//! Sockets for mods (docs/lua-mods.md 4.15, the `net` permission): TCP connections out and in, and UDP, for
//! companion programs. Each socket's reading runs on a thread; what it reads reaches the mod's handlers in the next
//! mod phase.

use super::api::{need, next_id, state};
use mlua::{Function, Lua, RegistryKey, Table, Value};
use std::io::{Read, Write};
use std::net::{Shutdown, SocketAddr, TcpListener, TcpStream, ToSocketAddrs, UdpSocket};
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::sync::mpsc::{Receiver, Sender, channel};
use std::sync::{Arc, Condvar, Mutex, PoisonError};
use std::time::Duration;

/// Ids of accepted connections, made on the accept threads (above the mod's ids).
static NEXT: AtomicU64 = AtomicU64::new(1 << 40);

/// What one socket's reader may hold read and not yet handed to the mod; past it the reader waits (for a stream, the
/// peer then waits too). This memory is the client's, outside the mod's Lua limit.
pub(super) const QUEUED_CAP: usize = 4 << 20;
/// What each report counts on top of its bytes, so empty reads and datagrams fill the queue too.
const REPORT_COST: usize = 64;

/// A socket's queued bytes (see `QUEUED_CAP`).
#[derive(Default)]
pub(super) struct Budget {
    used: Mutex<usize>,
    freed: Condvar,
}

impl Budget {
    /// Room for `n` bytes more (and the report's own cost), waiting while the queue is over the cap (one read always fits an empty queue);
    /// None once `stop` says the socket is closed.
    pub(super) fn take(self: &Arc<Self>, n: usize, stop: impl Fn() -> bool) -> Option<Credit> {
        let n = n + REPORT_COST;
        let mut used = self.used.lock().unwrap_or_else(PoisonError::into_inner);
        while *used > 0 && *used + n > QUEUED_CAP {
            if stop() {
                return None;
            }
            used = self.freed.wait_timeout(used, Duration::from_millis(200)).unwrap_or_else(PoisonError::into_inner).0;
        }
        *used += n;
        Some(Credit(self.clone(), n))
    }

    #[cfg(test)]
    fn used(&self) -> usize {
        *self.used.lock().unwrap_or_else(PoisonError::into_inner)
    }
}

/// Bytes queued on a socket's budget, given back when the data is handed over or dropped.
pub(super) struct Credit(Arc<Budget>, usize);

impl Drop for Credit {
    fn drop(&mut self) {
        *self.0.used.lock().unwrap_or_else(PoisonError::into_inner) -= self.1;
        self.0.freed.notify_all();
    }
}

/// What the threads report.
pub(super) enum Msg {
    Open(u64, Closer),
    Data(u64, Vec<u8>, Option<SocketAddr>, Credit),
    Accept(u64, u64, SocketAddr, Sender<Vec<u8>>, Closer, Arc<AtomicBool>),
    Closed(u64, Option<String>),
}

pub(super) enum Kind {
    /// A stream (TCP, a Unix socket, a pipe): its writer's queue and what shuts it.
    Tcp {
        tx: Sender<Vec<u8>>,
        stream: Option<Closer>,
        stop: Arc<AtomicBool>,
    },
    Listen {
        stop: Arc<AtomicBool>,
    },
    Udp {
        socket: UdpSocket,
        stop: Arc<AtomicBool>,
    },
}

pub(super) struct Sock {
    pub id: u64,
    pub handlers: Option<RegistryKey>,
    pub kind: Kind,
    /// A datagram was dropped past the mod's memory limit (said once per socket).
    pub dropped: bool,
}

impl Drop for Sock {
    fn drop(&mut self) {
        match &self.kind {
            Kind::Tcp { stream, stop, .. } => {
                stop.store(true, Ordering::Relaxed);
                if let Some(close) = stream {
                    close();
                }
            }
            Kind::Listen { stop } | Kind::Udp { stop, .. } => stop.store(true, Ordering::Relaxed),
        }
    }
}

/// The channel the threads report on, made with the first socket.
pub(super) fn reports(lua: &Lua) -> Sender<Msg> {
    let st = state(lua);
    let mut s = st.borrow_mut();
    if s.sock_tx.is_none() {
        let (tx, rx) = channel();
        (s.sock_tx, s.sock_rx) = (Some(tx), Some(rx));
    }
    s.sock_tx.clone().expect("made above")
}

/// Shuts a stream, which ends its reader.
pub(super) type Closer = Box<dyn Fn() + Send>;

fn tcp_closer(s: TcpStream) -> Closer {
    Box::new(move || {
        let _ = s.shutdown(Shutdown::Both);
    })
}

/// A TCP stream's reader and writer threads.
fn run_tcp(
    id: u64,
    s: &TcpStream,
    report: Sender<Msg>,
    out: Receiver<Vec<u8>>,
    stop: Arc<AtomicBool>,
) -> std::io::Result<()> {
    run_stream(id, s.try_clone()?, s.try_clone()?, report, out, stop).map(|_| ())
}

/// Where a reader thread is put once it runs, for a closer made before it (see `reader_closer`).
pub(super) type ReaderSlot = Arc<std::sync::Mutex<Option<std::thread::JoinHandle<()>>>>;

/// Windows: a blocking read on a file or pipe never ends by itself; closing cancels the reader thread's read.
#[cfg(windows)]
pub(super) fn reader_closer(slot: ReaderSlot) -> Closer {
    use std::os::windows::io::AsRawHandle;
    Box::new(move || {
        if let Some(h) = slot.lock().ok().as_ref().and_then(|g| g.as_ref()) {
            // SAFETY: the JoinHandle keeps the thread handle open.
            unsafe { windows_sys::Win32::System::IO::CancelSynchronousIo(h.as_raw_handle() as _) };
        }
    })
}

/// A connected stream's reader and writer threads (the writer takes what `out` queues); the reader's handle.
fn run_stream<R: Read + Send + 'static, W: Write + Send + 'static>(
    id: u64,
    mut r: R,
    mut w: W,
    report: Sender<Msg>,
    out: Receiver<Vec<u8>>,
    stop: Arc<AtomicBool>,
) -> std::io::Result<std::thread::JoinHandle<()>> {
    std::thread::Builder::new().name("lua-tcp-w".into()).spawn(move || {
        for b in out {
            if w.write_all(&b).is_err() {
                return;
            }
        }
    })?;
    std::thread::Builder::new().name("lua-tcp-r".into()).spawn(move || {
        let mut buf = vec![0u8; 65536];
        let budget = Arc::new(Budget::default());
        let closed = || stop.load(Ordering::Relaxed);
        // Closed (the socket's stop flag) ends it too: a Windows closer cancels only a read already running, not one
        // started after a wait for queue room.
        while !closed() {
            match r.read(&mut buf) {
                Ok(0) => {
                    let _ = report.send(Msg::Closed(id, None));
                    return;
                }
                Ok(n) => {
                    let Some(c) = budget.take(n, closed) else { return };
                    if report.send(Msg::Data(id, buf[..n].to_vec(), None, c)).is_err() {
                        return;
                    }
                }
                Err(e) => {
                    let _ = report.send(Msg::Closed(id, Some(e.to_string())));
                    return;
                }
            }
        }
    })
}

fn addr(host: &str, port: u16) -> mlua::Result<SocketAddr> {
    (host, port)
        .to_socket_addrs()
        .map_err(|e| mlua::Error::runtime(format!("{host}:{port}: {e}")))?
        .next()
        .ok_or_else(|| mlua::Error::runtime(format!("{host}:{port}: no address")))
}

pub(super) fn keep(lua: &Lua, id: u64, handlers: Table, kind: Kind) -> mlua::Result<()> {
    let key = lua.create_registry_value(handlers)?;
    state(lua).borrow_mut().socks.push(Sock { id, handlers: Some(key), kind, dropped: false });
    Ok(())
}

/// The system's control sockets, refused: each lets a client run programs (the session bus, systemd, agents, docker).
const PIPES_REFUSED: [&str; 12] = [
    "bus",
    "dbus-1",
    "systemd",
    "gnupg",
    "keyring",
    "gcr",
    "ssh-agent",
    "openssh-ssh-agent",
    "docker.sock",
    "docker_engine",
    "podman",
    "containerd",
];

/// A pipe name: a path inside the runtime folder (as `app/com.discordapp.Discord/discord-ipc-0`), never out of it,
/// and not one of the system's control sockets.
fn pipe_allowed(name: &str) -> Result<(), &'static str> {
    let parts: Vec<&str> = name.split('/').collect();
    if name.is_empty()
        || name.starts_with('/')
        || name.contains('\\')
        || parts.iter().any(|p| p.is_empty() || *p == "." || *p == "..")
    {
        return Err("a name in the runtime folder, not a path out of it");
    }
    if cfg!(windows) && parts.len() > 1 {
        return Err("a pipe name has no folders");
    }
    let first = parts[0].to_ascii_lowercase();
    if PIPES_REFUSED.iter().any(|r| first == *r || first.starts_with(&format!("{r}."))) || first.starts_with("ssh-") {
        return Err("a system control socket, refused");
    }
    Ok(())
}

/// A local endpoint another program offers: a Unix socket (a bare name in the runtime folder, as Discord's
/// `discord-ipc-0`), or on Windows the named pipe `\\.\pipe\<name>`.
#[cfg(unix)]
fn open_pipe(name: &str, _: ReaderSlot) -> std::io::Result<(Box<dyn Read + Send>, Box<dyn Write + Send>, Closer)> {
    use std::os::unix::net::UnixStream;
    let dir = std::env::var_os("XDG_RUNTIME_DIR")
        .or_else(|| std::env::var_os("TMPDIR"))
        .map_or_else(|| std::path::PathBuf::from("/tmp"), std::path::PathBuf::from);
    let path = dir.join(name);
    let s = UnixStream::connect(path)?;
    let (r, w, c) = (s.try_clone()?, s.try_clone()?, s);
    Ok((
        Box::new(r),
        Box::new(w),
        Box::new(move || {
            let _ = c.shutdown(Shutdown::Both);
        }),
    ))
}

#[cfg(windows)]
fn open_pipe(name: &str, slot: ReaderSlot) -> std::io::Result<(Box<dyn Read + Send>, Box<dyn Write + Send>, Closer)> {
    let path = format!(r"\\.\pipe\{name}");
    let f = std::fs::OpenOptions::new().read(true).write(true).open(path)?;
    let w = f.try_clone()?;
    Ok((Box::new(f), Box::new(w), reader_closer(slot)))
}

pub(super) fn install(lua: &Lua, t: &Table) -> mlua::Result<()> {
    t.set(
        "connect",
        lua.create_function(|lua, (host, port, handlers): (String, u16, Table)| {
            need(lua, "net")?;
            let id = next_id(&state(lua));
            let report = reports(lua);
            let (tx, rx) = channel::<Vec<u8>>();
            let stop = Arc::new(AtomicBool::new(false));
            let flag = stop.clone();
            std::thread::Builder::new()
                .name("lua-tcp".into())
                .spawn(move || {
                    let s = (host.as_str(), port)
                        .to_socket_addrs()
                        .map_err(|e| e.to_string())
                        .and_then(|mut a| a.next().ok_or_else(|| "no address".to_string()))
                        .and_then(|a| {
                            TcpStream::connect_timeout(&a, Duration::from_secs(10)).map_err(|e| e.to_string())
                        });
                    match s {
                        Ok(s) if !flag.load(Ordering::Relaxed) => {
                            let _ = s.set_nodelay(true);
                            // Open is reported before the reader starts, so no data comes ahead of it.
                            let r = s.try_clone().map(tcp_closer).and_then(|c| {
                                let _ = report.send(Msg::Open(id, c));
                                run_tcp(id, &s, report.clone(), rx, flag.clone())
                            });
                            if let Err(e) = r {
                                let _ = report.send(Msg::Closed(id, Some(e.to_string())));
                            }
                        }
                        Ok(_) => {}
                        Err(e) => {
                            let _ = report.send(Msg::Closed(id, Some(e)));
                        }
                    }
                })
                .map_err(|e| mlua::Error::runtime(format!("net.connect: {e}")))?;
            keep(lua, id, handlers, Kind::Tcp { tx, stream: None, stop })?;
            Ok(id)
        })?,
    )?;
    t.set(
        "pipe",
        lua.create_function(|lua, (name, handlers): (String, Table)| {
            need(lua, "net")?;
            pipe_allowed(&name).map_err(|e| mlua::Error::runtime(format!("net.pipe {name:?}: {e}")))?;
            let id = next_id(&state(lua));
            let report = reports(lua);
            let (tx, rx) = channel::<Vec<u8>>();
            let stop = Arc::new(AtomicBool::new(false));
            let flag = stop.clone();
            std::thread::Builder::new()
                .name("lua-pipe".into())
                .spawn(move || {
                    let slot = ReaderSlot::default();
                    let r = open_pipe(&name, slot.clone()).and_then(|(r, w, close)| {
                        let _ = report.send(Msg::Open(id, close));
                        let h = run_stream(id, r, w, report.clone(), rx, flag)?;
                        if let Ok(mut g) = slot.lock() {
                            *g = Some(h);
                        }
                        Ok(())
                    });
                    if let Err(e) = r {
                        let _ = report.send(Msg::Closed(id, Some(format!("{name}: {e}"))));
                    }
                })
                .map_err(|e| mlua::Error::runtime(format!("net.pipe: {e}")))?;
            keep(lua, id, handlers, Kind::Tcp { tx, stream: None, stop })?;
            Ok(id)
        })?,
    )?;
    t.set(
        "listen",
        lua.create_function(|lua, (port, handlers, opts): (u16, Table, Option<Table>)| {
            need(lua, "net")?;
            let host = opts
                .as_ref()
                .map(|o| o.get::<Option<String>>("host"))
                .transpose()?
                .flatten()
                .unwrap_or_else(|| "127.0.0.1".into());
            let l = TcpListener::bind(addr(&host, port)?)
                .map_err(|e| mlua::Error::runtime(format!("net.listen {host}:{port}: {e}")))?;
            let bound = l.local_addr().map(|a| a.port()).unwrap_or(port);
            l.set_nonblocking(true).map_err(|e| mlua::Error::runtime(e.to_string()))?;
            let id = next_id(&state(lua));
            let report = reports(lua);
            let stop = Arc::new(AtomicBool::new(false));
            let flag = stop.clone();
            std::thread::Builder::new()
                .name("lua-listen".into())
                .spawn(move || {
                    while !flag.load(Ordering::Relaxed) {
                        match l.accept() {
                            Ok((s, peer)) => {
                                let _ = s.set_nonblocking(false);
                                let _ = s.set_nodelay(true);
                                let cid = NEXT.fetch_add(1, Ordering::Relaxed);
                                let (tx, rx) = channel::<Vec<u8>>();
                                // Accept is reported before the reader starts, so no data comes ahead of it.
                                let Ok(c) = s.try_clone().map(tcp_closer) else { continue };
                                let stop = Arc::new(AtomicBool::new(false));
                                if report.send(Msg::Accept(id, cid, peer, tx, c, stop.clone())).is_err() {
                                    return;
                                }
                                if let Err(e) = run_tcp(cid, &s, report.clone(), rx, stop) {
                                    let _ = report.send(Msg::Closed(cid, Some(e.to_string())));
                                }
                            }
                            Err(e) if e.kind() == std::io::ErrorKind::WouldBlock => {
                                std::thread::sleep(Duration::from_millis(50))
                            }
                            Err(e) => {
                                let _ = report.send(Msg::Closed(id, Some(e.to_string())));
                                return;
                            }
                        }
                    }
                })
                .map_err(|e| mlua::Error::runtime(format!("net.listen: {e}")))?;
            keep(lua, id, handlers, Kind::Listen { stop })?;
            Ok((id, bound))
        })?,
    )?;
    t.set(
        "udp",
        lua.create_function(|lua, (port, handlers, opts): (Option<u16>, Table, Option<Table>)| {
            need(lua, "net")?;
            let host = opts
                .as_ref()
                .map(|o| o.get::<Option<String>>("host"))
                .transpose()?
                .flatten()
                .unwrap_or_else(|| "127.0.0.1".into());
            let port = port.unwrap_or(0);
            let socket = UdpSocket::bind(addr(&host, port)?)
                .map_err(|e| mlua::Error::runtime(format!("net.udp {host}:{port}: {e}")))?;
            let bound = socket.local_addr().map(|a| a.port()).unwrap_or(port);
            let r = socket.try_clone().map_err(|e| mlua::Error::runtime(e.to_string()))?;
            let _ = r.set_read_timeout(Some(Duration::from_millis(200)));
            let id = next_id(&state(lua));
            let report = reports(lua);
            let stop = Arc::new(AtomicBool::new(false));
            let flag = stop.clone();
            std::thread::Builder::new()
                .name("lua-udp".into())
                .spawn(move || {
                    let mut buf = vec![0u8; 65536];
                    let budget = Arc::new(Budget::default());
                    while !flag.load(Ordering::Relaxed) {
                        match r.recv_from(&mut buf) {
                            Ok((n, from)) => {
                                let Some(c) = budget.take(n, || flag.load(Ordering::Relaxed)) else { return };
                                if report.send(Msg::Data(id, buf[..n].to_vec(), Some(from), c)).is_err() {
                                    return;
                                }
                            }
                            Err(e)
                                if matches!(
                                    e.kind(),
                                    std::io::ErrorKind::WouldBlock | std::io::ErrorKind::TimedOut
                                ) => {}
                            Err(_) => {}
                        }
                    }
                })
                .map_err(|e| mlua::Error::runtime(format!("net.udp: {e}")))?;
            keep(lua, id, handlers, Kind::Udp { socket, stop })?;
            Ok((id, bound))
        })?,
    )?;
    t.set(
        "send",
        lua.create_function(|lua, (id, data, host, port): (u64, mlua::LuaString, Option<String>, Option<u16>)| {
            need(lua, "net")?;
            let st = state(lua);
            let s = st.borrow();
            let sock =
                s.socks.iter().find(|x| x.id == id).ok_or_else(|| mlua::Error::runtime(format!("no socket {id}")))?;
            match &sock.kind {
                Kind::Tcp { tx, .. } => {
                    tx.send(data.as_bytes().to_vec()).map_err(|_| mlua::Error::runtime("the connection is closed"))
                }
                Kind::Udp { socket, .. } => {
                    let (Some(h), Some(p)) = (host, port) else {
                        return Err(mlua::Error::runtime("a UDP send takes (id, data, host, port)"));
                    };
                    socket
                        .send_to(&data.as_bytes(), addr(&h, p)?)
                        .map(|_| ())
                        .map_err(|e| mlua::Error::runtime(e.to_string()))
                }
                Kind::Listen { .. } => Err(mlua::Error::runtime("send on a connection, not its listener")),
            }
        })?,
    )?;
    t.set("close", lua.create_function(|lua, id: u64| close(lua, id))?)?;
    Ok(())
}

/// Closes a socket or device by id and frees its handlers.
pub(super) fn close(lua: &Lua, id: u64) -> mlua::Result<()> {
    let st = state(lua);
    let gone: Vec<Sock> = {
        let mut s = st.borrow_mut();
        let (gone, kept) = std::mem::take(&mut s.socks).into_iter().partition(|x| x.id == id);
        s.socks = kept;
        gone
    };
    drop_all(lua, gone);
    Ok(())
}

/// Closes the sockets and frees their handlers.
fn drop_all(lua: &Lua, gone: Vec<Sock>) {
    for mut g in gone {
        if let Some(k) = g.handlers.take() {
            let _ = lua.remove_registry_value(k);
        }
    }
}

/// Takes the socket `id` out of the mod's list.
fn take(vm: &super::vm::Vm, id: u64) -> Vec<Sock> {
    let mut s = vm.st.borrow_mut();
    let (gone, kept) = std::mem::take(&mut s.socks).into_iter().partition(|x| x.id == id);
    s.socks = kept;
    gone
}

/// What the threads reported: each socket's handlers (open, data, accept, close).
pub(super) fn poll(vm: &super::vm::Vm) {
    let msgs: Vec<Msg> = match vm.st.borrow().sock_rx.as_ref() {
        Some(rx) => rx.try_iter().collect(),
        None => return,
    };
    for m in msgs {
        let id = match &m {
            Msg::Open(id, _) | Msg::Data(id, ..) | Msg::Accept(id, ..) | Msg::Closed(id, _) => *id,
        };
        let handlers: Option<Table> = vm
            .st
            .borrow()
            .socks
            .iter()
            .find(|s| s.id == id)
            .and_then(|s| s.handlers.as_ref())
            .and_then(|k| vm.lua.registry_value(k).ok());
        let Some(h) = handlers else {
            // Closed before the thread's report came: its stream is shut here, or the connection and its reader
            // stay open until the peer hangs up.
            if let Msg::Open(_, close) | Msg::Accept(.., close, _) = m {
                close();
            }
            continue;
        };
        let f = |name: &str| h.get::<Option<Function>>(name).ok().flatten();
        match m {
            Msg::Open(id, stream) => {
                if let Some(Kind::Tcp { stream: s, .. }) =
                    vm.st.borrow_mut().socks.iter_mut().find(|s| s.id == id).map(|s| &mut s.kind)
                {
                    *s = Some(stream);
                }
                if let Some(f) = f("open") {
                    vm.call(f, id);
                }
            }
            Msg::Data(id, bytes, from, credit) => {
                let Some(data) = f("data") else { continue };
                let b = vm.lua.create_string(&bytes);
                drop((bytes, credit));
                match (b, from) {
                    (Ok(b), Some(a)) => vm.call(data, (id, b, a.ip().to_string(), a.port())),
                    (Ok(b), None) => vm.call(data, (id, b)),
                    // Past the mod's memory limit, a datagram is lost as UDP may lose one: said (and logged) once per
                    // socket, not counted as the mod's error (a run of them would suspend it).
                    (Err(e), Some(_)) => {
                        let mut st = vm.st.borrow_mut();
                        if let Some(s) = st.socks.iter_mut().find(|s| s.id == id && !s.dropped) {
                            s.dropped = true;
                            log::warn!("{}: socket {id}: datagram dropped: {e}", vm.manifest.id);
                            let note =
                                format!("{}: socket {id}: datagrams dropped past the memory limit", vm.manifest.id);
                            st.notices.push(note);
                        }
                    }
                    // A stream would go on with a hole in it, so it is closed.
                    (Err(e), None) => {
                        let gone = take(vm, id);
                        drop_all(&vm.lua, gone);
                        if let Some(f) = f("close") {
                            let text = format!("data dropped: {e}");
                            vm.call(f, (id, vm.lua.create_string(&text).map_or(Value::Nil, Value::String)));
                        }
                    }
                }
            }
            Msg::Accept(lid, cid, peer, tx, stream, stop) => {
                let Ok(key) = vm.lua.create_registry_value(h.clone()) else { continue };
                vm.st.borrow_mut().socks.push(Sock {
                    id: cid,
                    handlers: Some(key),
                    kind: Kind::Tcp { tx, stream: Some(stream), stop },
                    dropped: false,
                });
                if let Some(f) = f("accept") {
                    vm.call(f, (lid, cid, peer.ip().to_string(), peer.port()));
                }
            }
            Msg::Closed(id, err) => {
                let gone = take(vm, id);
                drop_all(&vm.lua, gone);
                if let Some(f) = f("close") {
                    vm.call(
                        f,
                        (id, err.map_or(Value::Nil, |e| vm.lua.create_string(&e).map_or(Value::Nil, Value::String))),
                    );
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::super::{api, manifest, vm};
    use super::*;
    use std::path::PathBuf;

    fn vm() -> vm::Vm {
        let vm = vm::Vm::new(manifest::Manifest::default(), PathBuf::from("."), vm::Source::Dir(PathBuf::from(".")))
            .unwrap();
        api::install(&vm.lua, &vm, (serde_json::Value::Null, serde_json::Value::Null)).unwrap();
        vm
    }

    fn flag() -> (Arc<AtomicBool>, Closer) {
        let f = Arc::new(AtomicBool::new(false));
        let g = f.clone();
        (f, Box::new(move || g.store(true, Ordering::Relaxed)))
    }

    /// A reader waits while its socket's queue is past the cap, and goes on once the mod takes data.
    #[test]
    fn queue_is_capped() {
        let b = Arc::new(Budget::default());
        let first = b.take(QUEUED_CAP - 10, || false).unwrap();
        let (tx, rx) = channel();
        let b2 = b.clone();
        let t = std::thread::spawn(move || {
            let c = b2.take(65536, || false);
            tx.send(()).unwrap();
            c
        });
        assert!(rx.recv_timeout(Duration::from_millis(400)).is_err(), "the reader went past the cap");
        drop(first);
        rx.recv_timeout(Duration::from_secs(5)).expect("the reader went on once the data was taken");
        drop(t.join().unwrap());
        assert_eq!(b.used(), 0);
        // One read always fits an empty queue, even a large one; a closed socket stops the wait.
        let big = b.take(QUEUED_CAP * 2, || false).unwrap();
        assert!(b.take(1, || true).is_none());
        drop(big);
    }

    /// An Open or Accept that comes after the mod closed the socket shuts the connection (it leaked open, with its
    /// reader thread, until the peer hung up).
    #[test]
    fn late_open_and_accept_are_shut() {
        let vm = vm();
        let report = reports(&vm.lua);
        let (opened, close) = flag();
        report.send(Msg::Open(7, close)).unwrap();
        let (accepted, close) = flag();
        let (tx, _rx) = channel();
        let stop = Arc::new(AtomicBool::new(false));
        report.send(Msg::Accept(8, 1 << 41, "127.0.0.1:1".parse().unwrap(), tx, close, stop)).unwrap();
        poll(&vm);
        assert!(opened.load(Ordering::Relaxed), "late Open not shut");
        assert!(accepted.load(Ordering::Relaxed), "Accept on a closed listener not shut");
        assert!(vm.st.borrow().socks.is_empty());
    }

    /// A listener closed with a connection still queued: the peer sees the connection end.
    #[test]
    fn listener_closed_with_queued_accept() {
        let vm = vm();
        api::state(&vm.lua).borrow_mut().granted.push("net".into());
        let (lid, port): (u64, u16) = vm.lua.load("return net.listen(0, {})").eval().unwrap();
        let mut peer = TcpStream::connect(("127.0.0.1", port)).unwrap();
        // The accept thread polls every 50 ms.
        std::thread::sleep(Duration::from_millis(300));
        vm.lua.load(format!("net.close({lid})")).exec().unwrap();
        poll(&vm);
        peer.set_read_timeout(Some(Duration::from_secs(5))).unwrap();
        assert_eq!(peer.read(&mut [0u8; 8]).unwrap(), 0, "the accepted connection stayed open");
    }

    /// Data past the mod's memory limit closes the socket with an error, rather than leave a hole in the stream.
    #[test]
    fn data_past_the_memory_limit_closes() {
        let vm = vm();
        let report = reports(&vm.lua);
        let h: Table = vm
            .lua
            .load("got = {}; return {data = function() got.data = true end, close = function(id, e) got.err = e end}")
            .eval()
            .unwrap();
        keep(&vm.lua, 5, h, Kind::Listen { stop: Arc::new(AtomicBool::new(false)) }).unwrap();
        // A test-sized limit: a real read is at most 64 KiB.
        vm.lua.set_memory_limit(vm.lua.used_memory() + (32 << 10)).unwrap();
        let b = Arc::new(Budget::default());
        let c = b.take(1, || false).unwrap();
        report.send(Msg::Data(5, vec![b'x'; 65536], None, c)).unwrap();
        poll(&vm);
        let (data, err): (Option<bool>, Option<String>) = vm.lua.load("return got.data, got.err").eval().unwrap();
        assert_eq!(data, None);
        assert!(err.is_some_and(|e| e.starts_with("data dropped")));
        assert!(vm.st.borrow().socks.is_empty());
        assert_eq!(b.used(), 0);
    }

    /// A datagram past the limit is lost alone (as UDP may lose one), the error reported; the socket stays.
    #[test]
    fn datagram_past_the_memory_limit_is_dropped() {
        let vm = vm();
        let report = reports(&vm.lua);
        let h: Table =
            vm.lua.load("got = {}; return {close = function() got.closed = true end, data = print}").eval().unwrap();
        keep(&vm.lua, 6, h, Kind::Listen { stop: Arc::new(AtomicBool::new(false)) }).unwrap();
        vm.lua.set_memory_limit(vm.lua.used_memory() + (32 << 10)).unwrap();
        // More than the error limit's worth: not one counts toward suspending the mod.
        for _ in 0..30 {
            let c = Arc::new(Budget::default()).take(1, || false).unwrap();
            report.send(Msg::Data(6, vec![b'x'; 65536], Some("127.0.0.1:9".parse().unwrap()), c)).unwrap();
        }
        poll(&vm);
        let closed: Option<bool> = vm.lua.load("return got.closed").eval().unwrap();
        assert_eq!(closed, None);
        assert_eq!(vm.st.borrow().socks.len(), 1);
        assert!(vm.st.borrow().errors.is_empty());
        assert_eq!(vm.st.borrow().notices.len(), 1);
    }
}

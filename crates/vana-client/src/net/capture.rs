//! Packet capture (ours, opt-in): the map server's traffic each way, as the wire carries it after the cipher and
//! the coder, written one JSON object a line to `<data dir>/captures/<unix seconds>-<process>.jsonl`, one file a client run
//! (every zone and character of it), readable by this user only.
//!
//! On while config `packet_capture` is set or `VANA_CAPTURE` is in the environment (any value but 0; a value other
//! than 1 is the folder to write to). Only the map session is recorded: the lobby, PlayOnline and patch connections
//! never are, since they carry the password and the account's keys. The one map packet with sign-in data, c2s 0x00A,
//! is written with its account name and ticket zeroed. Chat and tells are recorded as they come, so a capture is
//! someone's game session: share it with care.
//!
//! A sub-packet line: `{"t":ms,"dir":"in"|"out","zone":n,"id":n,"hex":"…"}` (the whole sub-packet with its 4-byte
//! header). A datagram line: `{"t":ms,"dir":…,"zone":n,"dgram":n,"seq":n,"ack":n,"form":"huffman"|"plain"|"raw"}`
//! (its size in bytes, its header's sequence and ack, and the coder's form: retail's leading byte 1 is Huffman,
//! anything else the plain copy, and the first 0x00A goes raw). `t` is Unix time in ms. Each sub-packet is written
//! once: as it first goes out, and as it is first accepted (a resend of either shows only as a datagram line). Read
//! one back with `vana-client capture FILE`.

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use std::collections::VecDeque;
use std::fs::File;
use std::io::{BufRead, LineWriter, Write};
use std::path::{Path, PathBuf};
use std::sync::Mutex;
use std::sync::atomic::{AtomicBool, AtomicU16, Ordering::Relaxed};
use std::time::{SystemTime, UNIX_EPOCH};

/// One line of a capture.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Record {
    pub t: u64,
    pub dir: Dir,
    pub zone: u16,
    #[serde(flatten)]
    pub kind: Kind,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Dir {
    In,
    Out,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(untagged)]
pub enum Kind {
    Packet {
        id: u16,
        #[serde(with = "hex")]
        hex: Vec<u8>,
    },
    Datagram {
        dgram: usize,
        seq: u16,
        ack: u16,
        form: String,
    },
}

static ON: AtomicBool = AtomicBool::new(false);
static OUT: Mutex<Option<LineWriter<File>>> = Mutex::new(None);
/// The zone the last s2c 0x00A named.
static ZONE: AtomicU16 = AtomicU16::new(0);
/// The live feed (the Packets window's): records kept in memory while it is on, oldest dropped past `FEED_MAX`.
static LIVE: AtomicBool = AtomicBool::new(false);
static FEED: Mutex<VecDeque<Record>> = Mutex::new(VecDeque::new());
const FEED_MAX: usize = 100_000;

fn env() -> Option<String> {
    std::env::var("VANA_CAPTURE").ok().filter(|v| v != "0")
}

/// The config switch, set each frame; the environment variable turns it on whatever the config says.
pub fn set(on: bool) {
    let on = on || env().is_some();
    if !ON.swap(on, Relaxed) && on {
        log::info!("capture: on, writing to {}", dir().display());
    }
}

/// Whether a capture file is being written.
pub fn recording() -> bool {
    ON.load(Relaxed)
}

/// The live feed on or off (off empties it).
pub fn live(on: bool) {
    LIVE.store(on, Relaxed);
    if !on {
        FEED.lock().unwrap_or_else(|e| e.into_inner()).clear();
    }
}

/// The records fed since the last call.
pub fn take_live() -> Vec<Record> {
    FEED.lock().unwrap_or_else(|e| e.into_inner()).drain(..).collect()
}

#[cfg(test)]
static TEST_DIR: Mutex<Option<PathBuf>> = Mutex::new(None);

/// Capture on into `dir` (a test's own folder), or off.
#[cfg(test)]
pub fn test_capture(dir: Option<PathBuf>) {
    ON.store(dir.is_some(), Relaxed);
    *TEST_DIR.lock().unwrap() = dir;
}

/// Where capture files go.
pub fn dir() -> PathBuf {
    #[cfg(test)]
    if let Some(d) = TEST_DIR.lock().unwrap().clone() {
        return d;
    }
    match env() {
        Some(d) if d != "1" => PathBuf::from(d),
        _ => crate::config::data_dir().join("captures"),
    }
}

/// A new file only this user can read (a capture holds chat and tells), failing if it exists: never one a recording
/// is writing, nor an existing file's looser permissions.
fn private_file(path: &Path) -> std::io::Result<File> {
    let mut o = std::fs::OpenOptions::new();
    o.write(true).create_new(true);
    #[cfg(unix)]
    std::os::unix::fs::OpenOptionsExt::mode(&mut o, 0o600);
    o.open(path)
}

/// A new capture file in `dir`, `<unix seconds>-<process>.jsonl` (a suffix after it if taken): never one another
/// client is writing.
fn new_capture(dir: &Path) -> std::io::Result<(File, PathBuf)> {
    let stem = format!("{}-{}", ms() / 1000, std::process::id());
    for n in 0..100 {
        let path = dir.join(if n == 0 { format!("{stem}.jsonl") } else { format!("{stem}-{n}.jsonl") });
        match private_file(&path) {
            Ok(f) => return Ok((f, path)),
            Err(e) if e.kind() == std::io::ErrorKind::AlreadyExists => continue,
            Err(e) => return Err(e),
        }
    }
    Err(std::io::Error::new(std::io::ErrorKind::AlreadyExists, "no free capture name"))
}

fn ms() -> u64 {
    SystemTime::now().duration_since(UNIX_EPOCH).map(|d| d.as_millis() as u64).unwrap_or(0)
}

fn wanted() -> bool {
    ON.load(Relaxed) || LIVE.load(Relaxed)
}

/// Stamps a record with the time and zone (`packet` keeps it), feeds it live and writes it to the file (opened first; closed while
/// capture is off).
fn record(out: bool, kind: Kind) {
    let side = if out { Dir::Out } else { Dir::In };
    let rec = Record { t: ms(), dir: side, zone: ZONE.load(Relaxed), kind };
    if LIVE.load(Relaxed) {
        let mut feed = FEED.lock().unwrap_or_else(|e| e.into_inner());
        if feed.len() >= FEED_MAX {
            feed.pop_front();
        }
        feed.push_back(rec.clone());
    }
    let mut file = OUT.lock().unwrap_or_else(|e| e.into_inner());
    if !ON.load(Relaxed) {
        *file = None;
        return;
    }
    if file.is_none() {
        match std::fs::create_dir_all(dir()).and_then(|_| new_capture(&dir())) {
            Ok((f, path)) => {
                log::info!("capture: {}", path.display());
                *file = Some(LineWriter::new(f));
            }
            Err(e) => {
                log::warn!("capture: not opened in {}, capture off: {e}", dir().display());
                ON.store(false, Relaxed);
                return;
            }
        }
    }
    let w = file.as_mut().unwrap();
    if let Err(e) = serde_json::to_string(&rec).map_err(std::io::Error::other).and_then(|l| writeln!(w, "{l}")) {
        log::warn!("capture: write failed, capture off: {e}");
        ON.store(false, Relaxed);
        *file = None;
    }
}

/// One sub-packet (with its 4-byte header) as sent or received.
pub fn packet(out: bool, data: &[u8]) {
    if data.len() < 4 {
        return;
    }
    let id = u16::from_le_bytes([data[0], data[1]]) & 0x1ff;
    // The zone is kept while nothing records, so a capture or the live view started later has it.
    if !out && id == 0x00A && data.len() >= 0x32 {
        ZONE.store(u16::from_le_bytes([data[0x30], data[0x31]]), Relaxed);
    }
    if !wanted() {
        return;
    }
    let hex = if out && id == 0x00A { redact_login(data) } else { data.to_vec() };
    record(out, Kind::Packet { id, hex });
}

/// One datagram: its size, header sequence and ack, and the coder's form byte (`None` for the raw first 0x00A).
pub fn datagram(out: bool, bytes: usize, seq: u16, ack: u16, form: Option<u8>) {
    if !wanted() {
        return;
    }
    let form = match form {
        None => "raw",
        Some(1) => "huffman",
        Some(_) => "plain",
    };
    record(out, Kind::Datagram { dgram: bytes, seq, ack, form: form.into() });
}

/// c2s 0x00A without what signs in: the account name (packet +0x31, 15 bytes) and the ticket hashed from it and the
/// session key (+0x40, 16 bytes). Everything else (character id and name, language) stays.
fn redact_login(data: &[u8]) -> Vec<u8> {
    let mut d = data.to_vec();
    if let Some(s) = d.get_mut(0x31..0x50) {
        s.fill(0);
    }
    d
}

/// Common names for the ids a capture is read for (LSB's naming); an id not here has none.
pub fn name(dir: Dir, id: u16) -> &'static str {
    let s2c: &[(u16, &str)] = &[
        (0x009, "message"),
        (0x00A, "zone in"),
        (0x00B, "zone out"),
        (0x00D, "char update"),
        (0x00E, "npc update"),
        (0x017, "chat"),
        (0x01B, "job info"),
        (0x01C, "inventory size"),
        (0x01D, "inventory finish"),
        (0x01E, "inventory modify"),
        (0x01F, "inventory assign"),
        (0x020, "inventory item"),
        (0x028, "action"),
        (0x029, "action message"),
        (0x02A, "message special"),
        (0x02D, "message debug"),
        (0x032, "event"),
        (0x033, "event string"),
        (0x034, "event params"),
        (0x036, "npc message"),
        (0x037, "char status"),
        (0x038, "animation"),
        (0x050, "equip"),
        (0x051, "char look"),
        (0x053, "system message"),
        (0x055, "key items"),
        (0x057, "weather"),
        (0x05B, "position"),
        (0x061, "char stats"),
        (0x062, "skills"),
        (0x067, "char sync"),
        (0x076, "party effects"),
        (0x08C, "merits"),
        (0x08D, "job points"),
        (0x08F, "celestite"),
        (0x0AA, "spells"),
        (0x0AC, "abilities"),
        (0x0DD, "party member"),
        (0x0DF, "char health"),
    ];
    let c2s: &[(u16, &str)] = &[
        (0x00A, "login"),
        (0x00D, "leave zone"),
        (0x015, "position"),
        (0x01A, "action"),
        (0x05B, "event end"),
        (0x0B5, "chat"),
        (0x0B6, "tell"),
        (0x0C5, "celestite set"),
        (0x0C6, "celestite request"),
        (0x110, "fishing"),
    ];
    let t = if dir == Dir::Out { c2s } else { s2c };
    t.iter().find(|(i, _)| *i == id).map_or("", |(_, n)| n)
}

#[derive(Clone, Copy)]
enum Ty {
    U8,
    U16,
    U32,
    F32,
    /// Shift-JIS text cut at its first zero, this many bytes at most (0: to the end).
    Str(usize),
    /// Bytes in hex.
    Bytes(usize),
    /// Little-endian dwords in hex.
    Dwords(usize),
}

/// The fields this client reads from an id (the offsets of its own parsers, client/net.rs, and the decompile notes
/// in docs/modlog), for the Packets window. Ids not here show only their bytes.
fn layout(dir: Dir, id: u16) -> &'static [(&'static str, usize, Ty)] {
    use Ty::*;
    match (dir, id) {
        (Dir::In, 0x009) => &[("MesNo", 0xa, U16), ("Data", 0xd, Str(0))],
        (Dir::In, 0x00A) => &[
            ("zone", 0x30, U16),
            ("clock ms", 0x34, U32),
            ("clock s", 0x38, U32),
            ("header word 1", 0x94, U32),
            ("header word 2", 0x98, U32),
        ],
        (Dir::In, 0x00B) => &[("state", 4, U8), ("ip", 8, U32)],
        (Dir::In, 0x017) => {
            &[("Kind", 4, U8), ("Attr", 5, U8), ("Data", 6, U16), ("name", 8, Str(15)), ("text", 0x17, Str(0))]
        }
        (Dir::In, 0x029) => &[
            ("actor", 4, U32),
            ("target", 8, U32),
            ("data 0", 0xc, U32),
            ("data 1", 0x10, U32),
            ("actor index", 0x14, U16),
            ("target index", 0x16, U16),
            ("message", 0x18, U16),
            ("type", 0x1a, U8),
        ],
        (Dir::In, 0x02A) => &[
            ("actor", 4, U32),
            ("num 0", 8, U32),
            ("num 1", 0xc, U32),
            ("num 2", 0x10, U32),
            ("num 3", 0x14, U32),
            ("index", 0x18, U16),
            ("mes", 0x1a, U16),
            ("kind", 0x1c, U8),
            ("name", 0x1e, Str(32)),
        ],
        (Dir::In, 0x02D) => &[
            ("actor", 4, U32),
            ("target", 8, U32),
            ("actor index", 0xc, U16),
            ("target index", 0xe, U16),
            ("data 0", 0x10, U32),
            ("data 1", 0x14, U32),
            ("message", 0x18, U16),
            ("type", 0x1a, U8),
        ],
        (Dir::In, 0x036) => &[("actor", 4, U32), ("index", 8, U16), ("mes", 0xa, U16), ("kind", 0xc, U8)],
        (Dir::In, 0x053) => &[("para", 4, U32), ("para2", 8, U32), ("id", 0xc, U16)],
        (Dir::In, 0x05B) => {
            &[("x", 4, F32), ("y", 8, F32), ("z", 0xc, F32), ("id", 0x10, U32), ("mode", 0x16, U8), ("dir", 0x17, U8)]
        }
        // FUN_100e5290: 0x2b dwords from +4 (docs/modlog/2026-11-15-celestite-windows-153-159.md).
        (Dir::In, 0x08F) => &[("block", 4, Dwords(0x2b))],
        (Dir::Out, 0x00A) => &[("char id", 0xc, U32), ("char name", 0x22, Str(15))],
        // FUN_100e5200: the record number (set + 1) at +4, ten celestites at +5.
        (Dir::Out, 0x0C5) => &[("record", 4, U8), ("celestites", 5, Bytes(10))],
        _ => &[],
    }
}

/// A sub-packet's header (id, size, sync) and the fields `layout` knows, as (name, value) pairs.
pub fn decode(dir: Dir, data: &[u8]) -> Vec<(&'static str, String)> {
    if data.len() < 4 {
        return Vec::new();
    }
    let word = u16::from_le_bytes([data[0], data[1]]);
    let id = word & 0x1ff;
    let mut out = vec![
        ("id", format!("{id:#05x} {}", name(dir, id))),
        ("size", format!("{} ({:#x} bytes)", (word >> 9) * 4, (word >> 9) * 4)),
        ("sync", u16::from_le_bytes([data[2], data[3]]).to_string()),
    ];
    let at = |o: usize, n: usize| data.get(o..o + n);
    for &(field, o, ty) in layout(dir, id) {
        let v = match ty {
            Ty::U8 => at(o, 1).map(|b| format!("{0} ({0:#04x})", b[0])),
            Ty::U16 => at(o, 2).map(|b| format!("{0} ({0:#06x})", u16::from_le_bytes([b[0], b[1]]))),
            Ty::U32 => at(o, 4).map(|b| format!("{0} ({0:#010x})", u32::from_le_bytes(b.try_into().unwrap()))),
            Ty::F32 => at(o, 4).map(|b| format!("{:.3}", f32::from_le_bytes(b.try_into().unwrap()))),
            Ty::Str(n) => data.get(o..).map(|b| {
                let b = if n == 0 { b } else { &b[..n.min(b.len())] };
                let b = &b[..b.iter().position(|&c| c == 0).unwrap_or(b.len())];
                format!("{:?}", crate::dat::font::decode_sjis(b))
            }),
            Ty::Bytes(n) => at(o, n).map(hex::encode),
            Ty::Dwords(n) => at(o, n * 4).map(|b| {
                b.chunks(4)
                    .map(|d| format!("{:08x}", u32::from_le_bytes(d.try_into().unwrap())))
                    .collect::<Vec<_>>()
                    .join(" ")
            }),
        };
        out.push((field, v.unwrap_or_else(|| "(past the end)".into())));
    }
    out
}

/// Bytes to look for, typed as hex ("1e 09") or as text in quotes ("\"Celestite\""), the text as the packets carry
/// it (Shift-JIS).
pub fn parse_find(s: &str) -> Result<Vec<u8>, String> {
    let s = s.trim();
    if let Some(t) = s.strip_prefix('"') {
        return Ok(crate::dat::font::encode_sjis(t.strip_suffix('"').unwrap_or(t)));
    }
    hex::decode(s.replace([' ', ','], "")).map_err(|_| format!("not hex: {s}"))
}

/// Ids typed as hex, comma or space separated ("08f, 0x0c5").
pub fn parse_ids(s: &str) -> Result<Vec<u16>, String> {
    s.split([',', ' '])
        .filter(|t| !t.is_empty())
        .map(|t| u16::from_str_radix(t.trim_start_matches("0x"), 16).map_err(|_| format!("not a hex id: {t}")))
        .collect()
}

/// Which sub-packets a reader keeps.
#[derive(Clone, Default)]
pub struct Filter {
    pub ids: Vec<u16>,
    pub dir: Option<Dir>,
    pub zone: Option<u16>,
    /// Bytes that must appear somewhere in the sub-packet.
    pub find: Vec<u8>,
}

impl Filter {
    pub fn keeps(&self, r: &Record) -> bool {
        let Kind::Packet { id, hex } = &r.kind else { return false };
        (self.ids.is_empty() || self.ids.contains(id))
            && self.dir.is_none_or(|d| d == r.dir)
            && self.zone.is_none_or(|z| z == r.zone)
            && (self.find.is_empty() || hex.windows(self.find.len()).any(|w| w == self.find))
    }
}

/// A capture file's records, skipping lines that don't parse.
pub fn load(path: &Path) -> Result<Vec<Record>> {
    let f = File::open(path).with_context(|| format!("{} not opened", path.display()))?;
    let mut out = Vec::new();
    for line in std::io::BufReader::new(f).lines() {
        if let Ok(r) = serde_json::from_str(&line?) {
            out.push(r);
        }
    }
    Ok(out)
}

/// Writes records as a new capture file (an existing one, a recording among them, is left alone).
pub fn save<'a>(path: &Path, records: impl IntoIterator<Item = &'a Record>) -> Result<()> {
    let f =
        private_file(path).with_context(|| format!("{} not created (a file by that name exists?)", path.display()))?;
    let mut w = std::io::BufWriter::new(f);
    for r in records {
        writeln!(w, "{}", serde_json::to_string(r)?)?;
    }
    Ok(w.flush()?)
}

/// What `vana-client capture` prints: the kept sub-packets (with their bytes when `hex`), then a count of each id
/// each way and of the datagram forms each way.
pub fn report(records: &[Record], f: &Filter, hex_dump: bool) -> String {
    use std::fmt::Write;
    let mut s = String::new();
    let (mut ids, mut forms) = (BTreeMap::<(Dir, u16), u32>::new(), BTreeMap::<(Dir, &str), u32>::new());
    let start = records.first().map_or(0, |r| r.t);
    let d = |d: Dir| if d == Dir::Out { "out" } else { "in" };
    for r in records {
        match &r.kind {
            Kind::Datagram { form, .. } if f.dir.is_none_or(|d| d == r.dir) && f.zone.is_none_or(|z| z == r.zone) => {
                *forms.entry((r.dir, form)).or_default() += 1
            }
            Kind::Datagram { .. } => {}
            Kind::Packet { id, hex } if f.keeps(r) => {
                *ids.entry((r.dir, *id)).or_default() += 1;
                let secs = r.t.saturating_sub(start) as f64 / 1000.0;
                let _ = writeln!(
                    s,
                    "{secs:>9.3} {:<3} z{:<4} {id:#05x} {:<18} {:>4}",
                    d(r.dir),
                    r.zone,
                    name(r.dir, *id),
                    hex.len()
                );
                if hex_dump {
                    for (k, row) in hex.chunks(16).enumerate() {
                        let _ = writeln!(s, "    {:04x}  {}", k * 16, hex::encode(row));
                    }
                }
            }
            Kind::Packet { .. } => {}
        }
    }
    let _ = writeln!(s, "\nsub-packets:");
    for ((dir, id), n) in &ids {
        let _ = writeln!(s, "  {:<3} {id:#05x} {:<18} {n}", d(*dir), name(*dir, *id));
    }
    let _ = writeln!(s, "datagrams:");
    for ((dir, form), n) in &forms {
        let _ = writeln!(s, "  {:<3} {form:<8} {n}", d(*dir));
    }
    s
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The login packet keeps its character but loses the account and ticket.
    #[test]
    fn login_redacted() {
        let mut body = vec![0u8; 88];
        body[8..12].copy_from_slice(&0x1234u32.to_le_bytes());
        body[30..35].copy_from_slice(b"Livea");
        body[45..52].copy_from_slice(b"account");
        body[60..76].fill(0xAB);
        let p = crate::net::map::session::build(0x00A, &body);
        let r = redact_login(&p);
        assert_eq!(&r[..0x31], &p[..0x31]);
        assert!(r[0x31..0x50].iter().all(|&b| b == 0));
        assert_eq!(&r[0x50..], &p[0x50..]);
        assert!(!r.windows(7).any(|w| w == b"account"));
    }

    /// Known ids decode by their layout; a field past the packet's end says so.
    #[test]
    fn decodes() {
        let mut d = crate::net::map::session::build(0x0C5, &[0u8; 12]);
        d[4] = 3;
        d[5..15].copy_from_slice(&[1, 2, 3, 4, 5, 6, 7, 8, 9, 10]);
        let f = decode(Dir::Out, &d);
        assert_eq!(f[0], ("id", "0x0c5 celestite set".into()));
        assert!(
            f.contains(&("record", "3 (0x03)".into())) && f.contains(&("celestites", "0102030405060708090a".into()))
        );
        let f = decode(Dir::In, &crate::net::map::session::build(0x053, &[0u8; 4]));
        assert!(f.contains(&("id", "(past the end)".into())), "{f:?}");
        assert_eq!(parse_find("1e 09"), Ok(vec![0x1e, 9]));
        assert_eq!(parse_find("\"Cel\""), Ok(b"Cel".to_vec()));
        // Text as the packets carry it: Shift-JIS.
        assert_eq!(parse_find("\"ア\""), Ok(vec![0x83, 0x41]));
        assert_eq!(parse_ids("08f, 0x0C5"), Ok(vec![0x08F, 0x0C5]));
        assert!(parse_ids("zz").is_err());
    }

    /// The live feed gets what the hooks record while it is on, and nothing once it is off.
    #[test]
    fn live_feed() {
        live(true);
        packet(false, &crate::net::map::session::build(0x1FD, &[9; 4]));
        let got = take_live();
        assert!(got.iter().any(|r| matches!(&r.kind, Kind::Packet { id: 0x1FD, .. })));
        live(false);
        packet(false, &crate::net::map::session::build(0x1FD, &[9; 4]));
        assert!(!take_live().iter().any(|r| matches!(&r.kind, Kind::Packet { id: 0x1FD, .. })));
    }

    /// Two captures started in the same second get their own files, readable by this user only.
    #[test]
    fn captures_never_share_a_file() {
        let dir = std::env::temp_dir().join(format!("vana-capture-new-{}", std::process::id()));
        std::fs::create_dir_all(&dir).unwrap();
        let (_, a) = new_capture(&dir).unwrap();
        let (_, b) = new_capture(&dir).unwrap();
        assert_ne!(a, b);
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            assert_eq!(std::fs::metadata(&a).unwrap().permissions().mode() & 0o777, 0o600);
        }
        let _ = std::fs::remove_dir_all(&dir);
    }

    /// Lines round-trip through save and load in the documented shape, and the report filters and counts them.
    #[test]
    fn save_load_report() {
        let recs = vec![
            Record {
                t: 1000,
                dir: Dir::In,
                zone: 230,
                kind: Kind::Datagram { dgram: 60, seq: 1, ack: 0, form: "plain".into() },
            },
            Record {
                t: 1000,
                dir: Dir::In,
                zone: 230,
                kind: Kind::Packet { id: 0x08F, hex: vec![0x8f, 0x2c, 1, 0, 0x1e, 0x09] },
            },
            Record { t: 1500, dir: Dir::Out, zone: 230, kind: Kind::Packet { id: 0x0C6, hex: vec![0xc6, 0x02, 2, 0] } },
        ];
        let line = serde_json::to_string(&recs[1]).unwrap();
        assert_eq!(line, r#"{"t":1000,"dir":"in","zone":230,"id":143,"hex":"8f2c01001e09"}"#);
        let path = std::env::temp_dir().join(format!("vana-capture-{}.jsonl", std::process::id()));
        let _ = std::fs::remove_file(&path);
        save(&path, &recs).unwrap();
        assert_eq!(load(&path).unwrap(), recs);
        // An existing file (a recording, say) is never replaced.
        assert!(save(&path, &recs[..1]).is_err());
        assert_eq!(load(&path).unwrap(), recs);
        let _ = std::fs::remove_file(&path);

        let f = Filter { find: vec![0x1e, 0x09], ..Default::default() };
        let r = report(&recs, &f, true);
        assert!(r.contains("0x08f celestite") && !r.contains("0x0c6"), "{r}");
        assert!(r.contains("8f2c01001e09") && r.contains("in  plain    1"), "{r}");
        let f = Filter { dir: Some(Dir::Out), ids: vec![0x0C6], ..Default::default() };
        let r = report(&recs, &f, false);
        assert!(r.contains("out 0x0c6 celestite request  1") && !r.contains("in  plain"), "{r}");
    }
}

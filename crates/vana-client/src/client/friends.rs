//! The Friend List's data (MODLOG "The Friend List"): what polcore and FFXiMain's friend manager held, filled
//! from LSB's xi_profile (net/profile.rs): our open status, handle and characters, the friend and block lists, the
//! friends' presence from the IRC notices.

use super::World;
use crate::net::pol::polpro;
use crate::net::profile::{self, Cred, FriendInfo, Irc, IrcLink};
use crate::net::search::Record;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex, mpsc};
use std::time::{Duration, Instant};

/// One friend or block-list entry (sqPolFriend and FFXiMain's part of it, 2.2).
#[derive(Clone, Debug, Default)]
pub struct Friend {
    pub black: bool,
    /// The list index (0..199 friends, 0..99 block list) and display order.
    pub index: u8,
    pub order: u8,
    pub account: u64,
    /// What we call them (the handle name of the entry).
    pub name: String,
    pub pending: bool,
    /// From the notices: 0 offline, 1 online, 2 away (polcore's entry bits 13..15, `profile::entry_status`); the
    /// character they play; the newest notice's age.
    pub status: u8,
    pub char_id: Option<u32>,
    pub age: u64,
    /// The entry's playing bit (16) and the last character primitive a notice carried (polcore keeps one per
    /// character slot; ours the last): `char_id` is that character while playing.
    playing: bool,
    last_char: Option<u32>,
    /// The playing character's search record (+0xb0) and whether it is current (+0xfc bit 0).
    pub record: Option<Record>,
    pub fresh: bool,
    /// An unread message from them (+0xfc bit 1).
    pub mail: bool,
}

impl Friend {
    fn from_info(f: &FriendInfo) -> Self {
        Self {
            black: f.black,
            index: f.num,
            order: f.order,
            account: f.pol,
            name: f.name.clone(),
            pending: f.pending,
            ..Default::default()
        }
    }
}

/// The handle and character lists.
struct Tables {
    handles: Vec<[u8; 0x88]>,
    chars: Vec<[u8; 0x68]>,
}

/// polcore's side of the session: its status (what LoadMyStatus and each ChangeMyStatus that went through left) and
/// its handle and character tables. Shared with the status jobs, as polcore's globals are with FFXiMain's job runner.
pub struct PolSide {
    pub status: polpro::Status,
    pub tables: polpro::Tables,
}

impl Default for PolSide {
    fn default() -> Self {
        Self { status: polpro::Status::start(), tables: polpro::Tables::default() }
    }
}

impl PolSide {
    /// What polcore makes of a ChangeMyStatus with these arguments (`polpro::change_my_status`): `Ok(None)` sends
    /// nothing.
    #[allow(clippy::type_complexity)]
    pub fn plan(&self, args: [i32; 6]) -> Result<Option<([u8; 0x24], polpro::Status)>, i32> {
        let slots = &self.tables.slots;
        polpro::change_my_status(&self.status, args, |c| slots.get(c as usize).copied().unwrap_or_default())
    }
}

enum Done {
    /// LoadFriendList (at the start, apart from the IRC sign-on).
    List(anyhow::Result<Vec<FriendInfo>>),
    /// The handle and character lists (at the start).
    Tables(anyhow::Result<Tables>),
    /// LoadMyStatus's reply (after the sign-on).
    MyStatus(anyhow::Result<Vec<u8>>),
    /// ChangeMyStatus answered: the open status it set (None: only the in-game flag).
    Status(anyhow::Result<()>, Option<u8>),
    /// Anything else: applied on the main thread.
    Then(Box<dyn FnOnce(&mut World) + Send>),
}

/// A message in the local folders (retail's 7 received unread / 8 read, 2.4): downloaded, then deleted on the server.
#[derive(Clone, Debug)]
pub struct Mail {
    pub name: String,
    pub header: profile::MsgHeader,
    pub data: Vec<u8>,
    pub unread: bool,
}

#[derive(Default)]
pub struct Friends {
    cred: Option<Cred>,
    irc: Option<IrcLink>,
    jobs: Vec<mpsc::Receiver<Done>>,
    pub list: Vec<Friend>,
    /// Our open status (DAT_104df90c): 0 Online, 1 Away, 4 Invisible.
    pub my_status: u8,
    /// polcore's status and tables for this session.
    pol: Arc<Mutex<PolSide>>,
    /// The status object's block in force (DAT_104e0abc +0, FUN_100f78d0): ChangeMyStatus's six arguments, read from
    /// polcore's getters (slots 0x2f8, 0x2f4, 0x330, 0x338, -1, 0x33c) once our character is the active one. Its
    /// open status ([3]) is `my_status`.
    in_force: Option<[i32; 6]>,
    /// Our handle (LoadHandleNameList: LSB's account's first character) and the account's characters.
    pub handle: Option<(u64, String)>,
    pub chars: Vec<(u8, u32, String)>,
    pub signed_on: bool,
    /// This sign-on's LoadMyStatus is in: our status is known.
    pub loaded: bool,
    /// The friend list and the handle / character lists are in.
    lists: bool,
    tables: bool,
    started: bool,
    /// The in-game ChangeMyStatus went (after the map session exists, so friends see the character).
    announced: bool,
    /// A failed active-character call is made again from then.
    announce_at: Option<Instant>,
    /// Our status is not known yet: from `start` until this sign-on's LoadMyStatus answers.
    loading: bool,
    /// The status job runs (its busy byte, job +1): a change asked meanwhile is dropped (FUN_100f7bd0).
    status_job: bool,
    /// A change asked before the session could send it (not signed on, the loads not in, or our slot unknown): sent
    /// once it can, so a later LoadMyStatus does not undo it.
    pending: Option<u8>,
    /// Set when this session's Friends goes (a logout): status jobs still trying stop, so an old change cannot land
    /// after the next login has loaded its own status.
    stop: StopOnDrop,
    /// What the last zone-in still owes our status, held until the loads are in.
    zone_owed: Option<ZoneStatus>,
    retry_at: Option<Instant>,
    /// A failed handle / character list load is asked again from then.
    tables_at: Option<Instant>,
    /// A handle / character list load is out.
    tables_loading: bool,
    /// The friends' characters asked of xi_search (2.3): the queries out, the chunks left of a round, the next round.
    lookups: Vec<(mpsc::Receiver<anyhow::Result<crate::net::search::Reply>>, Vec<u32>)>,
    round: Vec<Vec<u32>>,
    next_round: Option<Instant>,
    /// The categories the list hides (+0xa4..+0xa8, kept for the session).
    pub hidden: [bool; 5],
    /// One PlayOnline message operation at a time (DAT_10663ac8).
    pub busy: bool,
    /// The help bar's busy strip bit 0x10: set with msgline's commit (FUN_101ff3e0), cleared by its answer
    /// (FUN_101ff8a0 / FUN_101ff920). Retail's other setters are not modelled (MODLOG "The busy strip's 0x10"); not
    /// /befriend's search (FUN_101ffa20) or Ignore (FUN_10200280), which hold only `busy`.
    pub strip: bool,
    /// The local messages, oldest first; messages that came since the count was last seen (netstat).
    pub mail: Vec<Mail>,
    pub mail_new: u32,
    fetching: bool,
}

/// What an s2c 0x00A does to our open status (FUN_100fae40 by LoginState, `net::pol_return`).
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum ZoneStatus {
    /// A plain zone-in (LoginState 1, 2, 0, 6 and up): Away goes back to Online.
    Reset,
    /// The game came back from the PlayOnline viewer (3, 4, 5; FUN_100daf30 state 4): the status is read again.
    Reread,
}

/// A plain zone-in's rule: only Away (1) is reset, to Online (0), and the away flag is cleared with it; Online and
/// Invisible (4) stay and nothing is sent (0x100faf3c / 0x100faf9b: `cmp eax, 1`).
pub fn zone_reset(status: u8) -> Option<u8> {
    (status == 1).then_some(0)
}

/// The open statuses one change of ours asks polcore for, in order. Retail's status job (FUN_100f1a10) sends the
/// object's current block, and FUN_100f7bd0 writes only the pending block: the first request repeats the status in
/// force, its answer (FUN_100f7b00) copies the pending block over and leaves the job busy, and the second request
/// carries the new status. The first changes nothing in polcore, so polcore sends nothing for it (`PolSide::plan`).
pub fn change_sends(current: u8, new: u8) -> [u8; 2] {
    [current, new]
}

/// The arguments of a request of the status job: the block in force with open status `open` (FUN_100f1a10).
pub fn job_args(in_force: [i32; 6], open: u8) -> [i32; 6] {
    let mut a = in_force;
    a[3] = open as i32;
    a
}

/// What `set_status` does with a change (FUN_100f7bd0).
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Change {
    /// Nothing: the status is already in force, or the status job runs.
    Drop,
    /// No PlayOnline session at all (no xi_profile): only kept, as polcore's object would keep it.
    Keep,
    /// The session is not ready to send: held until it is.
    Hold,
    /// ChangeMyStatus with these open statuses, in order.
    Send([u8; 2]),
}

/// The change decision: `ready` is signed on, loaded and our slot known.
pub fn change(open: u8, current: u8, job_busy: bool, has_cred: bool, ready: bool) -> Change {
    match () {
        _ if job_busy => Change::Drop,
        _ if !has_cred => {
            if open == current {
                Change::Drop
            } else {
                Change::Keep
            }
        }
        _ if !ready => Change::Hold,
        _ if open == current => Change::Drop,
        _ => Change::Send(change_sends(current, open)),
    }
}

/// When a zone-in's owed status step may run: at once without xi_profile, else not while the loads are out, nor
/// between them and the sign-on.
pub fn settle_ready(has_cred: bool, loading: bool, loaded: bool, signed_on: bool) -> bool {
    !has_cred || (!loading && (!loaded || signed_on))
}

/// Tries of a request in one run of the job (FUN_100f12a0: 0x32). Kept for the log only: the next run starts at
/// once, so the run boundary changes nothing.
pub const STATUS_TRIES: u32 = 0x32;

/// The job runner (FUN_100f12a0) runs once a game frame, 1/30 s.
pub const RETRY_DELAY: Duration = Duration::from_nanos(1_000_000_000 / 30);

/// A failure worth another try: the connection or its I/O (a timeout, a reset, a stream closed mid-request), not an
/// answer from xi_profile (retail retries only results above -0x800, not refusals). xi_profile closing the link at
/// the open and a failed TLS handshake are its answers (`profile::request`, `tls::HandshakeFailed`), so they are
/// final.
pub fn transient(e: &anyhow::Error) -> bool {
    e.chain().any(|c| c.is::<std::io::Error>())
}

/// `change_once`, decided again at each try as FUN_10019bc0 runs again, and tried again on a connection failure:
/// retail's job runner (FUN_100f12a0) tries a failed request again each frame, up to 0x32 times for results above
/// -0x800, and as the answer (FUN_100f7b00) leaves the job busy the whole job runs again after that, without end.
/// Ours does the same for `transient` errors until `stop` is set (the session's end, which ends retail's jobs too).
fn change_retried(c: &Cred, stop: &AtomicBool, pol: &Mutex<PolSide>, args: [i32; 6]) -> anyhow::Result<()> {
    let mut n = 0u32;
    loop {
        if stop.load(Ordering::Relaxed) {
            anyhow::bail!("the session ended");
        }
        match change_once(c, stop, pol, args) {
            Err(e) if transient(&e) => {
                n += 1;
                log::debug!("friends: ChangeMyStatus try {n} (run {}): {e:#}", n.div_ceil(STATUS_TRIES));
                std::thread::sleep(RETRY_DELAY);
            }
            r => return r,
        }
    }
}

/// One ChangeMyStatus through polcore (`PolSide::plan`): nothing is sent when it changes nothing; a request that
/// goes through sets polcore's status, unless its session ended meanwhile (`stop`: polcore has started over).
fn change_once(c: &Cred, stop: &AtomicBool, pol: &Mutex<PolSide>, args: [i32; 6]) -> anyhow::Result<()> {
    let plan = pol.lock().unwrap().plan(args).map_err(polpro::Local)?;
    let Some((payload, status)) = plan else {
        log::info!("friends: ChangeMyStatus {args:?} changes nothing, not sent");
        return Ok(());
    };
    profile::change_my_status(c, &payload)?;
    answered(stop, pol, status)
}

/// A ChangeMyStatus went through: polcore's status becomes the one it set, unless its session ended meanwhile.
fn answered(stop: &AtomicBool, pol: &Mutex<PolSide>, status: polpro::Status) -> anyhow::Result<()> {
    let mut p = pol.lock().unwrap();
    if stop.load(Ordering::Relaxed) {
        anyhow::bail!("the session ended");
    }
    p.status = status;
    Ok(())
}

impl Friends {
    pub fn new(cred: Option<Cred>) -> Self {
        Self { cred, ..Default::default() }
    }
}

/// A stop flag the status jobs share, set when its owner (Friends) is dropped.
#[derive(Default)]
struct StopOnDrop(Arc<AtomicBool>);

impl Drop for StopOnDrop {
    fn drop(&mut self) {
        self.0.store(true, Ordering::Relaxed);
    }
}

fn spawn(w: &mut World, job: impl FnOnce(&Cred) -> Done + Send + 'static) {
    let Some(c) = w.friends.cred.clone() else { return };
    let (tx, rx) = mpsc::channel();
    std::thread::spawn(move || {
        let _ = tx.send(job(&c));
    });
    w.friends.jobs.push(rx);
}

/// In game: the IRC sign-on; the loads follow it (`load`).
pub fn start(w: &mut World) {
    let f = &mut w.friends;
    if f.started || f.cred.is_none() {
        return;
    }
    (f.started, f.loading) = (true, true);
    f.irc = f.cred.clone().map(profile::irc);
    spawn(w, |c| Done::List(profile::load_friend_list(c)));
    load_tables(w);
}

/// The handle and character lists (LoadHandleNameList, LoadCharacterList); a failed load is asked again 30 s later.
fn load_tables(w: &mut World) {
    w.friends.tables_loading = true;
    spawn(w, |c| {
        Done::Tables((|| Ok(Tables { handles: profile::load_handles(c)?, chars: profile::load_characters(c)? }))())
    });
}

/// Once signed on, our status (the Viewer's order: the chat sign-in, then LoadMyStatus, so the status is the one the
/// session holds; xi_profile answers LoadMyStatus with its default, Online, until the sign-on has read the saved one).
fn load_status(w: &mut World) {
    w.friends.loading = true;
    spawn(w, |c| Done::MyStatus(profile::load_my_status(c)));
}

/// Our character's slot in LoadCharacterList's order.
fn slot(w: &World) -> Option<u8> {
    w.friends.chars.iter().find(|c| c.1 == w.my_id).map(|c| c.0)
}

/// Signed on, loaded, our slot known and the status object built (our character made the active one): a
/// ChangeMyStatus can say which character is in game.
fn ready(w: &World) -> bool {
    w.friends.signed_on && w.friends.loaded && slot(w).is_some() && w.friends.in_force.is_some()
}

/// Our status as it will be: a held change, else the one in force.
fn effective_status(f: &Friends) -> u8 {
    f.pending.unwrap_or(f.my_status)
}

/// FUN_100f7bd0 (/online 0, /away 1, /hide 4, olstat, the zone-in): ChangeMyStatus when it differs and the status
/// job is idle; it is ours once answered. See `change`: without xi_profile the status is only kept; before the
/// session can send (sign-on, the loads, our slot; or within 30 s of an IRC drop) it is held and sent by `poll`.
pub fn set_status(w: &mut World, open: u8) {
    let ready = ready(w);
    let f = &mut w.friends;
    match change(open, f.my_status, f.status_job, f.cred.is_some(), ready) {
        Change::Drop => {}
        Change::Keep => f.my_status = open,
        Change::Hold => f.pending = Some(open),
        Change::Send(sends) => {
            // `ready` holds only with the object built.
            let Some(in_force) = f.in_force else { return };
            (f.status_job, f.pending) = (true, None);
            let (pol, stop) = (f.pol.clone(), f.stop.0.clone());
            spawn(w, move |c| {
                let r = sends.iter().try_for_each(|&o| change_retried(c, &stop, &pol, job_args(in_force, o)));
                Done::Status(r, Some(open))
            });
        }
    }
}

/// s2c 0x00A (FUN_100fae40, connection state 2, so every zone-in): what it owes our status, settled by `poll` once
/// the status is known.
pub fn zone_in(w: &mut World, pol_return: bool) {
    w.friends.zone_owed = Some(if pol_return { ZoneStatus::Reread } else { ZoneStatus::Reset });
    settle(w);
}

/// The zone-in's status step. Retail has polcore's status in hand before the game starts; ours waits for
/// LoadMyStatus and the sign-on.
/// - Reset (0x100faf3c / 0x100faf9b): Away -> FUN_100f7bd0(0) and c2s 0x0DC away off (FUN_100e4710(1, 0)), the
///   packet even when the status job is busy and the change is dropped.
/// - Reread (FUN_100daf30 state 4, after the viewer gave the game back; ours has no viewer, so at once):
///   FUN_100f7aa0 reads polcore's status again into both blocks and sends it once, then FUN_100e4710(1, status ==
///   Away) sets or clears the away flag to match.
fn settle(w: &mut World) {
    let f = &w.friends;
    let Some(owed) = f.zone_owed.filter(|_| settle_ready(f.cred.is_some(), f.loading, f.loaded, f.signed_on)) else {
        return;
    };
    // FUN_100f7aa0 runs on the status job: wait for a change in flight.
    if owed == ZoneStatus::Reread && f.status_job {
        return;
    }
    w.friends.zone_owed = None;
    match owed {
        ZoneStatus::Reset => {
            if let Some(open) = zone_reset(effective_status(&w.friends)) {
                set_status(w, open);
                super::textcmd::config_flag(w, 1, false);
            }
        }
        // No session to read from or send to (no xi_profile, a failed load, our slot unknown): the flag follows the
        // status we have.
        ZoneStatus::Reread if !ready(w) => {
            super::textcmd::config_flag(w, 1, effective_status(&w.friends) == 1);
        }
        // FUN_100f7aa0: the flag and open status in force from polcore's getters (slots 0x330 / 0x338), the fifth
        // field -1, one request. Ours reads LoadMyStatus first for what the viewer would have left in polcore.
        ZoneStatus::Reread => {
            let Some(in_force) = w.friends.in_force else { return };
            let (pol, stop) = (w.friends.pol.clone(), w.friends.stop.0.clone());
            (w.friends.status_job, w.friends.pending) = (true, None);
            spawn(w, move |c| {
                let read = profile::load_my_status(c).and_then(|s| {
                    let args = {
                        let mut p = pol.lock().unwrap();
                        let p = &mut *p;
                        p.status.loaded(&s, &p.tables.slots);
                        let mut a = in_force;
                        (a[2], a[3], a[4]) = (p.status.flag, p.status.presence, -1);
                        a
                    };
                    change_retried(c, &stop, &pol, args)?;
                    Ok(args)
                });
                Done::Then(Box::new(move |w| {
                    w.friends.status_job = false;
                    match read {
                        Ok(a) => {
                            w.friends.in_force = Some(a);
                            w.friends.my_status = a[3] as u8;
                        }
                        Err(e) => log::warn!("friends: reading our status again: {e:#}"),
                    }
                    super::textcmd::config_flag(w, 1, w.friends.my_status == 1);
                }))
            });
        }
    }
}

/// Each frame: the jobs' answers and the IRC session's notices.
pub fn poll(w: &mut World) {
    let mut done = Vec::new();
    w.friends.jobs.retain(|rx| match rx.try_recv() {
        Ok(d) => {
            done.push(d);
            false
        }
        Err(mpsc::TryRecvError::Empty) => true,
        Err(mpsc::TryRecvError::Disconnected) => false,
    });
    for d in done {
        match d {
            Done::List(Ok(list)) => {
                let f = &mut w.friends;
                f.list = list.iter().map(Friend::from_info).collect();
                f.lists = true;
                log::info!("friends: {} entries", f.list.len());
            }
            Done::Tables(Ok(t)) => {
                w.friends.tables_loading = false;
                let f = &mut w.friends;
                {
                    let mut p = f.pol.lock().unwrap();
                    p.tables.load_handles(&t.handles);
                    p.tables.load_characters(&t.chars);
                }
                f.handle = t.handles.first().map(profile::handle_name);
                f.chars = t.chars.iter().map(profile::character_row).collect();
                f.tables = true;
                log::info!("friends: handle {:?}, {} characters", f.handle, f.chars.len());
            }
            Done::MyStatus(Ok(s)) => {
                let f = &mut w.friends;
                {
                    let mut p = f.pol.lock().unwrap();
                    let p = &mut *p;
                    p.status.loaded(&s, &p.tables.slots);
                    // FFXiMain's open status is polcore's presence (slot 0x338).
                    f.my_status = p.status.presence as u8;
                }
                (f.loaded, f.loading) = (true, false);
                log::info!("friends: status {}", f.my_status);
            }
            Done::List(Err(e)) => log::warn!("friends: loading the friend list: {e:#}"),
            Done::Tables(Err(e)) => {
                w.friends.tables_loading = false;
                log::warn!("friends: loading the handle and character lists: {e:#}");
                w.friends.tables_at = Some(Instant::now() + Duration::from_secs(30));
            }
            Done::MyStatus(Err(e)) => {
                w.friends.loading = false;
                log::warn!("friends: loading our status: {e:#}");
            }
            Done::Status(r, open) => {
                if open.is_some() {
                    w.friends.status_job = false;
                }
                match (r, open) {
                    (Ok(()), Some(o)) => {
                        w.friends.my_status = o;
                        if let Some(f) = &mut w.friends.in_force {
                            f[3] = o as i32;
                        }
                    }
                    (Ok(()), None) => {}
                    (Err(e), _) => log::warn!("friends: ChangeMyStatus: {e:#}"),
                }
                log::info!("friends: ChangeMyStatus {open:?} answered, status {}", w.friends.my_status);
            }
            Done::Then(f) => f(w),
        }
    }
    let events: Vec<Irc> = w.friends.irc.as_ref().map(|l| l.events.try_iter().collect()).unwrap_or_default();
    for e in events {
        match e {
            Irc::SignedOn => {
                log::info!("friends: signed on to xi_profile's IRC");
                w.friends.signed_on = true;
                load_status(w);
                load_mail(w);
                fetch(w);
            }
            Irc::Status { account, list_index, status, char_id, age } => {
                notice(w, account, list_index, status, char_id, age)
            }
            Irc::Mail => fetch(w),
            Irc::Closed(why) => {
                log::warn!("friends: IRC closed: {why}");
                let f = &mut w.friends;
                (f.irc, f.signed_on, f.announced, f.tables_at) = (None, false, false, None);
                f.retry_at = Some(Instant::now() + Duration::from_secs(30));
                // The session's status jobs end with it (as at a logout), so none builds a request from the tables
                // cleared below.
                f.stop = StopOnDrop::default();
                // A new sign-on is a new PlayOnline session: polcore starts over (FUN_10018d60: its status, no change
                // gone through yet, and its handle and character tables cleared), the lists and our status are read
                // again and our character made the active one again (xi_profile clears it at each sign-on).
                (f.loaded, f.loading, f.in_force, f.tables) = (false, true, None, false);
                *f.pol.lock().unwrap() = PolSide::default();
            }
        }
    }
    // A dropped session comes back and loads again.
    if w.friends.tables_at.is_some_and(|t| Instant::now() >= t)
        && w.friends.started
        && !w.friends.tables
        && !w.friends.tables_loading
    {
        w.friends.tables_at = None;
        load_tables(w);
    }
    if w.friends.retry_at.is_some_and(|t| Instant::now() >= t) {
        (w.friends.retry_at, w.friends.started) = (None, false);
        start(w);
    }
    refresh(w);
    // A change held until the session could send it.
    if w.friends.pending.is_some() && ready(w) && !w.friends.status_job {
        let p = w.friends.pending.take().unwrap_or_default();
        set_status(w, p);
    }
    settle(w);
    // This character is now the active one, once signed on and in a zone: retail's step 3 call (FUN_100ee2a0 subs
    // 0xe / 0xf: its handle, its entry, flag 0, presence kept, p5 kept, activity 1), then the status object is built
    // from polcore's getters (FUN_100f78d0).
    let f = &w.friends;
    if f.signed_on
        && f.loaded
        && f.tables
        && !f.announced
        && w.zone.is_some()
        && f.announce_at.is_none_or(|t| Instant::now() >= t)
    {
        w.friends.announced = true;
        let Some(index) = slot(w) else {
            log::warn!("friends: our character is not in PlayOnline's character list");
            return;
        };
        let handle = w.friends.pol.lock().unwrap().tables.entries[index as usize & 0x3f].handle;
        let (pol, stop) = (w.friends.pol.clone(), w.friends.stop.0.clone());
        spawn(w, move |c| {
            let r = change_once(c, &stop, &pol, [handle as i32, index as i32, 0, -2, -1, 1]);
            let st = pol.lock().unwrap().status.clone();
            Done::Then(Box::new(move |w| {
                // Answered after its session ended: the next sign-on makes the call again.
                if stop.load(Ordering::Relaxed) {
                    return;
                }
                // Retail goes no further than lobby step 3 when this call fails, so the status object is built only
                // after it went through; ours tries again 30 s later, and status changes wait until then.
                if let Err(e) = r {
                    log::warn!("friends: making our character the active one: {e:#}");
                    (w.friends.announced, w.friends.announce_at) =
                        (false, Some(Instant::now() + Duration::from_secs(30)));
                    return;
                }
                w.friends.announce_at = None;
                w.friends.in_force = Some([st.content, st.service, st.flag, st.presence, -1, st.activity]);
                w.friends.my_status = st.presence as u8;
            }))
        });
    }
}

/// A friend's status notice (3.4.3), as polcore's handler applies it (the function at 0x100160a0.., FUN_1001c9b0):
/// to the entry at its list index, only when newer than the last applied (an equal age is dropped too); the status
/// block only when the notice says it applies, the character only when a primitive follows.
fn notice(w: &mut World, account: u64, index: u8, status: Option<(u8, bool)>, char_id: Option<u32>, age: u64) {
    let Some(f) = w.friends.list.iter_mut().find(|f| !f.black && f.index == index && f.account == account) else {
        log::info!("friends: status of account {account} (index {index}) for no entry");
        return;
    };
    if age <= f.age {
        return;
    }
    f.age = age;
    if let Some((st, playing)) = status {
        (f.status, f.playing) = (st, playing);
    }
    if char_id.is_some() {
        f.last_char = char_id;
    }
    let now = f.last_char.filter(|_| f.playing);
    if f.char_id != now {
        (f.char_id, f.record, f.fresh) = (now, None, false);
    }
    log::info!("friends: {} is {} ({:?})", f.name, f.status, f.char_id);
}

/// FUN_100e63e0 / FUN_100e6650: the playing friends' characters from xi_search, 20 ids a query (wire type 1); a round
/// at login runs its chunks back to back, then one chunk every 60 s. A record sets the friend's (fresh); an id without
/// one loses it.
fn refresh(w: &mut World) {
    let mut answers = Vec::new();
    w.friends.lookups.retain(|(rx, ids)| match rx.try_recv() {
        Ok(r) => {
            answers.push((r, ids.clone()));
            false
        }
        Err(mpsc::TryRecvError::Empty) => true,
        Err(mpsc::TryRecvError::Disconnected) => false,
    });
    for (r, ids) in answers {
        let records = r.map(|r| r.records).unwrap_or_default();
        for f in w.friends.list.iter_mut().filter(|f| f.char_id.is_some_and(|c| ids.contains(&c))) {
            f.record = records.iter().find(|r| Some(r.id) == f.char_id).cloned();
            f.fresh = f.record.is_some();
        }
    }
    if !w.friends.lists || w.zone.is_none() || !w.friends.lookups.is_empty() {
        return;
    }
    let f = &mut w.friends;
    let now = Instant::now();
    if f.round.is_empty() && f.next_round.is_none_or(|t| now >= t) {
        let ids: Vec<u32> =
            f.list.iter().filter(|f| !f.black && !f.pending && f.status != 0).filter_map(|f| f.char_id).collect();
        f.round = ids.chunks(20).map(<[u32]>::to_vec).collect();
        f.next_round = Some(now + Duration::from_secs(60));
    }
    if let Some(ids) = f.round.pop() {
        let rx = crate::net::search::spawn(w.search_addr, crate::net::search::Cond::ids(ids.clone()));
        w.friends.lookups.push((rx, ids));
    }
}

fn now_unix() -> u32 {
    std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).map_or(0, |d| d.as_secs() as u32)
}

/// The local folders: <data>/mail/<account>/unread and read.
fn mail_dir(w: &World, unread: bool) -> Option<std::path::PathBuf> {
    let c = w.friends.cred.as_ref()?;
    Some(crate::config::data_dir().join("mail").join(c.account.to_string()).join(if unread {
        "unread"
    } else {
        "read"
    }))
}

/// The local folders read at sign-on (FUN_100f1f00: the counts), every friend's unread bit set from them.
fn load_mail(w: &mut World) {
    let mut mail = Vec::new();
    for unread in [true, false] {
        let Some(dir) = mail_dir(w, unread) else { return };
        for e in std::fs::read_dir(&dir).into_iter().flatten().flatten() {
            let name = e.file_name().to_string_lossy().into_owned();
            if let (Some(header), Ok(data)) = (profile::MsgHeader::from_name(&name), std::fs::read(e.path())) {
                mail.push(Mail { name, header, data, unread });
            }
        }
    }
    mail.sort_by_key(|m| (m.header.date, m.header.seq));
    w.friends.mail = mail;
    mark_mail(w);
}

/// +0xfc bit 1: an unread message from that friend.
fn mark_mail(w: &mut World) {
    let senders: Vec<u64> = w.friends.mail.iter().filter(|m| m.unread).map(|m| m.header.from).collect();
    for f in &mut w.friends.list {
        f.mail = senders.contains(&f.account);
    }
}

/// The mail engine (2.4): the server's mailbox listed, each message downloaded into the unread folder and deleted on the
/// server; new ones counted (netstat's SE 0039).
pub fn fetch(w: &mut World) {
    if w.friends.fetching {
        return;
    }
    w.friends.fetching = true;
    spawn(w, |c| {
        let got: Vec<(String, Vec<u8>)> = profile::mail_list(c)
            .unwrap_or_default()
            .into_iter()
            .filter_map(|(_, _, name)| {
                let path = format!("{}{name}", profile::MAILBOX);
                let data = profile::read_file(c, &path).ok()?;
                let _ = profile::remove_file(c, &path);
                Some((name, data))
            })
            .collect();
        Done::Then(Box::new(move |w: &mut World| {
            w.friends.fetching = false;
            let dir = mail_dir(w, true);
            for (name, data) in got {
                let Some(header) = profile::MsgHeader::from_name(&name) else { continue };
                if let Some(d) = &dir {
                    let _ = std::fs::create_dir_all(d);
                    let _ = std::fs::write(d.join(&name), &data);
                }
                log::info!("friends: message type {} from {} ({})", header.kind, header.sender, header.from);
                w.friends.mail.push(Mail { name, header, data, unread: true });
                w.friends.mail_new += 1;
            }
            // Oldest first by Date (FUN_102006b0).
            w.friends.mail.sort_by_key(|m| (m.header.date, m.header.seq));
            mark_mail(w);
        }))
    });
}

/// Mark a message read (FUN_100f7590): the file moves to the read folder.
pub fn mark_read(w: &mut World, name: &str) {
    if let (Some(from), Some(to)) = (mail_dir(w, true), mail_dir(w, false)) {
        let _ = std::fs::create_dir_all(&to);
        let _ = std::fs::rename(from.join(name), to.join(name));
    }
    if let Some(m) = w.friends.mail.iter_mut().find(|m| m.name == name) {
        m.unread = false;
    }
    mark_mail(w);
}

/// FUN_100e68a0: -3 ourselves, -2 already in that list, -1 full, 0 room.
pub fn room(w: &World, black: bool, account: u64) -> i8 {
    let f = &w.friends;
    match () {
        _ if f.cred.as_ref().is_some_and(|c| c.account as u64 == account) => -3,
        _ if f.list.iter().any(|e| e.black == black && e.account == account) => -2,
        _ if f.list.iter().filter(|e| e.black == black).count() >= if black { 100 } else { 200 } => -1,
        _ => 0,
    }
}

/// StoreFriendList with every index's order and `entry`; the list loaded again after.
fn store(c: &Cred, list: &[Friend], entry: FriendInfo) -> anyhow::Result<Vec<FriendInfo>> {
    let mut fo = [0u8; 200];
    let mut bo = [0u8; 100];
    for f in list {
        if f.black {
            bo[f.index as usize % 100] = f.order;
        } else {
            fo[f.index as usize % 200] = f.order;
        }
    }
    let i = entry.num as usize;
    if entry.black {
        bo[i % 100] = entry.order
    } else {
        fo[i % 200] = entry.order
    }
    let ans = profile::store_friend_list(c, &fo, &bo, &[entry])?;
    if ans.first().is_some_and(|a| a.result != 0) {
        anyhow::bail!("the store was refused");
    }
    profile::load_friend_list(c)
}

/// A set entry at the first free index of a list.
fn new_entry(w: &World, black: bool, account: u64, name: &str) -> FriendInfo {
    let used: Vec<u8> = w.friends.list.iter().filter(|f| f.black == black).map(|f| f.index).collect();
    let num = (0..if black { 100 } else { 200 }).find(|n| !used.contains(n)).unwrap_or(0);
    FriendInfo {
        op: 1,
        black,
        handle_id: account,
        num,
        order: num,
        pol: account,
        name: name.chars().take(15).collect(),
        ..Default::default()
    }
}

/// The store's answer applied: the entries as loaded, their presence kept.
fn apply_list(w: &mut World, list: Vec<FriendInfo>) {
    let old = std::mem::take(&mut w.friends.list);
    w.friends.list = list
        .iter()
        .map(|i| {
            let mut f = Friend::from_info(i);
            if let Some(o) = old.iter().find(|o| o.black == f.black && o.account == f.account) {
                (f.status, f.char_id, f.age, f.record, f.fresh) =
                    (o.status, o.char_id, o.age, o.record.clone(), o.fresh);
                (f.playing, f.last_char) = (o.playing, o.last_char);
            }
            f
        })
        .collect();
    mark_mail(w);
}

/// What a message operation is (msgline's modes, 2.6).
#[derive(Clone, Debug, PartialEq)]
pub enum Op {
    /// 0 / 1: a plain message.
    Plain { to: u64 },
    /// 2: a friend request: the account, our list name for them, their character's name (the text).
    Request { to: u64, list_name: String, char_name: String },
    /// 5: a reply.
    Reply { to: u64 },
    /// 7: an accept, listing them under `list_name`.
    Accept { to: u64, list_name: String },
    /// 8: a decline.
    Decline { to: u64 },
}

/// polcore's built-in titles (xiloader's English) and FFXiMain's (group 9).
fn title(w: &World, kind: u8, reply: bool) -> String {
    match kind {
        profile::FRIEND_REQUEST => "Let's be friends!".into(),
        profile::FRIEND_ACCEPTED => "Friend registration accepted".into(),
        profile::FRIEND_DECLINED => "Friend registration declined".into(),
        _ => w.battle.as_ref().and_then(|b| b.group9(if reply { 0x1f } else { 0x1c })).unwrap_or_default(),
    }
}

/// msgline's commit (FUN_101ff3e0): the job for the mode, busy until it answers; the success line (group 9) or the
/// failure (10 0x2d, reply / accept 10 0x6f).
pub fn send(w: &mut World, op: Op, text: Vec<u8>) {
    let Some(me) = w.friends.cred.as_ref().map(|c| c.account as u64) else { return };
    (w.friends.busy, w.friends.strip) = (true, true);
    let (to, kind) = match &op {
        Op::Plain { to } | Op::Reply { to } => (*to, profile::PLAIN),
        Op::Request { to, .. } => (*to, profile::FRIEND_REQUEST),
        Op::Accept { to, .. } => (*to, profile::FRIEND_ACCEPTED),
        Op::Decline { to } => (*to, profile::FRIEND_DECLINED),
    };
    let t = crate::dat::font::encode_sjis(&title(w, kind, matches!(op, Op::Reply { .. })));
    // FUN_100f3190's title lengths: 0x10 for a request (FUN_100f3ba0) and an accept (FUN_100f39a0), else 0x32. A
    // request's text is their character's name; an accept's is empty (the typed name only names the entry); a
    // decline's is msgline's buffer, which nothing typed into.
    let body = match &op {
        Op::Request { char_name, .. } => {
            profile::encode_body(&profile::body_title(&t, 0x10), &crate::dat::font::encode_sjis(char_name))
        }
        Op::Accept { .. } => profile::encode_body(&profile::body_title(&t, 0x10), &[]),
        _ => profile::encode_body(&profile::body_title(&t, 0x32), &text),
    };
    // The header's title comes from the data (FUN_100f4ff0), but an accept or a decline answers with the request's
    // own header turned round (polcore table +0x490, FUN_10015050), title kept.
    let (head_title, title_cut) = matches!(op, Op::Accept { .. } | Op::Decline { .. })
        .then(|| {
            let m = w.friends.mail.iter().rev();
            let mut m = m.filter(|m| m.header.from == to && m.header.kind == profile::FRIEND_REQUEST);
            m.next().map(|m| (m.header.title.clone(), m.header.title_cut))
        })
        .flatten()
        .unwrap_or_else(|| profile::header_title(&body));
    let nanos = std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).map_or(0, |d| d.subsec_nanos());
    let header = profile::MsgHeader {
        from: me,
        to,
        sender: w.name.clone(),
        title: head_title,
        title_cut,
        seq: nanos,
        date: now_unix(),
        length: body.len() as u32,
        kind,
        no_reply: false,
    };
    let entry = match &op {
        Op::Request { list_name, .. } | Op::Accept { list_name, .. } => Some(new_entry(w, false, to, list_name)),
        _ => None,
    };
    let list = w.friends.list.clone();
    let (ok_line, fail_line) = match op {
        Op::Plain { .. } => (0x11, 0x2d),
        Op::Request { .. } => (0x14, 0x2d),
        Op::Reply { .. } => (0x12, 0x6f),
        Op::Accept { .. } => (0x15, 0x6f),
        Op::Decline { .. } => (0x16, 0x2d),
    };
    spawn(w, move |c| {
        let r = (|| {
            let stored = match entry {
                Some(e) => Some(store(c, &list, e)?),
                None => None,
            };
            profile::send_message(c, &header, &body)?;
            Ok::<_, anyhow::Error>(stored)
        })();
        Done::Then(Box::new(move |w: &mut World| {
            (w.friends.busy, w.friends.strip) = (false, false);
            match r {
                Ok(stored) => {
                    if let Some(l) = stored {
                        apply_list(w, l);
                    }
                    super::textcmd::log9(w, ok_line);
                }
                Err(e) => {
                    log::warn!("friends: send: {e:#}");
                    line10(w, fail_line, 1);
                }
            }
        }))
    });
}

/// A group 10 line with its %d.
fn line10(w: &mut World, id: usize, code: i32) {
    if let Some(bt) = w.battle.clone() {
        let s = super::select::printf(&bt.group10(id).unwrap_or_default(), &[&code.to_string()]);
        let colour = bt.colour_in(0x79, &w.game);
        w.chat.push_colour(s, colour);
    }
}

/// Ignore (FUN_10200280): a friend's entry copied to the block list, else a new one; 9 0x18, then `then`.
pub fn ignore(w: &mut World, account: u64, name: &str, then: impl FnOnce(&mut World) + Send + 'static) {
    let e = new_entry(w, true, account, name);
    let list = w.friends.list.clone();
    w.friends.busy = true;
    spawn(w, move |c| {
        let r = store(c, &list, e);
        Done::Then(Box::new(move |w: &mut World| {
            w.friends.busy = false;
            match r {
                Ok(l) => {
                    apply_list(w, l);
                    super::textcmd::log9(w, 0x18);
                }
                Err(e) => log::warn!("friends: ignore: {e:#}"),
            }
            then(w);
        }))
    });
}

/// FUN_101ffa20 (/befriend, the search results' Befriend): SearchPolId for the character, then our lists: not listed ->
/// the composer for a request (its name box filled with theirs); listed -> 7 0xa1 from /befriend, a plain message from
/// Befriend; ourselves or not found -> 7 0xa2 *(inf)*. Busy: 7 0x7a.
pub fn befriend(w: &mut World, char_id: u32, char_name: String, from_command: bool) {
    if w.friends.busy {
        super::textcmd::refuse(w, 0x7a);
        return;
    }
    w.friends.busy = true;
    spawn(w, move |c| {
        let r = profile::search_pol_id(c, char_id);
        Done::Then(Box::new(move |w: &mut World| {
            w.friends.busy = false;
            let pol = match r {
                Ok(Some((pol, _))) => pol,
                other => {
                    log::info!("friends: SearchPolId {char_id}: {other:?}");
                    super::textcmd::refuse(w, 0xa2);
                    return;
                }
            };
            match room(w, false, pol) {
                0 => {
                    super::polmail::compose(
                        w,
                        super::polmail::Mode::Request { to: pol, char_name: char_name.clone() },
                        char_name,
                    );
                }
                -2 if !from_command => {
                    super::polmail::compose(w, super::polmail::Mode::Plain { to: pol }, String::new());
                }
                -2 => super::textcmd::refuse(w, 0xa1),
                _ => super::textcmd::refuse(w, 0xa2),
            }
        }))
    });
}

#[cfg(test)]
mod tests {
    use super::*;

    /// A plain zone-in resets only Away (FUN_100fae40: `cmp eax, 1` on FUN_100f7bc0's status); Online and Invisible
    /// stay, and so does anything polcore might hold besides.
    #[test]
    fn zone_in_resets_only_away() {
        assert_eq!(zone_reset(1), Some(0));
        for s in [0, 2, 3, 4, 5] {
            assert_eq!(zone_reset(s), None, "status {s}");
        }
    }

    /// The status job sends the block in force first, then the pending one (FUN_100f1a10 / FUN_100f7b00).
    #[test]
    fn a_change_repeats_the_old_status_first() {
        assert_eq!(change_sends(1, 0), [1, 0]);
        assert_eq!(change_sends(0, 4), [0, 4]);
    }

    /// Through polcore the job's first request (the status in force) changes nothing and is not sent; the second is:
    /// one ChangeMyStatus on the wire per change.
    #[test]
    fn a_change_is_sent_once() {
        let mut p = PolSide::default();
        p.tables.slots[0][2] = 2 << 1 | 1;
        p.status = polpro::Status { content: 0, service: 2, flag: 0, presence: 0, p5: 0, activity: 1, changed: true };
        let in_force = [0, 2, 0, 0, -1, 1];
        let [old, new] = change_sends(0, 1);
        assert_eq!(p.plan(job_args(in_force, old)), Ok(None));
        let (b, st) = p.plan(job_args(in_force, new)).unwrap().unwrap();
        assert_eq!(b[0x10..0x17], [0, 1, 2, 1, 1, 0, 2]);
        assert_eq!(st.presence, 1);
        // Before any change went through (a fresh polcore), the first goes too.
        p.status.changed = false;
        assert!(p.plan(job_args(in_force, old)).unwrap().is_some());
    }

    /// FUN_100f7bd0's gate, and ours around it: a busy job drops, no xi_profile keeps, a session not ready holds
    /// (even the status in force, so a held change can be taken back), a ready one sends only a difference.
    #[test]
    fn change_decision() {
        use Change::*;
        assert_eq!(change(1, 0, true, true, true), Drop);
        assert_eq!(change(1, 0, true, false, false), Drop);
        assert_eq!(change(1, 0, false, false, false), Keep);
        assert_eq!(change(0, 0, false, false, false), Drop);
        assert_eq!(change(1, 0, false, true, false), Hold);
        assert_eq!(change(0, 0, false, true, false), Hold);
        assert_eq!(change(0, 0, false, true, true), Drop);
        assert_eq!(change(4, 1, false, true, true), Send([1, 4]));
    }

    /// The zone-in's step waits while the loads are out and between them and the sign-on; it runs at once without
    /// xi_profile, and after a failed load.
    #[test]
    fn settle_gate() {
        assert!(settle_ready(false, false, false, false));
        assert!(!settle_ready(true, true, false, false));
        assert!(!settle_ready(true, true, false, true));
        assert!(!settle_ready(true, false, true, false));
        assert!(settle_ready(true, false, true, true));
        assert!(settle_ready(true, false, false, false));
    }

    /// Only a connection failure is tried again; an answer from xi_profile is final.
    #[test]
    fn retries_only_io() {
        let io = anyhow::Error::from(std::io::Error::from(std::io::ErrorKind::TimedOut)).context("no answer");
        assert!(transient(&io));
        assert!(!transient(&anyhow::anyhow!("ChangeMyStatus: xi_profile error 0x1")));
        let tls = std::io::Error::from(std::io::ErrorKind::InvalidData);
        assert!(!transient(&anyhow::Error::from(crate::net::tls::HandshakeFailed(tls)).context("connecting")));
    }

    /// An answer that comes after its IRC session ended leaves polcore's reset status alone.
    #[test]
    fn a_late_answer_keeps_the_reset_status() {
        let (stop, pol) = (AtomicBool::new(false), Mutex::new(PolSide::default()));
        let set = polpro::Status { content: 0, service: 3, flag: 0, presence: 0, p5: 0, activity: 1, changed: true };
        answered(&stop, &pol, set.clone()).unwrap();
        assert_eq!(pol.lock().unwrap().status, set);
        *pol.lock().unwrap() = PolSide::default();
        stop.store(true, Ordering::Relaxed);
        assert!(answered(&stop, &pol, set).is_err());
        assert_eq!(pol.lock().unwrap().status, polpro::Status::start());
    }

    /// A stopped session sends nothing more.
    #[test]
    fn dropped_friends_stop_their_jobs() {
        let f = Friends::new(None);
        let stop = f.stop.0.clone();
        assert!(!stop.load(Ordering::Relaxed));
        drop(f);
        assert!(stop.load(Ordering::Relaxed));
    }
}

//! statcom2 row 8, Job Points (MODLOG "Merit Points and Job Points"): merit2ca (the jobs) with levmeri2 and
//! bluehelp, jbpcat over it (page 0 the job's categories, page 1 its gifts), raising through merityn.

use super::World;
use super::game_ui::GameUi;
use super::merits::{PointList, PointRow, bluehelp};
use crate::audio::Ui;
use crate::dat::merits::{Gifts, JobPointTables};
use std::sync::Arc;

const JOBS: &str = "menu    merit2ca";
const PAGE: &str = "menu    jbpcat";
const LEV: &str = "menu    levmeri2";
const WHITE: u32 = 0x8080_8080;
const GREY: u32 = 0x8040_4040;
const BLUE: u32 = 0x8020_60a0;
const CURSOR: u32 = 0x8080_3040;
/// DAT_10371864: the cell columns of merit2ca and jbpcat.
const COLS: [f32; 6] = [0.0, 6.0, 252.0, 253.0, 290.0, 357.0];
const JOB_ROWS: usize = 11;
const PAGE_ROWS: usize = 10;
/// jbpcat's children (+0x58): rows 1..=10, then the page arrows 11 (left) and 12 (right).
const CHILDREN: u16 = 12;

struct Page {
    job: u8,
    l: List,
    gifts: Gifts,
}

/// jbpcat's list and window cursor. Retail keeps one top (+0x1e), the window's cursor child (+0x4c) and each page's
/// index (+0x60 / +0x62, saved by win_jbpcat_v58); ours keeps each page's index and top, the index following the cursor
/// row, and the cursor on a page arrow apart.
#[derive(Clone, Debug, PartialEq)]
struct List {
    /// +0x58: 0 the job point categories, 1 the gifts.
    page: usize,
    /// Per page: the index (the cursor row's while the cursor is on a row) and the top.
    cursor: [usize; 2],
    top: [usize; 2],
    /// The window's cursor on page arrow 11 / 12. No key links to them (links 0xff): only the mouse puts it there.
    arrow: Option<u16>,
    /// The window's +0x61 / +0x62, cleared while the pointer is on an arrow (win_jbpcat_onkey message 9): +0x61 lets
    /// the window move its cursor by the arrow keys (FUN_101196f0), +0x62 lets Enter raise a row and Ctrl / Alt page.
    keys: bool,
    /// The list's last cursor child (+0x24).
    last: u16,
    /// The child under the pointer (mouse +0x52; None = -1, kept while the pointer is hidden) and the pointer shown
    /// (+0x4e).
    ptr: Option<u16>,
    shown: bool,
}

impl List {
    /// FUN_1014c540 at the window's load: page 0 from index 0 (FUN_101f8690(0, 0) -> child 1).
    fn new() -> List {
        List { page: 0, cursor: [0; 2], top: [0; 2], arrow: None, keys: true, last: 1, ptr: None, shown: false }
    }

    /// The window's cursor child (+0x4c).
    fn child(&self) -> u16 {
        self.arrow.unwrap_or_else(|| (self.cursor[self.page] - self.top[self.page] + 1) as u16)
    }

    /// FUN_101f8710: the cursor child's index, past the rows on an arrow (top + 10 / top + 11).
    fn index(&self) -> usize {
        self.top[self.page] + self.child() as usize - 1
    }

    /// FUN_1011a320 / the window's cursor on child `c` (the top as it is).
    fn set_child(&mut self, c: u16) {
        if c > PAGE_ROWS as u16 {
            self.arrow = Some(c);
        } else {
            self.arrow = None;
            self.cursor[self.page] = self.top[self.page] + c as usize - 1;
        }
    }

    /// FUN_1011a240(c, 0): the child held to 1..=12 (+0x58); a new cursor child sends message 9 while the pointer is
    /// shown (mouse +0x48 is taken as past its glide: jbpcat turns no focus).
    fn put(&mut self, c: isize, lens: [usize; 2], snd: &mut Vec<u32>) {
        let old = self.child();
        self.put_from(old, c, lens, snd);
    }

    /// `put` from the window's cursor child `old` (a page load sets the new page's top first).
    fn put_from(&mut self, old: u16, c: isize, lens: [usize; 2], snd: &mut Vec<u32>) {
        let c = c.clamp(1, CHILDREN as isize) as u16;
        self.set_child(c);
        if c != old && self.shown {
            self.hover(lens, snd);
        }
    }

    /// Message 9 (win_jbpcat_onkey 0x1014bf50): the pointer on an arrow (+0x52 = 11 / 12) puts the cursor there
    /// (FUN_1011a320) and clears +0x61 / +0x62; elsewhere it sets them and win_common_onkey_3 runs: a pointer on a row
    /// (1..=count) other than the last cursor plays SE 0001 (FUN_10037b30), then the tail's clamp.
    fn hover(&mut self, lens: [usize; 2], snd: &mut Vec<u32>) {
        if let Some(a) = self.ptr.filter(|&a| a > PAGE_ROWS as u16) {
            self.set_child(a);
            self.keys = false;
            return;
        }
        self.keys = true;
        if let Some(k) = self.ptr.filter(|&k| k >= 1 && k as usize <= lens[self.page])
            && self.last != k
        {
            snd.push(Ui::Cursor as u32);
            self.last = self.child();
        }
        self.clamp(lens, snd);
    }

    /// win_common_onkey_3's tail (0x101f855b) and win_jbpcat_v28 every frame: an index at or past the count puts the
    /// cursor on the count's child (FUN_1011a240(count - top)).
    fn clamp(&mut self, lens: [usize; 2], snd: &mut Vec<u32>) {
        let n = lens[self.page];
        if n <= self.index() {
            self.put(n as isize - self.top[self.page] as isize, lens, snd);
            self.last = self.child();
        }
    }

    /// win_jbpcat_v58 3 / 4, and 5 on an arrow: the page's index kept (FUN_101f8710 into +0x60 / +0x62), the other page
    /// loaded (FUN_1014c540 / FUN_1014c770: FUN_101f8630 sets the last cursor to the window's, then FUN_101f8690(its
    /// index, 0): top = min(index, count - 10), the cursor on index - top + 1).
    fn turn(&mut self, lens: [usize; 2], snd: &mut Vec<u32>) {
        let old = self.child();
        self.cursor[self.page] = self.index();
        self.page = 1 - self.page;
        let (pg, max) = (self.page, lens[self.page].saturating_sub(PAGE_ROWS));
        let s = self.cursor[pg];
        self.top[pg] = s.min(max);
        self.last = old;
        self.put_from(old, (s - self.top[pg] + 1) as isize, lens, snd);
        self.last = self.child();
    }

    /// win_common_onkey_3 message 1 when the window moved no cursor (on an arrow, or +0x61 clear): on row 10 with the
    /// last cursor there the top goes on (SE 0001 unless at its end); a count below the child puts the cursor on the
    /// count's child; else SE 0001 only. Then the tail's clamp.
    fn down(&mut self, lens: [usize; 2], snd: &mut Vec<u32>) {
        let (pg, n, c) = (self.page, lens[self.page], self.child());
        if c == PAGE_ROWS as u16 && self.last == c {
            let max = n.saturating_sub(PAGE_ROWS);
            if self.top[pg] < max {
                self.top[pg] += 1;
                snd.push(Ui::Cursor as u32);
            } else {
                self.top[pg] = max;
            }
            self.set_child(c);
        } else if n < c as usize {
            self.put(n as isize, lens, snd);
        } else {
            snd.push(Ui::Cursor as u32);
        }
        self.last = self.child();
        self.clamp(lens, snd);
    }

    /// win_common_onkey_3 message 2 when the window moved no cursor: on row 1 with the last cursor there the top goes
    /// back (SE 0001 unless at 0); else SE 0001 only. Then the tail's clamp.
    fn up(&mut self, lens: [usize; 2], snd: &mut Vec<u32>) {
        let (pg, c) = (self.page, self.child());
        if c == 1 && self.last == 1 {
            if self.top[pg] > 0 {
                self.top[pg] -= 1;
                snd.push(Ui::Cursor as u32);
            }
            self.set_child(1);
        } else {
            snd.push(Ui::Cursor as u32);
        }
        self.last = self.child();
        self.clamp(lens, snd);
    }

    /// The row list's messages 0x16..0x19 / 0x1b (`scrollbar::rows`) on the window's cursor child, an arrow's too: a
    /// cursor child FUN_1011a240 moved sends message 9.
    fn rows(&mut self, m: super::scrollbar::Msg, lens: [usize; 2], snd: &mut Vec<u32>) -> bool {
        let (pg, old) = (self.page, self.child());
        let mut row = old as usize - 1;
        let moved = super::scrollbar::rows(m, &mut self.top[pg], &mut row, lens[pg], PAGE_ROWS);
        if self.arrow.is_none() || row + 1 != old as usize {
            self.set_child(row as u16 + 1);
        }
        if self.child() != old && self.shown {
            self.hover(lens, snd);
        }
        // 0x16 / 0x17 keep the last cursor; 0x18 / 0x19 / 0x1b and the tail's clamp set it.
        if !matches!(m, super::scrollbar::Msg::Up | super::scrollbar::Msg::Down) || self.child() != old {
            self.last = self.child();
        }
        moved
    }
}

pub struct JpUi {
    cursor: usize,
    top: usize,
    page: Option<Page>,
    /// DAT_1057844c: frames before another raise may ask.
    hold: f32,
    /// The gift names' file (language slot 0x1f).
    gift_names: Option<u32>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Ask {
    pub entry: u16,
}

pub(super) fn tables(w: &mut World) -> Option<Arc<JobPointTables>> {
    if w.jp_tables.is_none() {
        let index = w.index.clone()?;
        w.jp_tables = Some(Arc::new(JobPointTables::load(&index, w.point_files[1])));
    }
    w.jp_tables.clone()
}

/// s2c 0x08D: the entries' levels and next costs; session +0x40db5 bit 0.
pub fn on_list(w: &mut World, recs: &[(u16, u8, u16)]) {
    for &(id, level, next) in recs {
        w.jp_list.insert(id, (level, next));
    }
    w.jp_seen = true;
}

/// statcom2 Enter on row 8 (k 7): merit2ca; c2s 0x0C0 while no 0x08D came this session.
pub fn open(w: &mut World) {
    tables(w);
    if !w.jp_seen {
        let _ = w.net.commands.send(super::net::NetCommand::Packet { id: 0x0C0, body: Vec::new() });
    }
    w.jp_ui = Some(JpUi { cursor: 0, top: 0, page: None, hold: 0.0, gift_names: w.point_files[2] });
}

fn job_level(w: &World, job: u8) -> u8 {
    w.job_info.as_ref().and_then(|j| j.levels.get(job as usize).copied()).unwrap_or(0)
}

/// 0x063 type 5: job `job`'s (capacity, current, spent).
fn points(w: &World, job: u8) -> Option<[u16; 3]> {
    w.jp_jobs.and_then(|j| j.get(job as usize).copied())
}

/// The main job's spent job points (s2c 0x063 sub 5), which gate job-point gift spells (FUN_1022b7d0).
pub(super) fn main_spent(w: &World) -> u16 {
    w.stats.as_ref().and_then(|st| points(w, st.main.0)).map_or(0, |p| p[2])
}

/// jbpcat +0x5e: in the Mog House and not in Monstrosity.
fn can_spend(w: &World) -> bool {
    w.myroom.is_some() && !super::monstrosity::active(w)
}

/// A page-0 row's colours (FUN_1014b0d0): name, level (blue when maxed), cost (None = no cost cell).
fn colours(w: &World, t: &JobPointTables, job: u8, e: u16) -> (u32, u32, Option<u32>) {
    let Some(&(level, next)) = w.jp_list.get(&e).filter(|s| can_spend(w) && (s.0 != 0 || s.1 != 0)) else {
        return (GREY, GREY, None);
    };
    if level >= t.max.get(e as usize).copied().unwrap_or(0) {
        return (WHITE, BLUE, None);
    }
    let have = points(w, job).map_or(0, |p| p[1]);
    (WHITE, WHITE, Some(if have >= next { WHITE } else { GREY }))
}

/// The two pages' counts (+0x20 / +0x5c once loaded).
fn lens(p: &Page, t: &JobPointTables) -> [usize; 2] {
    [t.entries(p.job).len(), p.gifts.needs.iter().filter(|&&n| n != 0).count()]
}

fn page_len(p: &Page, t: &JobPointTables) -> usize {
    lens(p, t)[p.l.page]
}

/// The gifts shown (threshold != 0), in table order.
fn gift_rows(g: &Gifts) -> Vec<usize> {
    (0..g.needs.len()).filter(|&i| g.needs[i] != 0).collect()
}

/// merityn's Yes (LAB_1014b610): c2s 0x0BF {u16 entry}; no new prompt for 30 frames.
pub fn answer(w: &mut World, a: Ask) {
    let mut body = a.entry.to_le_bytes().to_vec();
    body.extend([0, 0]);
    let _ = w.net.commands.send(super::net::NetCommand::Packet { id: 0x0BF, body });
    if let Some(u) = w.jp_ui.as_mut() {
        u.hold = 30.0;
    }
}

/// Group 10 0x74 "Use <n> job point(s) to raise your "<name>" by one level?".
pub fn ask_text(w: &World, a: Ask) -> String {
    let (Some(bt), Some(t)) = (&w.battle, &w.jp_tables) else { return String::new() };
    let Some(raw) = bt.group10_raw(0x74) else { return String::new() };
    let name = crate::dat::font::encode_sjis(&t.name(a.entry));
    let cost = w.jp_list.get(&a.entry).map_or(0, |s| s.1 as i32);
    crate::dat::font::decode_sjis(&crate::dat::msgfmt::format(&raw, &[&name], &[cost], &|_, _, _| None))
}

/// The hold after a raise counts down (frames at 30 a second *(inf: retail counts its frames)*).
pub fn update(w: &mut World, ticks: f32) {
    if let Some(u) = w.jp_ui.as_mut() {
        u.hold = (u.hold - ticks / 2.0).max(0.0);
    }
    frame_clamp(w);
}

fn scroll(cursor: &mut usize, top: &mut usize, to: usize, rows: usize) {
    *cursor = to;
    super::scrollbar::keep_in_view(top, to, rows);
}

/// A key on the open windows; `shown`: the pointer shown (mouse +0x4e; the keys hide it when they hide the cursor).
pub fn key(w: &mut World, key: egui::Key, page_mod: bool, shown: bool) {
    use egui::Key as K;
    // Modern: Tab (the pad's RB / LB) turns the two pages as their tabs.
    let key = if key == K::Tab && w.modern && w.jp_ui.as_ref().is_some_and(|u| u.page.is_some()) {
        K::ArrowRight
    } else {
        key
    };
    let Some(t) = w.jp_tables.clone() else { return };
    let Some(u) = w.jp_ui.as_ref() else { return };
    let jobs = t.jobs.len();
    // jbpcat: FUN_101196f0 (the window: arrow keys move its cursor by the links while +0x61 is set), then
    // win_jbpcat_onkey -> win_common_onkey_3 -> win_jbpcat_v58.
    if let Some(p) = &u.page {
        let (job, lens) = (p.job, lens(p, &t));
        let mut l = p.l.clone();
        l.shown = shown;
        let (pg, n) = (l.page, lens[l.page]);
        let mut snd = Vec::new();
        let mut enter = None;
        match key {
            // win_common_onkey_3 3 / 4 (+0x47 clear): the tail's clamp, then win_jbpcat_v58 3 / 4: the other page,
            // back on its index, +0x61 / +0x62 set, SE 0001.
            K::ArrowLeft | K::ArrowRight if !page_mod => {
                l.clamp(lens, &mut snd);
                l.turn(lens, &mut snd);
                l.keys = true;
                snd.push(Ui::Cursor as u32);
            }
            // With Ctrl / Alt (key 0xa8), the row list's page (win_common_onkey_3 0x19 / 0x18): the top by 10, the
            // screen row kept; nothing while +0x62 is clear.
            K::ArrowLeft | K::ArrowRight => {
                if !l.keys {
                    return;
                }
                let m =
                    if key == K::ArrowRight { super::scrollbar::Msg::PageDown } else { super::scrollbar::Msg::PageUp };
                if l.rows(m, lens, &mut snd) {
                    snd.push(Ui::Cursor as u32);
                }
            }
            // The window moves no cursor from an arrow (links 0xff) or with +0x61 clear: the list's own steps.
            K::ArrowUp if l.arrow.is_some() || !l.keys => l.up(lens, &mut snd),
            K::ArrowDown if l.arrow.is_some() || !l.keys => l.down(lens, &mut snd),
            K::ArrowUp | K::ArrowDown => {
                let cursor = l.cursor[pg];
                let to =
                    if key == K::ArrowDown { (cursor + 1).min(n.saturating_sub(1)) } else { cursor.saturating_sub(1) };
                if to != cursor {
                    let (c, tp) = (&mut l.cursor[pg], &mut l.top[pg]);
                    scroll(c, tp, to, PAGE_ROWS);
                    snd.push(Ui::Cursor as u32);
                }
                l.last = l.child();
            }
            K::Escape => {
                w.sounds.push(Ui::Cancel as u32);
                if let Some(u) = w.jp_ui.as_mut() {
                    u.page = None;
                }
                return;
            }
            // Message 5 with the cursor child as its parameter, after the tail's clamp: on an arrow the other page
            // (win_jbpcat_v58 0xb / 0xc, SE 0001, +0x61 / +0x62 as they are); on a row nothing while +0x62 is clear.
            K::Enter => {
                let on = l.child();
                l.clamp(lens, &mut snd);
                if on > PAGE_ROWS as u16 {
                    l.turn(lens, &mut snd);
                    snd.push(Ui::Cursor as u32);
                } else if l.keys {
                    enter = Some(l.cursor[l.page]);
                }
            }
            _ => {}
        }
        let pg = l.page;
        if let Some(p) = w.jp_ui.as_mut().and_then(|u| u.page.as_mut()) {
            p.l = l;
        }
        w.sounds.extend(snd);
        let Some(cursor) = enter else { return };
        if w.myroom.is_none() {
            super::textcmd::refuse(w, 0xb2);
            w.sounds.push(Ui::Error as u32);
            return;
        }
        let e = t.entries(job).get(cursor).copied();
        let ok = pg == 0 && e.is_some_and(|e| colours(w, &t, job, e).2 == Some(WHITE));
        match e.filter(|_| ok) {
            Some(entry) => {
                w.sounds.push(Ui::Select as u32);
                if w.jp_ui.as_ref().is_some_and(|u| u.hold <= 0.0) {
                    w.confirm = Some((super::Confirm::JobPoint(Ask { entry }), false));
                }
            }
            None => w.sounds.push(Ui::Error as u32),
        }
        return;
    }
    // merit2ca.
    let cursor = u.cursor;
    match key {
        // The row list's page (FUN_101f6c70 3 / 4 with +0x47 set): the top by a page, the screen row kept.
        K::ArrowLeft | K::ArrowRight => {
            let m = if key == K::ArrowRight { super::scrollbar::Msg::PageDown } else { super::scrollbar::Msg::PageUp };
            if let Some(u) = w.jp_ui.as_mut()
                && super::scrollbar::rows_abs(m, &mut u.top, &mut u.cursor, jobs, JOB_ROWS)
            {
                w.sounds.push(Ui::Cursor as u32);
            }
        }
        K::ArrowUp | K::ArrowDown => {
            let to =
                if key == K::ArrowDown { (cursor + 1).min(jobs.saturating_sub(1)) } else { cursor.saturating_sub(1) };
            if to != cursor
                && let Some(u) = w.jp_ui.as_mut()
            {
                scroll(&mut u.cursor, &mut u.top, to, JOB_ROWS);
                w.sounds.push(Ui::Cursor as u32);
            }
        }
        K::Escape => {
            w.sounds.push(Ui::Cancel as u32);
            w.jp_ui = None;
        }
        K::Enter => {
            let Some(&(job, _)) = t.jobs.get(cursor) else { return };
            if job_level(w, job) < 99 {
                w.sounds.push(Ui::Error as u32);
                return;
            }
            w.sounds.push(Ui::Select as u32);
            let Some(index) = w.index.clone() else { return };
            let gifts = Gifts::load(&index, job, u.gift_names);
            if let Some(u) = w.jp_ui.as_mut() {
                u.page = Some(Page { job, l: List::new(), gifts });
            }
        }
        _ => {}
    }
}

/// The focused list's bar (FUN_101f6c70).
pub(super) fn bar(w: &mut World, m: super::scrollbar::Msg) -> bool {
    let Some(t) = w.jp_tables.clone() else { return false };
    let Some(u) = w.jp_ui.as_mut() else { return false };
    match u.page.as_mut() {
        // The bar is the mouse's: the pointer shown.
        Some(p) => {
            let lens = lens(p, &t);
            let mut snd = Vec::new();
            p.l.shown = true;
            let moved = p.l.rows(m, lens, &mut snd);
            w.sounds.extend(snd);
            moved
        }
        None => super::scrollbar::rows_abs(m, &mut u.top, &mut u.cursor, t.jobs.len(), JOB_ROWS),
    }
}

/// win_jbpcat_v28, every frame: the tail's clamp (an index at or past the count, an arrow's too, goes to the count's
/// child; message 9 puts it back on an arrow still under the pointer).
fn frame_clamp(w: &mut World) {
    let Some(t) = w.jp_tables.clone() else { return };
    let Some(p) = w.jp_ui.as_mut().and_then(|u| u.page.as_mut()) else { return };
    let lens = lens(p, &t);
    let mut snd = Vec::new();
    p.l.clamp(lens, &mut snd);
    w.sounds.extend(snd);
}

/// The Job Points windows as the modern layer draws them.
pub(super) struct Look {
    /// The job shown's (capacity, current, spent).
    pub points: Option<[u16; 3]>,
    pub jobs: PointList,
    /// jbpcat: its page (0 the categories, 1 the gifts) and rows.
    pub page: Option<(usize, PointList)>,
}

pub(super) fn look(w: &World) -> Option<Look> {
    let u = w.jp_ui.as_ref()?;
    let t = w.jp_tables.clone()?;
    let free = w.confirm.is_none();
    let job = u.page.as_ref().map(|p| p.job).or_else(|| t.jobs.get(u.cursor).map(|j| j.0)).unwrap_or(0);
    let rows = t
        .jobs
        .iter()
        .map(|&(j, _)| {
            let ok = job_level(w, j) >= 99;
            let level =
                points(w, j).filter(|_| ok).map(|p| (format!("{} P", p[1]), if p[1] >= 500 { BLUE } else { WHITE }));
            PointRow { name: t.name(j as u16), tint: if ok { WHITE } else { GREY }, level, ..Default::default() }
        })
        .collect();
    let help =
        t.jobs.get(u.cursor).map_or((String::new(), String::new()), |&(j, _)| (t.name(j as u16), t.desc(j as u16)));
    let jobs = PointList { rows, cursor: u.cursor, focused: free && u.page.is_none(), help };
    let page = u.page.as_ref().map(|p| {
        let pg = p.l.page;
        let cursor = p.l.cursor[pg];
        let (rows, help) = if pg == 0 {
            let es = t.entries(p.job);
            let rows = es
                .iter()
                .map(|&e| {
                    let (c, lc, cost) = colours(w, &t, p.job, e);
                    let (level, next) = w.jp_list.get(&e).copied().unwrap_or((0, 0));
                    let max = t.max.get(e as usize).copied().unwrap_or(0);
                    PointRow {
                        name: t.name(e),
                        tint: c,
                        level: Some((format!("{level}/{max}"), lc)),
                        cost: cost.map(|cc| (format!("{next} P"), cc)),
                        ..Default::default()
                    }
                })
                .collect();
            (rows, es.get(cursor).map_or((String::new(), String::new()), |&e| (t.name(e), t.desc(e))))
        } else {
            let gifts = gift_rows(&p.gifts);
            let have = points(w, p.job).map(|q| q[2]);
            let rows = gifts
                .iter()
                .map(|&g| {
                    let need = p.gifts.needs[g];
                    let c = if have.is_some_and(|h| h >= need) { WHITE } else { GREY };
                    PointRow {
                        name: p.gifts.name(g),
                        tint: c,
                        level: Some((need.to_string(), c)),
                        ..Default::default()
                    }
                })
                .collect();
            (rows, gifts.get(cursor).map_or((String::new(), String::new()), |&g| (p.gifts.name(g), p.gifts.desc(g))))
        };
        (pg, PointList { rows, cursor, focused: free, help })
    });
    Some(Look { points: points(w, job).filter(|_| (1..32).contains(&job)), jobs, page })
}

/// The mouse's cursor moves: a job (merit2ca), a row of jbpcat's page. True when it moved.
pub(super) fn set_job(w: &mut World, k: usize) -> bool {
    let n = w.jp_tables.as_ref().map_or(0, |t| t.jobs.len());
    let Some(u) = w.jp_ui.as_mut().filter(|u| u.page.is_none() && u.cursor != k && k < n) else { return false };
    scroll(&mut u.cursor, &mut u.top, k, JOB_ROWS);
    true
}

pub(super) fn set_row(w: &mut World, k: usize) -> bool {
    let Some(t) = w.jp_tables.clone() else { return false };
    let Some(p) = w.jp_ui.as_mut().and_then(|u| u.page.as_mut()) else { return false };
    let pg = p.l.page;
    if p.l.cursor[pg] == k || k >= page_len(p, &t) {
        return false;
    }
    let (c, tp) = (&mut p.l.cursor[pg], &mut p.l.top[pg]);
    scroll(c, tp, k, PAGE_ROWS);
    p.l.last = p.l.child();
    true
}

/// jbpcat's page by the mouse (a tab): as Left / Right.
pub(super) fn set_page(w: &mut World, pg: usize) {
    if w.jp_ui.as_ref().and_then(|u| u.page.as_ref()).is_some_and(|p| p.l.page != pg) {
        key(w, egui::Key::ArrowRight, false, false);
    }
}

/// The mouse on the focused list (the generic window's rows, jbpcat's page arrows too): hover moves the cursor, a
/// click on it is Enter, a right click over the window Escape.
pub fn pointer(w: &mut World, gu: &GameUi, p: (f32, f32), clicked: bool, right: bool) {
    // The modern layer draws them and takes their mouse.
    if w.modern {
        return;
    }
    let Some(t) = w.jp_tables.clone() else { return };
    let Some(u) = w.jp_ui.as_ref() else { return };
    let keys = |w: &mut World, _: &GameUi, k: egui::Key| key(w, k, false, true);
    match &u.page {
        // The mouse routine (FUN_101267f0): +0x52 the child under the pointer while it is shown, a new hit child takes
        // the cursor by FUN_1011a240 (its message 9 in `List::hover`: an arrow keeps it with no SE, a row other than
        // the last cursor SE 0001); the release on the cursor child is message 5 (Enter: on an arrow the page turns).
        Some(pg) => {
            let shown = super::winhits::shown(gu);
            let (child, on) = super::winhits::child_under(w, PAGE, p, 1..=CHILDREN, super::winhits::parts(gu, PAGE));
            let lens = lens(pg, &t);
            let mut l = pg.l.clone();
            l.shown = shown;
            if shown {
                l.ptr = child;
            }
            if let Some(pg) = w.jp_ui.as_mut().and_then(|u| u.page.as_mut()) {
                pg.l = l.clone();
            }
            if right && on {
                keys(w, gu, egui::Key::Escape);
                return;
            }
            let Some(id) = child else { return };
            let takes = super::winhits::takes(gu, PAGE, id, on);
            if id != l.child() && takes {
                let mut snd = Vec::new();
                l.put(id as isize, lens, &mut snd);
                if let Some(pg) = w.jp_ui.as_mut().and_then(|u| u.page.as_mut()) {
                    pg.l = l;
                }
                w.sounds.extend(snd);
            } else if clicked && id == l.child() {
                keys(w, gu, egui::Key::Enter);
            }
        }
        None if !t.jobs.is_empty() => {
            let (top, cursor) = (u.top, u.cursor);
            super::winhits::generic(
                w,
                gu,
                JOBS,
                (cursor - top + 1) as u16,
                1..=JOB_ROWS as u16,
                p,
                clicked,
                right,
                |w, id| {
                    if let Some(u) = w.jp_ui.as_mut() {
                        u.cursor = top + id as usize - 1;
                    }
                },
                keys,
            );
        }
        None => {}
    }
}

/// Help slot 1 0xe 0x320 "Select the job for which you wish to expend job points.", title "Categories".
pub fn bars(gu: &GameUi, w: &World, bars: &mut super::hudtop::BarText) {
    if w.jp_ui.is_none() {
        return;
    }
    bars.help[1] = gu.menu_msg(0x320, "");
    bars.help[2] = String::new();
    bars.title[2] = gu.title_msg(0x82).unwrap_or_default();
}

pub fn draw(gu: &mut GameUi, w: &World, sw: f32, sh: f32) -> Vec<[f32; 4]> {
    let mut rects = Vec::new();
    if w.modern {
        return rects;
    }
    let Some(u) = &w.jp_ui else { return rects };
    let Some(t) = w.jp_tables.clone() else { return rects };
    let free = w.confirm.is_none();
    let right = |gu: &mut GameUi, s: &str, x: f32, y: f32, tint: u32| gu.shape_text(s, x, y, tint, true);
    // The job: jbpcat's, else merit2ca's cursor row.
    let job = u.page.as_ref().map(|p| p.job).or_else(|| t.jobs.get(u.cursor).map(|j| j.0)).unwrap_or(0);
    // levmeri2 (FUN_1014a040): capacity / 30000 at (x1 - 8, 0x1c), job points / 500 at (x1 - 8, 0x44).
    if let Some(win) = gu.lobby_window_off(LEV, sw, sh, (0.0, 0.0), None, false, &[]) {
        let [x, y, fw, _] = win.frame;
        if let Some([cap, cur, _]) = points(w, job).filter(|_| (1..32).contains(&job)) {
            right(gu, &format!("{cap}/30000"), x + fw - 8.0, y + 28.0, WHITE);
            right(gu, &format!("{cur}/500 P"), x + fw - 8.0, y + 68.0, if cur >= 500 { BLUE } else { WHITE });
        }
        rects.push(win.frame);
    }
    let (name, desc);
    if let Some(p) = &u.page {
        let pg = p.l.page;
        let n = page_len(p, &t);
        let top = p.l.top[pg];
        // The row on the window's cursor: none while it is on an arrow (11 / 12, the hand there).
        let cursor = if p.l.arrow.is_none() { p.l.cursor[pg] } else { usize::MAX };
        let hide: Vec<u16> = (1..=PAGE_ROWS as u16).filter(|&k| top + k as usize > n).collect();
        let sel = (n > 0 || p.l.arrow.is_some()).then(|| p.l.child());
        let Some(win) = gu.lobby_window_off(PAGE, sw, sh, (0.0, 0.0), sel, free, &hide) else { return rects };
        super::winhits::record(w, PAGE, &win);
        let [x0, y0, fw, fh] = win.frame;
        gu.sprite("menu    windowps", 722 + pg, x0, y0, WHITE);
        if pg == 1 {
            let spent = points(w, p.job).map_or(0, |q| q[2]);
            gu.shape_text(&spent.to_string(), x0 + 258.0, y0 + 4.0, WHITE, false);
        }
        let gifts = gift_rows(&p.gifts);
        for i in top..(top + PAGE_ROWS).min(n) {
            let y = y0 + 17.0 + 17.0 * (i - top) as f32;
            let on = i == cursor;
            if pg == 0 {
                let e = t.entries(p.job)[i];
                let (c, lc, cost) = colours(w, &t, p.job, e);
                let (level, next) = w.jp_list.get(&e).copied().unwrap_or((0, 0));
                gu.text(&t.name(e), x0 + COLS[1], y + 2.0, 0, if on { CURSOR } else { c }.to_le_bytes());
                right(gu, &level.to_string(), x0 + COLS[2], y + 4.0, lc);
                gu.shape_text(
                    &format!("/{}", t.max.get(e as usize).copied().unwrap_or(0)),
                    x0 + COLS[3],
                    y + 4.0,
                    lc,
                    false,
                );
                if let Some(cc) = cost {
                    gu.shape_text(&gu.lobby_msg(0xc, 0x5a, "NEXT"), x0 + COLS[4], y + 4.0, cc, false);
                    right(gu, &format!("{next} P"), x0 + COLS[5], y + 4.0, cc);
                }
            } else {
                let g = gifts[i];
                let need = p.gifts.needs[g];
                let have = points(w, p.job).map(|q| q[2]);
                let c = if have.is_some_and(|h| h >= need) { WHITE } else { GREY };
                gu.text(&p.gifts.name(g), x0 + COLS[1], y + 2.0, 0, if on { CURSOR } else { c }.to_le_bytes());
                right(gu, &need.to_string(), x0 + COLS[4] + 52.0, y + 4.0, c);
            }
        }
        if n > PAGE_ROWS {
            gu.scroll_bar(x0 + fw - 8.0, y0, fh, n, PAGE_ROWS, top);
            if free {
                gu.bar_site(super::scrollbar::Owner::Points, x0 + fw - 8.0, y0, fh, n, PAGE_ROWS, top);
            }
        }
        rects.push(win.frame);
        // win_jbpcat_v50: the help of the entry at FUN_101f8710's index (on an arrow, the one below the shown rows or
        // after it), none past the count.
        let cursor = p.l.index();
        (name, desc) = if pg == 0 {
            t.entries(p.job).get(cursor).map_or((String::new(), String::new()), |&e| (t.name(e), t.desc(e)))
        } else {
            gifts.get(cursor).map_or((String::new(), String::new()), |&g| (p.gifts.name(g), p.gifts.desc(g)))
        };
    } else {
        let hide: Vec<u16> = (1..=JOB_ROWS as u16).filter(|&k| u.top + k as usize > t.jobs.len()).collect();
        let sel = (!t.jobs.is_empty()).then(|| (u.cursor - u.top + 1) as u16);
        let Some(win) = gu.lobby_window_off(JOBS, sw, sh, (0.0, 0.0), sel, free, &hide) else { return rects };
        super::winhits::record(w, JOBS, &win);
        let [x0, y0, fw, fh] = win.frame;
        for (i, &(job, _)) in t.jobs.iter().enumerate().skip(u.top).take(JOB_ROWS) {
            let y = y0 + 4.0 + 16.0 * (i - u.top) as f32;
            let c = if i == u.cursor {
                CURSOR
            } else if job_level(w, job) >= 99 {
                WHITE
            } else {
                GREY
            };
            gu.text(&t.name(job as u16), x0 + COLS[1], y + 2.0, 0, c.to_le_bytes());
        }
        if t.jobs.len() > JOB_ROWS {
            gu.scroll_bar(x0 + fw - 8.0, y0, fh, t.jobs.len(), JOB_ROWS, u.top);
            if free {
                gu.bar_site(super::scrollbar::Owner::Points, x0 + fw - 8.0, y0, fh, t.jobs.len(), JOB_ROWS, u.top);
            }
        }
        rects.push(win.frame);
        (name, desc) = (t.name(job as u16), t.desc(job as u16));
    }
    rects.extend(bluehelp(gu, sw, sh, &name, &desc));
    rects
}

#[cfg(test)]
mod tests {
    use super::*;

    const LENS: [usize; 2] = [25, 5];
    const SE: u32 = Ui::Cursor as u32;

    fn on_arrow(a: u16) -> (List, Vec<u32>) {
        let mut l = List::new();
        let mut snd = Vec::new();
        (l.shown, l.ptr) = (true, Some(a));
        l.put(a as isize, LENS, &mut snd);
        (l, snd)
    }

    /// The hover on an arrow (win_jbpcat_onkey message 9): the cursor there with no SE, +0x61 / +0x62 cleared, the
    /// last cursor kept; back on that row no SE either (win_common_onkey_3 9 tests the last cursor).
    #[test]
    fn hover_takes_an_arrow() {
        let (mut l, snd) = on_arrow(11);
        assert_eq!((l.arrow, l.keys, l.last, l.index(), snd), (Some(11), false, 1, 10, vec![]));
        let mut snd = Vec::new();
        l.ptr = Some(1);
        l.put(1, LENS, &mut snd);
        assert_eq!((l.arrow, l.keys, l.cursor[0], snd.clone()), (None, true, 0, vec![]));
        l.ptr = Some(3);
        l.put(3, LENS, &mut snd);
        assert_eq!((l.cursor[0], l.last, snd), (2, 3, vec![SE]));
    }

    /// A click on the arrow (message 5 with child 0xb): page 0's index kept as FUN_101f8710's (top + 10), page 1
    /// loaded on its index, and message 9 puts the cursor back on the arrow still under the pointer. Back on page 0
    /// that index is past the gifts' count, so the tail's clamp moves it (and message 9 back again): page 1 keeps 10,
    /// page 0 comes back with its top at 10.
    #[test]
    fn click_turns_and_stays_on_the_arrow() {
        let (mut l, _) = on_arrow(11);
        let mut snd = Vec::new();
        l.clamp(LENS, &mut snd);
        l.turn(LENS, &mut snd);
        assert_eq!((l.page, l.cursor, l.arrow, l.keys, l.last), (1, [10, 0], Some(11), false, 11));
        l.clamp(LENS, &mut snd);
        l.turn(LENS, &mut snd);
        assert_eq!((l.page, l.cursor, l.top, l.arrow), (0, [10, 10], [10, 0], Some(11)));
        assert!(snd.is_empty());
    }

    /// Enter by the keys on an arrow with the pointer hidden: the page turns, no message 9, so +0x61 / +0x62 stay
    /// clear and the window moves no cursor: Down plays SE 0001 only, Up on child 1 (the last cursor there) takes the
    /// top back, silent at 0.
    #[test]
    fn keys_after_an_arrow_with_the_pointer_hidden() {
        let (mut l, _) = on_arrow(12);
        l.shown = false;
        let mut snd = Vec::new();
        l.clamp(LENS, &mut snd);
        l.turn(LENS, &mut snd);
        assert_eq!((l.page, l.child(), l.keys), (1, 1, false));
        l.down(LENS, &mut snd);
        l.up(LENS, &mut snd);
        assert_eq!((l.child(), snd), (1, vec![SE]));
    }

    /// Down on an arrow (win_common_onkey_3 1): a count below the child puts the cursor on the count's child, else
    /// SE 0001 and the cursor stays; Up is SE 0001 only.
    #[test]
    fn down_and_up_on_an_arrow() {
        let (mut l, _) = on_arrow(11);
        l.shown = false;
        let mut snd = Vec::new();
        l.down(LENS, &mut snd);
        l.up(LENS, &mut snd);
        assert_eq!((l.arrow, snd), (Some(11), vec![SE, SE]));
        let mut l = List { page: 1, arrow: Some(12), ..List::new() };
        let mut snd = Vec::new();
        l.down(LENS, &mut snd);
        assert_eq!((l.arrow, l.cursor[1], l.last, snd), (None, 4, 5, vec![]));
    }

    /// win_jbpcat_v28's clamp every frame: an arrow past the count goes to the count's child while the pointer is
    /// hidden, and stays while it is shown on the arrow (message 9).
    #[test]
    fn frame_clamp_on_an_arrow() {
        let mut l = List { page: 1, arrow: Some(11), keys: false, ptr: Some(11), ..List::new() };
        let mut snd = Vec::new();
        l.shown = true;
        l.clamp(LENS, &mut snd);
        assert_eq!(l.arrow, Some(11));
        l.shown = false;
        l.clamp(LENS, &mut snd);
        assert_eq!((l.arrow, l.cursor[1], l.last), (None, 4, 5));
    }

    /// The wheel on an arrow moves the top under it; past the count the tail's clamp takes the cursor off.
    #[test]
    fn wheel_on_an_arrow() {
        let (mut l, _) = on_arrow(11);
        let mut snd = Vec::new();
        l.ptr = None;
        for _ in 0..14 {
            l.rows(super::super::scrollbar::Msg::Down, LENS, &mut snd);
        }
        assert_eq!((l.top[0], l.arrow, l.index()), (14, Some(11), 24));
        l.rows(super::super::scrollbar::Msg::Down, LENS, &mut snd);
        assert_eq!((l.top[0], l.arrow, l.cursor[0]), (15, None, 24));
    }
}

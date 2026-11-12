//! The Timers' dock (not retail: the user's Ashita "Timers" add-on): the Timers mod's three edge windows
//! (timers_recasts, timers_buffs, timers_debuffs) drawn as one framed window at a screen edge that slides in and out
//! like a drawer, its sections folding. The rows are the mod's (its `timers` elements); the client only draws them.
//! Also the retail charge rule for /recast.

use super::World;
use super::game_ui::GameUi;
use std::time::Instant;

/// Extras "Timers Position".
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Dock {
    /// Left edge, under the cast bar and the status icons, down to the compass.
    Left,
    /// Right edge, above the party, target and alliance windows, up to the main menu's foot.
    Right,
    /// Top, beside the status icons, under the help bar.
    Top,
}

impl Dock {
    pub fn of(v: u32) -> Self {
        match v {
            1 => Self::Right,
            2 => Self::Top,
            _ => Self::Left,
        }
    }
}

/// The sections in order (a fold bit each).
pub const SECTIONS: [&str; 3] = ["Recasts", "Buffs", "Debuffs"];

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Icon {
    None,
    Status(u16),
    /// A magic / ability list icon (library, sprite) and the record's picture (magic::picture, the modern UI's art).
    List(&'static str, usize, Option<u16>),
}

#[derive(Clone, Debug, PartialEq)]
pub struct Row {
    pub icon: Icon,
    pub name: String,
    /// Seconds left; at or below 0 the predicted end has passed and the server's word is awaited.
    pub left: f32,
    /// The whole length (0: unknown, no gauge).
    pub total: f32,
    /// Recasts fill toward ready; effects drain.
    pub fills: bool,
}

#[derive(Clone, Debug, Default, PartialEq)]
pub struct Group {
    pub name: Option<String>,
    pub rows: Vec<Row>,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Hit {
    Drawer,
    Section(usize),
}

/// The window's hot areas.
pub type Hits = Vec<(Hit, [f32; 4])>;

pub struct Timers {
    /// The drawer is out; `slide` runs 0 (in) .. 1 (out).
    pub out: bool,
    slide: f32,
    /// Folded sections, a bit each.
    pub folded: u8,
    last: Option<Instant>,
    /// Last frame's window and its hot areas.
    rect: Option<[f32; 4]>,
    hits: Hits,
    /// The drawer or a fold changed: the app keeps it in the config.
    pub dirty: bool,
    /// This frame's rows and place, from `prepare` to `draw`.
    frame: Option<Prepared>,
    /// The log window's rect last frame (the right dock stacks on it where it reaches under).
    pub log: Option<[f32; 4]>,
}

struct Prepared {
    secs: [Vec<Group>; 3],
    place: Place,
    pulse: f32,
    small: bool,
}

impl Default for Timers {
    fn default() -> Self {
        Self {
            out: true,
            slide: 1.0,
            folded: 0,
            last: None,
            rect: None,
            hits: Vec::new(),
            dirty: false,
            frame: None,
            log: None,
        }
    }
}

impl Timers {
    pub fn new(out: bool, folded: u8) -> Self {
        Self { out, slide: out as u8 as f32, folded, ..Default::default() }
    }

    pub fn toggle(&mut self) {
        self.out = !self.out;
        self.dirty = true;
    }

    /// Last frame's window (the training regime window stacks on it).
    pub fn rect(&self) -> Option<[f32; 4]> {
        self.rect
    }
}

// ---- Charges (/recast) ----

/// FUN_10222c90: the charge abilities by their moves' kind (9 Quick Draw, 0xf Stratagems, 0x12 Ready): the charges
/// at our job's level and the full recast in seconds; none when the job does not give them.
pub fn charge_rule(w: &World, kind: u8) -> Option<(i32, i32)> {
    let s = w.stats.as_ref()?;
    let ((mj, ml), (sj, sl)) = (s.main, s.sub);
    let (n, full) = match kind {
        9 => {
            let lv = if mj == 0x11 {
                Some(ml)
            } else if sj == 0x11 {
                Some(sl)
            } else {
                None
            };
            (lv.map_or(0, |l| if l < 0x28 { 0 } else { 2 }), 0x78)
        }
        0xf => {
            let lv = if mj == 0x14 {
                Some(ml)
            } else if sj == 0x14 {
                Some(sl)
            } else {
                None
            };
            (lv.map_or(0, |l| (l as i32 - 10) / 0x14 + 1), 0xf0)
        }
        // Retail tests the sub job against 0x14 here, not 9 (kept).
        0x12 => {
            let lv = if mj == 9 {
                Some(ml)
            } else if sj == 0x14 {
                Some(sl)
            } else {
                None
            };
            (lv.map_or(0, |l| if l < 0x19 { 0 } else { 3 }), 0x5a)
        }
        _ => return None,
    };
    Some((n, full))
}

/// FUN_10225120 / FUN_10225220 with charges: the full recast adjusted by the slot's Calc2 (added) and Calc1 (sixtieths
/// off), then the seconds to the next charge (`left` mod a charge's share, the share when it divides) and the charges
/// ready.
pub fn charges(left_secs: i32, n: i32, full: i32, calc: (u8, i16)) -> Option<(i32, i32, i32)> {
    if n <= 0 {
        return None;
    }
    let full = full + calc.1 as i32;
    let total = full as f32 - (calc.0 & 0x3f) as f32 * (1.0 / 60.0) * full as f32;
    let per = (total / n as f32) as i32;
    let total = total as i32;
    if per <= 0 || total / n <= 0 {
        return None;
    }
    let mut next = left_secs % per;
    if next == 0 && left_secs != 0 {
        next = per;
    }
    Some((next, (total - left_secs) / (total / n), per))
}

/// The sections as the mods' timers elements give them (the Timers mod's panels), each row's time left now; a
/// section whose panel is not open is empty.
pub fn sections(w: &World) -> [Vec<Group>; 3] {
    let now = Instant::now();
    w.lua.timers_sections().map(|s| {
        let Some((groups, at)) = s else { return Vec::new() };
        let gone = now.duration_since(at).as_secs_f32();
        groups
            .into_iter()
            .map(|g| Group {
                name: g.name,
                rows: g
                    .rows
                    .into_iter()
                    .map(|r| Row {
                        icon: icon_of(w, r.icon),
                        name: r.name,
                        left: r.left - gone,
                        total: r.total,
                        fills: r.fills,
                    })
                    .collect(),
            })
            .collect()
    })
}

/// A row's icon as the Timers draw it: a status icon, or a spell's or ability's list sprite and picture.
fn icon_of(w: &World, icon: Option<super::lua::window::Icon>) -> Icon {
    use super::lua::window::Icon as W;
    match icon {
        Some(W::Status(id)) => Icon::Status(id),
        Some(i @ (W::Spell(_) | W::Ability(_))) => {
            i.sprite(w).map_or(Icon::None, |(l, k)| Icon::List(l, k, i.picture(w)))
        }
        _ => Icon::None,
    }
}

// ---- The window ----

/// The window's width, its inner margin, a row's pitch and parts.
pub const WIDTH: f32 = 184.0;
const PAD: f32 = 8.0;
const HEAD_H: f32 = 16.0;
const GROUP_H: f32 = 13.0;
const ROW_H: f32 = 24.0;
const GAUGE_X: f32 = 20.0;
/// What stays on screen of a drawer slid in.
const SLIVER: f32 = 22.0;
/// The top dock's drawer slides up into the help bar's foot.
const TOP_EDGE: f32 = 48.0;
/// The slide takes 12 ticks.
const SLIDE_TICKS: f32 = 12.0;
const N: u32 = 0x8080_8080;
const GROUP_TINT: u32 = 0x80a0_a080;
const DIM: u32 = 0x8050_5050;

enum Line<'a> {
    Head(usize, usize),
    Group(&'a str),
    Row(&'a Row),
}

/// The place for the window: x, the top (Left / Top) or the foot (Right), and the most height it may take.
#[derive(Clone, Copy, Debug)]
pub struct Place {
    pub dock: Dock,
    pub x: f32,
    pub y: f32,
    pub max_h: f32,
}

/// Where the window goes beside the retail windows (layout pixels): `icons` the status icon rows, `log_top` the log's
/// top, `foot` the top of the party / target / alliance windows on the right, `mini` the minimap's rect.
pub fn place(dock: Dock, sw: f32, sh: f32, icons: usize, log_top: f32, foot: f32, mini: Option<[f32; 4]>) -> Place {
    // The status icons: 26-px rows from (142, 48).
    let icons_foot = if icons == 0 { 0.0 } else { 48.0 + 2.0 + 26.0 * icons.div_ceil(9) as f32 + 4.0 };
    // The minimap's foot where it stands over the column [x0, x1).
    let mini_foot =
        |x0: f32, x1: f32| mini.filter(|m| m[0] < x1 && m[0] + m[2] > x0).map_or(0.0, |m| m[1] + m[3] + 6.0);
    match dock {
        Dock::Left => {
            // Under the cast bar (16, 94) + 24 and the gauges, past the icons when they reach over; above the compass
            // (its centre 42 above the log, 50 each way); under a minimap there.
            let top =
                122f32.max(if 16.0 + WIDTH > 142.0 { icons_foot } else { 0.0 }).max(mini_foot(16.0, 16.0 + WIDTH));
            Place { dock, x: 16.0, y: top, max_h: (log_top - 96.0 - top).max(0.0) }
        }
        Dock::Right => {
            // Above the party / target / alliance windows, below the main menu (W - 128, 48) 112 x 202 and a minimap.
            let (foot, x) = (foot - 4.0, sw - 16.0 - WIDTH);
            Place { dock, x, y: foot, max_h: (foot - 252f32.max(mini_foot(x, x + WIDTH))).max(0.0) }
        }
        Dock::Top => {
            // Centred, clear of the status icons (to x 376), the main menu column (from W - 128) and a minimap at the
            // top right.
            let right = mini.filter(|m| m[1] < 100.0 && m[0] > sw / 2.0).map_or(sw - 132.0, |m| m[0] - 4.0);
            let x = ((sw - WIDTH) / 2.0).max(380.0).min(right - WIDTH);
            let top = 50.0;
            Place { dock, x, y: top, max_h: ((sh * 0.45).min(log_top - 4.0) - top).max(0.0) }
        }
    }
}

fn clock(left: f32) -> String {
    let s = left.max(0.0).ceil() as u32;
    if s >= 3600 {
        format!("{}:{:02}:{:02}", s / 3600, s % 3600 / 60, s % 60)
    } else {
        format!("{}:{:02}", s / 60, s % 60)
    }
}

/// The lines that fit: rows dropped from the last section up, each section's heading counting what it hides.
fn lines<'a>(secs: &'a [Vec<Group>; 3], folded: u8, max_h: f32) -> (Vec<Line<'a>>, [usize; 3]) {
    let mut hidden = [0usize; 3];
    let mut out: Vec<Line> = Vec::new();
    for (k, groups) in secs.iter().enumerate() {
        let n: usize = groups.iter().map(|g| g.rows.len()).sum();
        if n == 0 {
            continue;
        }
        out.push(Line::Head(k, n));
        if folded & (1 << k) != 0 {
            hidden[k] = n;
            continue;
        }
        for g in groups.iter().filter(|g| !g.rows.is_empty()) {
            if let Some(name) = &g.name {
                out.push(Line::Group(name));
            }
            out.extend(g.rows.iter().map(Line::Row));
        }
    }
    let height = |l: &[Line]| l.iter().map(line_h).sum::<f32>() + 2.0 * PAD;
    while height(&out) > max_h {
        let Some(i) = out.iter().rposition(|l| matches!(l, Line::Row(_))) else { break };
        let k = out[..i].iter().rev().find_map(|l| if let Line::Head(k, _) = l { Some(*k) } else { None }).unwrap_or(0);
        hidden[k] += 1;
        out.remove(i);
        // A group left without rows loses its name.
        if matches!(out.get(i - 1), Some(Line::Group(_))) && !matches!(out.get(i), Some(Line::Row(_))) {
            out.remove(i - 1);
        }
    }
    (out, hidden)
}

fn line_h(l: &Line) -> f32 {
    match l {
        Line::Head(..) => HEAD_H,
        Line::Group(_) => GROUP_H,
        Line::Row(_) => ROW_H,
    }
}

fn ease(s: f32) -> f32 {
    s * s * (3.0 - 2.0 * s)
}

/// Draws the window at `p` with the drawer at `slide` (0 in .. 1 out); returns its rect and hot areas. `pulse` is the
/// retail status icons' 40-tick triangle (their last 15 s dim with it).
pub fn draw_panel(
    gu: &mut GameUi,
    secs: &[Vec<Group>; 3],
    folded: u8,
    p: &Place,
    slide: f32,
    sw: f32,
    pulse: f32,
    small: bool,
) -> Option<([f32; 4], Hits)> {
    let (lines, hidden) = lines(secs, folded, p.max_h);
    if lines.is_empty() {
        return None;
    }
    let h = (lines.iter().map(line_h).sum::<f32>() + 2.0 * PAD).max(48.0);
    let (mut x, mut y) = (p.x, if p.dock == Dock::Right { p.y - h } else { p.y });
    let e = 1.0 - ease(slide.clamp(0.0, 1.0));
    match p.dock {
        Dock::Left => x -= (x + WIDTH - SLIVER) * e,
        Dock::Right => x += (sw - SLIVER - x) * e,
        // Into the help bar's foot (y 48): clipped there, so a faded top bar shows none of it.
        Dock::Top => y -= (y + h - (TOP_EDGE + SLIVER)) * e,
    }
    let mut hits = Vec::new();
    if p.dock == Dock::Top {
        gu.clip(Some([0.0, TOP_EDGE, sw, 1.0e5]));
    }
    gu.frame(x, y, WIDTH, h);
    // The drawer's arrow on its outer edge: pointing out of the screen while out, in while in.
    let inward = slide < 0.5;
    let (ax, ay) = match p.dock {
        Dock::Left => (x + WIDTH - 20.0, y + 6.0),
        Dock::Right => (x + 4.0, y + 6.0),
        // In: on the sliver below the help bar; out: the window's top right, clear of the rows.
        Dock::Top if inward => (x + WIDTH - 18.0, y + h - 18.0),
        Dock::Top => (x + WIDTH - 18.0, y + 4.0),
    };
    match p.dock {
        Dock::Left => gu.sprite("menu    frames", if inward { 26 } else { 25 }, ax, ay, N),
        Dock::Right => gu.sprite("menu    frames", if inward { 25 } else { 26 }, ax, ay, N),
        // The left arrow turned a quarter: down while in, up while out.
        Dock::Top => {
            let (m, dx, dy) =
                if inward { ([0.0, 1.0, -1.0, 0.0], 0.0, 18.0) } else { ([0.0, -1.0, 1.0, 0.0], 9.0, 0.0) };
            gu.sprite_affine("menu    frames", 25, ax + dx, ay + dy, m, N);
            true
        }
    };
    let drawer = match p.dock {
        Dock::Left => [x + WIDTH - SLIVER, y, SLIVER, h.min(48.0)],
        Dock::Right => [x, y, SLIVER, h.min(48.0)],
        Dock::Top if inward => [x, y + h - SLIVER, WIDTH, SLIVER],
        Dock::Top => [x + WIDTH - 24.0, y, 24.0, 24.0],
    };
    hits.push((Hit::Drawer, drawer));
    if slide <= 0.0 {
        gu.clip(None);
        return Some((visible(p.dock, [x, y, WIDTH, h]), hits));
    }
    let inner = WIDTH - 2.0 * PAD;
    let mut ly = y + PAD;
    for (i, l) in lines.iter().enumerate() {
        match l {
            Line::Head(k, n) => {
                let label = if hidden[*k] > 0 {
                    format!("{} ({})", SECTIONS[*k], if hidden[*k] == *n { *n } else { hidden[*k] })
                } else {
                    SECTIONS[*k].to_string()
                };
                // The first heading makes room for the drawer arrow on the right dock.
                let lx = if p.dock == Dock::Right && i == 0 { x + PAD + 16.0 } else { x + PAD };
                gu.label(&label, lx, ly, N, false);
                hits.push((Hit::Section(*k), [x + PAD, ly, inner - 14.0, HEAD_H]));
            }
            Line::Group(name) => {
                gu.shape_fit(name, x + PAD + 2.0, ly + 1.0, inner - 2.0, GROUP_TINT);
            }
            Line::Row(r) => {
                let (rx, right) = (x + PAD, x + WIDTH - PAD);
                let waiting = r.left <= 0.0;
                let c = if !r.fills && r.left > 0.0 && r.left <= 15.0 {
                    (0x40 + (64.0 * pulse) as u32) as f32 / 128.0
                } else {
                    1.0
                };
                match r.icon {
                    Icon::Status(id) => {
                        if let Some(t) = gu.status_icon(small, id) {
                            gu.image_rect(&t, rx, ly + 4.0, 16.0, 16.0, [c, c, c, 1.0]);
                        }
                    }
                    Icon::List(lib, k, _) => {
                        gu.sprite(lib, k, rx, ly + 4.0, N);
                    }
                    Icon::None => {}
                }
                gu.text_fit(
                    &r.name,
                    rx + GAUGE_X,
                    ly - 2.0,
                    inner - GAUGE_X + 16.0,
                    if waiting { 0x8060_6060 } else { N },
                );
                let t = if r.total <= 0.0 { "--:--".to_string() } else { clock(r.left) };
                gu.shape_text(&t, right, ly + 13.0, if waiting { DIM } else { N }, true);
                if r.total > 0.0 {
                    let f = (r.left / r.total).clamp(0.0, 1.0);
                    let f = if r.fills { 1.0 - f } else { f };
                    let (gx, gy) = (rx + GAUGE_X, ly + 14.0);
                    // Frame 38's own fill (97, 64 x 6), coloured by the time left.
                    if f > 0.0 {
                        gu.sprite_coloured("menu    windowps", 97, gx + 4.0, gy + 1.0, f, 1.0, bar_colour(r.left), 8);
                    }
                    gu.sprite("menu    windowps", 38, gx, gy, if f == 0.0 { 0x3050_5050 } else { N });
                }
            }
        }
        ly += line_h(l);
    }
    gu.clip(None);
    Some((visible(p.dock, [x, y, WIDTH, h]), hits))
}

/// The Ashita Timers add-on's bar colours (timer.lua), every bar: green from 30 s left, yellow from 20, orange from 10,
/// else red (vertex colours, 0x80 = 1.0; alpha as fill 97's).
fn bar_colour(left: f32) -> [u8; 4] {
    match left {
        l if l >= 30.0 => [0x00, 0x80, 0x00, 0xa9],
        l if l >= 20.0 => [0x80, 0x80, 0x1a, 0xa9],
        l if l >= 10.0 => [0x80, 0x53, 0x27, 0xa9],
        _ => [0x80, 0x00, 0x00, 0xa9],
    }
}

/// The part of the window on show (the top dock's is clipped at the help bar's foot).
fn visible(dock: Dock, r: [f32; 4]) -> [f32; 4] {
    if dock != Dock::Top || r[1] >= TOP_EDGE {
        return r;
    }
    [r[0], TOP_EDGE, r[2], (r[1] + r[3] - TOP_EDGE).max(0.0)]
}

/// The status icon rows in use (for the left dock's top).
fn icon_count(w: &World) -> usize {
    w.buffs.iter().filter(|&&b| b != 0xffff && b != 0).count()
}

/// Each frame from the HUD, before the party / target windows and the menus (they draw over it). Hidden in events,
/// under the map, and with nothing to show.
pub fn prepare(gu: &GameUi, w: &mut World, sw: f32, sh: f32, foot: f32) {
    let now = Instant::now();
    let dt = w.timers.last.map_or(0.0, |l| now.duration_since(l).as_secs_f32());
    w.timers.last = Some(now);
    let step = dt * 60.0 / SLIDE_TICKS;
    w.timers.slide = if w.timers.out { (w.timers.slide + step).min(1.0) } else { (w.timers.slide - step).max(0.0) };
    w.timers.frame = None;
    if !w.lua.has_timers() || super::event::active(w) || w.map.is_some() || w.cnq.is_some() {
        return;
    }
    let secs = sections(w);
    let place = base_place(gu, w, sw, sh, foot);
    let t = 40.0 - (w.started.elapsed().as_secs_f32() * 60.0) % 40.0;
    let pulse = if t <= 20.0 { t } else { 40.0 - t } * 0.05;
    w.timers.frame = Some(Prepared { secs, place, pulse, small: w.game.get(0x99) & 4 != 0 });
}

/// The dock's place this frame (the training regime window's too).
pub fn base_place(gu: &GameUi, w: &World, sw: f32, sh: f32, foot: f32) -> Place {
    let log_top = gu.log_top.unwrap_or(sh - 16.0 - 120.0);
    // On the right, the log reaching under the window is one more window to stand on.
    let foot = match w.timers.log {
        Some([lx, ly, lw, _]) if lx + lw > sw - 16.0 - WIDTH => foot.min(ly),
        _ => foot,
    };
    place(Dock::of(w.timers_dock), sw, sh, icon_count(w), log_top, foot, w.minimap_rect)
}

/// Draws what `prepare` set up (after the HUD's clear, before the party / target windows and the menus).
pub fn draw(gu: &mut GameUi, t: &mut Timers, sw: f32) {
    t.rect = None;
    t.hits.clear();
    let Some(f) = t.frame.take() else { return };
    if let Some((r, hits)) = draw_panel(gu, &f.secs, t.folded, &f.place, t.slide, sw, f.pulse, f.small) {
        t.rect = Some(r);
        t.hits = hits;
    }
}

/// After every window has drawn: a click on the drawer's arrow or a section heading, unless a window over ours has the
/// spot (`over`). Returns the window's rect for the world-click blockers.
pub fn pointer(
    w: &mut World,
    gu: &GameUi,
    p: Option<(f32, f32)>,
    clicked: bool,
    over: &[[f32; 4]],
) -> Option<[f32; 4]> {
    let r = w.timers.rect?;
    let Some(p) = p else { return Some(r) };
    let inside = |r: &[f32; 4]| p.0 >= r[0] && p.0 < r[0] + r[2] && p.1 >= r[1] && p.1 < r[1] + r[3];
    if !inside(&r) || over.iter().any(inside) {
        return Some(r);
    }
    let Some(&(hit, _)) = w.timers.hits.iter().find(|(_, h)| inside(h)) else { return Some(r) };
    gu.hover.set(true);
    if clicked {
        match hit {
            Hit::Drawer => w.timers.toggle(),
            Hit::Section(k) => {
                w.timers.folded ^= 1 << k;
                w.timers.dirty = true;
            }
        }
        w.sounds.push(crate::audio::Ui::Cursor as u32);
    }
    Some(r)
}

/// topbar-shot's sample (`--window timers:DOCK[:SLIDE[:FOLDED]]`): spell and ability icons from the action table.
pub fn shot(
    gu: &mut GameUi,
    acts: Option<&crate::dat::actions::Actions>,
    spec: &str,
    sw: f32,
    sh: f32,
    mini: Option<[f32; 4]>,
) {
    let f: Vec<&str> = spec.split(':').collect();
    let dock = Dock::of(f.first().and_then(|d| d.parse().ok()).unwrap_or(0));
    let slide = f.get(1).and_then(|d| d.parse().ok()).unwrap_or(1.0);
    let folded = f.get(2).and_then(|d| d.parse().ok()).unwrap_or(0);
    let list = |spell: bool, id: u16| -> Icon {
        let Some(a) = acts else { return Icon::None };
        let r = if spell {
            a.spell(id).map(|r| (r.icon, r.icon_hd))
        } else {
            a.abilities.get(id as usize).map(|r| (r.icon as u16, r.icon_hd))
        };
        r.map_or(Icon::None, |(i, hd)| match hd {
            0xffff => Icon::List("menu    magiconw", i as usize, Some(i)),
            1000.. => Icon::List("menu    magico2w", (hd - 1000) as usize, Some(hd)),
            _ => Icon::List("menu    magiconw", hd as usize, Some(hd)),
        })
    };
    let row = |icon: Icon, name: &str, left: f32, total: f32, fills: bool| Row {
        icon,
        name: name.into(),
        left,
        total,
        fills,
    };
    let secs = [
        vec![Group {
            name: None,
            rows: vec![
                row(list(false, 0x200 + 35), "Provoke", 18.0, 30.0, true),
                row(list(true, 57), "Haste", 9.5, 20.0, true),
                row(list(false, 0x200 + 0x1f), "Berserk", 252.0, 300.0, true),
                row(list(false, 0x200 + 0xd7), "Stratagems (2)", 22.0, 48.0, true),
            ],
        }],
        vec![
            Group {
                name: Some("Lnxcsfix".into()),
                rows: vec![
                    row(Icon::Status(43), "Refresh II", 9.0, 150.0, false),
                    row(Icon::Status(33), "Haste II", 134.0, 180.0, false),
                    row(Icon::Status(40), "Protect IV", 1561.0, 1800.0, false),
                ],
            },
            Group {
                name: Some("Tuxedo".into()),
                rows: vec![
                    row(Icon::Status(116), "Phalanx II", 90.0, 240.0, false),
                    row(Icon::Status(33), "Haste II", 170.0, 180.0, false),
                ],
            },
        ],
        vec![Group {
            name: Some("Valkurm Emperor".into()),
            rows: vec![
                row(Icon::Status(4), "Paralyze", -1.0, 120.0, false),
                row(Icon::Status(134), "Dia II", 70.0, 120.0, false),
                row(Icon::Status(13), "Slow II", 160.0, 180.0, false),
            ],
        }],
    ];
    let log_top = sh - 16.0 - 8.0 * 16.0 - 14.0;
    let foot = sh - 16.0 - 34.0 - 46.0;
    let p = place(dock, sw, sh, 4, log_top, foot, mini);
    draw_panel(gu, &secs, folded, &p, slide, sw, 0.5, false);
}

#[cfg(test)]
mod tests {
    use super::*;

    /// FUN_10225120 / FUN_10225220: Stratagems at SCH 99 (5 charges of 48 s, 240 s whole) with 100 s left: 4 s to
    /// the next, 2 ready; Ready (3 of 30 s) with Calc2 -15 (merits, 5 s each): 25-s shares.
    #[test]
    fn charge_maths() {
        assert_eq!(charges(100, 5, 240, (0, 0)), Some((4, 2, 48)));
        assert_eq!(charges(96, 5, 240, (0, 0)), Some((48, 3, 48)));
        assert_eq!(charges(0, 5, 240, (0, 0)), Some((0, 5, 48)));
        assert_eq!(charges(60, 3, 90, (0, -15)), Some((10, 0, 25)));
        assert_eq!(charges(10, 0, 90, (0, 0)), None);
    }

    #[test]
    fn clocks() {
        assert_eq!(
            [clock(0.2), clock(59.0), clock(61.5), clock(3600.0), clock(3725.0)],
            ["0:01", "0:59", "1:02", "1:00:00", "1:02:05"]
        );
    }

    fn row(left: f32) -> Row {
        Row { icon: Icon::None, name: "x".into(), left, total: 60.0, fills: false }
    }

    /// Rows go from the last section up when the height runs out; the headings count what is hidden.
    #[test]
    fn fitting() {
        let secs = [
            vec![Group { name: None, rows: vec![row(1.0), row(2.0)] }],
            vec![Group { name: Some("A".into()), rows: vec![row(3.0)] }],
            vec![Group { name: Some("Mob".into()), rows: vec![row(4.0), row(5.0)] }],
        ];
        let all = HEAD_H * 3.0 + GROUP_H * 2.0 + ROW_H * 5.0 + 2.0 * PAD;
        let (l, hidden) = lines(&secs, 0, all);
        assert_eq!((l.len(), hidden), (10, [0, 0, 0]));
        let (l, hidden) = lines(&secs, 0, all - ROW_H);
        assert_eq!((l.len(), hidden), (9, [0, 0, 1]));
        // Both of the monster's rows gone: its name goes too.
        let (l, hidden) = lines(&secs, 0, all - 2.0 * ROW_H);
        assert_eq!((l.len(), hidden), (7, [0, 0, 2]));
        // Folded: the heading alone.
        let (l, hidden) = lines(&secs, 0b001, all);
        assert_eq!((l.len(), hidden), (8, [2, 0, 0]));
    }

    #[test]
    fn places() {
        let p = place(Dock::Left, 1280.0, 800.0, 3, 650.0, 600.0, None);
        assert_eq!((p.x, p.y, p.max_h), (16.0, 122.0, 650.0 - 96.0 - 122.0));
        // Three icon rows reach past y 122.
        let p = place(Dock::Left, 1280.0, 800.0, 20, 650.0, 600.0, None);
        assert_eq!(p.y, 48.0 + 2.0 + 78.0 + 4.0);
        let p = place(Dock::Right, 1280.0, 800.0, 0, 650.0, 600.0, None);
        assert_eq!((p.x, p.y, p.max_h), (1280.0 - 16.0 - WIDTH, 596.0, 596.0 - 252.0));
        // A minimap at the top left: the left dock starts under it; at the top right the top dock keeps left of it.
        let mini = crate::client::minimap::place(1, 1, 1280.0, 0, None);
        assert_eq!(place(Dock::Left, 1280.0, 800.0, 0, 650.0, 600.0, Some(mini)).y, mini[1] + mini[3] + 6.0);
        let mini = crate::client::minimap::place(0, 2, 1280.0, 0, None);
        let p = place(Dock::Top, 1280.0, 800.0, 0, 650.0, 600.0, Some(mini));
        assert!(p.x + WIDTH <= mini[0] - 4.0, "{}", p.x);
        // A large one reaches below the main menu: the right dock stops above its foot.
        let p = place(Dock::Right, 1280.0, 800.0, 0, 650.0, 600.0, Some(mini));
        assert_eq!(p.max_h, 596.0 - (mini[1] + mini[3] + 6.0));
    }
}

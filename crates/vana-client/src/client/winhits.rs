//! Windows as drawn this frame (name, frame, children), for the mouse on windows drawn at a runtime place (stacked on
//! the log, beside a row): the generic window handler's hit tests on what was really drawn.

use super::World;
use super::game_ui::{GameUi, LobbyWin};
use crate::audio::Ui;

/// Forgets last frame's windows (before the windows draw).
pub fn clear(w: &World) {
    w.win_hits.borrow_mut().clear();
}

pub fn record(w: &World, name: &str, win: &LobbyWin) {
    record_rects(w, name, win.frame, win.children.clone());
}

/// `record` with the children the draw left out (`hide`: the disabled ones a window draws itself) put back at their
/// layout places, in layout order: retail's hit test (FUN_1011a140) takes a child in state 5 as any other.
pub fn record_all(w: &World, gu: &GameUi, name: &str, frame: [f32; 4], drawn: &[(u16, [f32; 4])]) {
    let layout: Vec<(u16, [f32; 4])> = gu.menu_def(name).map_or(Vec::new(), |m| {
        m.elements.iter().skip(1).map(|e| (e.id, [e.x as f32, e.y as f32, e.w as f32, e.h as f32])).collect()
    });
    record_rects(w, name, frame, put_back(&layout, drawn));
}

/// The layout's children (`layout`, places relative to the window) at the window's place this frame, which the drawn
/// ones give; a drawn child keeps its drawn rect. The drawn ones alone when none of them is in the layout.
fn put_back(layout: &[(u16, [f32; 4])], drawn: &[(u16, [f32; 4])]) -> Vec<(u16, [f32; 4])> {
    let shift =
        drawn.iter().find_map(|(id, r)| layout.iter().find(|e| e.0 == *id).map(|e| (r[0] - e.1[0], r[1] - e.1[1])));
    let Some((dx, dy)) = shift else { return drawn.to_vec() };
    layout
        .iter()
        .map(|&(id, [x, y, w, h])| match drawn.iter().find(|c| c.0 == id) {
            Some(&c) => c,
            None => (id, [dx + x, dy + y, w, h]),
        })
        .collect()
}

pub fn record_rects(w: &World, name: &str, frame: [f32; 4], children: Vec<(u16, [f32; 4])>) {
    w.win_hits.borrow_mut().push((name.to_string(), frame, children));
}

/// Both ends of a box take the mouse (FUN_1011a140: `jg` / `jl` around each bound).
pub(super) fn inside(r: [f32; 4], p: (f32, f32)) -> bool {
    p.0 >= r[0] && p.0 <= r[0] + r[2] && p.1 >= r[1] && p.1 <= r[1] + r[3]
}

/// The child of a window drawn this frame under the pointer (`ids` only), and whether the pointer is on the window.
/// Retail's hit test (FUN_1011a140) takes the first child whose box holds the mouse and whose state is not 0x10. The
/// box is the child's place moved by its header offset (+0xe, +0x10); `part` gives that offset, or None for a child in
/// state 0x10. The frame is not asked (only the hover while the button is held, FUN_1015f360, wants the mouse inside
/// it).
pub fn child_under(
    w: &World,
    name: &str,
    p: (f32, f32),
    ids: std::ops::RangeInclusive<u16>,
    part: impl Fn(u16) -> Option<(f32, f32)>,
) -> (Option<u16>, bool) {
    let hits = w.win_hits.borrow();
    let Some((_, frame, children)) = hits.iter().find(|h| h.0 == name) else { return (None, false) };
    (pick(children, p, ids, part), inside(*frame, p))
}

fn pick(
    children: &[(u16, [f32; 4])],
    p: (f32, f32),
    ids: std::ops::RangeInclusive<u16>,
    part: impl Fn(u16) -> Option<(f32, f32)>,
) -> Option<u16> {
    let under = |&&(id, r): &&(u16, [f32; 4])| {
        ids.contains(&id) && part(id).is_some_and(|(ox, oy)| inside([r[0] + ox, r[1] + oy, r[2], r[3]], p))
    };
    children.iter().find(under).map(|c| c.0)
}

/// Window `name`'s children for `child_under`: each one's header offset, None for the caption retail puts in state
/// 0x10 (`GameUi::caption`).
pub fn parts<'a>(gu: &'a GameUi, name: &'a str) -> impl Fn(u16) -> Option<(f32, f32)> + 'a {
    let caption = gu.caption(name);
    move |id| {
        if caption == Some(id) {
            return None;
        }
        let e = gu.layout(name).and_then(|m| m.elements.iter().skip(1).find(|e| e.id == id));
        Some(e.map_or((0.0, 0.0), |e| (e.offset().0 as f32, e.offset().1 as f32)))
    }
}

/// The hover hits of last frame and of this one (mouse object +0x52): each window and its child under the pointer
/// (ours runs more than one window's mouse in a frame, so one per window), and whether the pointer is hidden.
#[derive(Default)]
pub struct Hover {
    prev: Vec<(String, u16)>,
    now: Vec<(String, u16)>,
    down: bool,
    hidden: bool,
}

impl Hover {
    /// A frame of the mouse routine. With the pointer hidden (+0x4e = 0) the hover block does not run and +0x52 keeps
    /// its value, and the hit test answers -1 (FUN_1011a140 tests +0x4e), so nothing takes the cursor.
    fn frame(&mut self, down: bool, shown: bool) {
        self.hidden = !shown;
        if shown {
            self.prev = std::mem::take(&mut self.now);
        }
        self.down = down;
    }

    /// `takes` without the hand.
    fn take(&mut self, name: &str, child: u16, frame: bool) -> bool {
        if self.hidden {
            return false;
        }
        let fresh = !self.prev.iter().any(|(n, c)| n == name && *c == child);
        self.now.retain(|(n, _)| n != name);
        self.now.push((name.to_string(), child));
        fresh || (self.down && frame)
    }
}

/// A new frame of the mouse routine (FUN_101267f0): this frame's hits start as none (+0x52 = -1 when no window is
/// active or no child is hit) and the last ones are kept to compare with; `down` is the left button, `shown` the
/// pointer (hidden by the keys).
pub fn frame(gu: &GameUi, down: bool, shown: bool) {
    gu.ptr_hover.borrow_mut().frame(down, shown);
}

/// The mouse on child `child` of window `name`, hit under the pointer this frame (`frame`: the pointer is inside the
/// window's frame, FUN_1011a0a0). Whether the child takes the cursor (FUN_1011a240), by retail's two steps:
/// - the hover (tail of FUN_101267f0, 0x10127220): every frame, with no frame test, the cursor moves only when the
///   hit child is not the one of the frame before (`cmp ax, di; je` at 0x10127231); a pointer resting on a child does
///   not take the cursor back from the keys;
/// - the held button (FUN_1015f360, FUN_10124f00(0x3e, 0x8b, 4)): while the left button is down and the pointer is
///   inside the frame, a hit child other than the cursor's takes it every frame.
///
/// The caller moves the cursor only when the child is not already the cursor's. Also the pointer's hand: only over a
/// child not in state 5 (FUN_1011a140's out flag, mouse +0x4c = 1 at 0x10127422; `GameUi::disabled_parts`).
/// Ours keys the hit by window as well as child: ours runs more than one window's mouse in a frame, retail only the
/// active one's.
pub fn takes(gu: &GameUi, name: &str, child: u16, frame: bool) -> bool {
    if !gu.ptr_hover.borrow().hidden && !gu.disabled_part(name, child) {
        gu.hover.set(true);
    }
    gu.ptr_hover.borrow_mut().take(name, child, frame)
}

/// `takes` over a child that shows no hand all the same (the page arrows of the lists that disable themselves under
/// the pointer, FUN_10221cd0).
pub fn takes_no_hand(gu: &GameUi, name: &str, child: u16, frame: bool) -> bool {
    let hand = gu.hover.get();
    let t = takes(gu, name, child, frame);
    gu.hover.set(hand);
    t
}

/// The pointer shown (mouse +0x4e): the keys hide it until it moves.
pub fn shown(gu: &GameUi) -> bool {
    !gu.ptr_hover.borrow().hidden
}

/// The pointer inside window `name`'s frame as laid out (FUN_1011a0a0: both ends in), for `takes`.
pub fn in_frame(gu: &GameUi, name: &str, sw: f32, sh: f32, p: (f32, f32)) -> bool {
    gu.window_rect(name, sw, sh).is_some_and(|r| inside(r, p))
}

/// The pointer on a recorded window.
pub fn over(w: &World, name: &str, p: (f32, f32)) -> bool {
    child_under(w, name, p, 0..=0, |_| None).1
}

/// The left release (FUN_1015f420, state 5 of key function 0x8b) with no child of the focused window under the
/// pointer: retail plays SE 0004 (FUN_10037c50, "0004"). Its test: mouse mode 2 (a window has the focus, the pointer
/// shown), no press on a scroll bar (+0x86), FUN_1011a140 on the focused window (its children only, no frame test)
/// below 1, and mouse +0x87 clear (the caller's `hold`: rem4line / rem4li2 open, FUN_10137e70, or the map's marker
/// cursor, FUN_101fa130).
/// Ours also takes any child a window's mouse took this frame (`takes`) as a hit, for windows drawn where ours keeps no
/// record of them; and a window still opening (no place yet) as none.
pub fn release_off_child(w: &World, gu: &GameUi, sw: f32, sh: f32, p: (f32, f32)) -> bool {
    let Some(name) = gu.focused() else { return false };
    // Mode 2 wants the pointer shown (+0x4e); a child a window's mouse took this frame is a hit.
    let h = gu.ptr_hover.borrow();
    if h.hidden || !h.now.is_empty() {
        return false;
    }
    drop(h);
    let hits = w.win_hits.borrow();
    let child = match hits.iter().find(|h| crate::dat::ui::norm_name(h.0.as_bytes()) == name) {
        Some((n, _, children)) => pick(children, p, 1..=u16::MAX, parts(gu, n)),
        None if gu.window_xy(name, sw, sh).is_none() => return false,
        None => gu.hit(name, sw, sh, p),
    };
    child.is_none()
}

/// The generic handler on the focused window: a child under the pointer takes the cursor by `takes` (SE 0001), a
/// click (the release, FUN_1015f420) on the cursor's child is Enter and on another child nothing, a right click on the window Escape (ours only over it; retail's message 6 takes any right
/// click, FUN_1015def0). `key` is the window's key handler.
pub fn generic(
    w: &mut World,
    gu: &GameUi,
    name: &str,
    cursor: u16,
    ids: std::ops::RangeInclusive<u16>,
    p: (f32, f32),
    clicked: bool,
    right: bool,
    set: impl FnOnce(&mut World, u16),
    key: impl FnOnce(&mut World, &GameUi, egui::Key),
) {
    let (child, on) = child_under(w, name, p, ids, parts(gu, name));
    if right && on {
        key(w, gu, egui::Key::Escape);
        return;
    }
    let Some(id) = child else { return };
    let takes = takes(gu, name, id, on);
    if id != cursor && takes {
        set(w, id);
        w.sounds.push(Ui::Cursor as u32);
    } else if clicked && id == cursor {
        key(w, gu, egui::Key::Enter);
    }
}

/// What a click on an itemctrl / moneyctr child re-sends (message 5, FUN_1013a6c0 / FUN_10140bc0's tables): 1 the
/// confirm (Enter), 2 message 4 (Left: all, or ten times the step), 3 message 3 (Right: 1, or a tenth of the step),
/// 4 message 1 (Down: one step less), 5 message 2 (Up: one step more). The windows' keys do the rest.
pub fn ctr_key(child: u16) -> Option<egui::Key> {
    use egui::Key as K;
    Some(match child {
        1 => K::Enter,
        2 => K::ArrowLeft,
        3 => K::ArrowRight,
        4 => K::ArrowDown,
        5 => K::ArrowUp,
        _ => return None,
    })
}

/// itemctrl / moneyctr's cursor (window +0x4c, which ours keeps nowhere else) and the held button's count (+0xb0).
#[derive(Default)]
pub struct Held {
    /// The cursor child and the open window it belongs to (name, raise stamp): a window opened again starts on child
    /// 1, as every window load does (FUN_101193d0 -> FUN_1011a320(1)).
    cursor: Option<(String, i64, u16)>,
    count: i16,
    /// Real ticks not yet a whole retail frame, and when they were counted.
    acc: f32,
    last: Option<std::time::Instant>,
}

/// The count passes this, and every retail frame after it is a held repeat (`cmp word [esi + 0xb0], 0x1e; jle` at
/// 0x1015f3e3).
const REPEAT_AFTER: i16 = 0x1e;

impl Held {
    /// One frame of ours with the button held on the cursor's child: the retail frames that went by in it, each adding
    /// its ticks to the count (`fiadd` of FUN_10014cf0, at least 1, then `__ftol`) or, once the count is past 0x1e,
    /// sending the held message 0x1d (FUN_1015f2d0(0x1d)) instead. Returns how many held messages to send.
    fn tick(&mut self, now: std::time::Instant, frame: f32) -> u32 {
        // The first held frame is one retail frame.
        let dt = self.last.map_or(frame, |l| (now - l).as_secs_f32().min(0.5) * 60.0);
        self.last = Some(now);
        self.acc += dt;
        let mut sent = 0;
        while self.acc >= frame {
            self.acc -= frame;
            if self.count > REPEAT_AFTER {
                sent += 1;
            } else {
                self.count = (self.count as f32 + frame.max(1.0)) as i16;
            }
        }
        sent
    }

    /// The release (FUN_1015f420) and any frame without the button: the count starts again.
    fn reset(&mut self) {
        (self.count, self.acc, self.last) = (0, 0.0, None);
    }

    /// A held frame off the cursor's child or outside the frame: FUN_1015f360 counts nothing then, so the time does
    /// not count either; the next frame on the child counts as one retail frame. The count itself is kept.
    fn pause(&mut self) {
        (self.acc, self.last) = (0.0, None);
    }
}

/// itemctrl / moneyctr's cursor child (1 until the mouse moves it), for the draw's hand and the keys' Enter.
pub fn ctr_cursor(w: &World, gu: &GameUi, name: &str) -> u16 {
    match (&w.ctr_hold.cursor, gu.open_stamp(name)) {
        (Some((n, at, c)), Some(now)) if n == name && *at == now => *c,
        _ => 1,
    }
}

/// Enter on itemctrl / moneyctr while it has the focus is message 5 on its cursor child (FUN_1013bac0 /
/// FUN_10141fc0 case 5): what that child re-sends (`ctr_key`). Other keys and windows pass unchanged.
pub fn ctr_enter(w: &World, gu: &GameUi, key: egui::Key) -> egui::Key {
    let Some(name) = gu.focused().filter(|n| *n == "menu    itemctrl" || *n == "menu    moneyctr") else { return key };
    match key {
        egui::Key::Enter => ctr_key(ctr_cursor(w, gu, name)).unwrap_or(key),
        _ => key,
    }
}

/// The mouse on itemctrl / moneyctr (`name`, as drawn this frame), by the shared window steps on the window's own
/// cursor (+0x4c):
/// - the release (FUN_1015f420, run before the hover in a frame) on the cursor's child is its message 5 (ctr_key);
/// - a new hit (the hover) or the button held inside the frame (FUN_1015f360) moves the cursor onto the child under the
///   pointer (FUN_1011a240, message 9: SE 0001, the windows keep +0x85 set), so the hand stands on - / + while they
///   are held or pointed at;
/// - held on the cursor's child past 0x1e ticks, message 0x1d every retail frame, which re-sends - / + (FUN_1013bac0
///   / FUN_10141fc0 case 0x1d: children 4 / 5 only).
///
/// A right click on the window is Escape. `key` is the window's key handler.
pub fn ctr(
    w: &mut World,
    gu: &GameUi,
    name: &str,
    p: (f32, f32),
    clicked: bool,
    right: bool,
    mut key: impl FnMut(&mut World, &GameUi, egui::Key),
) {
    let (_, down) = w.ptr_button;
    let (child, on) = child_under(w, name, p, 1..=5, parts(gu, name));
    if right && on {
        key(w, gu, egui::Key::Escape);
        return;
    }
    let cursor = ctr_cursor(w, gu, name);
    let Some(id) = child else {
        if down {
            w.ctr_hold.pause()
        } else {
            w.ctr_hold.reset()
        }
        return;
    };
    let takes = takes(gu, name, id, on);
    if !down {
        if clicked && id == cursor {
            ctr_key(id).inspect(|&k| key(w, gu, k));
        }
        w.ctr_hold.reset();
    }
    let cursor = ctr_cursor(w, gu, name);
    if id != cursor {
        if takes && let Some(stamp) = gu.open_stamp(name) {
            w.ctr_hold.cursor = Some((name.to_string(), stamp, id));
            w.sounds.push(Ui::Cursor as u32);
        }
        // A held frame off the cursor's child counts nothing (FUN_1015f360).
        w.ctr_hold.pause();
        return;
    }
    if !down {
        return;
    }
    if !on {
        w.ctr_hold.pause();
        return;
    }
    let n = w.ctr_hold.tick(std::time::Instant::now(), super::ground::frame_ticks());
    if matches!(id, 4 | 5)
        && let Some(k) = ctr_key(id)
    {
        for _ in 0..n {
            key(w, gu, k);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use egui::Key as K;

    /// FUN_1011a140: the first child in order whose box (moved by its header offset, both ends in) holds the mouse;
    /// a child in state 0x10 (the caption of a window with registry flag 0x200000) is passed over.
    #[test]
    fn hit_test_skips_the_caption() {
        // Three rows 16 apart and a caption drawn across the top edge (place (0, 0), offset (19, -3)).
        let kids = [(1, [16.0, 6.0, 88.0, 16.0]), (2, [16.0, 22.0, 88.0, 16.0]), (3, [0.0, 0.0, 73.0, 9.0])];
        let live = |id: u16| Some(if id == 3 { (19.0, -3.0) } else { (0.0, 0.0) });
        let dead = |id: u16| if id == 3 { None } else { Some((0.0, 0.0)) };
        // A row's bottom edge is still the row; where two boxes meet the first child wins.
        assert_eq!(pick(&kids, (16.0, 6.0), 1..=3, live), Some(1));
        assert_eq!(pick(&kids, (104.0, 22.0), 1..=3, live), Some(1));
        assert_eq!(pick(&kids, (50.0, 23.0), 1..=3, live), Some(2));
        assert_eq!(pick(&kids, (105.0, 23.0), 1..=3, live), None);
        // A live caption is hit where it is drawn, above the frame too, not at its place.
        assert_eq!(pick(&kids, (20.0, -3.0), 1..=3, live), Some(3));
        assert_eq!(pick(&kids, (5.0, 2.0), 1..=3, live), None);
        // In state 0x10 it is not hit at all; nor is a child outside `ids`.
        assert_eq!(pick(&kids, (20.0, -3.0), 1..=3, dead), None);
        assert_eq!(pick(&kids, (30.0, 4.0), 1..=2, live), None);
        assert_eq!(pick(&kids, (30.0, 6.0), 1..=3, dead), Some(1));
    }

    /// FUN_101267f0's hover moves the cursor only onto a child that was not the hit of the frame before; the held
    /// button (FUN_1015f360) moves it every frame, inside the frame only.
    #[test]
    fn hover_takes_only_a_new_hit() {
        let mut h = Hover::default();
        let mut step = |down: bool, hit: Option<u16>, frame: bool| {
            h.frame(down, true);
            hit.map(|c| h.take("menu    w", c, frame))
        };
        // The pointer comes onto child 2, rests there (the keys may move the cursor away meanwhile), moves within it.
        assert_eq!(step(false, Some(2), true), Some(true));
        assert_eq!(step(false, Some(2), true), Some(false));
        assert_eq!(step(false, Some(2), true), Some(false));
        // Onto child 3, then off every child for a frame and back onto 3: a new hit again.
        assert_eq!(step(false, Some(3), true), Some(true));
        assert_eq!(step(false, None, true), None);
        assert_eq!(step(false, Some(3), true), Some(true));
        // The button held on the same child: every frame inside the frame, never outside it.
        assert_eq!(step(true, Some(3), true), Some(true));
        assert_eq!(step(true, Some(3), true), Some(true));
        assert_eq!(step(true, Some(3), false), Some(false));
        // Another window's child of the same number is another hit (ours keys by window too).
        h.frame(false, true);
        assert!(h.take("menu    v", 3, false));
    }

    /// Two windows hit in the same frame keep their own last hits; the hidden pointer takes nothing and keeps them.
    #[test]
    fn hover_per_window_and_hidden() {
        let mut h = Hover::default();
        h.frame(false, true);
        assert!(h.take("menu    a", 1, true) && h.take("menu    b", 4, true));
        h.frame(false, true);
        assert!(!h.take("menu    a", 1, true) && !h.take("menu    b", 4, true));
        // Hidden by a key: no hit, even held, and the memory stays.
        h.frame(true, false);
        assert!(!h.take("menu    a", 2, true));
        h.frame(false, true);
        assert!(!h.take("menu    a", 1, true));
        assert!(h.take("menu    b", 5, true));
    }

    /// Off the cursor's child the count stands still: the time away is not counted.
    #[test]
    fn held_count_pauses_off_the_child() {
        let t0 = std::time::Instant::now();
        let at = |n: u64| t0 + std::time::Duration::from_millis(n * 34);
        let mut h = Held::default();
        assert_eq!(h.tick(at(0), 2.0), 0);
        h.pause();
        // A long time away, then one frame back on: one retail frame counted.
        assert_eq!(h.tick(at(100), 2.0), 0);
        assert_eq!(h.count, 4);
    }

    /// FUN_1015f360: the count takes each retail frame's ticks (2 at 30 frames a second) up to past 0x1e, then
    /// every retail frame is a held message.
    #[test]
    fn held_count_repeats_past_thirty_ticks() {
        let t0 = std::time::Instant::now();
        let mut h = Held::default();
        // A little over 1/30 s a frame: one retail frame (2 ticks) each.
        let at = |n: u64| t0 + std::time::Duration::from_millis(n * 34);
        let sent: Vec<u32> = (0..20).map(|f| h.tick(at(f), 2.0)).collect();
        // Frames 0..=15 count 2..32; the count is past 30 after frame 15, so frame 16 on repeat.
        assert_eq!(&sent[..16], &[0; 16]);
        assert_eq!(&sent[16..], &[1, 1, 1, 1]);
        h.reset();
        assert_eq!(h.tick(at(30), 2.0), 0);
        assert_eq!(h.count, 2);
    }

    /// FUN_1011a140 hits a disabled child (state 5) as any other: the children a window's draw leaves out go back at
    /// their layout places, in layout order.
    #[test]
    fn disabled_children_are_put_back() {
        // iuse: Use (1) disabled and drawn by hand, Drop (2) drawn at (300, 120); the caption (3) drawn too.
        let layout = [(1, [8.0, 6.0, 96.0, 16.0]), (2, [8.0, 22.0, 96.0, 16.0]), (3, [19.0, -3.0, 73.0, 9.0])];
        let drawn = [(2, [300.0, 120.0, 96.0, 16.0]), (3, [311.0, 95.0, 73.0, 9.0])];
        let all = put_back(&layout, &drawn);
        assert_eq!(all, vec![(1, [300.0, 104.0, 96.0, 16.0]), drawn[0], drawn[1]]);
        assert_eq!(pick(&all, (310.0, 110.0), 1..=2, |_| Some((0.0, 0.0))), Some(1));
        // Nothing drawn that the layout knows: the drawn ones as they are.
        assert_eq!(put_back(&layout, &[(9, [0.0; 4])]), vec![(9, [0.0; 4])]);
    }

    #[test]
    fn ctr_children_send_retails_messages() {
        let keys: Vec<_> = (0..=6).map(ctr_key).collect();
        assert_eq!(
            keys,
            [None, Some(K::Enter), Some(K::ArrowLeft), Some(K::ArrowRight), Some(K::ArrowDown), Some(K::ArrowUp), None]
        );
    }
}

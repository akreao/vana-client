//! The skillchain helper's classic window (the skillchain add-ons): the Skillchain mod's edge window (panel
//! "skillchains") drawn in the Timers' dock, under the Timers and the training regime, with its drawer. The mod builds
//! the lines (its `lines` element); the client only draws them.

use super::World;
use super::game_ui::GameUi;
use super::lua::window::{self as model, TextLine};
use super::timers::{Dock, Place};
use std::time::Instant;

#[derive(Default)]
pub struct Tracker {
    /// The window folded (the modern tab, the classic drawer in).
    pub folded: bool,
    /// The classic window: the drawer's slide 0 (in) .. 1 (out), this frame's lines, title and place, last frame's
    /// rects.
    slide: f32,
    last: Option<Instant>,
    frame: Option<(Vec<TextLine>, String, Place)>,
    rect: Option<[f32; 4]>,
    drawer: Option<[f32; 4]>,
}

// ---- The window (classic) ----

/// The window's width (wider than the Timers': skill names) and pitches.
const WIDTH: f32 = 240.0;
const PAD: f32 = 8.0;
const HEAD_H: f32 = 16.0;
const SLIVER: f32 = 22.0;
const SLIDE_TICKS: f32 = 12.0;
const TOP_EDGE: f32 = 48.0;
const N: u32 = 0x8080_8080;
const GOLD: u32 = 0x80a0_a080;

/// The mod's lines and title in its open skillchains window.
fn panel_lines(w: &World) -> Option<(Vec<TextLine>, String)> {
    let win = w.lua.edge_panel("skillchains")?;
    let lines = win.elems.iter().find_map(|e| match e {
        model::Elem::Lines { lines, .. } => Some(lines.clone()),
        _ => None,
    })?;
    let title = if win.title.is_empty() { "Skillchains".to_string() } else { win.title };
    (!lines.is_empty()).then_some((lines, title))
}

/// Each frame from the HUD after the training regime's prepare: hidden in events, under the map, and without lines.
pub fn prepare(w: &mut World, base: Place) {
    let now = Instant::now();
    let dt = w.skillchain.last.map_or(0.0, |l| now.duration_since(l).as_secs_f32());
    w.skillchain.last = Some(now);
    let step = dt * 60.0 / SLIDE_TICKS;
    let out = !w.skillchain.folded;
    w.skillchain.slide = if out { (w.skillchain.slide + step).min(1.0) } else { (w.skillchain.slide - step).max(0.0) };
    w.skillchain.frame = None;
    if w.lua.hides("skillchains") || super::event::active(w) || w.map.is_some() || w.cnq.is_some() {
        return;
    }
    let Some((lines, title)) = panel_lines(w) else { return };
    // Under the Timers and the training regime windows (above them on the right).
    let over = [w.timers.rect(), w.regime.rect()];
    let place = over.iter().flatten().fold(base, |p, r| match p.dock {
        Dock::Right if r[1] - 4.0 < p.y => Place { y: r[1] - 4.0, max_h: (p.max_h - (p.y - r[1] + 4.0)).max(0.0), ..p },
        Dock::Right => p,
        _ if r[1] + r[3] + 4.0 > p.y => {
            Place { y: r[1] + r[3] + 4.0, max_h: (p.max_h - (r[1] + r[3] + 4.0 - p.y)).max(0.0), ..p }
        }
        _ => p,
    });
    w.skillchain.frame = Some((lines, title, place));
}

fn ease(s: f32) -> f32 {
    s * s * (3.0 - 2.0 * s)
}

/// Draws the window at `p` with the drawer at `slide`; its rect and the drawer's.
pub fn draw_panel(
    gu: &mut GameUi,
    lines: &[TextLine],
    title: &str,
    p: &Place,
    slide: f32,
    sw: f32,
) -> Option<([f32; 4], [f32; 4])> {
    let mut shown = 0;
    let mut h = 2.0 * PAD + HEAD_H;
    for l in lines {
        if h + model::classic_line_h(l) > p.max_h {
            break;
        }
        h += model::classic_line_h(l);
        shown += 1;
    }
    if p.max_h < 2.0 * PAD + HEAD_H {
        return None;
    }
    let (mut x, mut y) = (p.x, if p.dock == Dock::Right { p.y - h } else { p.y });
    let e = 1.0 - ease(slide.clamp(0.0, 1.0));
    match p.dock {
        Dock::Left => x -= (x + WIDTH - SLIVER) * e,
        Dock::Right => x += (sw - SLIVER - x) * e,
        Dock::Top => y -= (y + h - (TOP_EDGE + SLIVER)) * e,
    }
    if p.dock == Dock::Top {
        gu.clip(Some([0.0, TOP_EDGE, sw, 1.0e5]));
    }
    gu.frame(x, y, WIDTH, h);
    let inward = slide < 0.5;
    match p.dock {
        Dock::Left => gu.sprite("menu    frames", if inward { 26 } else { 25 }, x + WIDTH - 20.0, y + 6.0, N),
        Dock::Right => gu.sprite("menu    frames", if inward { 25 } else { 26 }, x + 4.0, y + 6.0, N),
        Dock::Top => {
            let (m, dx, dy, ay) = if inward {
                ([0.0, 1.0, -1.0, 0.0], 0.0, 18.0, y + h - 18.0)
            } else {
                ([0.0, -1.0, 1.0, 0.0], 9.0, 0.0, y + 4.0)
            };
            gu.sprite_affine("menu    frames", 25, x + WIDTH - 18.0 + dx, ay + dy, m, N);
            true
        }
    };
    let drawer = match p.dock {
        Dock::Left => [x + WIDTH - SLIVER, y, SLIVER, h.min(48.0)],
        Dock::Right => [x, y, SLIVER, h.min(48.0)],
        Dock::Top if inward => [x, y + h - SLIVER, WIDTH, SLIVER],
        Dock::Top => [x + WIDTH - 24.0, y, 24.0, 24.0],
    };
    let rect = if p.dock == Dock::Top && y < TOP_EDGE {
        [x, TOP_EDGE, WIDTH, (y + h - TOP_EDGE).max(0.0)]
    } else {
        [x, y, WIDTH, h]
    };
    if slide <= 0.0 {
        gu.clip(None);
        return Some((rect, drawer));
    }
    let (rx, inner) = (x + PAD, WIDTH - 2.0 * PAD);
    let hx = if p.dock == Dock::Right { rx + 16.0 } else { rx };
    gu.shape_fit(title, hx, y + PAD + 1.0, inner - 22.0, GOLD);
    let mut ly = y + PAD + HEAD_H;
    for l in lines.iter().take(shown) {
        model::draw_classic_line(gu, l, rx, ly, inner);
        ly += model::classic_line_h(l);
    }
    gu.clip(None);
    Some((rect, drawer))
}

pub fn draw(gu: &mut GameUi, t: &mut Tracker, sw: f32) {
    (t.rect, t.drawer) = (None, None);
    let Some((lines, title, p)) = t.frame.take() else { return };
    if let Some((rect, drawer)) = draw_panel(gu, &lines, &title, &p, t.slide, sw) {
        (t.rect, t.drawer) = (Some(rect), Some(drawer));
    }
}

/// After every window: a click on the drawer's arrow, unless a window over ours has the spot. The window's rect for
/// the world-click blockers.
pub fn pointer(
    w: &mut World,
    gu: &GameUi,
    p: Option<(f32, f32)>,
    clicked: bool,
    over: &[[f32; 4]],
) -> Option<[f32; 4]> {
    let r = w.skillchain.rect?;
    let Some(p) = p else { return Some(r) };
    let inside = |r: &[f32; 4]| p.0 >= r[0] && p.0 < r[0] + r[2] && p.1 >= r[1] && p.1 < r[1] + r[3];
    if !inside(&r) || over.iter().any(inside) || !w.skillchain.drawer.as_ref().is_some_and(inside) {
        return Some(r);
    }
    gu.hover.set(true);
    if clicked {
        w.skillchain.folded = !w.skillchain.folded;
        w.sounds.push(crate::audio::Ui::Cursor as u32);
    }
    Some(r)
}

/// topbar-shot's sample (`--window skillchain:DOCK[:SLIDE]`): a Liquefaction made on a monster, two trusts.
pub fn shot(gu: &mut GameUi, spec: &str, sw: f32, sh: f32, mini: Option<[f32; 4]>) {
    use model::{Span, Tone};
    let f: Vec<&str> = spec.split(':').collect();
    let dock = Dock::of(f.first().and_then(|d| d.parse().ok()).unwrap_or(0));
    let slide = f.get(1).and_then(|d| d.parse().ok()).unwrap_or(1.0);
    let text = |t: &str, tone: Tone| TextLine {
        spans: vec![Span { text: t.into(), tone, ..Default::default() }],
        ..Default::default()
    };
    let lines = vec![
        text("Skillchain: Liquefaction", Tone::Gold),
        text("Burst Fire: 5.5 s", Tone::Text),
        text("Close within 5.5 s", Tone::Muted),
        TextLine { bar: Some((0.7, Tone::Rgba([0x80, 0x70, 0x20, 0xa9]))), ..Default::default() },
        text("Naji: Vorpal Blade", Tone::Text),
        text("  > Fusion (Light, Fire)", Tone::Gold),
        text("You: Fast Blade", Tone::Muted),
        text("  > Scission (Earth)", Tone::Bronze),
        TextLine { heading: true, ..text("Party chains", Tone::Gold) },
        text("Fusion (Light, Fire)", Tone::Gold),
        text("You: Burning Blade", Tone::Text),
        text("  > Naji: Vorpal Blade", Tone::Muted),
        text("Gravitation (Dark, Earth)", Tone::Gold),
        text("You: Red Lotus Blade", Tone::Text),
        text("  > Ayame: Tachi: Kasha", Tone::Muted),
    ];
    let log_top = sh - 16.0 - 8.0 * 16.0 - 14.0;
    let foot = sh - 16.0 - 34.0 - 46.0;
    let p = super::timers::place(dock, sw, sh, 4, log_top, foot, mini);
    draw_panel(gu, &lines, "Skillchains", &p, slide, sw);
}

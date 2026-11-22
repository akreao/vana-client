//! Frame timing for the F12 overlay and FFXI_PERF=1 log lines (not part of retail). With FFXI_PERF set, every
//! frame whose start-to-start gap is over FFXI_PERF_SPIKE ms (default 25) is logged with its spans and the
//! notes of what was loaded or created in it. The F12 overlay also graphs the last FRAMES frames (`graph`).

use std::sync::Mutex;
use std::sync::atomic::{AtomicBool, Ordering};
use std::time::{Duration, Instant};

/// Spans of one redraw, in the order they run.
pub const SPANS: [&str; 6] = ["update", "ui", "acquire", "actors", "encode", "present"];

static ON: AtomicBool = AtomicBool::new(false);
static NOTES: Mutex<Vec<String>> = Mutex::new(Vec::new());

/// Notes work done this frame for the spike log (only collected with FFXI_PERF).
pub fn note(what: impl FnOnce() -> String) {
    if ON.load(Ordering::Relaxed)
        && let Ok(mut n) = NOTES.lock()
    {
        n.push(what());
    }
}

/// Runs `f`; with FFXI_PERF, notes it when it took 0.5 ms or more.
pub fn timed<T>(what: impl FnOnce() -> String, f: impl FnOnce() -> T) -> T {
    if !ON.load(Ordering::Relaxed) {
        return f();
    }
    let t = Instant::now();
    let r = f();
    let ms = t.elapsed().as_secs_f32() * 1000.0;
    if ms >= 0.5 {
        note(|| format!("{} {ms:.1} ms", what()));
    }
    r
}

#[derive(Default, Clone, Copy)]
pub struct Counts {
    pub entities: usize,
    pub actors: usize,
    pub posed: usize,
    pub drawn: usize,
    pub zone_draws: usize,
    pub ui_vertices: usize,
    /// Live / allocated effect tasks, generator instances and elements.
    pub fx: [(usize, usize); 3],
}

/// Frames the F12 graph keeps: 5 s at 60 fps.
pub const FRAMES: usize = 300;

/// The newest `FRAMES` values, oldest first when iterated; a push past the end drops the oldest.
pub struct Ring<T> {
    buf: Vec<T>,
    next: usize,
}

impl<T: Copy> Default for Ring<T> {
    fn default() -> Self {
        Self { buf: Vec::with_capacity(FRAMES), next: 0 }
    }
}

impl<T: Copy> Ring<T> {
    pub fn push(&mut self, v: T) {
        if self.buf.len() < FRAMES {
            self.buf.push(v);
        } else {
            self.buf[self.next] = v;
        }
        self.next = (self.next + 1) % FRAMES;
    }

    pub fn len(&self) -> usize {
        self.buf.len()
    }

    pub fn iter(&self) -> impl Iterator<Item = &T> {
        let split = if self.buf.len() < FRAMES { 0 } else { self.next };
        self.buf[split..].iter().chain(&self.buf[..split])
    }
}

/// One frame for the graph, in ms: the start-to-start gap (what the player sees) and each span.
#[derive(Clone, Copy, Default)]
pub struct Sample {
    pub gap: f32,
    pub spans: [f32; SPANS.len()],
}

/// Gap statistics over the graph's frames, in ms: mean, the 99th percentile ("1% low" as a frame time) and the worst.
#[derive(Debug, Clone, Copy, Default, PartialEq)]
pub struct Stats {
    pub mean: f32,
    pub p99: f32,
    pub worst: f32,
}

/// Stats of the gaps; `scratch` is reused so the overlay allocates nothing a frame once it has grown.
pub fn stats(gaps: impl Iterator<Item = f32>, scratch: &mut Vec<f32>) -> Stats {
    scratch.clear();
    scratch.extend(gaps);
    if scratch.is_empty() {
        return Stats::default();
    }
    let n = scratch.len();
    let mean = scratch.iter().sum::<f32>() / n as f32;
    let k = ((n as f32 * 0.99).ceil() as usize).clamp(1, n) - 1;
    let (_, p99, _) = scratch.select_nth_unstable_by(k, f32::total_cmp);
    let p99 = *p99;
    let worst = scratch.iter().copied().fold(0.0, f32::max);
    Stats { mean, p99, worst }
}

/// The graph's top in ms: the smallest of 20 / 40 / 80 / 160 at or above `ms` (160 caps it). Fed the 99th percentile,
/// so one hitch doesn't flatten the rest; taller bars are cut at the top and drawn red.
pub fn scale(ms: f32) -> f32 {
    [20.0, 40.0, 80.0].into_iter().find(|&t| ms <= t).unwrap_or(160.0)
}

/// Colours of the spans in SPANS order (acquire is mostly the wait for the swap chain).
const SPAN_COLOURS: [egui::Color32; SPANS.len()] = [
    egui::Color32::from_rgb(90, 150, 255),
    egui::Color32::from_rgb(190, 120, 255),
    egui::Color32::from_rgb(120, 120, 120),
    egui::Color32::from_rgb(90, 210, 120),
    egui::Color32::from_rgb(255, 170, 60),
    egui::Color32::from_rgb(240, 90, 90),
];
const GAP_COLOUR: egui::Color32 = egui::Color32::from_rgb(70, 70, 80);
const GPU_COLOUR: egui::Color32 = egui::Color32::from_rgb(255, 230, 80);

/// Draws the frame-time graph into `ui`: per frame a grey bar for the gap with the CPU spans stacked on it, the GPU
/// time (when the device has timestamps) as a yellow line, 60 / 30 fps guides, then a legend and the gap stats.
pub fn graph(ui: &mut egui::Ui, perf: &mut Perf, gpu: Option<&Ring<f32>>) {
    const W: f32 = 360.0;
    const H: f32 = 90.0;
    let st = stats(perf.hist.iter().map(|s| s.gap), &mut perf.scratch);
    let top = scale(st.p99);
    let (rect, _) = ui.allocate_exact_size(egui::vec2(W, H), egui::Sense::hover());
    let painter = ui.painter_at(rect);
    painter.rect_filled(rect, 0.0, egui::Color32::from_black_alpha(120));
    let bar = W / FRAMES as f32;
    let y = |ms: f32| rect.bottom() - (ms / top).min(1.0) * H;
    // Newest frame at the right edge.
    let x0 = rect.right() - perf.hist.len() as f32 * bar;
    let mut mesh = egui::Mesh::default();
    for (i, s) in perf.hist.iter().enumerate() {
        let x = x0 + i as f32 * bar;
        let r = |a: f32, b: f32| egui::Rect::from_min_max(egui::pos2(x, y(b)), egui::pos2(x + bar, y(a)));
        mesh.add_colored_rect(r(0.0, s.gap), if s.gap > top { egui::Color32::RED } else { GAP_COLOUR });
        let mut at = 0.0;
        for (k, v) in s.spans.iter().enumerate() {
            if *v > 0.0 && at < top {
                mesh.add_colored_rect(r(at, at + v), SPAN_COLOURS[k]);
            }
            at += v;
        }
    }
    painter.add(egui::Shape::mesh(mesh));
    for (ms, label) in [(1000.0 / 60.0, "60"), (1000.0 / 30.0, "30")] {
        if ms < top {
            let yy = y(ms);
            painter.line_segment(
                [egui::pos2(rect.left(), yy), egui::pos2(rect.right(), yy)],
                egui::Stroke::new(1.0, egui::Color32::from_white_alpha(70)),
            );
            painter.text(
                egui::pos2(rect.left() + 2.0, yy),
                egui::Align2::LEFT_BOTTOM,
                label,
                egui::FontId::monospace(10.0),
                egui::Color32::from_white_alpha(160),
            );
        }
    }
    if let Some(g) = gpu.filter(|g| g.len() > 1) {
        let gx0 = rect.right() - g.len() as f32 * bar;
        let pts: Vec<egui::Pos2> =
            g.iter().enumerate().map(|(i, v)| egui::pos2(gx0 + (i as f32 + 0.5) * bar, y(*v))).collect();
        painter.add(egui::Shape::line(pts, egui::Stroke::new(1.0, GPU_COLOUR)));
    }
    painter.text(
        rect.right_top() + egui::vec2(-2.0, 1.0),
        egui::Align2::RIGHT_TOP,
        format!("{top:.0} ms"),
        egui::FontId::monospace(10.0),
        egui::Color32::from_white_alpha(160),
    );
    ui.horizontal(|ui| {
        ui.spacing_mut().item_spacing.x = 8.0;
        let key = |ui: &mut egui::Ui, c: egui::Color32, t: &str| {
            ui.label(egui::RichText::new(format!("■ {t}")).monospace().size(11.0).color(c));
        };
        key(ui, GAP_COLOUR.gamma_multiply(2.0), "frame");
        for (k, s) in SPANS.iter().enumerate() {
            key(ui, SPAN_COLOURS[k], s);
        }
        if gpu.is_some() {
            key(ui, GPU_COLOUR, "gpu");
        }
    });
    ui.label(
        egui::RichText::new(format!(
            "frame {:.1} ms avg  {:.1} 1% low  {:.1} worst  (last {} frames)",
            st.mean,
            st.p99,
            st.worst,
            perf.hist.len()
        ))
        .monospace()
        .color(egui::Color32::WHITE),
    );
}

pub struct Perf {
    window: Instant,
    frames: u32,
    busy: Duration,
    worst: Duration,
    spans: [Duration; SPANS.len()],
    /// Previous frame start, for the gap; spikes and the worst gap in this window.
    last_start: Option<Instant>,
    spike: Duration,
    spikes: u32,
    worst_gap: Duration,
    /// Last finished window: fps, mean / worst CPU ms per frame, mean ms per span.
    pub fps: f32,
    pub cpu_ms: f32,
    pub worst_ms: f32,
    pub span_ms: [f32; SPANS.len()],
    /// Buffer bytes written to the GPU (render::WRITTEN) in this window, and their mean KB per frame in the last.
    written: u64,
    pub upload_kb: f32,
    pub counts: Counts,
    /// The frame limit and present mode, for the overlay.
    pub limit: String,
    /// The last FRAMES frames, for the F12 graph.
    pub hist: Ring<Sample>,
    /// Reused by `graph`'s stats.
    scratch: Vec<f32>,
    log: bool,
}

impl Default for Perf {
    fn default() -> Self {
        let log = std::env::var_os("FFXI_PERF").is_some();
        ON.store(log, Ordering::Relaxed);
        let spike = std::env::var("FFXI_PERF_SPIKE").ok().and_then(|v| v.parse().ok()).unwrap_or(25.0f32);
        Self {
            window: Instant::now(),
            frames: 0,
            busy: Duration::ZERO,
            worst: Duration::ZERO,
            spans: [Duration::ZERO; SPANS.len()],
            last_start: None,
            spike: Duration::from_secs_f32(spike / 1000.0),
            spikes: 0,
            worst_gap: Duration::ZERO,
            fps: 0.0,
            cpu_ms: 0.0,
            worst_ms: 0.0,
            span_ms: [0.0; SPANS.len()],
            written: 0,
            upload_kb: 0.0,
            counts: Counts::default(),
            limit: String::new(),
            hist: Ring::default(),
            scratch: Vec::new(),
            log,
        }
    }
}

impl Perf {
    /// One redraw: its span end times from `start`; the CPU time excludes the acquire wait.
    pub fn frame(&mut self, start: Instant, marks: [Instant; SPANS.len()]) {
        let mut prev = start;
        let mut this = [Duration::ZERO; SPANS.len()];
        for (i, m) in marks.iter().enumerate() {
            this[i] = m.saturating_duration_since(prev);
            self.spans[i] += this[i];
            prev = *m;
        }
        let busy =
            marks[SPANS.len() - 1].saturating_duration_since(start) - marks[2].saturating_duration_since(marks[1]);
        self.busy += busy;
        self.worst = self.worst.max(busy);
        self.frames += 1;
        self.written += crate::render::WRITTEN.swap(0, Ordering::Relaxed);
        let gap = self.last_start.map_or(Duration::ZERO, |l| start.saturating_duration_since(l));
        self.last_start = Some(start);
        let notes =
            if self.log { NOTES.lock().map(|mut n| std::mem::take(&mut *n)).unwrap_or_default() } else { Vec::new() };
        self.worst_gap = self.worst_gap.max(gap);
        // The first frame has no gap; its busy time stands in.
        let ms = |d: Duration| d.as_secs_f32() * 1000.0;
        self.hist.push(Sample { gap: ms(if gap.is_zero() { busy } else { gap }), spans: this.map(ms) });
        if self.log && (gap > self.spike || busy > self.spike) {
            self.spikes += 1;
            let ms = |d: Duration| d.as_secs_f32() * 1000.0;
            let spans: Vec<String> = SPANS.iter().zip(this).map(|(s, d)| format!("{s} {:.1}", ms(d))).collect();
            // The gap less this redraw is time spent before it started (events, the pacing wait).
            let before = gap.saturating_sub(marks[SPANS.len() - 1].saturating_duration_since(start));
            log::warn!(
                "spike: gap {:.1} ms (before redraw {:.1}, busy {:.1}); {}; {}",
                ms(gap),
                ms(before),
                ms(busy),
                spans.join(", "),
                if notes.is_empty() { "nothing noted".to_string() } else { notes.join("; ") }
            );
        }
        let el = self.window.elapsed();
        if el >= Duration::from_secs(2) {
            let n = self.frames.max(1) as f32;
            self.fps = self.frames as f32 / el.as_secs_f32();
            self.cpu_ms = self.busy.as_secs_f32() * 1000.0 / n;
            self.worst_ms = self.worst.as_secs_f32() * 1000.0;
            self.span_ms = self.spans.map(|d| d.as_secs_f32() * 1000.0 / n);
            self.upload_kb = self.written as f32 / 1024.0 / n;
            if self.log {
                let spans: Vec<String> = SPANS.iter().zip(self.span_ms).map(|(s, v)| format!("{s} {v:.2}")).collect();
                let c = self.counts;
                log::warn!(
                    "perf: {:.1} fps ({}), cpu {:.2} ms (worst {:.2}), worst gap {:.1} ms, spikes {}; {}; entities {} actors {} posed {} drawn {} zone draws {} ui verts {}, upload {:.0} KB/frame",
                    self.fps,
                    self.limit,
                    self.cpu_ms,
                    self.worst_ms,
                    self.worst_gap.as_secs_f32() * 1000.0,
                    self.spikes,
                    spans.join(", "),
                    c.entities,
                    c.actors,
                    c.posed,
                    c.drawn,
                    c.zone_draws,
                    c.ui_vertices,
                    self.upload_kb
                );
            }
            self.window = Instant::now();
            self.frames = 0;
            self.written = 0;
            self.busy = Duration::ZERO;
            self.worst = Duration::ZERO;
            self.spans = [Duration::ZERO; SPANS.len()];
            self.spikes = 0;
            self.worst_gap = Duration::ZERO;
        }
    }

    /// Overlay lines.
    pub fn lines(&self) -> Vec<String> {
        let c = self.counts;
        let spans: Vec<String> = SPANS.iter().zip(self.span_ms).map(|(s, v)| format!("{s} {v:.1}")).collect();
        vec![
            format!(
                "{:.1} fps ({})   cpu {:.2} ms/frame (worst {:.1})",
                self.fps, self.limit, self.cpu_ms, self.worst_ms
            ),
            spans.join("  "),
            format!(
                "entities {}  actors {} (posed {}, drawn {})  zone draws {}  ui vertices {}",
                c.entities, c.actors, c.posed, c.drawn, c.zone_draws, c.ui_vertices
            ),
            format!("upload {:.0} KB/frame", self.upload_kb),
            format!(
                "effects: tasks {}/{}  generators {}/{}  elements {}/{}",
                c.fx[0].0, c.fx[0].1, c.fx[1].0, c.fx[1].1, c.fx[2].0, c.fx[2].1
            ),
        ]
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ring_keeps_newest_in_order() {
        let mut r = Ring::default();
        for i in 0..FRAMES + 5 {
            r.push(i);
        }
        assert_eq!(r.len(), FRAMES);
        let v: Vec<usize> = r.iter().copied().collect();
        assert_eq!(v.first(), Some(&5));
        assert_eq!(v.last(), Some(&(FRAMES + 4)));
        assert!(v.windows(2).all(|w| w[1] == w[0] + 1));
    }

    #[test]
    fn ring_partial_iterates_from_start() {
        let mut r = Ring::default();
        r.push(1);
        r.push(2);
        assert_eq!(r.iter().copied().collect::<Vec<_>>(), vec![1, 2]);
    }

    #[test]
    fn stats_mean_p99_worst() {
        let mut scratch = Vec::new();
        assert_eq!(stats(std::iter::empty(), &mut scratch), Stats::default());
        // 99 frames at 10 ms and one at 50: the 99th percentile is still 10, the worst 50.
        let gaps = std::iter::repeat_n(10.0, 99).chain([50.0]);
        let s = stats(gaps, &mut scratch);
        assert!((s.mean - 10.4).abs() < 1e-4);
        assert_eq!(s.p99, 10.0);
        assert_eq!(s.worst, 50.0);
        // Two slow frames in 100 reach the 99th percentile.
        let s = stats(std::iter::repeat_n(10.0, 98).chain([30.0, 50.0]), &mut scratch);
        assert_eq!(s.p99, 30.0);
    }

    #[test]
    fn scale_steps() {
        assert_eq!(scale(5.0), 20.0);
        assert_eq!(scale(20.0), 20.0);
        assert_eq!(scale(33.4), 40.0);
        assert_eq!(scale(70.0), 80.0);
        assert_eq!(scale(500.0), 160.0);
    }

    #[test]
    fn frame_feeds_history() {
        let mut p = Perf::default();
        let t = Instant::now();
        let ms = |n: u64| t + Duration::from_millis(n);
        p.frame(t, [ms(1), ms(2), ms(3), ms(4), ms(5), ms(6)]);
        p.frame(ms(16), [ms(17), ms(18), ms(19), ms(20), ms(21), ms(22)]);
        let v: Vec<Sample> = p.hist.iter().copied().collect();
        assert_eq!(v.len(), 2);
        assert!((v[0].gap - 5.0).abs() < 0.01, "first frame: busy (6 less the 1 ms acquire)");
        assert!((v[1].gap - 16.0).abs() < 0.01);
        assert!(v[1].spans.iter().all(|s| (s - 1.0).abs() < 0.01));
    }
}

//! Ours (a developer tool, not retail): the Packets window (/packets with config `dev_tools`), drawn over the modern UI.
//! The controls and filters across the top, the rows down the left, the chosen row's fields and bytes at the right.
//! Its state and logic are client/packets.rs; the capture itself net/capture.rs.

use super::super::World;
use super::Panels;
use super::kit::{self, Btn};
use crate::net::capture::{self, Dir, Kind};
use egui::{FontFamily, FontId, RichText, Ui, vec2};

const W: f32 = 1240.0;
const LIST_W: f32 = 500.0;
const ROW_H: f32 = 18.0;

const FIELDS: [&str; 4] = ["save", "ids", "zone", "find"];

fn field_id(name: &str) -> egui::Id {
    egui::Id::unique(("packets field", name))
}

/// One of the window's text fields has egui's focus: the modern UI keeps it there (it drops every other focus).
pub fn keeps_focus(w: &World, id: egui::Id) -> bool {
    shown(w) && FIELDS.iter().any(|n| field_id(n) == id)
}

pub fn shown(w: &World) -> bool {
    w.modern && w.packets.open
}

pub fn draw(p: &mut Panels, w: &mut World) {
    if shown(w) {
        window(p, &mut w.packets);
    }
}

/// The window with a few sample records, one picked (modern-shot --packets).
pub(super) fn sample(p: &mut Panels) {
    use crate::net::capture::Record;
    let pk = |id: u16, body: &[u8]| crate::net::map::session::build(id, body);
    let mut celestite = [0u8; 0xb0];
    celestite[..8].copy_from_slice(&[0x21, 0, 0, 0, 0x05, 0, 0, 0]);
    let mut chat = [0u8; 0x30];
    chat[4..8].copy_from_slice(&[4, 0, 0, 0]);
    chat[8..13].copy_from_slice(b"Livea");
    chat[0x17..0x24].copy_from_slice(b"\x1e\x09Hello there");
    let rows = [
        (0, Dir::In, Kind::Datagram { dgram: 312, seq: 41, ack: 17, form: "huffman".into() }),
        (0, Dir::In, Kind::Packet { id: 0x00D, hex: pk(0x00D, &[0; 0x40]) }),
        (180, Dir::In, Kind::Packet { id: 0x017, hex: pk(0x017, &chat[4..]) }),
        (400, Dir::Out, Kind::Datagram { dgram: 96, seq: 18, ack: 41, form: "huffman".into() }),
        (400, Dir::Out, Kind::Packet { id: 0x015, hex: pk(0x015, &[0; 0x1c]) }),
        (812, Dir::Out, Kind::Packet { id: 0x0C6, hex: pk(0x0C6, &[]) }),
        (1020, Dir::In, Kind::Packet { id: 0x08F, hex: pk(0x08F, &celestite[4..]) }),
        (1400, Dir::Out, Kind::Packet { id: 0x0C5, hex: pk(0x0C5, &[2, 7, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0]) }),
    ];
    let mut pk = super::super::packets::Packets::new(true);
    pk.records =
        rows.into_iter().map(|(t, dir, kind)| Record { t: 1_763_300_000_000 + t, dir, zone: 26, kind }).collect();
    (pk.source, pk.datagrams, pk.selected) = ("1763300000.jsonl".into(), true, Some(6));
    window(p, &mut pk);
}

fn window(p: &mut Panels, pk: &mut super::super::packets::Packets) {
    let m = p.m;
    let small = m.theme.small;
    let mono = FontId::new(small, FontFamily::Monospace);
    let at = p.ctx.content_rect().center() - vec2(W / 2.0, 300.0);
    let tall = (p.ctx.content_rect().height() * 0.55).max(240.0);
    let note = format!("{} ({} records)", pk.source, pk.records.len());
    let mut close = false;
    let mut typing = false;
    p.panel_at(
        "packets",
        Some(super::Title { icon: None, text: "Packets", note: Some(&note), name: false }),
        W,
        at,
        |ui| {
            let btn = |ui: &mut Ui, label: &str, lit: bool| {
                let s = Btn { lit, cursor: false, off: false };
                kit::clicked(&kit::button(ui, m, label, 0.0, s), s)
            };
            // Source and recording.
            ui.horizontal(|ui| {
                ui.spacing_mut().item_spacing.x = 6.0;
                let paused = pk.source == "live" && !pk.live;
                if btn(
                    ui,
                    if pk.live {
                        "Live"
                    } else if paused {
                        "Paused"
                    } else {
                        "Go live"
                    },
                    pk.live,
                ) {
                    if pk.live {
                        pk.live = false;
                    } else {
                        pk.go_live();
                    }
                }
                if btn(ui, "Clear", false) {
                    pk.clear();
                }
                if btn(ui, if pk.record { "Recording to file" } else { "Record to file" }, pk.record) {
                    pk.record = !pk.record;
                }
                let files: Vec<_> = pk.files.clone();
                let mut pick = None;
                egui::ComboBox::from_id_salt("packets open").selected_text("Open capture").width(220.0).show_ui(
                    ui,
                    |ui| {
                        for f in &files {
                            let name = f.file_name().map_or_else(String::new, |n| n.to_string_lossy().into_owned());
                            if ui.selectable_label(false, name).clicked() {
                                pick = Some(f.clone());
                            }
                        }
                        if files.is_empty() {
                            ui.label("No captures yet");
                        }
                    },
                );
                if let Some(f) = pick {
                    pk.load(&f);
                }
                if btn(ui, "Rescan", false) {
                    pk.refresh_files();
                }
                typing |= ui
                    .add(
                        egui::TextEdit::singleline(&mut pk.save_name)
                            .id(field_id("save"))
                            .hint_text("file name")
                            .desired_width(150.0),
                    )
                    .has_focus();
                if btn(ui, "Save shown", false) {
                    pk.save();
                }
                if btn(ui, "Close", false) {
                    close = true;
                }
            });
            // Filters.
            ui.horizontal(|ui| {
                ui.spacing_mut().item_spacing.x = 6.0;
                ui.label("Ids");
                typing |= ui
                    .add(
                        egui::TextEdit::singleline(&mut pk.ids)
                            .id(field_id("ids"))
                            .hint_text("08f, 0c5")
                            .desired_width(110.0),
                    )
                    .has_focus();
                for (label, d) in [("Both", None), ("In", Some(Dir::In)), ("Out", Some(Dir::Out))] {
                    if btn(ui, label, pk.dir == d) {
                        pk.dir = d;
                    }
                }
                ui.label("Zone");
                typing |= ui
                    .add(egui::TextEdit::singleline(&mut pk.zone).id(field_id("zone")).desired_width(46.0))
                    .has_focus();
                ui.label("Find");
                typing |= ui
                    .add(
                        egui::TextEdit::singleline(&mut pk.find)
                            .id(field_id("find"))
                            .hint_text("1e 09 or \"text\"")
                            .desired_width(150.0),
                    )
                    .has_focus();
                if btn(ui, "Datagrams", pk.datagrams) {
                    pk.datagrams = !pk.datagrams;
                }
                if btn(ui, "Follow", pk.follow) {
                    pk.follow = !pk.follow;
                }
            });
            let status = if !pk.bad_filter.is_empty() { pk.bad_filter.clone() } else { pk.status.clone() };
            let forms = pk.forms();
            let rec =
                if capture::recording() { format!("writing to {}", capture::dir().display()) } else { String::new() };
            let line = [status, if forms.is_empty() { String::new() } else { format!("datagrams: {forms}") }, rec]
                .into_iter()
                .filter(|s| !s.is_empty())
                .collect::<Vec<_>>()
                .join("  ·  ");
            ui.label(RichText::new(line).size(small).color(m.theme.color("muted")));
            ui.add_space(4.0);
            ui.horizontal_top(|ui| {
                // The rows.
                let view = pk.view().to_vec();
                let start = pk.records.first().map_or(0, |r| r.t);
                ui.allocate_ui_with_layout(vec2(LIST_W, tall), egui::Layout::top_down(egui::Align::Min), |ui| {
                    let mut area = egui::ScrollArea::vertical()
                        .id_salt("packets rows")
                        .max_height(tall)
                        .auto_shrink([false, false]);
                    if pk.follow {
                        area = area.stick_to_bottom(true);
                    }
                    area.show_rows(ui, ROW_H, view.len(), |ui, range| {
                        for &i in &view[range] {
                            let r = &pk.records[i];
                            let d = if r.dir == Dir::In { "in " } else { "out" };
                            let secs = r.t.saturating_sub(start) as f64 / 1000.0;
                            let text = match &r.kind {
                                Kind::Packet { id, hex } => {
                                    format!(
                                        "{secs:>9.3} {d} z{:<4} {id:#05x} {:<18} {:>4}",
                                        r.zone,
                                        capture::name(r.dir, *id),
                                        hex.len()
                                    )
                                }
                                Kind::Datagram { dgram, seq, ack, form } => {
                                    format!("{secs:>9.3} {d} z{:<4} datagram #{seq} ack {ack} {form} {dgram}", r.zone)
                                }
                            };
                            let colour = match r.kind {
                                Kind::Datagram { .. } => m.theme.color("muted"),
                                _ if r.dir == Dir::Out => m.theme.color("gold"),
                                _ => m.theme.color("text"),
                            };
                            let (rect, resp) = ui.allocate_exact_size(vec2(LIST_W - 14.0, ROW_H), egui::Sense::click());
                            if pk.selected == Some(i) {
                                ui.painter().rect_filled(rect, 2.0, m.theme.color("focus").gamma_multiply(0.45));
                            } else if resp.hovered() {
                                ui.painter().rect_filled(rect, 2.0, m.theme.color("focus").gamma_multiply(0.15));
                            }
                            let left = egui::pos2(rect.min.x + 4.0, rect.center().y);
                            ui.painter().text(left, egui::Align2::LEFT_CENTER, text, mono.clone(), colour);
                            if resp.clicked() {
                                pk.selected = Some(i);
                                pk.follow = false;
                            }
                        }
                    });
                });
                ui.separator();
                // The chosen row.
                ui.vertical(|ui| {
                    let Some(r) = pk.selected.and_then(|i| pk.records.get(i)) else {
                        ui.label(
                            RichText::new("Pick a row to see its fields and bytes.")
                                .size(small)
                                .color(m.theme.color("muted")),
                        );
                        return;
                    };
                    egui::ScrollArea::vertical()
                        .id_salt("packets detail")
                        .max_height(tall)
                        .auto_shrink([false, false])
                        .show(ui, |ui| match &r.kind {
                            Kind::Packet { hex, .. } => {
                                egui::Grid::new("packets fields").num_columns(2).spacing(vec2(12.0, 2.0)).show(
                                    ui,
                                    |ui| {
                                        ui.label(RichText::new("dir").size(small).color(m.theme.color("muted")));
                                        ui.label(RichText::new(format!("{:?}, zone {}", r.dir, r.zone)).size(small));
                                        ui.end_row();
                                        for (k, v) in capture::decode(r.dir, hex) {
                                            ui.label(RichText::new(k).size(small).color(m.theme.color("muted")));
                                            ui.add(egui::Label::new(RichText::new(v).font(mono.clone())).wrap());
                                            ui.end_row();
                                        }
                                    },
                                );
                                ui.add_space(6.0);
                                let mut d = super::super::packets::dump(hex);
                                ui.add(
                                    egui::TextEdit::multiline(&mut d)
                                        .font(mono.clone())
                                        .desired_width(f32::INFINITY)
                                        .interactive(true),
                                );
                                if ui.small_button("Copy hex").clicked() {
                                    ui.ctx().copy_text(hex::encode(hex));
                                }
                            }
                            Kind::Datagram { dgram, seq, ack, form } => {
                                ui.label(format!(
                                    "{:?} datagram, zone {}: {dgram} bytes, sequence {seq}, ack {ack}, form {form}",
                                    r.dir, r.zone
                                ));
                            }
                        });
                });
            });
        },
    );
    pk.typing = typing;
    if close {
        pk.open = false;
    }
}

#[cfg(test)]
mod tests {
    /// A clicked field keeps egui's focus through the modern UI's per-frame surrender (as `keeps_focus` exempts it)
    /// and takes typed text.
    #[test]
    fn field_keeps_focus_and_types() {
        let ctx = egui::Context::default();
        let mut text = String::new();
        let mut rect = egui::Rect::NOTHING;
        let frame = |events: Vec<egui::Event>, text: &mut String, rect: &mut egui::Rect| {
            let raw = egui::RawInput { events, ..Default::default() };
            let mut out = ctx.run_ui(raw, |ui| {
                // The surrender in Modern::ui, with this window's exemption.
                ui.ctx().memory_mut(|mem| {
                    if let Some(id) = mem.focused()
                        && !super::FIELDS.iter().any(|n| super::field_id(n) == id)
                    {
                        mem.surrender_focus(id);
                    }
                });
                *rect = ui.add(egui::TextEdit::singleline(text).id(super::field_id("ids"))).rect;
            });
            out.textures_delta.clear();
        };
        frame(Vec::new(), &mut text, &mut rect);
        let at = rect.center();
        let click = |pressed| egui::Event::PointerButton {
            pos: at,
            button: egui::PointerButton::Primary,
            pressed,
            modifiers: Default::default(),
        };
        frame(vec![egui::Event::PointerMoved(at), click(true), click(false)], &mut text, &mut rect);
        assert_eq!(ctx.memory(|m| m.focused()), Some(super::field_id("ids")));
        frame(vec![egui::Event::Text("08f".into())], &mut text, &mut rect);
        frame(vec![egui::Event::Text(",0c5".into())], &mut text, &mut rect);
        assert_eq!(text, "08f,0c5");
    }
}

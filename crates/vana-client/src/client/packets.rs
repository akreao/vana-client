//! Ours (a developer tool, not retail): the Packets window's state. It shows the map session's sub-packets live
//! (net/capture.rs's feed) or from a capture file, filtered by id, direction, zone and bytes, with the fields the
//! client reads from known ids and the raw bytes. Opened with /packets while config `dev_tools` is on; the window
//! itself is client/modern/packetwin.rs.

use crate::net::capture::{self, Dir, Filter, Kind, Record};
use std::path::{Path, PathBuf};

/// Records kept; past it the oldest quarter goes.
const MAX: usize = 200_000;

#[derive(Default)]
pub struct Packets {
    /// Config `dev_tools`: /packets opens the window.
    pub enabled: bool,
    pub open: bool,
    pub records: Vec<Record>,
    /// Taking the live feed (else showing what was loaded, or paused).
    pub live: bool,
    /// The window's own switch for writing a capture file (config `packet_capture` turns it on too).
    pub record: bool,
    /// What `records` came from: "live" or a file name.
    pub source: String,
    pub ids: String,
    pub dir: Option<Dir>,
    pub zone: String,
    pub find: String,
    /// Datagram rows shown between the sub-packets.
    pub datagrams: bool,
    /// The list keeps the newest row in view.
    pub follow: bool,
    /// The chosen record (an index into `records`).
    pub selected: Option<usize>,
    /// The last load or save message.
    pub status: String,
    /// What is wrong with the filter typed; empty when it reads.
    pub bad_filter: String,
    /// A text field has the keys (set by the window each frame).
    pub typing: bool,
    /// Capture files to open, newest first.
    pub files: Vec<PathBuf>,
    pub save_name: String,
    /// The rows shown (indices into `records`), the records looked at so far, and the filter they were built for.
    view: Vec<usize>,
    scanned: usize,
    view_key: Option<(String, Option<Dir>, String, String, bool)>,
}

impl Packets {
    pub fn new(enabled: bool) -> Self {
        Self { enabled, ..Default::default() }
    }

    /// Opens or closes the window; opening starts the live feed when nothing is loaded.
    pub fn toggle(&mut self) {
        self.open = !self.open;
        if self.open {
            self.refresh_files();
            if self.records.is_empty() {
                self.live = true;
                self.follow = true;
                self.source = "live".into();
            }
        }
    }

    /// Each frame: the live feed on while the window takes it, its new records appended.
    pub fn tick(&mut self) {
        let live = self.open && self.live;
        capture::live(live);
        if live {
            let new = capture::take_live();
            if !new.is_empty() {
                self.records.extend(new);
                if self.records.len() > MAX {
                    self.records.drain(..MAX / 4);
                    self.selected = self.selected.and_then(|s| s.checked_sub(MAX / 4));
                    self.view_key = None;
                }
            }
        }
    }

    pub fn clear(&mut self) {
        self.records.clear();
        (self.selected, self.view_key) = (None, None);
        self.status.clear();
    }

    /// Back to the live feed (what was loaded goes).
    pub fn go_live(&mut self) {
        if self.source != "live" {
            self.clear();
            self.source = "live".into();
        }
        self.live = true;
    }

    pub fn refresh_files(&mut self) {
        let mut files: Vec<_> = std::fs::read_dir(capture::dir())
            .map(|d| d.flatten().map(|e| e.path()).filter(|p| p.extension().is_some_and(|e| e == "jsonl")).collect())
            .unwrap_or_default();
        files.sort_by_key(|p| std::cmp::Reverse(std::fs::metadata(p).and_then(|m| m.modified()).ok()));
        self.files = files;
    }

    pub fn load(&mut self, path: &Path) {
        match capture::load(path) {
            Ok(r) => {
                self.clear();
                self.status = format!("{} records from {}", r.len(), path.display());
                self.records = r;
                self.live = false;
                self.source = path.file_name().map_or_else(String::new, |n| n.to_string_lossy().into_owned());
            }
            Err(e) => self.status = format!("{e:#}"),
        }
    }

    /// Writes the rows the filter keeps (and the datagram rows, when shown) to the captures folder.
    pub fn save(&mut self) {
        let name = self.save_name.trim();
        let name = if name.is_empty() { format!("{}-filtered", capture_stamp()) } else { name.to_string() };
        let name = if name.ends_with(".jsonl") { name } else { format!("{name}.jsonl") };
        if name.contains(['/', '\\']) {
            self.status = "Save takes a file name, not a path.".into();
            return;
        }
        let path = capture::dir().join(&name);
        let view = self.view().to_vec();
        let rows: Vec<_> = view.iter().map(|&i| &self.records[i]).collect();
        let n = rows.len();
        self.status = match std::fs::create_dir_all(capture::dir())
            .map_err(anyhow::Error::from)
            .and_then(|_| capture::save(&path, rows))
        {
            Ok(()) => format!("{n} records saved to {}", path.display()),
            Err(e) => format!("{e:#}"),
        };
        self.refresh_files();
    }

    /// The filter typed, or what is wrong with it.
    pub fn filter(&self) -> Result<Filter, String> {
        let zone = self.zone.trim();
        Ok(Filter {
            ids: capture::parse_ids(&self.ids)?,
            dir: self.dir,
            zone: if zone.is_empty() { None } else { Some(zone.parse().map_err(|_| format!("not a zone: {zone}"))?) },
            find: if self.find.trim().is_empty() { Vec::new() } else { capture::parse_find(&self.find)? },
        })
    }

    /// The rows shown, brought up to date (a changed filter starts over, new records are added).
    pub fn view(&mut self) -> &[usize] {
        let key = (self.ids.clone(), self.dir, self.zone.clone(), self.find.clone(), self.datagrams);
        if self.view_key.as_ref() != Some(&key) || self.scanned > self.records.len() {
            (self.view, self.scanned, self.view_key) = (Vec::new(), 0, Some(key));
        }
        let f = match self.filter() {
            Ok(f) => f,
            Err(e) => {
                self.bad_filter = e;
                return &self.view;
            }
        };
        self.bad_filter.clear();
        for i in self.scanned..self.records.len() {
            let r = &self.records[i];
            let shown = match r.kind {
                Kind::Packet { .. } => f.keeps(r),
                // Datagrams carry no id or bytes to match: shown only while neither is filtered on.
                Kind::Datagram { .. } => {
                    self.datagrams
                        && f.ids.is_empty()
                        && f.find.is_empty()
                        && f.dir.is_none_or(|d| d == r.dir)
                        && f.zone.is_none_or(|z| z == r.zone)
                }
            };
            if shown {
                self.view.push(i);
            }
        }
        self.scanned = self.records.len();
        &self.view
    }

    /// Datagram counts by form each way, over everything held.
    pub fn forms(&self) -> String {
        let mut n = std::collections::BTreeMap::<(Dir, &str), u32>::new();
        for r in &self.records {
            if let Kind::Datagram { form, .. } = &r.kind {
                *n.entry((r.dir, form)).or_default() += 1;
            }
        }
        n.iter()
            .map(|((d, f), n)| format!("{} {f} {n}", if *d == Dir::In { "in" } else { "out" }))
            .collect::<Vec<_>>()
            .join(", ")
    }
}

fn capture_stamp() -> u64 {
    std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).map_or(0, |d| d.as_secs())
}

/// A hex dump: offset, 16 bytes, and the printable ones as text.
pub fn dump(data: &[u8]) -> String {
    data.chunks(16)
        .enumerate()
        .map(|(k, row)| {
            let hex: Vec<_> = row.iter().map(|b| format!("{b:02x}")).collect();
            let text: String =
                row.iter().map(|&b| if b.is_ascii_graphic() || b == b' ' { b as char } else { '.' }).collect();
            format!("{:04x}  {:<47}  {text}", k * 16, hex.join(" "))
        })
        .collect::<Vec<_>>()
        .join("\n")
}

#[cfg(test)]
mod tests {
    use super::*;

    fn rec(t: u64, dir: Dir, zone: u16, kind: Kind) -> Record {
        Record { t, dir, zone, kind }
    }

    /// The view follows the filter and takes new records as they come; datagram rows only when asked for.
    #[test]
    fn view_filters() {
        let mut p = Packets {
            records: vec![
                rec(1, Dir::In, 230, Kind::Packet { id: 0x08F, hex: vec![0x8f, 0x2c, 0, 0, 0x1e, 0x09] }),
                rec(2, Dir::In, 230, Kind::Datagram { dgram: 60, seq: 1, ack: 0, form: "plain".into() }),
                rec(3, Dir::Out, 231, Kind::Packet { id: 0x0C6, hex: vec![0xc6, 2, 0, 0] }),
            ],
            ..Default::default()
        };
        assert_eq!(p.view(), &[0, 2]);
        p.datagrams = true;
        assert_eq!(p.view(), &[0, 1, 2]);
        p.zone = "231".into();
        assert_eq!(p.view(), &[2]);
        p.zone.clear();
        p.find = "1e09".into();
        assert_eq!(p.view(), &[0]);
        p.records.push(rec(4, Dir::In, 230, Kind::Packet { id: 0x017, hex: vec![0x17, 2, 0, 0, 0x1e, 0x09, 0, 0] }));
        assert_eq!(p.view(), &[0, 3]);
        p.find = "zz".into();
        assert!(p.view().is_empty() && p.bad_filter.starts_with("not hex"));
    }

    #[test]
    fn dumps() {
        assert_eq!(dump(b"AB\x00"), format!("0000  41 42 00{}  AB.", " ".repeat(39)));
    }
}

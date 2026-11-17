//! Battlefield info: c2s 0x112 (GP_CLI_COMMAND_BATTLEFIELD_REQ) and its reply s2c 0x072 (FFXiMain 30261102_0).
//!
//! mapframe's slot 0x30 (win_mapframe_v30 -> FUN_101fbc90(.., 0)) asks with kind 0 at every zone-in, from the handler
//! walk FUN_101604c0 (client/mod.rs `zone_walk`). The senders set one-shot reply callbacks the s2c 0x072 handler
//! (FUN_100999a0) runs: kind 0 -> FUN_101fbd00 (the sidebar block), 1 -> FUN_101fbdd0 (the map overlay). The zone load
//! (FUN_10093720) clears both callbacks.
//!
//! What reads the blocks is gated to zone 0x11E: the sidebar (FUN_10220a20 -> FUN_10220f60, session +0x3c400 ==
//! 0x11e) and the map overlay (win_mapframe_v10 -> FUN_101fc130, FUN_101fc0c0: the map's zone +0x100 == 0x11e), and
//! so is the only kind 2 send (FUN_101fb060 -> FUN_101fc100). Kind 1 has no caller (FUN_101fbc90 is called with 0 and 2
//! only). Zone 0x11E has no area name in the client's data (file 55466 entry 0x11e is empty), so those are not built;
//! the blocks are kept as retail keeps them.

/// The one c2s 0x112 sender FUN_101fbc90(.., kind) picks: 0 FUN_10098080, 1 FUN_100980c0, 2 FUN_10098100.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Kind {
    /// Both blocks (the zone-in's).
    Both = 0,
    #[allow(dead_code)] // FUN_100980c0 has no caller
    Sidebar = 1,
    #[allow(dead_code)] // sent for a map of zone 0x11E only (FUN_101fc100), not built
    MapOverlay = 2,
}

/// FUN_101fbd00's block: three records of four words from 0x10664964 (record 0 stays zero), a flag (_DAT_10664994)
/// and a 3-bit value (DAT_10664998). The getters (FUN_101f8810 / 8830 / 8850 / 88a0 by record and field, FUN_101f8790,
/// FUN_101f87b0) read zero while the block is not valid (DAT_10664960).
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Sidebar {
    #[allow(dead_code)] // read by the zone 0x11E sidebar (FUN_10220f60), not built
    pub records: [[u32; 4]; 3],
    #[allow(dead_code)] // FUN_101f8790, no caller
    pub flag: u32,
    #[allow(dead_code)] // FUN_101f87b0, read by the zone 0x11E sidebar
    pub mode: u32,
}

/// FUN_101fbdd0's block: ten records of four words from 0x1066499c. Not valid (DAT_10664961 clear), the getters give
/// 0, 3, -1, -1 (FUN_101f88d0 / 88f0 / 8910 / 8930).
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Overlay {
    #[allow(dead_code)] // read by the zone 0x11E map overlay (FUN_101fc150), not built
    pub records: [[i32; 4]; 10],
}

#[derive(Default)]
pub struct Battlefield {
    /// DAT_1048904c / DAT_10489050: the reply callbacks set, one-shot.
    sidebar_cb: bool,
    overlay_cb: bool,
    /// The blocks; None while not valid (DAT_10664960 / DAT_10664961).
    #[allow(dead_code)] // read by the zone 0x11E sidebar, not built
    pub sidebar: Option<Sidebar>,
    #[allow(dead_code)] // read by the zone 0x11E map overlay, not built
    pub overlay: Option<Overlay>,
}

fn word(p: &[u8], o: usize) -> u32 {
    u32::from_le_bytes(std::array::from_fn(|i| p.get(o + i).copied().unwrap_or(0)))
}

/// One packed word of the sidebar block (FUN_101fbd00): 7, 10, 14 and 1 bits.
fn split(v: u32) -> [u32; 4] {
    [v & 0x7f, v >> 7 & 0x3ff, v >> 17 & 0x3fff, v >> 31]
}

impl Sidebar {
    /// FUN_101fbd00 over the reply from @8: record 1 from the word @0xc, record 2 from @8, the flag and mode from @0x10.
    fn decode(p: &[u8]) -> Self {
        let c = word(p, 0x10);
        Sidebar { records: [[0; 4], split(word(p, 0xc)), split(word(p, 8))], flag: c & 1, mode: c >> 1 & 7 }
    }
}

impl Overlay {
    /// FUN_101fbdd0 over the reply from @8: byte i, then the word @0x14 + 4i's low 2 bits, bits 2..16 and 17..31,
    /// both signed.
    fn decode(p: &[u8]) -> Self {
        Overlay {
            records: std::array::from_fn(|i| {
                let w = word(p, 0x14 + 4 * i);
                [p.get(8 + i).copied().unwrap_or(0) as i32, (w & 3) as i32, ((w << 15) as i32) >> 17, (w as i32) >> 17]
            }),
        }
    }
}

impl Battlefield {
    /// FUN_101fbc90 with the sender's gate: kind 0 always goes and sets both callbacks; kinds 1 and 2 only while the
    /// sidebar callback is clear (FUN_10098100 checks DAT_1048904c too, though it sets the overlay's). A sent kind
    /// marks its blocks not valid. The c2s 0x112 body (4 bytes, kind @4) when one goes out.
    pub fn request(&mut self, kind: Kind) -> Option<Vec<u8>> {
        match kind {
            Kind::Both => {
                (self.sidebar_cb, self.overlay_cb, self.sidebar, self.overlay) = (true, true, None, None);
            }
            _ if self.sidebar_cb => return None,
            Kind::Sidebar => (self.sidebar_cb, self.sidebar) = (true, None),
            Kind::MapOverlay => (self.overlay_cb, self.overlay) = (true, None),
        }
        Some(vec![kind as u8, 0, 0, 0])
    }

    /// s2c 0x072 (FUN_100999a0): Kind @4, 0 the sidebar block, 1 the overlay, each taken only while its callback is
    /// set; the callback is cleared.
    pub fn on_reply(&mut self, p: &[u8]) {
        match p.get(4) {
            Some(0) if self.sidebar_cb => (self.sidebar_cb, self.sidebar) = (false, Some(Sidebar::decode(p))),
            Some(1) if self.overlay_cb => (self.overlay_cb, self.overlay) = (false, Some(Overlay::decode(p))),
            _ => {}
        }
    }

    /// FUN_10093720 (the zone load): both callbacks cleared; the blocks stay.
    pub fn zone_load(&mut self) {
        (self.sidebar_cb, self.overlay_cb) = (false, false);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn zone_in_request_is_kind_0() {
        let mut b = Battlefield::default();
        assert_eq!(b.request(Kind::Both), Some(vec![0, 0, 0, 0]));
        // A second kind 0 goes out as well (FUN_10098080 has no gate).
        assert_eq!(b.request(Kind::Both), Some(vec![0, 0, 0, 0]));
    }

    #[test]
    fn kinds_1_and_2_wait_on_the_sidebar_callback() {
        let mut b = Battlefield::default();
        b.request(Kind::Both);
        assert_eq!(b.request(Kind::Sidebar), None);
        assert_eq!(b.request(Kind::MapOverlay), None);
        b.zone_load();
        assert_eq!(b.request(Kind::MapOverlay), Some(vec![2, 0, 0, 0]));
        // Kind 2 set the overlay's callback only, so the sidebar's gate is still open.
        assert_eq!(b.request(Kind::Sidebar), Some(vec![1, 0, 0, 0]));
        assert_eq!(b.request(Kind::MapOverlay), None);
    }

    fn reply(kind: u8, body: &[u8]) -> Vec<u8> {
        let mut p = vec![0x72, 0x20, 0, 0, kind, 0, 0, 0];
        p.extend_from_slice(body);
        p.resize(0x3c, 0);
        p
    }

    #[test]
    fn sidebar_reply() {
        let mut b = Battlefield::default();
        b.request(Kind::Both);
        let a: u32 = 5 | 300 << 7 | 9000 << 17 | 1 << 31;
        let bw: u32 = 0x7f | 0x3ff << 7 | 15000 << 17;
        let c: u32 = 1 | 5 << 1 | 0xf0;
        let mut body = Vec::new();
        for v in [a, bw, c] {
            body.extend_from_slice(&v.to_le_bytes());
        }
        b.on_reply(&reply(0, &body));
        let s = b.sidebar.clone().unwrap();
        assert_eq!(s.records, [[0; 4], [0x7f, 0x3ff, 15000, 0], [5, 300, 9000, 1]]);
        assert_eq!((s.flag, s.mode), (1, 5));
        // One-shot: a second reply is dropped.
        b.on_reply(&reply(0, &[0xff; 12]));
        assert_eq!(b.sidebar, Some(s));
        // The overlay's callback is still set.
        assert!(b.overlay.is_none());
    }

    #[test]
    fn overlay_reply() {
        let mut b = Battlefield::default();
        b.request(Kind::Both);
        let mut body: Vec<u8> = (1..=12).collect();
        // Record 0: 2 | x = -3 | y = 100; record 9: 1 | x = 16383 | y = -1.
        let w0: u32 = 2 | ((-3i32 as u32) & 0x7fff) << 2 | (100u32 << 17);
        let w9: u32 = 1 | 0x3fff << 2 | ((-1i32 as u32) << 17);
        for i in 0..10 {
            let w = match i {
                0 => w0,
                9 => w9,
                _ => 3,
            };
            body.extend_from_slice(&w.to_le_bytes());
        }
        b.on_reply(&reply(1, &body));
        let o = b.overlay.clone().unwrap();
        assert_eq!(o.records[0], [1, 2, -3, 100]);
        assert_eq!(o.records[1], [2, 3, 0, 0]);
        assert_eq!(o.records[9], [10, 1, 16383, -1]);
        assert!(b.sidebar.is_none());
    }

    #[test]
    fn reply_without_a_request_is_dropped() {
        let mut b = Battlefield::default();
        b.on_reply(&reply(0, &[1; 12]));
        b.on_reply(&reply(1, &[1; 52]));
        assert!(b.sidebar.is_none() && b.overlay.is_none());
        b.request(Kind::Both);
        b.zone_load();
        b.on_reply(&reply(0, &[1; 12]));
        assert!(b.sidebar.is_none());
    }
}

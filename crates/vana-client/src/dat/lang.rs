//! The client language, one per process as retail's: DAT_10457bd8 +8 (0 ja, 1 en, 2 de, 3 fr). PlayOnline hands it
//! to the game in its start-up block (FUN_100157d0 gives the host &DAT_10457b58; FUN_100157e0 copies the block's +0x28,
//! DAT_10457b80, to +8). Ours takes it from config.json's `language`, set once at process start (main).

use super::items::Lang;
use std::sync::atomic::{AtomicU8, Ordering};

static LANG: AtomicU8 = AtomicU8::new(Lang::En as u8);

/// The block's +0x4c bit 0 (DAT_10457ba4 -> DAT_10457bd8 +0x18 bit 7, FUN_100157e0): German and French read the English
/// files and compare as English (FUN_1025cb90, FUN_101b67e0, FUN_101bde00). PlayOnline sets it for an install without
/// the European files; ours has no such install, so it stays clear.
const EUROPEAN_AS_ENGLISH: bool = false;

/// The client language.
pub fn client() -> Lang {
    match LANG.load(Ordering::Relaxed) {
        0 => Lang::Ja,
        2 => Lang::De,
        3 => Lang::Fr,
        _ => Lang::En,
    }
}

pub fn set(l: Lang) {
    LANG.store(l as u8, Ordering::Relaxed);
}

/// The language whose files and word rules apply (the test FUN_1025cb90 and the dictionary's readers make): the
/// client's, but English for German and French while EUROPEAN_AS_ENGLISH.
pub fn files() -> Lang {
    match client() {
        Lang::De | Lang::Fr if EUROPEAN_AS_ENGLISH => Lang::En,
        l => l,
    }
}

/// Whether German and French are played as English (the block's flag, DAT_10457bd8 +0x18 bit 7).
pub fn european_as_english() -> bool {
    EUROPEAN_AS_ENGLISH
}

/// FUN_1025cbd0's table (0x1038b998, rows of ja / en / de / fr): the layouts and sprite libraries each language has its
/// own of. Rows 3 (compass) and 4 (sc_item) have no caller.
const LIBS: [[&str; 4]; 8] = [
    ["menu    frames", "menu    framesus", "menu    framesde", "menu    framesfr"],
    ["menu    comwin", "menu    comwin", "menu    comwinde", "menu    comwinfr"],
    ["menu    resyn", "menu    resynus", "menu    resynde", "menu    resynfr"],
    ["menu    compass", "menu    compass", "menu    compassd", "menu    compassf"],
    ["menu    sc_item", "menu    sc_item_", "menu    sc_item_", "menu    sc_item_"],
    ["menu    key1top", "menu    key1topu", "menu    key1topu", "menu    key1topu"],
    ["menu    jobcsel", "menu    jobcselu", "menu    jobcselu", "menu    jobcselu"],
    ["menu    prty5", "menu    prty5us", "menu    prty5de", "menu    prty5fr"],
];

/// FUN_1025cbd0(row): row's name for the file language (trailing blanks dropped, as ui::norm_name keeps names).
pub fn lib(row: usize) -> &'static str {
    LIBS[row][files() as usize]
}

/// Ids SE gave every language's copy of a file: one per language (ja, en, de, fr) as the slot tables hold them.
/// `file(ids)` picks the file language's.
pub fn file(ids: &[u32; 4]) -> u32 {
    ids[files() as usize]
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Every per-language id set ours keeps is one slot of the DLL's four slot tables (FUN_1025cb20), and the library
    /// names are FUN_1025cbd0's table (0x1038b998). Needs the installed game; skipped without it.
    #[test]
    fn files_are_slots() {
        let dir = crate::config::Config::load().ffxi_dir();
        let Ok(dll) = std::fs::read(crate::dat::ffximain(&dir)) else { return };
        let slot = |l: u8, s: usize| crate::dat::msg::current_slot_file(&dll, l, s);
        let slots = (0..0x70).filter(|&s| slot(1, s).is_some()).collect::<Vec<_>>();
        let check = |what: &str, ids: [u32; 4]| {
            let found = slots.iter().find(|&&s| (0..4u8).all(|l| slot(l, s) == Some(ids[l as usize])));
            assert!(found.is_some(), "{what} {ids:?} is no slot");
        };
        use crate::dat::names::*;
        for (n, ids) in [
            ("key items", KEY_ITEMS),
            ("spells", SPELLS),
            ("abilities", ABILITIES),
            ("spell texts", SPELL_TEXTS),
            ("ability texts", ABILITY_TEXTS),
            ("mounts", MOUNTS),
            ("mount texts", MOUNT_TEXTS),
            ("emote texts", EMOTE_TEXTS),
            ("places", PLACES),
            ("titles", TITLES),
            ("weather", WEATHER),
            ("menu", crate::dat::ui::MENU),
            ("lobby", crate::dat::ui::LOBBY),
        ] {
            check(n, ids);
        }
        for r in crate::dat::items::RANGES {
            // 0xeff8 and 0xf600 are fixed files (FUN_1015b720 does not take them from a slot).
            if !matches!(r.0, 0xeff8 | 0xf600) {
                check("items", r.3);
            }
        }
        for ids in crate::dat::journal::MISSION_FILES.iter().chain(&crate::dat::journal::QUEST_FILES) {
            check("journal", *ids);
        }
        // The dialogue ranges' first files: 0x6e has none in German and French (0).
        for (k, s) in [0x6b, 0x6e, 0x6c, 0x6d].into_iter().enumerate() {
            assert_eq!(
                (0..4u8).map(|l| slot(l, s).unwrap()).collect::<Vec<_>>(),
                crate::dat::dialog::ZONE_FILES[k].to_vec()
            );
        }
        // FUN_1025cbd0's pointer table: found by a pointer to "menu    framesus" whose row and column all hold the
        // names (row 0 English is the table's second entry).
        let u32_at = |o: usize| u32::from_le_bytes(dll[o..o + 4].try_into().unwrap());
        let name_at = |p: u32| crate::dat::battle::va_bytes(&dll, p, 16).map(crate::dat::ui::norm_name);
        let fits = |at: usize| {
            LIBS.iter().enumerate().all(|(row, names)| {
                names.iter().enumerate().all(|(l, n)| {
                    dll.get(at + (row * 4 + l) * 4 + 3).is_some()
                        && name_at(u32_at(at + (row * 4 + l) * 4)).as_deref() == Some(*n)
                })
            })
        };
        let strings = dll.windows(17).enumerate().filter(|(_, w)| &w[..] == b"menu    framesus\0").map(|(o, _)| o);
        let found = strings.filter_map(|o| crate::dat::tables::va_of(&dll, o)).any(|va| {
            dll.windows(4).enumerate().filter(|(_, w)| *w == va.to_le_bytes()).any(|(o, _)| o >= 4 && fits(o - 4))
        });
        assert!(found, "FUN_1025cbd0's table");
    }
}

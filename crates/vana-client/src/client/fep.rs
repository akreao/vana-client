//! Auto-translate input in the chat line (FUN_101b5d10, English's direct path): Tab completes the word at the end
//! of the line from the dictionary into "menu    fep" (FUN_1018f550); a phrase goes in as one unit (`FD id FD`,
//! four private-use chars here, font::PHRASE_BASE). MODLOG "Auto-translate input".

use super::World;
use super::game_ui::GameUi;
use crate::audio::Ui as Se;
use crate::dat::font::{PHRASE_BASE, is_phrase};

/// Raw and expanded limits of the chat line (E+0xa54 = 0x78).
pub(super) const MAX: usize = 120;
/// Rows a page (FUN_1018f550).
pub(super) const ROWS: usize = 7;
/// The list's window (registry flags 0x0081000a: bit 0 clear, so it takes the focus while up; layer 7, over every
/// window of layer 4 or less). Its children: 1 the "n/m" line, 2..8 the page's rows, (3, 20 + 16 k) 68 x 16.
pub(super) const WINDOW: &str = "menu    fep";
/// The chat line's window (flags 0xa: it takes the focus too; layer 6). Its only part is the frame, so a focus that
/// moves to it puts the pointer nowhere (FUN_1011c080 finds no cursor part).
pub(super) const INLINE: &str = "menu    inline";

#[derive(Clone, Debug)]
pub struct Cand {
    pub text: String,
    /// A dictionary block; None for plain text (the typed word, a name).
    pub id: Option<[u8; 4]>,
}

#[derive(Debug)]
pub struct Fep {
    pub cands: Vec<Cand>,
    pub sel: usize,
    /// The category the list shows (T+0xec2c, its title's block id), and the root word and selection to go back to
    /// (DAT_10662d58 / d54).
    filter: Option<[u8; 4]>,
    root: (String, usize),
    /// The chars of the word being completed at the end of the line.
    word_len: usize,
    /// "menu    fep" as a window: its open hook, cursor and mouse.
    pub win: ListWin,
}

/// The list's window state (win_fep, object DAT_10663f54).
#[derive(Debug)]
pub struct ListWin {
    /// The window's open hook ran (win_fep_v04: FUN_1011a240(2, 1)) and moved its cursor to the first row: the
    /// pointer goes there at the next draw.
    pub warp: std::cell::Cell<bool>,
    /// The window's cursor child (+0x4c): 1 on a new window (FUN_101193d0), then the hook's 2, the hovered row, and
    /// the selection's row as each draw puts it back (win_fep_v10: FUN_1011a240(sel % 7 + 2, 0)).
    cursor: std::cell::Cell<u16>,
    /// +0x2201: message 9 reached the window's handler since the last update (win_fep_v28 clears it), so another one
    /// steps nothing.
    stepped: std::cell::Cell<bool>,
    /// The windows saw the pointer this frame (shown, no glide): a cursor move then sends message 9 (FUN_1011a240's
    /// test of mouse +0x48 and +0x4e).
    seen: std::cell::Cell<bool>,
}

impl ListWin {
    /// A new list window: the open hook still to run.
    pub(super) fn new() -> Self {
        Self {
            warp: std::cell::Cell::new(true),
            cursor: std::cell::Cell::new(1),
            stepped: std::cell::Cell::new(false),
            seen: std::cell::Cell::new(false),
        }
    }
}

/// The line's unit boundaries (FUN_1018fb00 / FUN_1018fa80): a phrase block is one unit, any other char one.
fn bounds(chars: &[char]) -> Vec<usize> {
    let mut out = vec![0];
    let mut i = 0;
    while i < chars.len() {
        i += if is_phrase(chars[i]) && i + 4 <= chars.len() { 4 } else { 1 };
        out.push(i);
    }
    out
}

/// The caret (E+0xa58) as a char index: None is the end of the line; never inside a block.
pub(super) fn caret(w: &World) -> usize {
    let chars: Vec<char> = w.chat_input.chars().collect();
    let at = w.chat_caret.map_or(chars.len(), |c| c.min(chars.len()));
    bounds(&chars).into_iter().filter(|&b| b <= at).max().unwrap_or(0)
}

fn set_caret(w: &mut World, at: usize) {
    w.chat_caret = (at < w.chat_input.chars().count()).then_some(at);
}

/// The selected chars (an addition: retail's line has no selection): between the anchor Shift left and the caret.
pub(super) fn selection(w: &World) -> Option<(usize, usize)> {
    let a = w.chat_anchor?.min(w.chat_input.chars().count());
    let c = caret(w);
    (a != c).then(|| (a.min(c), a.max(c)))
}

/// The selection as plain text (phrases written out), for the clipboard.
pub(super) fn selected_text(w: &World) -> Option<String> {
    let (s, e) = selection(w)?;
    Some(plain(w, &w.chat_input.chars().skip(s).take(e - s).collect::<String>()))
}

/// Removes the selection; false when there is none.
pub(super) fn delete_selection(w: &mut World) -> bool {
    let Some((s, e)) = selection(w) else { return false };
    w.chat_input = w.chat_input.chars().take(s).chain(w.chat_input.chars().skip(e)).collect();
    w.chat_anchor = None;
    set_caret(w, s);
    true
}

pub(super) fn select_all(w: &mut World) {
    w.chat_anchor = Some(0);
    w.chat_caret = None;
}

/// Left / Right (FUN_101ac4e0 / FUN_101ac520): one unit; Home / End (VK 0x24 / 0x23): the start / the end. With
/// Shift the selection grows from where the caret was.
pub(super) fn caret_key(w: &mut World, key: egui::Key, shift: bool) {
    let chars: Vec<char> = w.chat_input.chars().collect();
    let (at, b) = (caret(w), bounds(&chars));
    w.chat_anchor = if shift { Some(w.chat_anchor.unwrap_or(at)) } else { None };
    let to = match key {
        egui::Key::ArrowLeft => b.into_iter().filter(|&x| x < at).max().unwrap_or(0),
        egui::Key::ArrowRight => b.into_iter().find(|&x| x > at).unwrap_or(chars.len()),
        egui::Key::Home => 0,
        _ => chars.len(),
    };
    set_caret(w, to);
}

fn spliced(line: &str, at: usize, text: &str) -> String {
    line.chars().take(at).chain(text.chars()).chain(line.chars().skip(at)).collect()
}

/// Typed or pasted text at the caret (FUN_101abc50): in whole only while the raw and the expanded line stay within
/// 120 bytes.
pub(super) fn insert(w: &mut World, text: &str) {
    // A selection is replaced, when the result fits.
    let (line, at) = match selection(w) {
        Some((s, e)) => (w.chat_input.chars().take(s).chain(w.chat_input.chars().skip(e)).collect(), s),
        None => (w.chat_input.clone(), caret(w)),
    };
    let next = spliced(&line, at, text);
    let (raw, plain) = lengths(w, &next);
    if raw <= MAX && plain <= MAX {
        w.chat_input = next;
        w.chat_anchor = None;
        set_caret(w, at + text.chars().count());
    }
}

/// The word being completed: the chars after the last space at the end of the line (FUN_101b7c20).
fn word(line: &str) -> &str {
    let tail = line.rsplit(' ').next().unwrap_or("");
    if tail.chars().any(is_phrase) { "" } else { tail }
}

/// FUN_101b6ee0: the dictionary's candidates, then the typed word (plain) and the names that start with it (tell
/// history, party: FUN_1018d7f0).
fn query(w: &World, word: &str, filter: Option<[u8; 4]>) -> Vec<Cand> {
    let Some(d) = w.battle.as_ref().and_then(|b| b.phrases.as_ref()) else { return Vec::new() };
    // The root category list (no word, no category) builds the Items and Key Items groups again (FUN_101bd060 ->
    // FUN_101bcd10): the item record cache as it stands, and the key items we hold in the table's order.
    if filter.is_none() && word.is_empty() {
        let (recent, keys) = w.names.as_ref().map_or_else(Default::default, |n| {
            let keys: Vec<(u32, String, bool)> =
                n.key_item_list().into_iter().map(|(id, name)| (id, name, w.has_key_item(id as u16))).collect();
            (n.items.recent(), keys)
        });
        d.rebuild(&recent, &keys);
    }
    let (hits, matched) = d.query(word, filter);
    let mut out: Vec<Cand> = hits.into_iter().map(|(text, id)| Cand { text, id: Some(id) }).collect();
    if filter.is_none() && !word.is_empty() {
        if matched {
            out.push(Cand { text: word.to_string(), id: None });
        }
        let lw = word.to_lowercase();
        let party: Vec<String> = w.party.members().map(|m| m.name.clone()).collect();
        for n in w.tells.iter().chain(party.iter()) {
            if n.to_lowercase().starts_with(&lw) && !out.iter().any(|c| c.id.is_none() && c.text == *n) {
                out.push(Cand { text: n.clone(), id: None });
            }
        }
    }
    out
}

/// Tab / Pause with the list closed: open it on the word at the end of the line.
pub(super) fn open(w: &mut World) {
    let wd = word(&w.chat_input).to_string();
    let cands = query(w, &wd, None);
    if cands.is_empty() {
        return;
    }
    w.sounds.push(Se::Select as u32);
    // A new window's cursor is child 1 (FUN_101193d0), so the hook's move to child 2 always moves the pointer.
    w.fep = Some(Fep {
        cands,
        sel: 0,
        filter: None,
        root: (wd.clone(), 0),
        word_len: wd.chars().count(),
        win: ListWin::new(),
    });
}

/// The list opened again on new candidates (FUN_1015f920 on the open window: its hook win_fep_v04 again): the cursor
/// goes to child 2 and the pointer with it, unless the cursor was there already (the last draw left it on the row of
/// `was`, the selection before). The draw then puts it back on the selection's row with no pointer move.
fn reopened(was: usize, warp: &std::cell::Cell<bool>) {
    if !was.is_multiple_of(ROWS) {
        warp.set(true);
    }
}

/// The line's raw (Shift-JIS, a phrase 6 bytes) and expanded lengths (FUN_101abc50 / FUN_101acc30).
pub(super) fn lengths(w: &World, line: &str) -> (usize, usize) {
    let raw = crate::dat::font::encode_sjis(line).len();
    let plain = plain(w, line);
    (raw, crate::dat::font::encode_sjis(&plain).len())
}

/// The line with its phrases as plain text (no brackets).
pub(super) fn plain(w: &World, line: &str) -> String {
    plain_with(w, line, w.names.as_deref().map(|n| n as &dyn crate::dat::names::NameSource))
}

/// `plain` for a mod: its item lookups take no slot of the record cache (`names::Peek`).
pub(super) fn plain_peek(w: &World, line: &str) -> String {
    let peek = w.names.as_deref().map(crate::dat::names::Peek);
    plain_with(w, line, peek.as_ref().map(|n| n as &dyn crate::dat::names::NameSource))
}

fn plain_with(w: &World, line: &str, names: Option<&dyn crate::dat::names::NameSource>) -> String {
    match w.battle.as_ref().and_then(|b| b.phrases.as_ref()) {
        Some(d) => d.expand_plain(line, names),
        None => line.chars().filter(|c| !is_phrase(*c)).collect(),
    }
}

/// The line with its phrases between the auto-translate brackets (FUN_101b8850 with braces).
pub(super) fn braced(w: &World, line: &str) -> String {
    braced_cap(w, line, usize::MAX)
}

/// `braced` into a `cap`-byte buffer (FUN_101b8850's fourth argument; `Dictionary::expand_cap`).
pub(super) fn braced_cap(w: &World, line: &str, cap: usize) -> String {
    let names = w.names.clone();
    match w.battle.as_ref().and_then(|b| b.phrases.as_ref()) {
        Some(d) => d.expand_cap(line, names.as_deref().map(|n| n as &dyn crate::dat::names::NameSource), cap),
        None => line.chars().filter(|c| !is_phrase(*c)).collect(),
    }
}

/// A key while the list is open; true when it was taken.
pub(super) fn key(w: &mut World, key: egui::Key, shift: bool) -> bool {
    let Some(mut f) = w.fep.take() else { return false };
    let n = f.cands.len().max(1);
    match key {
        egui::Key::Tab | egui::Key::Space | egui::Key::ArrowDown if !(shift && key == egui::Key::Tab) => {
            f.sel = (f.sel + 1) % n
        }
        egui::Key::Tab | egui::Key::ArrowUp => f.sel = (f.sel + n - 1) % n,
        egui::Key::PageDown => f.sel = (f.sel + ROWS).min(n - 1),
        egui::Key::PageUp => f.sel = f.sel.saturating_sub(ROWS),
        // These move the Japanese composition cursor (FUN_101b4fc0 / FUN_101b5020); nothing in English.
        egui::Key::ArrowLeft | egui::Key::ArrowRight | egui::Key::Home | egui::Key::End => {
            w.fep = Some(f);
            return true;
        }
        egui::Key::Escape | egui::Key::Backspace | egui::Key::Delete => {
            // A category goes back to the root list (FUN_101b84b0 then FUN_1015f920), else the list closes.
            if f.filter.take().is_some() {
                reopened(f.sel, &f.win.warp);
                let (wd, sel) = f.root.clone();
                f.cands = query(w, &wd, None);
                f.sel = sel.min(f.cands.len().saturating_sub(1));
            } else {
                w.sounds.push(Se::Cancel as u32);
                return true;
            }
        }
        egui::Key::Enter => {
            let Some(c) = f.cands.get(f.sel).cloned() else { return true };
            // The word being completed goes from the end of the line (FUN_101ad0d0), then the candidate in at the
            // caret (FUN_101b6c00).
            let keep = w.chat_input.chars().count().saturating_sub(f.word_len);
            let line: String = w.chat_input.chars().take(keep).collect();
            let at_end = caret(w) >= keep;
            let at = caret(w).min(keep);
            let put = |w: &mut World, line: String, n: usize| {
                w.chat_input = line;
                w.chat_caret = if at_end { None } else { Some(at + n) };
            };
            match c.id {
                None => put(w, spliced(&line, at, &c.text), c.text.chars().count()),
                Some(id) if id[3] == 0 => {
                    // A category (FUN_101ae200: the id's last byte is 0, which no item or key item block has): nothing
                    // goes in; the list reopens on its phrases.
                    put(w, line, 0);
                    // menu_fep_fn_8: the category's list, then FUN_1015f920.
                    reopened(f.sel, &f.win.warp);
                    if f.filter.is_none() {
                        f.root.1 = f.sel;
                    }
                    f.filter = Some(id);
                    f.word_len = 0;
                    f.cands = query(w, "", f.filter);
                    f.sel = 0;
                    w.sounds.push(Se::Select as u32);
                    w.fep = Some(f);
                    return true;
                }
                Some(id) => {
                    let block: String = id
                        .iter()
                        .enumerate()
                        .map(|(k, &b)| char::from_u32(PHRASE_BASE + 0x100 * k as u32 + b as u32).unwrap())
                        .collect();
                    let next = spliced(&line, at, &block);
                    let (raw, plain) = lengths(w, &next);
                    if raw > MAX || plain > MAX {
                        w.sounds.push(Se::Error as u32);
                        return true;
                    }
                    put(w, next, 4);
                }
            }
            w.sounds.push(Se::Select as u32);
            return true;
        }
        _ => {
            w.fep = Some(f);
            return false;
        }
    }
    w.sounds.push(Se::Cursor as u32);
    w.fep = Some(f);
    true
}

/// Backspace / Delete with the list closed (FUN_101abef0 / FUN_101abf40(1 / 0)): the unit before / at the caret goes;
/// a phrase becomes its plain text minus its last / first letter.
pub(super) fn erase(w: &mut World, back: bool) {
    if delete_selection(w) {
        return;
    }
    let chars: Vec<char> = w.chat_input.chars().collect();
    let (at, b) = (caret(w), bounds(&chars));
    let unit = if back { b.windows(2).find(|u| u[1] == at) } else { b.windows(2).find(|u| u[0] == at) };
    let Some(&[s, e]) = unit else { return };
    let mut text = String::new();
    if e - s == 4 {
        text = plain(w, &chars[s..e].iter().collect::<String>());
        if back {
            text.pop();
        } else if !text.is_empty() {
            text.remove(0);
        }
    }
    let n = text.chars().count();
    w.chat_input = chars[..s].iter().collect::<String>() + &text + &chars[e..].iter().collect::<String>();
    if w.chat_caret.is_some() {
        set_caret(w, if back { s + n } else { s });
    }
}

/// FUN_101ad4b0: a submitted line's first word has its phrase expanded (so 【Text Commands】 phrases work); a chat
/// command (table 0x1037ab98) keeps the rest's blocks, any other command gets plain text; a line without a command
/// (said in the chat mode) keeps its blocks.
pub(super) fn outgoing(w: &World, line: String) -> String {
    if !line.chars().any(is_phrase) {
        return line;
    }
    let lead = line.len() - line.trim_start().len();
    let (first, rest) = line[lead..].split_once(' ').map_or((&line[lead..], ""), |(a, b)| (a, b));
    let first = plain(w, first);
    if !first.starts_with('/') {
        return line;
    }
    const CHAT: [&str; 34] = [
        "/say",
        "/s",
        "/shout",
        "/sh",
        "/tell",
        "/t",
        "/party",
        "/p",
        "/linkshell",
        "/l",
        "/linkshell2",
        "/l2",
        "/unity",
        "/u",
        "/assistj",
        "/aj",
        "/assiste",
        "/ae",
        "/emote",
        "/em",
        "/message",
        "/me",
        "/yell",
        "/yl",
        "//worldmes",
        "//wm",
        "/linkshellmes",
        "/lsmes",
        "/linkshell2mes",
        "/ls2mes",
        "/nominate",
        "/propose",
        "/seacom",
        "/sc",
    ];
    if CHAT.contains(&first.to_lowercase().as_str()) {
        format!("{first} {rest}")
    } else {
        plain(w, &format!("{first} {rest}"))
    }
}

/// The line being typed as drawn (FUN_101b5470): plain runs and phrases (coloured, no brackets) as (text, phrase).
pub(super) fn pieces(w: &World, line: &str) -> Vec<(String, bool)> {
    let mut out: Vec<(String, bool)> = Vec::new();
    let mut block = String::new();
    for ch in line.chars() {
        if is_phrase(ch) {
            block.push(ch);
            if block.chars().count() == 4 {
                out.push((plain(w, &block), true));
                block.clear();
            }
        } else {
            match out.last_mut() {
                Some((t, false)) => t.push(ch),
                _ => out.push((ch.to_string(), false)),
            }
        }
    }
    out
}

/// Tab completion in a text field other than the chat line (ours: retail's FEP there is the Japanese IME's; the
/// macro editor's title and lines and the comment editor's lines are FUN_101c5be0 mode 8 / 9 boxes, which take phrase
/// blocks): the same list, Tab / Space / arrows / Page keys to move, Enter to put the pick in place of the word before
/// the caret, Escape / Backspace to back out of a category or close.
#[derive(Debug)]
pub struct FieldFep {
    pub cands: Vec<Cand>,
    pub sel: usize,
    filter: Option<[u8; 4]>,
    root: (String, usize),
    /// The word being completed (the chars after the last space before the caret).
    pub word: String,
    /// As `Fep::win` (ours takes no mouse on these lists).
    pub win: ListWin,
}

/// What a key did to a field's list.
#[derive(Debug, PartialEq)]
pub(super) enum FieldPick {
    /// Not the list's key: the field takes it (the list stays).
    Pass,
    /// Taken; the list stays open.
    Keep,
    /// Taken; the list closes.
    Close,
    /// Put this text (a plain word or a phrase block) in place of the word.
    Put(String),
}

/// Tab in a field: the list on the word before the caret (`before`, the field's text up to it); None when nothing
/// matches.
pub(super) fn field_open(w: &mut World, before: &str) -> Option<FieldFep> {
    let wd = word(before).to_string();
    let cands = query(w, &wd, None);
    if cands.is_empty() {
        w.sounds.push(Se::Error as u32);
        return None;
    }
    w.sounds.push(Se::Select as u32);
    Some(FieldFep { cands, sel: 0, filter: None, root: (wd.clone(), 0), word: wd, win: ListWin::new() })
}

/// A phrase id as the four private-use chars a line holds (font::PHRASE_BASE).
pub(super) fn block(id: [u8; 4]) -> String {
    id.iter().enumerate().map(|(k, &b)| char::from_u32(PHRASE_BASE + 0x100 * k as u32 + b as u32).unwrap()).collect()
}

/// A key while a field's list is open (as `key`'s list keys).
pub(super) fn field_key(w: &mut World, f: &mut FieldFep, key: egui::Key, shift: bool) -> FieldPick {
    let n = f.cands.len().max(1);
    match key {
        egui::Key::Tab | egui::Key::Space | egui::Key::ArrowDown if !(shift && key == egui::Key::Tab) => {
            f.sel = (f.sel + 1) % n
        }
        egui::Key::Tab | egui::Key::ArrowUp => f.sel = (f.sel + n - 1) % n,
        egui::Key::PageDown => f.sel = (f.sel + ROWS).min(n - 1),
        egui::Key::PageUp => f.sel = f.sel.saturating_sub(ROWS),
        egui::Key::ArrowLeft | egui::Key::ArrowRight | egui::Key::Home | egui::Key::End => return FieldPick::Keep,
        egui::Key::Escape | egui::Key::Backspace | egui::Key::Delete => {
            if f.filter.take().is_some() {
                reopened(f.sel, &f.win.warp);
                let (wd, sel) = f.root.clone();
                f.cands = query(w, &wd, None);
                f.sel = sel.min(f.cands.len().saturating_sub(1));
            } else {
                w.sounds.push(Se::Cancel as u32);
                return FieldPick::Close;
            }
        }
        egui::Key::Enter => {
            let Some(c) = f.cands.get(f.sel).cloned() else { return FieldPick::Close };
            return match c.id {
                None => FieldPick::Put(c.text),
                // A category: its phrases replace the list (the word stays until a phrase is put).
                Some(id) if id[3] == 0 => {
                    reopened(f.sel, &f.win.warp);
                    if f.filter.is_none() {
                        f.root.1 = f.sel;
                    }
                    f.filter = Some(id);
                    f.cands = query(w, "", f.filter);
                    f.sel = 0;
                    w.sounds.push(Se::Select as u32);
                    FieldPick::Keep
                }
                Some(id) => FieldPick::Put(block(id)),
            };
        }
        _ => return FieldPick::Pass,
    }
    w.sounds.push(Se::Cursor as u32);
    FieldPick::Keep
}

/// `text` with `word` (ending at char `caret`) replaced by `put`; the new text and caret. None when `fits` refuses it.
pub(super) fn field_put(
    text: &str,
    caret: usize,
    word: &str,
    put: &str,
    fits: impl Fn(&str) -> bool,
) -> Option<(String, usize)> {
    let n = word.chars().count().min(caret);
    let before: String = text.chars().take(caret - n).collect();
    let after: String = text.chars().skip(caret).collect();
    let next = format!("{before}{put}{after}");
    fits(&next).then(|| (next, caret - n + put.chars().count()))
}

/// "menu    fep" (FUN_1018f550): above the chat line at the word's start (inf: where retail puts it); width the widest
/// row, 90..200 + 12; the header "%d/%d" at (+0x23, 0) after the ▲ at (+5, 0); seven rows 16 px apart; a row too wide
/// cut with "...".
pub(super) fn draw(gu: &mut GameUi, f: &Fep, x: f32, line_y: f32) -> [f32; 4] {
    draw_list(gu, &f.cands, f.sel, &f.win, x, line_y)
}

/// The list's draw for any field: above `line_y` from `x`. The hand is the window's cursor: child 2 + the row
/// (win_fep_v10: FUN_1011a240(sel % 7 + 2, 0)), at that child's rect; `win.warp` the open hook's pointer move to
/// child 2.
pub(super) fn draw_list(gu: &mut GameUi, cands: &[Cand], sel: usize, win: &ListWin, x: f32, line_y: f32) -> [f32; 4] {
    let widest = cands.iter().map(|c| gu.text_width(&c.text)).fold(0.0, f32::max);
    let w = (widest.max(90.0) + 12.0).min(200.0);
    let h = 134.0;
    let y = line_y - h - 2.0;
    gu.frame(x, y, w, h);
    const C: [u8; 4] = [0x80; 4];
    gu.text("\u{25b2}", x + 5.0, y + 4.0, 0, C);
    gu.text(&format!("{}/{}", if cands.is_empty() { 0 } else { sel + 1 }, cands.len()), x + 35.0, y + 4.0, 0, C);
    gu.want(WINDOW);
    let first = sel / ROWS * ROWS;
    // win_fep_v28, each frame after the mouse: the step mark goes.
    win.stepped.set(false);
    // The open hook's FUN_1011a240(2, 1) ran at the open (before the update), so its message 9 marks nothing that
    // lasts; its pointer move glides, so the draw's cursor move below sends none.
    let warped = win.warp.take();
    if warped {
        win.cursor.set(2);
        gu.warp_to_part(row_rect(x, y, 0));
    }
    // win_fep_v10's FUN_1011a240(sel % 7 + 2, 0): a cursor elsewhere (a hovered row the steps did not reach) comes
    // back to the selection's row, and with the pointer seen that sends message 9, which steps nothing (the row is
    // the selection's) but marks the window until the next update: the next frame's mouse steps nothing.
    let row = (sel % ROWS + 2) as u16;
    if win.cursor.replace(row) != row && win.seen.get() && !warped {
        win.stepped.set(true);
    }
    for (k, c) in cands.iter().enumerate().skip(first).take(ROWS) {
        let mut t = c.text.clone();
        // The text from x + 6 (inf: retail cuts at the width - 15 from its start).
        if gu.text_width(&t) > w - 12.0 {
            while !t.is_empty() && gu.text_width(&format!("{t}...")) > w - 12.0 {
                t.pop();
            }
            t = format!("{}...", t.trim_end());
        }
        let ry = y + 20.0 + 16.0 * (k - first) as f32;
        gu.text(&t, x + 6.0, ry, 0, C);
        if k == sel {
            gu.cursor_on(WINDOW, 2 + (k - first) as u16, x + 3.0, ry, row_rect(x, y, k - first));
        }
    }
    [x, y, w, h]
}

/// The mouse on the chat line's list (classic layer; the modern layer's egui list takes its own), once a frame before
/// the draw. FUN_10160fd0 runs the release (FUN_1015f420), the held step (FUN_1015f360), then the hover (tail of
/// FUN_101267f0) on the focused window, then the windows' updates (win_fep_v28), then the draw:
/// - the release on the cursor's child is message 5 (win_fep_onkey: command 5 -> menu_fep_fn_6(0xd, 1, 0), the list's
///   Enter on the selection; the child is not read in English); on another child nothing;
/// - the hover or the held button moving the cursor to child c sends message 9 (FUN_1011a240), which steps the
///   selection to c's row (`hover_step`);
/// - no right click: message 6 closes no window with flag 2 set (FUN_101196f0) and the handler takes none.
pub(super) fn pointer(w: &mut World, gu: &GameUi, p: Option<(f32, f32)>, released: bool) {
    let Some(f) = w.fep.as_ref() else { return };
    f.win.seen.set(false);
    let cursor = f.win.cursor.get();
    // The mouse reaches the focused window only (FUN_10160380).
    let focus = crate::dat::ui::norm_name(WINDOW.as_bytes());
    let Some(p) = p.filter(|_| !w.modern && gu.focused() == Some(focus.as_str()) && super::winhits::shown(gu)) else {
        return;
    };
    f.win.seen.set(true);
    let (child, on) = super::winhits::child_under(w, WINDOW, p, 1..=8, super::winhits::parts(gu, WINDOW));
    let Some(c) = child else { return };
    let takes = super::winhits::takes(gu, WINDOW, c, on);
    if released && c == cursor {
        key(w, egui::Key::Enter, false);
    } else if c != cursor && takes {
        let Some(f) = w.fep.as_mut() else { return };
        f.win.cursor.set(c);
        let was = f.sel;
        hover_step(f, c);
        // win_fep_v28: a selection other than the last update's plays SE 0001 (FUN_10037b30).
        if f.sel != was {
            w.sounds.push(Se::Cursor as u32);
        }
    }
}

/// win_fep_onkey message 9 on child `c`, once between updates (+0x2201): from the selection's row (sel % 7 + 2) the
/// list's next (command 2 -> FUN_101b8670, which wraps to the first) up to c while the count (taken before, not
/// wrapped) stays under the candidates, or its previous (command 1 -> FUN_101b86a0, wrapping to the last) down to c.
/// A row past the last candidate on the last page steps on past it and wraps to the first page; child 1 (the "n/m"
/// line) steps back past the page's first row.
fn hover_step(f: &mut Fep, c: u16) {
    if f.win.stepped.replace(true) {
        return;
    }
    let n = f.cands.len();
    let (c, mut sel, mut row) = (c as usize, f.sel, f.sel % ROWS + 2);
    while row < c && sel < n {
        f.sel = (f.sel + 1) % n;
        (row, sel) = (row + 1, sel + 1);
    }
    for _ in c..row {
        f.sel = (f.sel + n - 1) % n;
    }
}

/// The list's parts as drawn at `frame` (`draw`'s rect) for the mouse: rows 2..8 at `row_rect` and the "n/m" line
/// from the layout. win_fep_v28 sets the frame's size (FUN_1011ba30), which the held step's frame test reads; the
/// rows past the last candidate keep their places and take the mouse (the draw hides no child).
pub(super) fn record(w: &World, gu: &GameUi, frame: [f32; 4]) {
    let rows: Vec<(u16, [f32; 4])> = (0..ROWS).map(|r| (2 + r as u16, row_rect(frame[0], frame[1], r))).collect();
    super::winhits::record_all(w, gu, WINDOW, frame, &rows);
}

/// Row `r`'s child (2 + r) of the list at (x, y): (3, 20 + 16 r), 68 x 16 in the layout (the frame's width set by
/// win_fep_v28 leaves the children as they are).
fn row_rect(x: f32, y: f32, r: usize) -> [f32; 4] {
    [x + 3.0, y + 20.0 + 16.0 * r as f32, 68.0, 16.0]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn units_and_splices() {
        let p = |b: u32| char::from_u32(PHRASE_BASE + b).unwrap();
        let line: Vec<char> = vec!['a', p(2), p(0x101), p(0x203), p(0x300), 'b'];
        assert_eq!(bounds(&line), vec![0, 1, 5, 6]);
        assert_eq!(spliced("abcdef", 3, "X"), "abcXdef");
        assert_eq!(spliced("ab", 2, "c"), "abc");
    }

    /// win_fep_v04's FUN_1011a240(2, 1) moves the pointer only when the cursor was off the first row; the rows are
    /// children 2..8 at (3, 20 + 16 r), 68 x 16.
    #[test]
    fn reopen_warps_off_the_first_row() {
        let warp = std::cell::Cell::new(false);
        reopened(7, &warp);
        assert!(!warp.get());
        reopened(9, &warp);
        assert!(warp.get());
        assert_eq!(row_rect(100.0, 50.0, 2), [103.0, 102.0, 68.0, 16.0]);
    }

    fn list(n: usize, sel: usize) -> Fep {
        Fep {
            cands: (0..n).map(|k| Cand { text: k.to_string(), id: None }).collect(),
            sel,
            filter: None,
            root: (String::new(), 0),
            word_len: 0,
            win: ListWin::new(),
        }
    }

    /// win_fep_onkey message 9: the selection steps to the hovered row (next / previous, wrapping), once between
    /// updates; the count bound is taken before the wrap.
    #[test]
    fn hover_steps_to_the_row() {
        let mut f = list(10, 1);
        hover_step(&mut f, 6);
        assert_eq!(f.sel, 4);
        // Marked until the update: a second message 9 steps nothing.
        hover_step(&mut f, 2);
        assert_eq!(f.sel, 4);
        f.win.stepped.set(false);
        hover_step(&mut f, 2);
        assert_eq!(f.sel, 0);
        // The last page (7..9): a row past the last candidate steps on and wraps to the first.
        let mut f = list(10, 8);
        hover_step(&mut f, 8);
        assert_eq!(f.sel, 0);
        // Child 1, the "n/m" line, steps back past the page's first row (wrapping to the last).
        let mut f = list(10, 0);
        hover_step(&mut f, 1);
        assert_eq!(f.sel, 9);
        let mut f = list(10, 9);
        hover_step(&mut f, 1);
        assert_eq!(f.sel, 6);
    }

    #[test]
    fn field_puts() {
        assert_eq!(field_put("/p hel x", 6, "hel", "Hello!", |_| true), Some(("/p Hello! x".to_string(), 9)));
        assert_eq!(field_put("hel", 3, "hel", "Hello!", |t| t.len() <= 4), None);
        let b = block([2, 1, 3, 0]);
        assert_eq!(b.chars().count(), 4);
        assert_eq!(field_put("go", 2, "go", &b, |_| true).map(|r| r.1), Some(4));
    }
}

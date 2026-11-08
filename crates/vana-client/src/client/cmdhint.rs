//! Ours: the chat line's hint for the mods' text commands. While a line starting with `/` is typed, the running mods'
//! command forms (their mod.toml `[commands]`) that what is typed fits, each with its mod and help, over the chat line
//! where the auto-translate list goes: literal words by prefix, a `<word>` or `[word]` taking anything. Up / Down pick a
//! row, Tab puts its fixed words on the line; Enter stays the line's.

use super::World;
use super::game_ui::GameUi;
use crate::audio::Ui as Se;

/// A row: the form as typed (`//loot list`), its mod, its help.
#[derive(Clone, Debug, PartialEq)]
pub struct Row {
    pub text: String,
    pub owner: String,
    pub help: String,
}

/// What the hint shows: its rows and the one picked.
#[derive(Clone, Debug, PartialEq)]
pub struct Hint {
    pub rows: Vec<Row>,
    pub sel: Option<usize>,
}

/// The most rows shown.
pub const ROWS: usize = 7;

/// A line's slashes ("/" or "//") and the rest.
fn split(line: &str) -> Option<(&str, &str)> {
    let slashes = if line.starts_with("//") {
        "//"
    } else if line.starts_with('/') {
        "/"
    } else {
        return None;
    };
    Some((slashes, &line[slashes.len()..]))
}

/// A form's words: a `<...>` or `[...]` one word with what it holds (`[lot | pass | drop]`).
pub(super) fn words(form: &str) -> Vec<&str> {
    let mut out = Vec::new();
    let (mut depth, mut start) = (0i32, None);
    for (i, c) in form.char_indices() {
        match c {
            '<' | '[' => depth += 1,
            '>' | ']' => depth = (depth - 1).max(0),
            _ => {}
        }
        if c.is_whitespace() && depth == 0 {
            if let Some(s) = start.take() {
                out.push(&form[s..i]);
            }
        } else if start.is_none() {
            start = Some(i);
        }
    }
    if let Some(s) = start {
        out.push(&form[s..]);
    }
    out
}

fn literal(word: &str) -> bool {
    !word.starts_with('<') && !word.starts_with('[')
}

/// Whether what is typed (the line after its slashes) fits a form (`name words`): each typed word its literal word
/// (the last one, still being typed, by prefix) or any word for a placeholder; past the form's end only when it ends
/// in a placeholder (the rest of the line). How many typed words met literal ones, when it fits.
fn fits(typed: &str, form: &str) -> Option<usize> {
    let fw = words(form);
    let tw: Vec<&str> = typed.split_whitespace().collect();
    let open = !typed.ends_with(char::is_whitespace);
    let mut met = 0;
    for (i, t) in tw.iter().enumerate() {
        let Some(f) = fw.get(i) else { return fw.last().filter(|l| !literal(l)).map(|_| met) };
        if literal(f) {
            let (f, t) = (f.to_ascii_lowercase(), t.to_ascii_lowercase());
            let ok = if open && i + 1 == tw.len() { f.starts_with(&t) } else { f == t };
            if !ok {
                return None;
            }
            met += 1;
        }
    }
    Some(met)
}

/// A form's fixed words: its literal words before the first placeholder.
fn fixed(form: &str) -> String {
    words(form).into_iter().take_while(|w| literal(w)).collect::<Vec<_>>().join(" ")
}

/// One slash starts every game command (/p, /sh, /ma ...): the hint waits for three letters there; `//` is the mods'.
fn enough(slashes: &str, typed: &str) -> bool {
    slashes == "//" || typed.split_whitespace().next().is_some_and(|w| w.chars().count() >= 3)
}

/// Every form (name and words, unslashed) the line before the caret fits, with its mod and help: those whose literal
/// words met more of the typed ones first.
fn rows(w: &World) -> Option<(String, Vec<(String, String, String)>)> {
    if !w.typing || w.fep.is_some() {
        return None;
    }
    let before: String = w.chat_input.chars().take(super::fep::caret(w)).collect();
    let (slashes, typed) = split(&before)?;
    if !enough(slashes, typed) {
        return None;
    }
    let mut out = Vec::new();
    for c in w.lua.command_list() {
        for (a, h) in &c.forms {
            let form = if a.is_empty() { c.name.clone() } else { format!("{} {a}", c.name) };
            if let Some(met) = fits(typed, &form) {
                out.push((met, (form, c.owner.clone(), h.clone())));
            }
        }
    }
    out.sort_by_key(|r| std::cmp::Reverse(r.0));
    (!out.is_empty()).then(|| (slashes.to_string(), out.into_iter().map(|r| r.1).collect()))
}

/// The row Up / Down picked, while the line is still what it was then.
fn picked(w: &World) -> Option<usize> {
    w.cmd_tab.as_ref().filter(|t| t.1 == w.chat_input).map(|t| t.0)
}

/// The hint for the line as typed up to the caret, if any.
pub(super) fn hint(w: &World) -> Option<Hint> {
    let (slashes, found) = rows(w)?;
    let rows = found.into_iter().map(|(text, owner, help)| Row { text: format!("{slashes}{text}"), owner, help });
    Some(Hint { rows: rows.collect(), sel: picked(w) })
}

/// Up / Down while the hint lists two rows or more: the pick moves (wrapping); otherwise the line history's.
pub(super) fn arrow(w: &mut World, up: bool) -> bool {
    let Some((_, found)) = rows(w).filter(|r| r.1.len() > 1) else { return false };
    let n = found.len();
    let next = match picked(w) {
        Some(i) if up => (i + n - 1) % n,
        Some(i) => (i + 1) % n,
        None if up => n - 1,
        None => 0,
    };
    w.cmd_tab = Some((next, w.chat_input.clone()));
    w.sounds.push(Se::Cursor as u32);
    true
}

/// Tab: the picked row's (else the first's) fixed words on the line, a space after them when words follow; when the
/// line already has them, the next row is picked (Shift+Tab: the one before). False when no form fits or the caret is
/// not at the end (the auto-translate list opens instead).
pub(super) fn tab(w: &mut World, back: bool) -> bool {
    if super::fep::caret(w) != w.chat_input.chars().count() {
        return false;
    }
    let Some((slashes, found)) = rows(w) else { return false };
    let at = picked(w).unwrap_or(0).min(found.len() - 1);
    let form = &found[at].0;
    let fix = fixed(form);
    let more = words(form).len() > words(&fix).len();
    let want = format!("{slashes}{fix}{}", if more { " " } else { "" });
    let (_, typed) = split(&w.chat_input).unwrap_or(("", ""));
    if want.len() > w.chat_input.len() && want.to_ascii_lowercase().starts_with(&w.chat_input.to_ascii_lowercase())
        || typed.trim().is_empty()
    {
        w.chat_input = want;
        w.chat_caret = None;
        w.cmd_tab = None;
    } else if found.len() > 1 {
        let n = found.len();
        let next = if back { (at + n - 1) % n } else { (at + 1) % n };
        w.cmd_tab = Some((next, w.chat_input.clone()));
    } else {
        return false;
    }
    w.sounds.push(Se::Cursor as u32);
    true
}

/// The rows shown: the page of ROWS holding the pick.
pub fn page(h: &Hint) -> (usize, &[Row]) {
    let first = h.sel.map_or(0, |s| s / ROWS * ROWS);
    (first, &h.rows[first..h.rows.len().min(first + ROWS)])
}

/// A row's note: its mod and help.
pub fn note(r: &Row, sep: &str) -> String {
    if r.help.is_empty() { r.owner.clone() } else { format!("{}{sep}{}", r.owner, r.help) }
}

/// The classic hint: a framed list above the chat line from `x` on a dark backing, the form white (the pick gold), its
/// mod and help grey; "n more" under a full page.
pub(super) fn draw(gu: &mut GameUi, h: &Hint, x: f32, line_y: f32) -> [f32; 4] {
    const WHITE: [u8; 4] = [0x80; 4];
    const GOLD: [u8; 4] = [0x40, 0x70, 0x80, 0x80];
    const GREY: [u8; 4] = [0x58, 0x58, 0x58, 0x80];
    let (first, rows) = page(h);
    let notes: Vec<String> = rows.iter().map(|r| note(r, ": ")).collect();
    let left = rows.iter().map(|r| gu.text_width(&r.text)).fold(0.0, f32::max);
    let right = notes.iter().map(|n| gu.text_width(n)).fold(0.0, f32::max);
    let gap = if right > 0.0 { 18.0 } else { 0.0 };
    let w = (left + gap + right).clamp(120.0, 720.0) + 12.0;
    let more = h.rows.len() > ROWS;
    let lines = rows.len() + usize::from(more);
    let hh = 8.0 + 16.0 * lines as f32;
    let y = line_y - hh - 2.0;
    // A dark backing: the frame is translucent and sits over the log's lines.
    gu.rect(x, y, w, hh, [0.02, 0.02, 0.05, 0.85]);
    gu.frame(x, y, w, hh);
    let fit = |gu: &mut GameUi, s: &str, tx: f32, ty: f32, max: f32, c: [u8; 4]| {
        let mut t = s.to_string();
        if gu.text_width(&t) > max {
            while !t.is_empty() && gu.text_width(&format!("{t}...")) > max {
                t.pop();
            }
            t = format!("{}...", t.trim_end());
        }
        gu.text(&t, tx, ty, 0, c);
    };
    for (k, (r, n)) in rows.iter().zip(&notes).enumerate() {
        let ry = y + 4.0 + 16.0 * k as f32;
        fit(gu, &r.text, x + 6.0, ry, w - 12.0, if h.sel == Some(first + k) { GOLD } else { WHITE });
        let hx = x + 6.0 + left + gap;
        fit(gu, n, hx, ry, x + w - 6.0 - hx, GREY);
    }
    if more {
        let ry = y + 4.0 + 16.0 * rows.len() as f32;
        fit(gu, &format!("{} of {} (Up / Down)", first + rows.len(), h.rows.len()), x + 6.0, ry, w - 12.0, GREY);
    }
    [x, y, w, hh]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn forms_fit() {
        assert_eq!(words("loot [lot | pass | drop] add <item>"), ["loot", "[lot | pass | drop]", "add", "<item>"]);
        assert_eq!(words("cancel <effect>[, <effect>...]"), ["cancel", "<effect>[, <effect>...]"]);
        assert_eq!(
            (fits("lo", "loot list"), fits("loot l", "loot list"), fits("loot x", "loot list")),
            (Some(1), Some(2), None)
        );
        // A placeholder takes any word, meeting no literal one; past the form's end only after a placeholder.
        assert_eq!(fits("loot lot add Kupo", "loot [lot | pass | drop] add <item>"), Some(2));
        assert_eq!((fits("find red mush", "find <words>"), fits("hb show x", "hb show")), (Some(1), None));
        assert_eq!(
            (fits("", "hb"), fits("trusts s", "trusts save <lineup>"), fits("trusts s", "trusts list")),
            (Some(0), Some(2), None)
        );
        assert_eq!(fits("trusts save ", "trusts <lineup>"), Some(1));
        assert_eq!(fixed("trusts save <lineup> [trust, trust...]"), "trusts save");
        assert_eq!(fixed("hb"), "hb");
        assert_eq!(split("//hb sh"), Some(("//", "hb sh")));
        assert!(enough("//", "") && !enough("/", "p") && enough("/", "rid"));
    }
}

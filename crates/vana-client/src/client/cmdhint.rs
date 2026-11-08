//! Ours: the chat line's hint for the mods' text commands. While a line starting with `/` is typed, the running mods'
//! commands its first word starts with, each with its help, over the line where the auto-translate list goes; once a
//! command is named and a space follows, that command's forms (its mod.toml `[commands]`). Tab completes the name,
//! again the next match; the hint never takes Enter or the arrows.

use super::World;
use super::game_ui::GameUi;
use super::lua::CmdInfo;
use crate::audio::Ui as Se;

/// What the hint shows: rows of (the command as it would be typed, its help), the row Tab put on the line, and for one
/// command's forms its help and mod under them.
#[derive(Clone, Debug, PartialEq)]
pub struct Hint {
    pub rows: Vec<(String, String)>,
    pub sel: Option<usize>,
    pub footer: Option<String>,
}

/// The most rows shown.
pub const ROWS: usize = 7;

/// A line's slashes ("/" or "//"), its first word, and whether a space follows it.
fn split(line: &str) -> Option<(&str, &str, bool)> {
    let slashes = if line.starts_with("//") {
        "//"
    } else if line.starts_with('/') {
        "/"
    } else {
        return None;
    };
    let rest = &line[slashes.len()..];
    Some(match rest.find(' ') {
        Some(i) => (slashes, &rest[..i], true),
        None => (slashes, rest, false),
    })
}

fn typed(slashes: &str, name: &str, form: &str) -> String {
    if form.is_empty() { format!("{slashes}{name}") } else { format!("{slashes}{name} {form}") }
}

/// One slash starts every game command (/p, /sh, /ma ...): the hint waits for three letters there; `//` is the mods'.
fn enough(slashes: &str, word: &str) -> bool {
    slashes == "//" || word.chars().count() >= 3
}

/// The commands a first word's root names, and the row Tab is on, while the line is still what Tab made it.
fn matches(w: &World, list: &[CmdInfo], word: &str) -> (Vec<usize>, Option<usize>) {
    let tab = w.cmd_tab.as_ref().filter(|t| t.2 == w.chat_input);
    let root = tab.map_or_else(|| word.to_ascii_lowercase(), |t| t.0.clone());
    let found = list.iter().enumerate().filter(|(_, c)| c.name.starts_with(&root)).map(|(i, _)| i).collect();
    (found, tab.map(|t| t.1))
}

/// The hint for the line as typed up to the caret, if any.
pub(super) fn hint(w: &World) -> Option<Hint> {
    if !w.typing || w.fep.is_some() {
        return None;
    }
    let before: String = w.chat_input.chars().take(super::fep::caret(w)).collect();
    let (slashes, word, spaced) = split(&before)?;
    let list = w.lua.command_list();
    // One command's forms, its help and mod under them.
    let forms = |c: &CmdInfo| {
        let rows = c.usage.iter().map(|u| (typed(slashes, &c.name, u), String::new())).collect();
        let footer = if c.help.is_empty() { c.owner.clone() } else { format!("{} ({})", c.help, c.owner) };
        Hint { rows, sel: None, footer: Some(footer) }
    };
    if spaced {
        return list.iter().find(|c| c.name == word.to_ascii_lowercase()).map(forms);
    }
    if !enough(slashes, word) {
        return None;
    }
    let (found, sel) = matches(w, &list, word);
    match found.len() {
        0 => return None,
        // The only match: its forms at once.
        1 => return Some(forms(&list[found[0]])),
        _ => {}
    }
    let rows = found
        .iter()
        .map(|&i| {
            let c = &list[i];
            let form = if c.usage.len() == 1 { c.usage[0].clone() } else { "...".into() };
            (typed(slashes, &c.name, &form), c.help.clone())
        })
        .collect();
    Some(Hint { rows, sel, footer: None })
}

/// Tab on a line whose first word names commands (the caret at its end): the line takes the next match's name, a
/// space after it when it is the only one. False when it does not apply (the auto-translate list opens instead).
pub(super) fn tab(w: &mut World, back: bool) -> bool {
    if !w.typing || w.fep.is_some() {
        return false;
    }
    let line = w.chat_input.clone();
    let Some((slashes, word, spaced)) = split(&line) else { return false };
    if spaced || super::fep::caret(w) != line.chars().count() || !enough(slashes, word) {
        return false;
    }
    let list = w.lua.command_list();
    let (found, sel) = matches(w, &list, word);
    if found.is_empty() {
        return false;
    }
    let n = found.len();
    let next = match sel {
        Some(i) if back => (i + n - 1) % n,
        Some(i) => (i + 1) % n,
        None => 0,
    };
    let root = w.cmd_tab.as_ref().filter(|t| t.2 == line).map_or_else(|| word.to_ascii_lowercase(), |t| t.0.clone());
    let name = &list[found[next]].name;
    w.chat_input = if n == 1 { format!("{slashes}{name} ") } else { format!("{slashes}{name}") };
    w.chat_caret = None;
    w.cmd_tab = (n > 1).then(|| (root, next, w.chat_input.clone()));
    w.sounds.push(Se::Cursor as u32);
    true
}

/// The classic hint: a framed list above the chat line from `x`, the command white (the Tab row gold), its help grey.
pub(super) fn draw(gu: &mut GameUi, h: &Hint, x: f32, line_y: f32) -> [f32; 4] {
    const WHITE: [u8; 4] = [0x80; 4];
    const GOLD: [u8; 4] = [0x40, 0x70, 0x80, 0x80];
    const GREY: [u8; 4] = [0x58, 0x58, 0x58, 0x80];
    let rows = &h.rows[..h.rows.len().min(ROWS)];
    let left = rows.iter().map(|r| gu.text_width(&r.0)).fold(0.0, f32::max);
    let right = rows.iter().map(|r| gu.text_width(&r.1)).fold(0.0, f32::max);
    let foot = h.footer.as_ref().map_or(0.0, |f| gu.text_width(f));
    let gap = if right > 0.0 { 18.0 } else { 0.0 };
    let w = (left + gap + right).max(foot).clamp(120.0, 720.0) + 12.0;
    let lines = rows.len() + usize::from(h.footer.is_some()) + usize::from(h.rows.len() > ROWS);
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
    for (k, (cmd, help)) in rows.iter().enumerate() {
        let ry = y + 4.0 + 16.0 * k as f32;
        let lit = h.sel == Some(k);
        fit(gu, cmd, x + 6.0, ry, w - 12.0, if lit { GOLD } else { WHITE });
        if !help.is_empty() {
            let hx = x + 6.0 + left + gap;
            fit(gu, help, hx, ry, x + w - 6.0 - hx, GREY);
        }
    }
    let mut ry = y + 4.0 + 16.0 * rows.len() as f32;
    if h.rows.len() > ROWS {
        fit(gu, &format!("{} more", h.rows.len() - ROWS), x + 6.0, ry, w - 12.0, GREY);
        ry += 16.0;
    }
    if let Some(f) = &h.footer {
        fit(gu, f, x + 6.0, ry, w - 12.0, GREY);
    }
    [x, y, w, hh]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn splits_lines() {
        assert_eq!(split("//hb sh"), Some(("//", "hb", true)));
        assert_eq!(split("/ahb"), Some(("/", "ahb", false)));
        assert_eq!(split("//"), Some(("//", "", false)));
        assert_eq!(split("hello"), None);
        assert!(enough("//", "") && !enough("/", "p") && enough("/", "ahb"));
        assert_eq!(typed("//", "hb", ""), "//hb");
        assert_eq!(typed("/", "ahbuy", "<item>"), "/ahbuy <item>");
    }
}

//! The game's line breaker (30261102_0 FUN_1025ccf0) and its language rules, on the game's Shift-JIS bytes: one line
//! of a text cut to a width, Japanese moving a char down before 、。！, the European languages breaking after words,
//! dashes and ellipses, French keeping a space before ! ? : ; and after an opening guillemet, and soft hyphens (0x15)
//! that end a line with '-'. Its callers: the log and event lines (FUN_1012b980, tail 0x18), the text boxes
//! (FUN_10158200 / FUN_101582a0, tail 0x14) and the item descriptions (FUN_1015c9e0, tail 0x14).

use super::font::is_lead;
use super::items::Lang;

/// A soft hyphen: never drawn (FUN_1025cc10 skips it), a line may end at one with '-'.
pub const SOFT_HYPHEN: u8 = 0x15;

/// What a line ends with at a soft hyphen (DAT_10354e04).
const HYPHEN: &[u8] = b"-";

/// FUN_1025d130's table (0x1038bd94): what a Japanese line never starts with: 、。！
const NO_START_JA: [&[u8]; 3] = [b"\x81\x41", b"\x81\x42", b"\x81\x49"];

/// FUN_1025d210's tables (0x1038bda4 / 0x1038bdcc, their lengths filled on each call): a word may end after one of A
/// (85 45, "...", ★, '-') unless one of A or B (' ', '?') follows.
const AFTER: [&[u8]; 4] = [b"\x85\x45", b"...", b"\x81\x9a", b"-"];
const NOT_BEFORE: [&[u8]; 2] = [b" ", b"?"];

/// FUN_1011ef10: a char's bytes (2 from a lead byte).
fn clen(b: u8) -> usize {
    1 + is_lead(b) as usize
}

fn at(s: &[u8], i: usize) -> u8 {
    s.get(i).copied().unwrap_or(0)
}

/// FUN_1011ef30's code: a lead byte's pair, else the byte.
fn code(s: &[u8]) -> u16 {
    let b = at(s, 0);
    if is_lead(b) { u16::from(b) << 8 | u16::from(at(s, 1)) } else { b as i8 as i16 as u16 }
}

/// FUN_1025d450: the chars that end a line (07, CR, LF).
fn is_break(b: u8) -> bool {
    matches!(b, 7 | 0xd | 0xa)
}

/// strncmp(s, t, t.len()) == 0 on a string that ends at its slice.
fn starts(s: &[u8], t: &[u8]) -> bool {
    (0..t.len()).all(|k| at(s, k) == t[k])
}

/// FUN_1025d130: the rest is at least 2 bytes and starts with a char a Japanese line never starts with.
fn no_start_ja(s: &[u8]) -> bool {
    s.len() > 1 && NO_START_JA.iter().any(|t| starts(s, t))
}

/// FUN_1025d190: a '-' that is no minus before a digit 0..8 (9 is not in its range).
fn dash(s: &[u8]) -> bool {
    at(s, 0) == b'-' && !(b'0'..=b'8').contains(&at(s, 1))
}

/// FUN_1025d1b0: the length of the first entry of `table` that `s` starts with, among those no longer than `room`.
fn starts_any(s: &[u8], room: isize, table: &[&[u8]]) -> usize {
    table.iter().find(|t| t.len() as isize <= room && starts(s, t)).map_or(0, |t| t.len())
}

/// FUN_1025ccb0: the bytes of the first `n` that are drawn (soft hyphens left out).
fn drawn_len(s: &[u8], n: usize) -> usize {
    let (mut i, mut out) = (0, 0);
    while i < n {
        if at(s, i) == SOFT_HYPHEN {
            i += 1;
        } else {
            let l = clen(at(s, i));
            i += l;
            out += l;
        }
    }
    out
}

/// FUN_1025cc10: `out` becomes the first `n` bytes of `src` from `base` on, soft hyphens left out, at most `cap` - 1
/// bytes in all (0: no cap); where it stopped in `src`.
fn copy(out: &mut Vec<u8>, base: usize, cap: usize, src: &[u8], n: usize) -> usize {
    out.truncate(base);
    let (mut i, mut w) = (0, 0);
    if n == 0 {
        return 0;
    }
    loop {
        if at(src, i) == SOFT_HYPHEN {
            i += 1;
        } else {
            let l = clen(at(src, i));
            if cap != 0 && cap <= l + w {
                return i;
            }
            out.extend((0..l).map(|k| at(src, i + k)));
            w += l;
            i += l;
        }
        if n <= i {
            return i;
        }
    }
}

/// FUN_1025d410: where the next soft hyphen is in the first `room` bytes, or -1.
fn next_soft_hyphen(s: &[u8], room: isize) -> isize {
    let mut i = 0;
    while at(s, i) != 0 && (i as isize) < room {
        if at(s, i) == SOFT_HYPHEN {
            return i as isize;
        }
        i += clen(at(s, i));
    }
    -1
}

/// FUN_1025d210(lang, s, room): where the first word of `s` ends (the next one starts), 0 for none in `room` bytes;
/// the whole string when it ends first. `lang` is the language its rules are taken from (3: French).
fn word_end(lang: u8, s: &[u8], room: usize) -> usize {
    let room = room as isize;
    let (mut i, mut brk, mut quote) = (0usize, 0usize, false);
    if at(s, 0) != 0 {
        loop {
            if at(s, i) == b' ' {
                if lang == 3 {
                    if quote {
                        quote = false;
                    } else {
                        if room <= (i + 1) as isize {
                            brk = 0;
                            break;
                        }
                        // French keeps the space before 82 49, 85 7B, ! ? : ;
                        if !matches!(code(&s[i + 1..]), 0x8249 | 0x857b | 0x21 | 0x3f | 0x3a | 0x3b) {
                            brk = i + 1;
                        }
                    }
                } else {
                    brk = i + 1;
                }
                if brk != 0 {
                    if !dash(&s[(i + 1).min(s.len())..]) {
                        break;
                    }
                    brk = 0;
                }
                i += 1;
            } else {
                let rest = &s[i..];
                let mut l = if dash(rest) { clen(at(s, i)) } else { starts_any(rest, room - i as isize, &AFTER) };
                if l == 0 {
                    // French: the space after 85 6B / 85 4B (its opening guillemets) does not end a word.
                    if lang == 3 && matches!(code(rest), 0x856b | 0x854b) {
                        quote = true;
                    }
                    l = clen(at(s, i));
                } else {
                    let next = &s[(i + l).min(s.len())..];
                    let r = room - l as isize - i as isize;
                    if starts_any(next, r, &AFTER) == 0 && starts_any(next, r, &NOT_BEFORE) == 0 {
                        if ((i + l) as isize) < room {
                            brk = i + l;
                        }
                        break;
                    }
                }
                i += l;
            }
            if room <= i as isize || at(s, i) == 0 {
                break;
            }
        }
    }
    if at(s, i) != 0 { brk } else { i }
}

/// The width a font gives one char: its bytes (one, or a lead byte's two) to pixels (FUN_1011e350's role).
pub type CharWidth<'a> = &'a dyn Fn(&[u8]) -> i32;

/// FUN_1011e350's codes below 0x20 have no width; the font gives the rest.
fn char_width(cw: CharWidth, s: &[u8]) -> i32 {
    if s.len() == 1 && s[0] < 0x20 { 0 } else { cw(s) }
}

/// FUN_1011e810(s, 0, max) (FUN_1011ee00): the bytes of `s` that fit in `max` px (a char that goes past it is left
/// out; 04 has no width). Retail's widths are shorts in whole pixels; ours are i32 so 1/16 px units ([`str_lines`])
/// keep retail's 32767 px reach.
pub fn fit(s: &[u8], max: i32, cw: CharWidth) -> usize {
    let (mut i, mut w) = (0usize, 0i32);
    while at(s, i) != 0 {
        let next = if at(s, i) == 4 {
            i + 1
        } else {
            let l = clen(at(s, i));
            w = w.saturating_add(char_width(cw, &s[i..(i + l).min(s.len())]));
            i + l
        };
        if max < w {
            return i;
        }
        i = next;
    }
    i
}

/// FUN_1011ee70, the Japanese item descriptions' fit: a 、。！ where the line would end hangs on it, with the
/// widthless chars after it.
pub fn fit_hanging(s: &[u8], max: i32, cw: CharWidth) -> usize {
    let n = fit(s, max, cw);
    if at(s, n) != 0 && no_start_ja(&s[n..]) {
        let l = clen(at(s, n));
        let rest = &s[(n + l).min(s.len())..];
        return n + l + fit(rest, 0, cw);
    }
    n
}

/// FUN_1011e400(s, n): the width of the first `n` bytes (0: all).
pub fn width(s: &[u8], n: usize, cw: CharWidth) -> i32 {
    let (mut i, mut w) = (0, 0);
    while at(s, i) != 0 && (n == 0 || i < n) {
        if at(s, i) == 4 {
            i += 1;
        } else {
            let l = clen(at(s, i));
            w += char_width(cw, &s[i..(i + l).min(s.len())]);
            i += l;
        }
    }
    w
}

/// One line as FUN_1025ccf0 cuts it.
#[derive(Debug, PartialEq)]
pub struct Line {
    /// The line's bytes: soft hyphens left out, '-' after the one it ends at.
    pub text: Vec<u8>,
    /// The bytes of the source it took (the next line starts there; past a line-break char).
    pub used: usize,
    /// The bytes of the source the line shows (`text` is them without soft hyphens, then the '-' if `hyphen`).
    pub shown: usize,
    pub hyphen: bool,
}

/// How a caller cuts its lines.
pub struct Cut<'a> {
    /// The output buffer's size (param_2): a line holds at most this - 1 bytes.
    pub cap: usize,
    /// The line's width (param_5, a short; i32 here, see [`fit`]).
    pub max: i32,
    /// param_6: the width added to the line when a soft hyphen is weighed against a word break, and (FUN_1025ccf0's
    /// first test) the bytes copied when the char where the width ends is a line break. Every caller passes 0.
    pub extra: i32,
    /// param_7: a word that does not fit moves to the next line only when less than this many of its bytes would fit
    /// on this one; else it is cut where the width ends (0x18 for the log and event lines, 0x14 for the others).
    pub tail: usize,
    /// param_8: what fits (FUN_1011ee00, or FUN_1011ee70 for Japanese item descriptions).
    pub fit: &'a dyn Fn(&[u8], i32) -> usize,
    /// The font's char widths (param_9 is FUN_1011e400 over them).
    pub cw: CharWidth<'a>,
}

/// FUN_1025ccf0: the first line of `src` (cut at its first NUL), for the client language's rules.
pub fn line(src: &[u8], c: &Cut) -> Line {
    line_for(super::lang::client(), super::lang::european_as_english(), src, c)
}

/// FUN_1025ccf0 for language `lang`, `european` the PlayOnline flag that plays German and French as English.
pub fn line_for(lang: Lang, european: bool, src: &[u8], c: &Cut) -> Line {
    let src = &src[..src.iter().position(|&b| b == 0).unwrap_or(src.len())];
    let w = |s: &[u8], n: usize| width(s, n, c.cw);
    let mut out = Vec::new();
    let mut used = 0;
    let n = (c.fit)(src, c.max);
    let mut s18 = n;
    let mut hyphen = false;
    if at(src, n) == 0 {
        // All of it fits: up to its first line break, '-' after a soft hyphen that ends it.
        let (mut i, mut last) = (0, 0u8);
        while i < n {
            if is_break(at(src, i)) {
                s18 = clen(at(src, i)) + i;
                break;
            }
            last = at(src, i);
            i += clen(at(src, i));
        }
        let shown = copy(&mut out, 0, c.cap.saturating_sub(1), src, i);
        let hyphen = last == SOFT_HYPHEN;
        if hyphen {
            out.extend_from_slice(HYPHEN);
        }
        return Line { text: out, used: s18, shown, hyphen };
    }
    let mut cut = None;
    s18 = 0;
    if !is_break(at(src, n)) {
        // A line break inside the width ends the line there.
        let mut i = 0;
        while i < n {
            if is_break(at(src, i)) {
                s18 = clen(at(src, i)) + i;
                cut = Some(i);
                break;
            }
            i += clen(at(src, i));
        }
    } else {
        // Unreachable in retail (a line break has no width, so the width never ends on one): param_6's bytes.
        used = n + 1;
        s18 = n;
        cut = Some(c.extra.max(0) as usize);
    }
    if let Some(k) = cut
        && s18 != 0
    {
        let shown = copy(&mut out, 0, c.cap, src, k);
        return Line { text: out, used: if used == 0 { s18 } else { used }, shown, hyphen: false };
    }
    let l = lang as u8;
    let eu = european && matches!(lang, Lang::De | Lang::Fr);
    let rules = if eu { 1 } else { l };
    let end = if !eu && l == 0 {
        // Japanese: a char where the width ends that no line starts with takes the last char down with it.
        if no_start_ja(&src[n..]) {
            let mut s;
            let mut j = 0;
            loop {
                s = j;
                j = s + clen(at(src, s));
                if j >= n {
                    break;
                }
            }
            if at(src, s) == 4 { s + 1 } else { s }
        } else {
            n
        }
    } else if !eu && !(1..=3).contains(&l) {
        0
    } else {
        // Words: as many whole ones as fit, then soft hyphens inside the next while the line with '-' still fits.
        let (mut s10, mut acc) = (0usize, 0usize);
        if at(src, 0) != 0 {
            loop {
                let k = word_end(rules, &src[s18..], n - s18);
                if k == 0 {
                    break;
                }
                let u = k + s18;
                if u > n {
                    break;
                }
                let t = drawn_len(&src[s18..], k) + acc;
                if c.cap <= t {
                    break;
                }
                acc = t;
                s18 = u;
                if at(src, u) == 0 {
                    break;
                }
            }
        }
        copy(&mut out, 0, 0, src, s18);
        let (mut p, mut u10, mut u7) = (s18, s18, s18);
        if at(src, s18) != 0 {
            loop {
                let k = next_soft_hyphen(&src[p..], n as isize - u10 as isize);
                if k < 1 {
                    break;
                }
                let k = k as usize + 1;
                u10 += k;
                if n < u10 {
                    break;
                }
                let b = drawn_len(&src[p..], k);
                let tot = b + 1 + acc;
                if c.cap <= tot {
                    break;
                }
                copy(&mut out, acc, b + 1, &src[p..], k);
                out.truncate(acc + b);
                out.extend_from_slice(HYPHEN);
                if c.max < w(&out, tot) {
                    break;
                }
                out.truncate(acc + b);
                p = u10;
                acc += b;
                s10 = u10;
                if at(src, p) == 0 {
                    break;
                }
            }
            if s10 != 0 {
                u7 = s10;
            }
        }
        let mut end = n;
        if n - u7 < c.tail {
            if s18 == 0 {
                if s10 != 0 {
                    hyphen = true;
                    end = s10;
                }
            } else {
                end = s18;
                if s10 != 0 && w(src, s18) + c.extra < c.max {
                    hyphen = true;
                    end = s10;
                }
            }
        }
        end
    };
    s18 = end;
    let shown = copy(&mut out, 0, c.cap, src, s18);
    if hyphen {
        out.extend_from_slice(HYPHEN);
    }
    Line { text: out, used: if used == 0 { s18 } else { used }, shown, hyphen }
}

/// The lines a caller loops FUN_1025ccf0 over until the text ends; byte offset in `src` where each starts. Retail's
/// loops never end on a char wider than the whole line (nothing is taken); ours takes that char alone.
#[cfg(test)]
pub fn lines_for(lang: Lang, european: bool, src: &[u8], c: &Cut) -> Vec<(usize, Line)> {
    let src = &src[..src.iter().position(|&b| b == 0).unwrap_or(src.len())];
    let mut out = Vec::new();
    let mut at_ = 0;
    while at_ < src.len() {
        let mut l = line_for(lang, european, &src[at_..], c);
        if l.used == 0 {
            let k = clen(src[at_]).min(src.len() - at_);
            l = Line { text: src[at_..at_ + k].to_vec(), used: k, shown: k, hyphen: false };
        }
        let used = l.used;
        out.push((at_, l));
        at_ += used;
    }
    out
}

/// A line of a decoded text (str offsets).
#[derive(Debug, Clone, PartialEq)]
pub struct StrLine {
    /// Where it starts.
    pub start: usize,
    /// Where what it shows ends (`Line::shown`).
    pub end: usize,
    /// Where the next starts.
    pub next: usize,
    pub hyphen: bool,
}

impl StrLine {
    /// The line as drawn: its chars without soft hyphens, then '-' after the one it ends at.
    pub fn text(&self, s: &str) -> String {
        let mut t: String = s[self.start..self.end].chars().filter(|&c| c != SOFT_HYPHEN as char).collect();
        if self.hyphen {
            t.push('-');
        }
        t
    }
}

/// How a caller cuts a decoded text: the font's widths, the width of each line by its index, the tail, the buffer,
/// and whether a Japanese line ending may hang (FUN_1011ee70).
pub struct StrCut<'a> {
    pub width: &'a dyn Fn(&str) -> f32,
    pub max: &'a dyn Fn(usize) -> f32,
    pub tail: usize,
    pub cap: usize,
    pub hang: bool,
    /// A line the buffer cut short of what it took goes on at what it did not show (retail's own texts never fill
    /// the buffer; a mod's long chat line can). Off: the rest is skipped, as retail.
    pub resume: bool,
}

/// FUN_1025ccf0 looped over a decoded text until it ends. The text goes back to the game's Shift-JIS (font::
/// encode_sjis) one char at a time; a char that is no single game char (one Shift-JIS lacks, a colour code, an
/// auto-translate block's) stands as a pair `FC xx` of its own width. Widths are in 1/16 px.
pub fn str_lines(text: &str, c: &StrCut) -> Vec<StrLine> {
    use super::font::{decode_sjis, encode_sjis};
    let mut bytes = Vec::with_capacity(text.len() * 2);
    let mut starts = Vec::new();
    let mut widths = std::collections::HashMap::new();
    let mut spare = 0x40u8;
    let mut buf = [0u8; 4];
    let sixteenths = |w: f32| (w * 16.0).round() as i32;
    for (si, ch) in text.char_indices() {
        let one: &str = ch.encode_utf8(&mut buf);
        let mut e = encode_sjis(one);
        let single = !e.is_empty() && e.len() == clen(e[0]) && (e != b"?" || ch == '?');
        if !single {
            e = vec![0xFC, spare];
            spare = if spare == 0xFC { 0x40 } else { spare + 1 };
        }
        starts.push((bytes.len(), si));
        widths.entry(e.clone()).or_insert_with(|| sixteenths((c.width)(one)));
        bytes.extend_from_slice(&e);
    }
    let cw = |s: &[u8]| widths.get(s).copied().unwrap_or_else(|| sixteenths((c.width)(&decode_sjis(s))));
    let fit_ = |s: &[u8], m: i32| if c.hang { fit_hanging(s, m, &cw) } else { fit(s, m, &cw) };
    let px = |w: f32| (w * 16.0).floor().clamp(0.0, (i16::MAX as i32 * 16) as f32) as i32;
    let str_at = |b: usize| match starts.binary_search_by_key(&b, |&(bo, _)| bo) {
        Ok(k) => starts[k].1,
        Err(k) if k >= starts.len() => text.len(),
        Err(k) => starts[k.saturating_sub(1)].1,
    };
    let mut out = Vec::new();
    let mut at_ = 0;
    while at_ < bytes.len() {
        let cut = Cut { cap: c.cap, max: px((c.max)(out.len())), extra: 0, tail: c.tail, fit: &fit_, cw: &cw };
        let mut l = self::line(&bytes[at_..], &cut);
        if l.used == 0 {
            // Retail's loops never end on a char wider than the whole line; ours takes it alone.
            let next = starts.iter().map(|&(b, _)| b).find(|&b| b > at_).unwrap_or(bytes.len());
            (l.used, l.shown, l.hyphen) = (next - at_, next - at_, false);
        } else if c.resume && l.shown != 0 && l.shown < l.used && !is_break(at(&bytes[at_..], l.shown)) {
            l.used = l.shown;
        }
        out.push(StrLine {
            start: str_at(at_),
            end: str_at(at_ + l.shown),
            next: str_at(at_ + l.used),
            hyphen: l.hyphen,
        });
        at_ += l.used;
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    /// 6 px a one-byte char, 12 a pair: widths in tests are in chars.
    fn cw(s: &[u8]) -> i32 {
        if s.len() == 2 { 12 } else { 6 }
    }

    fn cut(max_chars: i32, tail: usize) -> Cut<'static> {
        Cut { cap: 0x200, max: max_chars * 6, extra: 0, tail, fit: &|s, m| fit(s, m, &cw), cw: &cw }
    }

    fn texts(lang: Lang, s: &[u8], c: &Cut) -> Vec<Vec<u8>> {
        lines_for(lang, false, s, c).into_iter().map(|(_, l)| l.text).collect()
    }

    fn v(ls: &[&[u8]]) -> Vec<Vec<u8>> {
        ls.iter().map(|l| l.to_vec()).collect()
    }

    /// English: whole words, the space kept at the end of a line; a word with fewer than `tail` bytes on the line
    /// moves down, a longer one is cut where the width ends.
    #[test]
    fn english_words() {
        let c = cut(10, 0x18);
        assert_eq!(texts(Lang::En, b"one two three four", &c), v(&[b"one two ", b"three four"]));
        let long = b"a abcdefghijklmnopqrstuvwxyz";
        assert_eq!(texts(Lang::En, long, &c), v(&[b"a ", b"abcdefghij", b"klmnopqrst", b"uvwxyz"]));
        // With a tail of 8 the 8 bytes that fit stay.
        assert_eq!(texts(Lang::En, long, &cut(10, 8)), v(&[b"a abcdefgh", b"ijklmnopqr", b"stuvwxyz"]));
    }

    /// A line break inside the width ends the line, past it.
    #[test]
    fn breaks() {
        let c = cut(10, 0x18);
        assert_eq!(texts(Lang::En, b"ab\ncd", &c), v(&[b"ab", b"cd"]));
        assert_eq!(texts(Lang::En, b"ab\x07cd efgh ijkl", &c), v(&[b"ab", b"cd efgh ", b"ijkl"]));
    }

    /// After a dash or an ellipsis a word may end, not before a space or '?'.
    #[test]
    fn dashes() {
        let c = cut(10, 0x18);
        assert_eq!(texts(Lang::En, b"well-known words", &c), v(&[b"well-", b"known ", b"words"]));
        assert_eq!(texts(Lang::En, b"wait... what now", &c), v(&[b"wait... ", b"what now"]));
        assert_eq!(texts(Lang::En, b"x -5 apples", &c), v(&[b"x -5 ", b"apples"]));
    }

    /// French keeps the space before ! ? : ; and the one after an opening guillemet (85 6B).
    #[test]
    fn french_spaces() {
        let c = cut(12, 0x18);
        assert_eq!(texts(Lang::Fr, b"Il dit oui ! Non", &c), v(&[b"Il dit ", b"oui ! Non"]));
        assert_eq!(texts(Lang::En, b"Il dit oui ! Non", &c), v(&[b"Il dit oui ", b"! Non"]));
        let c = cut(10, 0x18);
        assert_eq!(texts(Lang::Fr, b"un mot\x85\x6b oui", &c), v(&[b"un ", b"mot\x85\x6b oui"]));
        assert_eq!(texts(Lang::En, b"un mot\x85\x6b oui", &c), v(&[b"un mot\x85\x6b ", b"oui"]));
        // The European flag plays French as English.
        let eu = lines_for(Lang::Fr, true, b"Il dit oui ! Non", &cut(12, 0x18));
        assert_eq!(eu.into_iter().map(|(_, l)| l.text).collect::<Vec<_>>(), v(&[b"Il dit oui ", b"! Non"]));
    }

    /// A soft hyphen is never drawn; a line may end at one with '-', and a text ending in one gets '-'.
    #[test]
    fn soft_hyphens() {
        let c = cut(10, 0x18);
        assert_eq!(texts(Lang::En, b"ab Donau\x15dampf\x15schiff", &c), v(&[b"ab Donau-", b"dampf-", b"schiff"]));
        assert_eq!(texts(Lang::En, b"ab\x15", &c), v(&[b"ab-"]));
    }

    /// Japanese: chars, not words; a line never starts with 、。！ (the char before moves down with it), except
    /// where the description's fit lets it hang.
    #[test]
    fn japanese() {
        let a = b"\x82\xa0"; // あ, 12 px
        let mut s = Vec::new();
        for _ in 0..5 {
            s.extend_from_slice(a);
        }
        s.extend_from_slice(b"\x81\x42");
        s.extend_from_slice(a);
        let c = cut(10, 0x18); // 60 px: 5 pairs
        let ls = lines_for(Lang::Ja, false, &s, &c);
        assert_eq!(ls.iter().map(|(at, l)| (*at, l.text.len())).collect::<Vec<_>>(), [(0, 8), (8, 6)]);
        let hang = Cut { fit: &|s, m| fit_hanging(s, m, &cw), ..cut(10, 0x18) };
        let ls = lines_for(Lang::Ja, false, &s, &hang);
        assert_eq!(ls.iter().map(|(at, l)| (*at, l.text.len())).collect::<Vec<_>>(), [(0, 12), (12, 2)]);
    }

    /// A decoded text: a char Shift-JIS lacks is one char of its own width; offsets are the str's.
    #[test]
    fn decoded() {
        let width = |s: &str| s.chars().map(|c| if c == '\u{2192}' { 12.0 } else { 6.0 }).sum::<f32>();
        let c = StrCut { width: &width, max: &|_| 60.0, tail: 0x18, cap: 0x200, hang: false, resume: false };
        let s = "ab \u{2192}\u{2192} cdefgh ij";
        let ls = str_lines(s, &c);
        assert_eq!(ls.iter().map(|l| l.text(s)).collect::<Vec<_>>(), ["ab \u{2192}\u{2192} ", "cdefgh ij"]);
        assert_eq!(ls[1].start, s.find('c').unwrap());
    }

    /// Lines wider than 2048 px still wrap (1/16 px units past a short).
    #[test]
    fn wide_lines() {
        let width = |s: &str| s.chars().count() as f32 * 6.0;
        let c = StrCut { width: &width, max: &|_| 2400.0, tail: 0x18, cap: 0x200, hang: false, resume: true };
        let s = "abcd ".repeat(90);
        let ls = str_lines(&s, &c);
        assert_eq!(ls.iter().map(|l| l.end - l.start).collect::<Vec<_>>(), [400, 50]);
    }

    /// A line the buffer cuts short goes on at what it did not show when `resume` is on; off, the rest is skipped.
    #[test]
    fn buffer_cut() {
        let width = |s: &str| s.chars().count() as f32 * 6.0;
        let s = "x".repeat(600);
        let c = StrCut { width: &width, max: &|_| 4000.0, tail: 0x18, cap: 0x200, hang: false, resume: true };
        let ls = str_lines(&s, &c);
        assert_eq!(ls.iter().map(|l| (l.start, l.end)).collect::<Vec<_>>(), [(0, 510), (510, 600)]);
        let ls = str_lines(&s, &StrCut { resume: false, ..c });
        assert_eq!(ls.iter().map(|l| (l.start, l.end, l.next)).collect::<Vec<_>>(), [(0, 510, 600)]);
    }
}

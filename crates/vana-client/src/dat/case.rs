//! The game's capitals by language (30261102_0): FUN_1025d5e0 (one char up), FUN_1025d580 (one char down),
//! FUN_1025d4b0 (the bytes a char takes up), FUN_1025d720 (a text with its first char up, cut to a buffer) and
//! FUN_1025d7a0 / FUN_1025d770 (the first char only). Japanese leaves every char as it is; English and German (and
//! French with the European-as-English flag) take FUN_1011f090 / FUN_1011f150; French puts its accented small vowels
//! up as plain capitals (85 BF / C1 A, 85 C7..CA E, 85 CD / CE I, 85 D3 O, 85 D8 / DA / DB U) and the other chars as
//! English does.

use super::font::{decode_sjis, encode_sjis, is_lead};
use super::items::Lang;

/// The rule a language takes (FUN_1025d4b0 / FUN_1025d580 / FUN_1025d5e0's test of DAT_10457bd8 +8 and +0x18 bit 7).
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
enum Rule {
    /// Japanese: chars as they are.
    Copy,
    /// English and German: FUN_1011f090 / FUN_1011f150.
    Latin,
    /// French: Latin, but an accented small vowel goes up to its plain capital.
    French,
}

fn rule(lang: Lang, european_as_english: bool) -> Rule {
    match lang {
        Lang::De | Lang::Fr if european_as_english => Rule::Latin,
        Lang::En | Lang::De => Rule::Latin,
        Lang::Fr => Rule::French,
        Lang::Ja => Rule::Copy,
    }
}

fn client_rule() -> Rule {
    rule(super::lang::client(), super::lang::european_as_english())
}

/// FUN_1011ef10: the bytes of the char at `s` (2 for a lead byte, else 1).
fn char_len(s: &[u8]) -> usize {
    1 + s.first().is_some_and(|&b| is_lead(b)) as usize
}

/// FUN_1011ef30: the char's code, a lead byte and the byte after it as one word, else the byte sign-extended (a lone
/// byte from 0x80 reads 0xff80..).
fn code(s: &[u8]) -> u16 {
    match s {
        [l, t, ..] if is_lead(*l) => u16::from_be_bytes([*l, *t]),
        [b, ..] => *b as i8 as i16 as u16,
        [] => 0,
    }
}

/// FUN_1011ef70: a code under 0x80 as one byte, else two (high first).
fn put(out: &mut Vec<u8>, c: u16) {
    if c < 0x80 {
        out.push(c as u8);
    } else {
        out.extend(c.to_be_bytes());
    }
}

/// FUN_1011f090: a to z, 85 BF..D5 and 85 D7..DD less 0x20; 85 5C / 5A / 5E to 85 4C / 4A / 4E, 85 DE to 85 5F.
fn upper_code(c: u16) -> u16 {
    match c {
        0x61..=0x7a | 0x85bf..=0x85d5 | 0x85d7..=0x85dd => c - 0x20,
        0x855c => 0x854c,
        0x855a => 0x854a,
        0x855e => 0x854e,
        0x85de => 0x855f,
        _ => c,
    }
}

/// FUN_1011f150: FUN_1011f090 the other way.
fn lower_code(c: u16) -> u16 {
    match c {
        0x41..=0x5a | 0x859f..=0x85b5 | 0x85b7..=0x85bd => c + 0x20,
        0x854c => 0x855c,
        0x854a => 0x855a,
        0x854e => 0x855e,
        0x855f => 0x85de,
        _ => c,
    }
}

/// French's plain capital for an accented small vowel (FUN_1025d5e0's switch), None for any other char.
fn french_plain(c: u16) -> Option<u8> {
    Some(match c {
        0x85bf | 0x85c1 => b'A',
        0x85c7..=0x85ca => b'E',
        0x85cd | 0x85ce => b'I',
        0x85d3 => b'O',
        0x85d8 | 0x85da | 0x85db => b'U',
        _ => return None,
    })
}

/// The char at `s` copied as it is.
fn copy(out: &mut Vec<u8>, s: &[u8]) {
    out.extend(&s[..char_len(s).min(s.len())]);
}

fn upper_char_by(r: Rule, out: &mut Vec<u8>, s: &[u8]) {
    let plain = if r == Rule::French { french_plain(code(s)) } else { None };
    match (r, plain) {
        (Rule::Copy, _) => copy(out, s),
        (_, Some(b)) => out.push(b),
        _ => put(out, upper_code(code(s))),
    }
}

fn lower_char_by(r: Rule, out: &mut Vec<u8>, s: &[u8]) {
    match r {
        Rule::Copy => copy(out, s),
        _ => put(out, lower_code(code(s))),
    }
}

/// FUN_1025d5e0: the char at `s` up, onto `out`.
pub fn upper_char(out: &mut Vec<u8>, s: &[u8]) {
    upper_char_by(client_rule(), out, s)
}

/// FUN_1011ef10 for the callers that step over the source.
pub fn source_len(s: &[u8]) -> usize {
    char_len(s)
}

/// FUN_1025d720(out, cap, s): `s` with its first char up, then whole chars while the text stays under `cap` bytes
/// (FUN_1011f310); None where retail returns 0 and writes nothing (`cap` no more than the first char up takes,
/// FUN_1025d4b0). An empty text stays empty (retail would read past its end).
fn cap_first_by(r: Rule, s: &[u8], cap: usize) -> Option<Vec<u8>> {
    let mut out = Vec::new();
    if s.is_empty() {
        return Some(out);
    }
    upper_char_by(r, &mut out, s);
    if cap <= out.len() {
        return None;
    }
    let mut at = char_len(s);
    while at < s.len() && s[at] != 0 {
        let n = char_len(&s[at..]);
        if out.len() + n >= cap {
            break;
        }
        out.extend(&s[at..(at + n).min(s.len())]);
        at += n;
    }
    Some(out)
}

/// FUN_1025d720 on Shift-JIS bytes.
pub fn cap_first(s: &[u8], cap: usize) -> Option<Vec<u8>> {
    cap_first_by(client_rule(), s, cap)
}

/// FUN_1025d720 on a decoded text: None where retail writes nothing.
pub fn cap_first_str(s: &str, cap: usize) -> Option<String> {
    cap_first(&encode_sjis(s), cap).map(|b| decode_sjis(&b))
}

/// FUN_1025d7a0: the first char up (FUN_1025d770, a 3-byte buffer), the rest as it is.
pub fn cap_str(s: &str) -> String {
    let b = encode_sjis(s);
    if b.is_empty() {
        return String::new();
    }
    let n = char_len(&b).min(b.len());
    let mut out = Vec::with_capacity(b.len() + 1);
    upper_char(&mut out, &b);
    out.extend(&b[n..]);
    decode_sjis(&out)
}

/// FUN_100c2bf0's modes (the message formatter's case, FUN_100be760 +0xc): 1 the first char up (whatever it is), 2 every
/// char up, 3 every char down, 4 up and down by turns from the first char; a 1E colour code is kept and not counted;
/// any other mode copies.
pub fn apply_case(t: &[u8], mode: u8) -> Vec<u8> {
    apply_case_by(client_rule(), t, mode)
}

fn apply_case_by(r: Rule, t: &[u8], mode: u8) -> Vec<u8> {
    let mut out = Vec::with_capacity(t.len() + 4);
    let (mut mode, mut turn) = (mode, 0u8);
    let mut i = 0;
    while i < t.len() && t[i] != 0 {
        if t[i] == 0x1e {
            out.extend(&t[i..(i + 2).min(t.len())]);
            i += 2;
            continue;
        }
        let s = &t[i..];
        match mode {
            1 => {
                upper_char_by(r, &mut out, s);
                mode = 0;
            }
            2 => upper_char_by(r, &mut out, s),
            3 => lower_char_by(r, &mut out, s),
            4 => {
                if turn & 1 != 0 {
                    lower_char_by(r, &mut out, s)
                } else {
                    upper_char_by(r, &mut out, s)
                }
                turn = turn.wrapping_add(1);
            }
            _ => copy(&mut out, s),
        }
        i += char_len(s);
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rules_by_language() {
        assert_eq!(rule(Lang::Ja, false), Rule::Copy);
        assert_eq!(rule(Lang::Ja, true), Rule::Copy);
        assert_eq!(rule(Lang::En, false), Rule::Latin);
        assert_eq!(rule(Lang::De, false), Rule::Latin);
        assert_eq!(rule(Lang::Fr, false), Rule::French);
        assert_eq!(rule(Lang::Fr, true), Rule::Latin);
    }

    #[test]
    fn first_char_up() {
        let up = |r, s: &[u8]| cap_first_by(r, s, 0x80).unwrap();
        assert_eq!(up(Rule::Latin, b"bronze sword"), b"Bronze sword");
        assert_eq!(up(Rule::Copy, b"bronze"), b"bronze");
        // French: an accented small vowel goes up plain; English and German keep the accent (85 C7 -> 85 A7).
        assert_eq!(up(Rule::French, &[0x85, 0xc7, b't', b'e']), b"Ete");
        assert_eq!(up(Rule::Latin, &[0x85, 0xc7, b't', b'e']), [0x85, 0xa7, b't', b'e']);
        // A French char outside the vowels takes the plain rule: 85 D1 (a small n tilde) up by 0x20.
        assert_eq!(up(Rule::French, &[0x85, 0xd1]), [0x85, 0xb1]);
        assert_eq!(up(Rule::Latin, &[0x85, 0x5c, b'x']), [0x85, 0x4c, b'x']);
        assert_eq!(up(Rule::Latin, &[0x85, 0xde]), [0x85, 0x5f]);
        // 85 D6 is not a letter.
        assert_eq!(up(Rule::Latin, &[0x85, 0xd6]), [0x85, 0xd6]);
        // A lone byte from 0x80 reads sign-extended and goes out as two bytes FF xx (FUN_1011ef30 / FUN_1011ef70).
        assert_eq!(up(Rule::Latin, &[0xb1, b'a']), [0xff, 0xb1, b'a']);
        assert_eq!(up(Rule::Copy, &[0xb1, b'a']), [0xb1, b'a']);
        assert_eq!(up(Rule::Latin, b""), b"");
    }

    #[test]
    fn first_char_up_cut() {
        // The text stays under the cap; a cap no more than the first char's bytes gives nothing.
        assert_eq!(cap_first_by(Rule::Latin, b"abcdef", 4).unwrap(), b"Abc");
        assert_eq!(cap_first_by(Rule::Latin, b"a", 1), None);
        assert_eq!(cap_first_by(Rule::French, &[0x85, 0xc7, b'x'], 2).unwrap(), b"E");
        assert_eq!(cap_first_by(Rule::Latin, &[b'a', 0x82, 0xa0, b'b'], 3).unwrap(), b"A");
    }

    #[test]
    fn formatter_case() {
        let c = |r, s: &[u8], m| apply_case_by(r, s, m);
        assert_eq!(c(Rule::Latin, b"the goblin", 1), b"The goblin");
        // Mode 1 puts up the first char whatever it is, not the first letter.
        assert_eq!(c(Rule::Latin, b"3 goblins", 1), b"3 goblins");
        assert_eq!(c(Rule::Latin, &[0x1e, 2, b'a', b'b'], 1), [0x1e, 2, b'A', b'b']);
        assert_eq!(c(Rule::Latin, b"Haste Up", 2), b"HASTE UP");
        assert_eq!(c(Rule::Latin, b"Haste Up", 3), b"haste up");
        assert_eq!(c(Rule::Latin, b"haste", 4), b"HaStE");
        // The turns count every char, a space too.
        assert_eq!(c(Rule::Latin, b"ab cd", 4), b"Ab cD");
        assert_eq!(c(Rule::Copy, b"haste", 2), b"haste");
        assert_eq!(c(Rule::French, &[0x85, 0xc9, 0x85, 0xc9], 2), b"EE");
        assert_eq!(c(Rule::Latin, b"abc", 0), b"abc");
    }
}

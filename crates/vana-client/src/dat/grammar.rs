//! German and French names (30261102_0): the forms the DLL builds for item, key item and title names in those client
//! languages. The German ones are declined (adjective and noun endings by article, case, gender and number); the
//! French ones are fields of the record as they are. The engine works on the Shift-JIS bytes, one character at a time
//! (FUN_1011ef10: a lead byte starts two), so the marks never match a trail byte.
//!
//! German marks in the text: byte 1 ends an adjective stem whose ending follows the next noun mark's gender; bytes
//! 2..15 end a noun and give its class (FUN_10259a60 / FUN_10259840). In item names '*' ends the adjective stem and '~'
//! is where the noun ending goes (FUN_10157530).

use super::font::{decode_sjis, encode_sjis, is_lead};

/// Article: none (strong endings).
pub const NONE: i32 = -1;
/// Article: definite (weak endings).
pub const DEFINITE: i32 = 0;

/// FUN_1011ef10: the bytes of the character at `s[i]`.
fn clen(s: &[u8], i: usize) -> usize {
    if s.get(i).is_some_and(|&b| is_lead(b)) { 2 } else { 1 }
}

/// The C string at `s[i..]`: up to the end or a NUL.
fn cstr(s: &[u8], i: usize) -> &[u8] {
    let t = s.get(i..).unwrap_or(&[]);
    &t[..t.iter().position(|&b| b == 0).unwrap_or(t.len())]
}

type Table = [[&'static str; 4]; 4];

/// 0x1038ae78: no article (rows nominative, genitive, dative, accusative; columns masculine, feminine, neuter, plural).
const STRONG: Table =
    [["er", "e", "es", "e"], ["en", "er", "en", "er"], ["em", "er", "em", "en"], ["en", "e", "es", "e"]];
/// 0x1038aeb8: no article, a masculine noun of class 4 (the flag; only the masculine column is filled).
const STRONG_FLAG: Table = [["er", "", "", ""], ["es", "", "", ""], ["em", "", "", ""], ["en", "", "", ""]];
/// 0x1038aef8: after the definite article.
const WEAK: Table = [["e", "e", "e", "en"], ["en", "en", "en", "en"], ["en", "en", "en", "en"], ["en", "e", "e", "en"]];
/// 0x1038af38: after the indefinite article.
const MIXED: Table =
    [["er", "e", "es", "en"], ["en", "en", "en", "en"], ["en", "en", "en", "en"], ["en", "e", "es", "en"]];

/// FUN_10259780: an adjective's ending. `article` -1 none, 0 definite, else indefinite; `case` 0..3 nominative,
/// genitive, dative, accusative; `gender` 0 masculine, 1 feminine, 2 neuter (any other: no ending); `flag` takes
/// table 0x1038aeb8 for a masculine noun with no article.
pub fn adjective(article: i32, case: i32, plural: bool, gender: i32, flag: bool) -> &'static str {
    let t = match article {
        NONE if flag && gender == 0 => &STRONG_FLAG,
        NONE => &STRONG,
        DEFINITE => &WEAK,
        _ => &MIXED,
    };
    let Some(row) = usize::try_from(case).ok().and_then(|c| t.get(c)) else { return "" };
    if plural {
        return row[3];
    }
    match gender {
        0..=2 => row[gender as usize],
        _ => "",
    }
}

/// FUN_10259670: a noun's ending by its class (0..7; any other none).
pub fn noun(case: i32, plural: bool, class: i32) -> &'static str {
    if plural {
        // Class 1: dative plural "n"; class 0 and the rest none.
        return if class == 1 && case == 2 { "n" } else { "" };
    }
    match (class, case) {
        (0, 1) => "es",
        (2, 1) => "s",
        (3, 1) => "ns",
        (3, 2 | 3) => "n",
        (4, 1..=3) => "en",
        (5, 1) => "ens",
        (5, 2) => "en",
        (6, 1..=3) => "n",
        (7, 1) => "ses",
        _ => "",
    }
}

/// The text form (FUN_10259a60) and the buffer form (FUN_10259840) read the marks alike but for 12 and 13: the buffer
/// form gives them no adjective gender and no noun class.
#[derive(Clone, Copy, PartialEq, Eq)]
enum Form {
    Text,
    Buffer,
}

/// The adjective's (plural, gender, flag) by the next noun mark (the jump tables at 0x10259bdc / 0x10259a18).
fn adjective_of(code: u8, form: Form) -> (bool, i32, bool) {
    match (code, form) {
        (12 | 13, Form::Buffer) => (false, -1, false),
        (2 | 5 | 7 | 12, _) => (false, 0, false),
        (8 | 10, _) => (false, 0, true),
        (3 | 6 | 9 | 11 | 13, _) => (false, 2, false),
        (4, _) => (false, 1, false),
        (14 | 15, _) => (true, -1, false),
        _ => (false, -1, false),
    }
}

/// The noun's (plural, class) by its mark (the jump tables at 0x10259bec / 0x10259a28).
fn class_of(code: u8, form: Form) -> (bool, i32) {
    match code {
        2 | 3 => (false, 0),
        4 => (false, 1),
        12 | 13 if form == Form::Text => (false, 1),
        5 | 6 => (false, 2),
        7 => (false, 3),
        8 => (false, 4),
        9 => (false, 5),
        10 => (false, 6),
        11 => (false, 7),
        14 => (true, 0),
        15 => (true, 1),
        _ => (false, -1),
    }
}

/// FUN_10259800: the first noun mark (2..15) from `s[i]` on, 0 when none.
fn next_mark(s: &[u8], mut i: usize) -> u8 {
    while let Some(&b) = s.get(i).filter(|&&b| b != 0) {
        if (2..=15).contains(&b) {
            return b;
        }
        i += clen(s, i);
    }
    0
}

/// The engine of FUN_10259a60 / FUN_10259840, appending to `out`; `size` stops the buffer form once that many bytes
/// are out (a piece is never cut). After a noun the article is none.
fn decline(out: &mut Vec<u8>, mut article: i32, case: i32, s: &[u8], form: Form, size: usize) {
    let s = cstr(s, 0);
    let mut i = 0;
    while i < s.len() && out.len() < size {
        match s[i] {
            1 => {
                let code = next_mark(s, i);
                if code != 0 {
                    let (plural, gender, flag) = adjective_of(code, form);
                    out.extend_from_slice(adjective(article, case, plural, gender, flag).as_bytes());
                }
                i += 1;
            }
            code @ 2..=15 => {
                let (plural, class) = class_of(code, form);
                out.extend_from_slice(noun(case, plural, class).as_bytes());
                article = NONE;
                i += 1;
            }
            _ => {
                let n = clen(s, i).min(s.len() - i);
                out.extend_from_slice(&s[i..i + n]);
                i += n;
            }
        }
    }
}

/// FUN_10259a60: `s` declined, appended to `out`.
pub fn decline_text(out: &mut Vec<u8>, article: i32, case: i32, s: &[u8]) {
    decline(out, article, case, s, Form::Text, usize::MAX);
}

/// FUN_10259840: `s` declined into a buffer of `size` bytes.
pub fn decline_buffer(size: usize, article: i32, case: i32, s: &[u8]) -> Vec<u8> {
    let mut out = Vec::new();
    decline(&mut out, article, case, s, Form::Buffer, size);
    out
}

/// A German key item's names (FUN_10155670 -> FUN_10156400; object vtable 0x10336240): fields 2 (whole), 4 / 5 / 7
/// (singular, plural, the rest). Field 3 (the gender, +8) is read only by the message grammar.
pub struct DeKeyItem {
    pub whole: bool,
    pub singular: Vec<u8>,
    pub plural: Vec<u8>,
    pub rest: Vec<u8>,
}

impl DeKeyItem {
    /// vt+0x10 (FUN_101561c0) / vt+0x18 (FUN_10156220): the singular or plural, then " " and the rest declined with
    /// no article when the name is whole.
    fn phrase(&self, s: &[u8], article: i32, case: i32) -> Vec<u8> {
        let mut out = Vec::new();
        decline_text(&mut out, article, case, s);
        if self.whole {
            if !s.is_empty() {
                out.push(b' ');
            }
            decline_text(&mut out, NONE, case, &self.rest);
        }
        out
    }

    /// vt+0x20 (FUN_10156300): the rest alone when whole, else the singular phrase.
    pub fn display(&self, article: i32, case: i32) -> Vec<u8> {
        if self.whole {
            let mut out = Vec::new();
            decline_text(&mut out, article, case, &self.rest);
            out
        } else {
            self.phrase(&self.singular, article, case)
        }
    }

    pub fn plural(&self, article: i32, case: i32) -> Vec<u8> {
        self.phrase(&self.plural, article, case)
    }
}

/// A German item name template (FUN_10157530): the last '*' (0 when none) and the last '~' (the length when none).
#[derive(Clone, Debug, Default)]
struct Template {
    s: Vec<u8>,
    star: usize,
    tilde: usize,
}

impl Template {
    fn new(s: &[u8], fix: bool) -> Template {
        let s = cstr(s, 0).to_vec();
        let (mut star, mut tilde, mut i) = (0, None, 0);
        while i < s.len() {
            match s[i] {
                b'*' => star = i,
                b'~' => tilde = Some(i),
                _ => {}
            }
            i += clen(&s, i);
        }
        let tilde = tilde.unwrap_or(s.len());
        // FUN_10157530 drops a '*' after the '~'; FUN_10157650 (a two-part name) keeps it.
        if fix && tilde < star {
            star = 0;
        }
        Template { s, star, tilde }
    }

    fn at(&self, i: usize) -> u8 {
        self.s.get(i).copied().unwrap_or(0)
    }

    /// FUN_10156cc0 / FUN_10157070 / FUN_10156e70: the adjective stem and its ending, the noun to the '~', its ending
    /// (an "n" first with `en`: FUN_10157070's feminine noun ending in 'e', declined as a plural), then what follows
    /// the '~'. Appended to `out`.
    fn build(&self, out: &mut Vec<u8>, article: i32, case: i32, plural: bool, class: i32, gender: i32, en: bool) {
        let mut i = 0;
        if self.star != 0 {
            out.extend_from_slice(&self.s[..self.star]);
            i = self.star + 1;
            out.extend_from_slice(adjective(article, case, plural, gender, class == 4).as_bytes());
        }
        let tilde = self.at(self.tilde) == b'~';
        let end = if tilde { self.tilde } else { self.s.len() };
        out.extend_from_slice(self.s.get(i..end.max(i)).unwrap_or(&[]));
        let mut plural = plural;
        if en && gender == 1 && self.tilde > 0 && self.at(self.tilde - 1) == b'e' {
            out.push(b'n');
            plural = true;
        }
        out.extend_from_slice(noun(case, plural, class).as_bytes());
        if tilde {
            out.extend_from_slice(cstr(&self.s, self.tilde + 1));
        }
    }
}

/// A German item's names (FUN_1015ec00; object vtable 0x103362c8): fields 1 (two-part), 2 / 3 (gender, class), 4 the
/// singular, 5 / 6 the plural's gender and class, 7 the plural.
#[derive(Clone, Debug, Default)]
pub struct DeItem {
    two_part: bool,
    gender: i32,
    class: i32,
    singular: Template,
    plural_gender: i32,
    plural_class: i32,
    plural: Template,
}

impl DeItem {
    pub fn new(two_part: u32, gender: u32, class: u32, singular: &[u8], pg: u32, pc: u32, plural: &[u8]) -> DeItem {
        let fix = two_part == 0;
        DeItem {
            two_part: !fix,
            gender: gender as i32,
            class: class as i32,
            singular: Template::new(singular, fix),
            plural_gender: pg as i32,
            plural_class: pc as i32,
            plural: Template::new(plural, fix),
        }
    }

    fn plural_into(&self, out: &mut Vec<u8>, article: i32, case: i32) {
        self.plural.build(out, article, case, true, self.plural_class, self.plural_gender, false);
    }

    /// vt+0x20 (FUN_10157ac0): a two-part name shows its plural, else vt+0x10 (FUN_101578d0): the singular, a two-part
    /// name's first part then its plural with no article.
    pub fn display(&self, article: i32, case: i32) -> Vec<u8> {
        let mut out = Vec::new();
        if self.two_part {
            self.plural_into(&mut out, article, case);
        } else {
            self.singular.build(&mut out, article, case, false, self.class, self.gender, false);
        }
        out
    }

    /// vt+0x18 (FUN_10157960): the plural; a two-part name's first part ("n" added to a feminine 'e') then its plural.
    pub fn plural(&self, article: i32, case: i32) -> Vec<u8> {
        let mut out = Vec::new();
        if self.two_part {
            self.singular.build(&mut out, article, case, false, self.class, self.gender, true);
            self.plural_into(&mut out, NONE, case);
        } else {
            self.plural_into(&mut out, article, case);
        }
        out
    }
}

/// A French key item's names (FUN_10155670 -> FUN_101569d0 -> FUN_10157fc0, vtable 0x10336320): fields 5 / 6 the
/// singular and plural, 9 the rest (mode 2 when it has one, else 3).
pub struct FrKeyItem {
    pub singular: Vec<u8>,
    pub plural: Vec<u8>,
    pub rest: Vec<u8>,
}

impl FrKeyItem {
    fn whole(&self) -> bool {
        !self.rest.is_empty()
    }

    /// vt+0x1c (FUN_101580e0): the singular, or the rest alone.
    pub fn display(&self) -> Vec<u8> {
        if self.whole() { self.rest.clone() } else { self.singular.clone() }
    }

    /// vt+0x10 (FUN_10158080): the plural, then " " and the rest.
    pub fn plural(&self) -> Vec<u8> {
        let mut out = self.plural.clone();
        if self.whole() {
            if !self.plural.is_empty() {
                out.push(b' ');
            }
            out.extend_from_slice(&self.rest);
        }
        out
    }
}

/// Bytes back to our text.
pub fn text(b: &[u8]) -> String {
    decode_sjis(b)
}

/// Our text as the DLL's bytes.
pub fn bytes(s: &str) -> Vec<u8> {
    encode_sjis(s)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn d(article: i32, case: i32, s: &[u8]) -> Vec<u8> {
        let mut out = Vec::new();
        decline_text(&mut out, article, case, s);
        out
    }

    /// The four ending tables (0x1038ae78 / aeb8 / aef8 / af38) by article, case, gender and number.
    #[test]
    fn adjective_tables() {
        assert_eq!(adjective(NONE, 0, false, 0, false), "er");
        assert_eq!(adjective(NONE, 1, false, 0, false), "en");
        assert_eq!(adjective(NONE, 1, false, 0, true), "es");
        assert_eq!(adjective(NONE, 1, true, 0, true), "");
        assert_eq!(adjective(NONE, 3, true, -1, false), "e");
        assert_eq!(adjective(DEFINITE, 0, false, 1, false), "e");
        assert_eq!(adjective(DEFINITE, 3, false, 0, false), "en");
        assert_eq!(adjective(1, 0, false, 2, false), "es");
        assert_eq!(adjective(1, 2, true, -1, false), "en");
        assert_eq!(adjective(NONE, 0, false, -1, false), "");
        assert_eq!(adjective(NONE, 4, false, 0, false), "");
    }

    /// FUN_10259670's classes.
    #[test]
    fn noun_classes() {
        let row = |class| (0..4).map(|c| noun(c, false, class)).collect::<Vec<_>>();
        assert_eq!(row(0), ["", "es", "", ""]);
        assert_eq!(row(1), ["", "", "", ""]);
        assert_eq!(row(2), ["", "s", "", ""]);
        assert_eq!(row(3), ["", "ns", "n", "n"]);
        assert_eq!(row(4), ["", "en", "en", "en"]);
        assert_eq!(row(5), ["", "ens", "en", ""]);
        assert_eq!(row(6), ["", "n", "n", "n"]);
        assert_eq!(row(7), ["", "ses", "", ""]);
        assert_eq!((0..4).map(|c| noun(c, true, 1)).collect::<Vec<_>>(), ["", "", "n", ""]);
        assert_eq!((0..4).map(|c| noun(c, true, 0)).collect::<Vec<_>>(), ["", "", "", ""]);
        assert_eq!(noun(1, false, -1), "");
    }

    /// The marks: an adjective takes the next noun's gender, the article is none after a noun, the text form and the
    /// buffer form differ on marks 12 and 13.
    #[test]
    fn marks() {
        assert_eq!(d(NONE, 0, b"blau\x01 S\x85\xc3uretester\x05"), b"blauer S\x85\xc3uretester");
        assert_eq!(d(NONE, 0, b"dunkl\x01 Mana-Kugel\x04"), b"dunkle Mana-Kugel");
        assert_eq!(d(DEFINITE, 1, b"alt\x01 Stiefel\x0f"), b"alten Stiefel");
        assert_eq!(d(NONE, 2, b"alt\x01 Stiefel\x0f"), b"alten Stiefeln");
        assert_eq!(d(1, 0, b"klein\x01 Hut\x02 gr\x01 Band\x03"), b"kleiner Hut gres Band");
        assert_eq!(d(NONE, 1, b"Hut\x02"), b"Hutes");
        assert_eq!(d(NONE, 0, b"neu\x01 Typ\x0c"), b"neuer Typ");
        assert_eq!(decline_buffer(64, NONE, 0, b"neu\x01 Typ\x0c"), b"neu Typ");
        assert_eq!(decline_buffer(3, NONE, 0, b"abcdef"), b"abc");
        // A mark byte never matches a trail byte: 0x85 0x05 is one character.
        assert_eq!(d(NONE, 0, b"\x85\x05x\x04"), b"\x85\x05x");
    }

    /// '*' and '~' in item names: "rostig* Eimer", "Aquarium~ (Fluss)", "Hand~ voll ...".
    #[test]
    fn item_templates() {
        let eimer = DeItem::new(0, 0, 2, b"rostig* Eimer", 0, 1, b"rostig* Eimer");
        assert_eq!(eimer.display(NONE, 0), b"rostiger Eimer");
        assert_eq!(eimer.display(DEFINITE, 1), b"rostigen Eimers");
        assert_eq!(eimer.plural(NONE, 2), b"rostigen Eimern");
        let aq = DeItem::new(0, 2, 2, b"Aquarium~ (Fluss)", 0, 0, b"Aquarien~ (Fluss)");
        assert_eq!(aq.display(NONE, 1), b"Aquariums (Fluss)");
        assert_eq!(aq.plural(NONE, 0), b"Aquarien (Fluss)");
        let hand = DeItem::new(0, 1, 1, b"Hand~ voll Bolzen", 0, 1, b"H\x85\xc3nde~ voll Bolzen");
        assert_eq!(hand.display(NONE, 2), b"Hand voll Bolzen");
        assert_eq!(hand.plural(NONE, 2), b"H\x85\xc3nden voll Bolzen");
        // A '*' after the '~' is dropped for a one-part name, kept for a two-part one.
        let odd = DeItem::new(0, 0, 0, b"a~b*c", 0, 0, b"");
        assert_eq!(odd.display(NONE, 1), b"aesb*c");
        // A two-part name shows its plural; its plural form is the first part (a feminine 'e' takes "n" and a plural
        // ending) then the plural with no article.
        let two = DeItem::new(1, 1, 1, b"Kiste ", 0, 1, b"gut* Dinge");
        assert_eq!(two.display(DEFINITE, 0), b"guten Dinge");
        assert_eq!(two.plural(DEFINITE, 2), b"Kiste guten Dingen");
        let two = DeItem::new(1, 1, 1, b"Kiste", 0, 1, b"gut* Dinge");
        assert_eq!(two.plural(DEFINITE, 2), b"Kistennguten Dingen");
    }
}

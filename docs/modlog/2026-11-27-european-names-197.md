<!-- dll-build: 30261102_0 -->
## German and French names: items, key items, titles (2026-11-27)

- The client language picks each name builder (DAT_10457bd8 +8; the European flag does not switch these to English).
  German names are declined; French names are record fields as they are. Ours had the first text field (or the short
  item name) for both, with the German marks left in.
- German engine (dat::grammar), on Shift-JIS bytes one character at a time (FUN_1011ef10). Byte 1 ends an adjective
  stem; bytes 2..15 end a noun and give its class. An adjective's ending follows the next noun mark (FUN_10259800):
  marks 2 5 7 12 masculine, 8 10 masculine with the flag, 3 6 9 11 13 neuter, 4 feminine, 14 15 plural. Endings
  FUN_10259780(article, case, plural, gender, flag): no article 0x1038ae78 (0x1038aeb8 for a flagged masculine, its
  other columns empty), definite 0x1038aef8, indefinite 0x1038af38; rows nominative, genitive, dative, accusative,
  columns masculine, feminine, neuter, plural. Noun endings FUN_10259670(case, plural, class): 0 "es" genitive, 1
  none, 2 "s" genitive, 3 "ns" / "n" / "n", 4 "en" outside the nominative, 5 "ens" / "en" / none, 6 "n" outside the
  nominative, 7 "ses" genitive; plural only class 1 "n" in the dative. Marks 2 3 class 0, 4 12 13 class 1, 5 6 2, 7..11
  3..7, 14 plural 0, 15 plural 1. After a noun the article is none. FUN_10259a60 (text) and FUN_10259840 (buffer of n
  bytes, stopped once n are out, a piece never cut) differ only on marks 12 and 13: the buffer form gives them no
  adjective gender and no noun class.
- Items (FUN_1015d3d0 -> FUN_1015ec00 / FUN_1015ecd0, only when the record has exactly 8 / 5 fields before its
  descriptions). German fields 1 two-part, 2 gender, 3 class, 4 singular, 5 / 6 the plural's gender and class, 7
  plural (vtable 0x103362c8). In the name texts '*' ends the adjective stem and '~' is where the noun's ending goes,
  the last of each counting (FUN_10157530; a one-part name drops a '*' after its '~', a two-part one, FUN_10157650,
  keeps it). FUN_10156cc0 / FUN_10156e70: the stem, its ending (flag: class 4), the noun to the '~', its ending, the
  rest. The log name FUN_1015b4f0 shows is vt+0x20 with no article in the nominative (a two-part name its plural);
  the plural vt+0x18 (FUN_10157960; a two-part name's first part, an "n" and a plural ending after a feminine 'e'
  (FUN_10157070), then its plural with no article). French fields 1 / 2 (unread here; *(inf: gender, elision)*), 3 singular, 4 plural: the
  log name is field 3 (vt+0x1c of vtable 0x10336320, mode 3). Ours: Item.singular / plural hold these for German and
  French, so the item window's title (FUN_1013e2c0), the puppet help and `log_name` have them, and Items::noun gives
  them to message text.
- Key items (FUN_10155670). German fields 1, 2 whole, 3 gender, 4 singular, 5 plural, 6, 7 the rest (FUN_10156400,
  noun vtable 0x10336240): shown (vt+0x20, FUN_10156300) the rest alone when whole, else vt+0x10 (FUN_101561c0) the
  singular declined, then " " and the rest with no article when whole; plural vt+0x18 the same with field 5. French
  fields 5 / 6 singular / plural, 9 the rest (FUN_101569d0 -> FUN_10157fc0: mode 2 with a rest, else 3): shown
  (vt+0x1c) the rest alone or the singular, plural (vt+0x10) field 6 then " " and the rest. The key item list takes
  the shown name (FUN_10155b40 -> FUN_10155880). Descriptions were already by language (FUN_10155a30).
- Titles (FUN_10158450): German [gender, male, gender, female] (FUN_10259ce0) declined in the buffer form with no
  article in the nominative (FUN_10259d40); French [x, y, male, x, y, female] as they are (FUN_1025a090 /
  FUN_1025a100); Japanese and English field 0. The gender: the profile (win_persona_v10_2 -> FUN_101586f0, into the
  rest of a 200-byte line) our actor's flags (German FUN_1008aab0 bits 4..5, French FUN_1008ab00 bit 6, which
  FUN_1009afe0 sets female for races 2, 4, 6, 7); s2c 0x0CA's title line (FUN_10158940, 0x80 bytes) the race at
  @0x7f >> 2 the same way. Ours: Names::title_of(id, female, size); the plain `title` (message text, mods, the GM
  list) is the male form.
- Item names typed in /equip and /item (FUN_1007b276, FUN_101f12d0, FUN_101f1630 through DAT_10664118, set by
  FUN_101c9f20): Japanese the short name byte for byte (FUN_101ca100), English the short or the log name in any case
  (FUN_101ca150, _stricmp), German and French the short or the log name through FUN_101c9d00(.., 0) (table
  DAT_1037e358: case off, accented letters apart). Ours matched the short name in any case in every language:
  dat::items::named now.
- Tests: dat::grammar::tests (the tables, the classes, the marks, the item templates), dat::names::tests::
  european_names (the installed German and French files).
- Left (queue rows 235, 236): the German and French message grammar (articles and cases in text) and the actors'
  grammar flags it reads.

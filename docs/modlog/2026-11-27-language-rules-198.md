<!-- dll-build: 30261102_0 -->
## Capitals, party languages and search languages by client language (2026-11-27)

- Capitals (FUN_1025d4b0 / FUN_1025d580 / FUN_1025d5e0, client language DAT_10457bd8 +8, European flag +0x18 bit 7):
  Japanese leaves every char as it is; English and German (and French with the European flag) take FUN_1011f090 up
  (a..z, 85 BF..D5, 85 D7..DD less 0x20; 85 5C / 5A / 5E to 85 4C / 4A / 4E; 85 DE to 85 5F) and FUN_1011f150 down (the
  same the other way); French puts its accented small vowels up as plain capitals (85 BF / C1 A, 85 C7..CA E, 85 CD /
  CE I, 85 D3 O, 85 D8 / DA / DB U) and the other chars as English. A char's code is FUN_1011ef30's: a lead byte and
  the next as one word, else the byte sign-extended, so a lone byte from 0x80 (half-width kana) goes out of the Latin
  rule as FF xx (FUN_1011ef70).
- FUN_1025d720(out, cap, s): the first char up, then whole chars while the text stays under `cap` (FUN_1011f310); with
  `cap` no more than the first char's bytes it returns 0 and writes nothing. Its callers: the item info name
  (FUN_1013e2c0, 0x80), the automaton's attachment help (FUN_1020a3a0, 0x80), the auto-translate list's rows
  (win_fep_v10, 0x168), key item blocks (FUN_101bd6d0 kind 0x13, the record name FUN_10155af0; item blocks, kind 7,
  stay as the record has them) and the help bar's FUN_101522b0. FUN_1025d7a0 (first char only) draws the key item,
  mission and quest entries (FUN_10234890 rows, FUN_102347a0 description names, FUN_10234990 row sizes).
- The message formatter's case (FUN_100c2bf0, mode from FUN_100be760 +0xc): 1 the first char up whatever it is (a
  digit stays and ends it), 2 every char up, 3 every char down, 4 up and down by turns from the first char (every char
  counts, a space too); 1E colour codes are kept and not counted.
- Party languages (prty5's handler, FUN_10207070 / FUN_10207120 / FUN_102071b0): row 1..5 -> mask bit by table
  0x10386450 at client language x 5: ja 0 1 3 2 4, en 1 3 2 0 4, de 2 1 3 0 4, fr 3 1 2 0 4 (bits 0 Japanese,
  1 English, 2 German, 3 French, 4 Other; the European flag not applied). The layout is the file language's (prty5 /
  prty5us / prty5de / prty5fr).
- The auction's bid list query (FUN_1020bfe0 case 5 -> FUN_100e2ac0) puts the client language as a word in the search
  request @0x0e; the histories (cases 6 / 7) put 0. /itemsearch (FUN_1007b0a0 -> FUN_100ec490): c2s 0x02C @4 the
  client language, 1 for German and French with the European flag; @8 the name strncpy'd into 0x40 bytes.
- German quotation marks in the log (FUN_10129090 -> FUN_10191710, buffer 0x800): only for a line whose mode word's
  second byte has no low five bits. s2c 0x017's lines (FUN_100d9b20: 0, 0x20, 0x40, 0x80) and many text command
  echoes (a mode stored as a plain int) have them; the group 7 / 9 printers FUN_1012b270 / FUN_1012b2c0 / FUN_1012b310 /
  FUN_1012b340 ({0x79 or 0x7b, 3}) and the search comment ({0xcc, 3}) do not. The search comment printer FUN_101ded50
  puts the marks on each row itself (buffer 0x52: a row of 0x29 bytes or more stays as it is) after the "..." cut.
- Ours: dat::case (cap_first / cap_str / apply_case); the formatter's mode 1 no longer waits for a letter and mode 4
  is the turns (the status title's word capitals were ours, kept as battle.rs word_capitals); event text's 7f 80 1
  (dialog.rs) takes FUN_1025d5e0. ChatLog keeps whether each line takes the log's quotes (chat lines; not the search
  comment's or /translate's lines).
- Not built (rows 237..240): the help bar's capitals (FUN_101522b0's callers), German quotes in the other text fields
  of FUN_101c76b0, the log quotes of the non-chat printers, the item rows of FUN_10232860; whether PlayOnline sets the
  European flag is not known (polcore not here).

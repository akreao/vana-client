## Grammar 0x81 / 0x83 and the choice mask's sign (2026-11-17)

Addresses are 30261006_0's.
- 0x81 / 0x83 (FUN_100bdc50 cases 0 / 2): a table byte by FUN_100bdb80 (0x40 | k: slot k's byte; low 6 bits 1:
  (b1 & 0x7f) ^ (b2 << 7) as a byte; else 0xff), then entry num[n] by FUN_100bdbe0. Nothing is appended when the
  lookup gives no string.
  - 0x83 FUN_1022ec10: mission log table (objects at 0x10667a90 + t * 0x94, t < 0xf, else none) entry n, its leading
    '_'s skipped. The 15 tables are language slots 0x49..0x4c, 0x4d..0x51, 0x55..0x57, 0x52..0x54 (FUN_1022d7a0):
    dat/journal.rs MISSION_FILES in that order.
  - 0x81 FUN_1022f330: quest area table (0x10668380 + t * 0x94; slots 0x40..0x48, 0x58, 0x59, FUN_1022f3d0) entry n,
    one leading '+' skipped (no bound on t). dat/journal.rs QUEST_FILES.
  - Both index the file's entries in order (FUN_1022edf0: none past the count).
- First seen in Beaucedine Glacier [S]'s Regal Pawprints (event 0x17, message 8608 "Attempt which objective?"): row
  1 mission log 7 num[0], row 2 log 8 (WotG) num[1], rows 3..8 quest area 7 (Crystal War) num[2..7]; the script
  hides each row whose param is 0. LSB's params [0, 22] show "None." and "Dungeons and Dancers.". Ours printed the
  rows as "." before.
- FUN_100b0960 shifts the choice's hide mask as a signed int (`(int)mask >> 1`), so past row 32 its sign bit hides
  (or shows) every further row. Ours shifted a u32: rows 33 and on always showed (the battlefield menus' trailing
  "." rows, e.g. Horlais Peak 7611).

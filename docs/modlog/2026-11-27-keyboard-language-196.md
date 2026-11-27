## The software keyboard follows the client language (2026-11-27)

<!-- dll-build: 30261102_0 -->

Modlog-gaps item 196 (split from 183): every part of the software keyboard that reads the client language
(DAT_10457bd8 +8, 0 Japanese) or the text engine's language (T+0xf0ec, 0 Japanese). German and French are English
throughout. Ours had the English keyboard only, in every language. The window is the handler DAT_10663f64 (vtable
0x10338a88, made once by FUN_101ba440).

- Window: FUN_101b0100 / FUN_101b0150 / FUN_101b0180 open, close and test "menu    keypad  " in Japanese and
  "menu    keypadus" in the others. The Japanese layout is 366 x 88 in the Japanese menu file (39532): children 1..5
  (tabs) at the left, the 75 keys from x 57, 6 / 7 / 0x53 / 0x54 / 0x55..0x57 at the right.
- Caps: FUN_101b0090 (the draw's library) and win_keylayou_v04 (Config's keyboard picture) load key1top in
  Japanese and key1topu in the others. Ours' library lookup already maps key1top* by language (FUN_1025cbd0 row 5).
- Open (win_keypad_v04): the 75 keys' sprites are off (FUN_1011a740, caps are drawn over them). In Japanese also
  child 4, SHIFT, SPACE and Enter, and the tabs 1..3 when the page at the open is a single-byte one. In the others
  children 1..5 and 0x54 are state 0x10 (FUN_10119ff0: neither drawn nor hit).
- Draw (win_keypad_v10): each key's cap at the child + (1, 1) (FUN_101b0d20; the cursor's 0x80805020, a dim cap
  0x40402810). In Japanese also child 4 the page's cap ([3]), SHIFT 0x13 / 0xc and SPACE 0x13 / 0x1ad (single-byte
  page / not), Enter 0x1b1, and the tabs 0x11 on a single-byte page (FUN_101b0dc0, no dim).
- Dim caps (FUN_101b0200): 0x15a..0x171, 0x177..0x18d, 0x172 / 0x173 / 0x1ae..0x1b0; 0x114 / 0x132 / 0x133 only
  outside Japanese.
- Pages (table 0x1037d320, 13 records of 7 dwords: keys, caps, kind, child 4's cap, single-byte flag, page after
  SHIFT, page after X): 0 hiragana, 1 katakana, 2 full-width letters, 3 half-width letters (kind 1), 4 symbols, 5
  half-width symbols and European letters; 6..12 the others' (as before).
- First page (FUN_101b0b50(T+0xf10c, T+0xf110)): with mode 1 the field's class picks it (0 -> 9, 1 -> 10, 2 / 5 -> 3
  in Japanese and 11 in the others, 3 -> 6, 4 -> 8); mode 2 -> 9; other modes keep the page (+0x2c: FUN_101b0010
  starts it at 0 in Japanese and 0xb in the others), or 0 on a single-byte page.
- The mode (T+0xf10c, FUN_101b6e20): 1 outside Japanese, the field's own in Japanese, where 0 sends what is typed to
  the IME's composition (FUN_101b2dd0 kind 6 -> FUN_101b2b40). The chat line opens with 0. Ours has no composition,
  so ours' fields are direct (mode 1) in every language.
- Page changes: the tabs (FUN_101b11b0: child 1 page 0, child 2 page 2 / 3, child 3 page 4 / 5, the lower while on
  page 0, 1, 2 or 4 by FUN_101b01b0; mode 1 lets a class 2 field have page 3 only, a class 5 field 3 and 5), SHIFT
  (FUN_101b1150, page [5]) and X (FUN_101b1180, page [6]), both through FUN_101b1110 (mode 1 refuses 0, 1, 2, 4). A
  tab or SHIFT that does not turn buzzes (SE 0004); X does not.
- Keys (FUN_101b0e30, table 0x101b10ac): child 5 closes (kind 8); 6 Backspace; 7 Tab, Escape in Japanese; 0x53
  Config, nothing in Japanese, where Config is 0x54; 0x55 is SHIFT outside Japanese, in Japanese Tab in mode 0 and a
  buzz in mode 1; 0x56 a space outside Japanese, in Japanese the conversion (edit mode +0x14) when there is a
  composition in mode 0, else a buzz; 0x57 Enter.
- Typing (FUN_101b0c20): kind 1 the byte. Kind 2 the pair, made one byte outside Japanese or on page 5
  (FUN_101b5a20: tables 0x1037d568 from 0x20 and 0x1037d730 from 0xa0; a European letter, lead 0x85, types nothing;
  0x8f81 and pairs with no byte stay pairs, typed as they are, kind 6); the full-width space 0x8140 types a space.
- Pad (win_keypad_onkey): Y (message 0xc) is Enter and RB (0x11) a space outside Japanese; in Japanese RB is Enter
  and Y the conversion (mode 0, else a buzz).
- Ours (client/swkbd.rs): the window, caps, pages, first page, page changes, keys and pad by language as above, with
  mode 1 and class 5; the modern layer's keyboard gets the tabs, Close (5) and Config (0x54) in Japanese, its SHIFT and
  SPACE shown inert there (they act on the composition only) and the space keys marked ␣.
- Not language reads: FUN_1019f5c0 (installp in Japanese, installu in English, nothing in German / French) is called
  only by FUN_1019fe50 (the install credits, no caller) and by FUN_1019ff10 unless +0x57 is set, which the ending
  credits (FUN_10164700) set first with FUN_101a0330(1). 0x101b029a is FUN_101b0200's jump table, not code.
- Not built: the Japanese composition (mode 0, the edit mode, feppalle) and the compact keyboard (config 0xc9 = 2
  in Japanese: FUN_101b29c0 opens keyselec through FUN_10191f50).
- Tests: `client::swkbd::tests::pages`, `pairs`, `tables_are_ffximain`, `layouts_and_caps`, `japanese`.

<!-- dll-build: 30261102_0 -->
## The client language's other reads, sorted (2026-11-27)

- The reads of the client language (DAT_10457bd8 +8; German and French as English with the European flag +0x18
  bit 7) that rows 196..198 left, sorted by what they do. Built here:
  - Key function 0xb4 (k2assign row 28, config 0x86, Q by default; Ctrl / Alt as the other channel keys): its entry
    (+0x1ffc of FUN_101af710's table, made by FUN_101af490) holds two texts, "/assiste " then "/assistj " (Japanese
    the other way round). FUN_101af4f0 with FUN_101af5b0 steps to the other text when the key is pressed while the
    line is in an assist mode (FUN_101b9c10: fep +0x2c is 0xe / 0xf), wrapping after the second; any other time the
    last text stays. FUN_101afb40 puts it before the line's body as the other channel keys do. Ours: Ctrl / Alt + Q,
    World.assist_key.
  - The "New Challenge" scheduler (FUN_10087230): file 0xaf0 + table 0x1035c53c by language, 0x396 Japanese and
    English, 0x398 German, 0x397 French (roe.rs NEW_CHALLENGE; ours took 0x396 in every language).
  - /translate (FUN_10079b30, argument count _DAT_1047e640): no letters, from and to the client language; one letter,
    from the client language to it; two, from and to (later letters unread). A letter (FUN_10079c10) is the first
    byte of message 0xc 0x5e..0x61 or that + 0x20 (J, E, G, F in English): G / F only without the European flag; any
    other letter is the client language (English for German and French with the flag). Ours sent English as the
    source and English for an unknown letter.
  - The server message request (FUN_100a30e0 -> FUN_100a3040): language 1 Japanese, 2 English, 3 German, 4 French;
    the pieces must carry the one asked for (FUN_100a2c30, +0x5b). Ours always asked for English.
- Already built elsewhere: /itemsearch's byte (FUN_1007b0a0, row 198), the dictionary's compares and its `@` names
  and French resolver (FUN_101b67e0, FUN_101bde00, FUN_101bded0, FUN_101bbba0, FUN_101bc410, row 162), typed item
  names (DAT_10664118 from FUN_101c9f20, row 197), the party language bits (row 198), the item info's text x
  (FUN_1013e5f0). Nothing to build: the debug switch FUN_101d3fe0 ("japanese" .. "french"); FUN_10225120 /
  FUN_102278c0 / FUN_10227930 / FUN_102279a0 make the same call in every language (Rizin: each jump table entry
  pushes the same string number).
- Split to rows 241..252 (not built): the message formatter's language cases; the date and time text FUN_101ca5a0;
  typed spell and ability names (DAT_10664118, string 1 in German and French); buff names' grammar; text widths and
  line cuts; the item info's layout; window layouts (battle and status, item and conquest, the lobby); keyboards and
  text input; Help Desk; the rest (movie subtitles, the region word, tskill2, message ids). FUN_10191520 /
  FUN_101915f0 (German text with its quotes) go with row 238.

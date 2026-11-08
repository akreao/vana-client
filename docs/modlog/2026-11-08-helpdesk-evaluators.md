## Help Desk: the evaluators, the '#' page, 9 0x7c, "Input:REQUIRE" (2026-11-08)

- The parameter evaluators are registered by FUN_10114b60 (FUN_1010fc50(group, key, callback)) when the session is
  built: GENERIC.PLATFORM "win" (LAB_10114cf0); GENERIC.VERSION (LAB_10114d00): "JP/" for language 0 else "EN/", then
  "S%08x/C%08x/" with S = DAT_10456a28 + 0x14 (the lobby's content bits 1..31, FUN_100ed7e0 step 0xf) and C = + 0x10
  (the installed mask), then the string at DAT_10662dcc + 0x10 *(inf: the client version)*; GAME.INFIELD "off" when
  session flag 0x100 is set (S+0x3c35c, FUN_100fc160; set with 0x200 / 0x2000 / 1 by FUN_100fa9f0 on 0x00A: our
  room_flag) else "on" (LAB_10114dc0); GAME.GMTIME "1" when the UTC minute (time() through gmtime FUN_101c8d10, tm_min)
  % 10 < 5, else "0" (LAB_10114df0); EXEC.LOGOUT / POLEXIT / RESCUE and GMREPORT.NOTICE / GMCALL / HARASSMENT / STUCK /
  BLOCK "on" (LAB_10114db0). No callback for GENERIC.AREACODE / CLILANG / SELLANG (Execute sets them) or
  GMCALL.INPUT (the field). This also answers "session flag 0x100": the room flag.
- The '#' pages: in session state 5 a non-question node whose LABEL starts with '#' is drawn by FUN_10219b80 instead
  of the leaf page: the text 9 0x7c "Carry out selected process?" (the only place 9 0x7c is drawn), row 1 9 0x7d
  (Execute), row 0 Go back (9 0x79) with history, row -1 Return (9 0x7a). +0x14 starts at 0 (no row marked). DAT 5813
  has no '#' labels, so nothing in today's script reaches it.
- "Input:REQUIRE": FFXiMain has neither "Input:" nor "REQUIRE" among its strings, so the client does not parse the
  keyword at all and nothing enforces it (any check is the server's). The err_%03d.bin viewer (str_err_bin*) is a
  debug path, not built.
- Ours (faqmain.rs): Env::of / value (the evaluators, taken when the page opens), Page::Confirm (rows [1, -1] plus Go
  back with history; Enter on 1 runs Execute; the cursor starts on Execute without history *(ours: retail's 0 there
  would be the GM reply's row 0)*; leaf pads *(inf)*). Not seen live (no '#' node; INFIELD "off" only in a Mog House).

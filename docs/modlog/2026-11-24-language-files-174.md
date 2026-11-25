<!-- dll-build: 30261102_0 -->
## Every language file and menu library follows the client language (2026-11-24)

- The client language is DAT_10457bd8 +8 (0 ja, 1 en, 2 de, 3 fr; row 162). Every language-dependent file the DLL
  reads comes from one place: FUN_1025cb90(slot) = FUN_1025cb20(language) then the slot's id in that language's table
  (ja 0x1038bbd0, en 0x1038b610, de 0x1038b450, fr 0x1038b7d0, 112 slots each). German and French read the English
  table while +0x18 bit 7 is set. Nothing reads DAT_1066a61c or the four tables any other way, so a file that has
  per-language copies is always the file language's. FUN_1025cb90 has 40 callers.
- Slots read here (current numbering): 0 the in-game menus (FUN_10010d20), 1 the lobby / title menus (FUN_10011a00),
  0x40..0x48, 0x58, 0x59 the quest areas' logs, 0x49..0x57 the mission logs, 0x6b / 0x6e / 0x6c / 0x6d the zone dialogue
  ranges 0..255 / 256..999 / 1000..1999 / 2000.. (FUN_100ae1e0; German and French have no 0x6e, its entry is 0, so
  retail reads file zone - 256 there), and the message groups (FUN_101551e0: groups 8 and 10 by language are the same
  slots 8 and 10 through FUN_101550c0 / FUN_10155100; group 0 has no German or French file).
- FUN_1025cbd0(row) (table 0x1038b998, four names a row): 0 frames / framesus / framesde / framesfr, 1 comwin (de /
  fr comwinde / comwinfr), 2 resyn (resynus / de / fr), 3 compass, 4 sc_item, 5 key1top (key1topu in the others),
  6 jobcsel (jobcselu), 7 prty5 (prty5us / de / fr). Rows 3 and 4 have no caller. FUN_1011c840 / FUN_1011cb40 /
  FUN_1011cda0 (layout loads) replace a sprite reference whose library starts "menu    frames" or "menu    comwin"
  (_strncmp 0xe) or "menu    key1top" (0xf) by rows 0 / 1 / 5, and "menu    lobbyps2" by lobbywin. Row 0 is also
  asked by FUN_101c9f20 and FUN_10217c60, row 2 by the raise prompt (FUN_101d2790), row 6 by the job list
  (FUN_10146a80 / FUN_10146dd0 / FUN_10146e90), row 7 by the party language window (FUN_10206fb0).
- Ours: dat::lang::set runs once in main before anything loads (it ran at the world entry, after the menus and the
  battle text were already loaded in English). Every slot and group read (GameUi data, battle text, command help, the
  blue magic, point and message files) passes dat::lang::files(); names, items, the menu and lobby files (ui::MENU /
  LOBBY), the journal logs and the zone dialogue take the file language's id from their per-language tables; Font::load
  reads the language's menu file. ui::lang_lib is FUN_1011c840's prefix rule over lang::lib's table, so our code's
  "menu    framesus" names framesde in a German client as retail's FUN_1025cbd0(0) does; ui::lang_menu maps the three
  windows opened by rows 2 / 6 / 7. The event VM's language (op 0x9c, FUN_100ba100) reads dat::lang, so the European
  flag is lang's (clear) there too; it had German and French as English.
- Status names (battle text grammar kind 0x13, the title bar): FUN_102287e0 loads slot 0x5c whole (English 55732, the
  same bytes as the 55725 ours read by a fixed id); ours now reads the slot (test dat::battle::tests::status_names_slot).
- German and French data in current installs look frozen: de slot 0x6b + 230 = 58207 is an hm_0 block, and the de / fr
  item files for 0x0000..0x4000 hold no item text (0x2200's do). Retail with the European flag clear reads the same
  files and fails the same way (FUN_100ae1e0 logs "EventMess file ga nee"), so ours does not fall back to English;
  whether PlayOnline sets the flag is queue row 183's question.
- Test dat::lang::tests::files_are_slots: every per-language id set ours keeps is one slot of the DLL's tables, the
  dialogue ranges match, and lang's library names are FUN_1025cbd0's table.
- Left (queue row 183): the DLL's other reads of the language, which pick per-language rules rather than files.

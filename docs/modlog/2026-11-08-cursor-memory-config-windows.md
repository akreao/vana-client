## Cursor memory: myroom, mogext, cmbmenu and configwi (2026-11-08)

- The last KEEP windows from "Cursor memory and the held action menu": myroom / mogext (open hook win_myroom_v04 only
  dims rows), cmbmenu (FUN_101e2950 only dims rows) and configwi (menu_configwi_open) are opened by name without
  registry flag 0x10000, so FUN_10118870 puts the cursor back on the child they were closed on. Ours: World::cursor_keep
  keeps them and moogle::open, synth::open_menu and the Config opens (the main menu, the software keyboard) recall
  them; 1 the first time.

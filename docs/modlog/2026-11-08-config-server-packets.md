## Config: the Show / Hide packet and the macro palette's narrow-screen rule (2026-11-08)

- Misc.'s Show / Hide (conf3win children 3 / 4, FFXiMain.dll checked 2026-11-08): the page opens on our anonymous flag
  (win_conf3win_v04: FUN_100e47c0(0) = flags +0x2cfd8 bit 2), a pick only marks the page (+0x38 value, +0x3c = 1), and
  closing it (win_conf3win_v08) sends c2s 0x0DC which 0 (flag byte +4 bit 2, SetFlg 1 hide / 2 show) when anything was
  picked, even the value already held. Ours: ConfigUi::anon follows 0x037 flags bit 5 until a pick, configui2::closed
  hands it to config_out, which sends textcmd::config_flag(0, hide). The modern Config page has no conf3win window,
  so its pick goes out at once.
- Misc. 2's macro palettes (FUN_1019b6d0, run by conf6win's draw and by config 0x9d / 0x9e changes): under 1024 wide
  the long palette row (child 12) is off and config 0x9d is written 1; under 640 wide the palette position rows
  (children 14 / 15) are off too (window manager +0xb6 = -1, the palettes never move). Ours: configui::disabled and
  ConfigUi::narrow (while conf6win is open, and on the modern page's conf6win rows). The palette placement itself
  (macroedit::palette_dx) already matched.
- Auto-disconnect (Global, 0x41) stays as it is: its timer is PlayOnline's.

## Config: the chat filters go to the server (2026-11-08)

- Config 0x32 / 0x33 (the Chat Filters masks) and 0x35 (System Lv.) live on the server too (FFXiMain.dll checked
  2026-11-08). Their setter LAB_101aa980 writes them into the session's SAVE_CONF copy (+0x2cfdc, +0x2cfe0, +0x2cfd9
  bits 3..4) and sends c2s 0x0DB kind 0 (FUN_100e4610 -> FUN_100e4630: @8..@0x13 the three words, @0x24 the party
  languages +0x2cff8) for every change; "Config send failed" is logged when it cannot. s2c 0x0B4 has a second handler
  FUN_101aaa50 (registered by FUN_101ab6a0 after the session's LAB_100e4f20) that sets 0x32 = @8, 0x33 = @0xc,
  0x35 = byte @5 bits 3..4; the setter only runs on a change, so the first 0x0B4 after login is echoed back once (LSB's
  0x0DB handler says it expects the client to send the filters at login).
- Ours: cfgapply::server_conf (from config_out's changed ids) and cfgapply::from_server (NetEvent::ConfSys);
  partymenu::send_0db builds the packet for both kinds. The Mouse/Cam. mouse reference (0x90) stays off (our mouse
  camera, Steven's choice), and 0x3e clipping / 0x40 animation rate were already in use.

## Cursor memory: myroom, mogext, cmbmenu and configwi (2026-11-08)

- The last KEEP windows from "Cursor memory and the held action menu": myroom / mogext (open hook win_myroom_v04 only
  dims rows), cmbmenu (FUN_101e2950 only dims rows) and configwi (menu_configwi_open) are opened by name without
  registry flag 0x10000, so FUN_10118870 puts the cursor back on the child they were closed on. Ours: World::cursor_keep
  keeps them and moogle::open, synth::open_menu and the Config opens (the main menu, the software keyboard) recall
  them; 1 the first time.

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

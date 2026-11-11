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

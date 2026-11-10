## The chat line with a window open, and the Full keyboard's typing (2026-11-10)

Issue lenonk/vana-client#173.

- The chat line's gate is the input object's vtable +0x40 (vtable 0x1032c3e0; FUN_101a7fa0 in 2019, FUN_101b0060 in
  30261006_0): the UI not hidden and no cinema (FUN_101573f0), the window manager not busy (+0xae), game mode 0x60
  and the zone loaded (FUN_10085930: DAT_10472fe0 clear, DAT_1047054b set at the zone-in's end *(inf: "loaded")*);
  or game mode 0x40 / 0. Space (FUN_101ab630, WM_CHAR) also needs mode 0x60 and the input idle (+0xf108 == 3). No
  window is tested: the line opens over the Items window and every other. MODLOG "Input layer"'s "in the field" is
  the game mode, not the absence of windows; the layout mode note's "no layout window up" was ours.
- FUN_101aa930 on a key down, not typing: nothing with Ctrl or Alt held (DIK 0x1d / 0x9d / 0x38 / 0xb8); VK 0xbf
  opens the line (the '/' that follows is typed); else on the Full keyboard (FUN_101afe20: config 0x42 = 0) a key
  FUN_101ac1b0 takes (VK A..Z, 0..9, 0xba..0xbe, 0xc0, 0xdb..0xdf, 0xe2; FUN_10253940: not Compact 2) opens it and its
  character is typed. The numpad's digits are VK 0x60..0x69: not typed. Ours had no such typing at all.
- Ours: Space, '/' and those keys are taken before every window and event (`chat_opens`: not in cinema mode; ours
  leaves Key Assignment's key wait its keys). The keys are matched by the event's physical key (Shift + 1 is still the
  1 key, as a VK is). winit's key location tells the numpad from the main block (egui maps both to one key); a mod's
  keys go by DIK (0x37, 0x47..0x53, 0x9c, 0xb5); a test drive's keys are the main block's. The modern list search
  keeps the letters it takes (ours); the plain-T Timers toggle (ours) is left to the compact keyboards.
- Seen live (Lnxcsfix, classic, Full keyboard): with the Items window open, Space, '/' and H each opened the chat line
  ("", "/", "h") over the window, and Escape closed it with the window kept. Not tried live: the numpad's digits on
  the Full keyboard (a drive cannot press them), the modern UI.

## Middle mouse button: message 7, the item info page; Maze Tabula open points (2026-11-08)

- FFXiMain.dll (checked 2026-11-08): the mouse object DAT_104e1d4c keeps its buttons as bits 1 left, 2 right, 4
  middle (+0x5c held, +0x60 press edge, +0x58 release); key functions 0x8b / 0x8c / 0x96 read them (callbacks
  0x10120980 / 0x10120a20 / 0x10120ad0). Key function 0x18's callback (keyfn_18_19_25_cb, FUN_10121000) reads the
  pad object DAT_104e1d44 (bit 2), not the mouse: Num- / - with Ctrl or Alt and the pad, no mouse button.
- FUN_1015fa00 each frame: 0x96 release -> FUN_1015df20: not with keyboard type 2 (FUN_1025e070, config 0x42 == 2,
  where 0x96 replaces the left button in the move drag) and only with input-context flag 4 (FUN_10123e50(4)):
  FUN_1015dd00(7): nothing in cinema mode / with the UI hidden; with a focused window (FUN_1015edb0), message 7 to it
  (FUN_10118260); else (or with WM +0xb4 == 1) menuwind opens. Message 7 is the one key function 0x18 gives a window
  (prvdungp's onkey FUN_101fe290 case 7 = FUN_101fdbf0, the turn).
- The middle press bit (+0x60 & 4) turns pages beside key function 0x18: the RoE description (FUN_102326b0, already
  built) and the item info window (FUN_1013cba0 -> FUN_1013c220, also iteminfo / itemxinf's per-frame hooks).
- Ours: a middle click (a press and release without a look drag; ours drags the camera with it) runs
  `middle_release`: the layout / gardening contexts (no flag 4) and cinema mode skip it, keyboard type 2 does not
  (ours: every type looks with a middle drag, and 0x42 = 2 is the default, so the retail skip lost the click); the
  maze editor turns the carried rune (its grid focused, as before, but now wherever the pointer is); with no window
  or only the main / action menu up, the next frame gets a Num- press (the main menu opens / closes); with another
  window up nothing happens *(inf: the windows ours has take no message 7 of their own)*. Not over a modern panel
  (egui keeps its pointer; the modern maze grid has its own middle click). A middle press also
  turns the item info window's page (`info_page_turn`, shared with key function 0x18). maze::pointer no longer reads
  the middle button.
- Maze Tabula open points (MODLOG "Moblin Maze Mongers: the Maze Tabula editor", Open):
  - d5 bits 4..10: the encoder FUN_101fd600 starts from a zeroed buffer and writes d5 bits 0..3 only (runes 10 / 11's
    rotation), so a client save clears them; only the R iteminfo reads them. Their meaning is server-side (not in
    the DLL); ours matches (encode leaves them 0).
  - The late 0x0D8: confirmed. FUN_1015ef60 broadcasts 0x800 through FUN_1015f3e0 when FUN_10087070(0) turns on
    (DAT_10487e48, the event running); prvdungp (flags 0x00420002, 0x400000 set) closes by FUN_1015e910, whose close
    hook FUN_101fd1d0 only closes bluinven / bluehelp. FUN_101fcfb0 (mode 0, slot >= 0, decoded) then saves at the
    event end. Ours keeps it.
  - helpwind with the grid focused: empty. The help line is the focused window's +0x3c (FUN_1011ae10); the WM window's
    ctor FUN_1012ab20 zeroes +0x36..+0x4d and no prvdungp function sets it (only prvdunsi's v04 / v08, 0xe 0x2a4).
    Ours already shows nothing.
  - Pulse / glint timing: stepped once per draw call (FUN_101fe0b0, the window's +0x10 draw), so once per rendered
    frame; ours steps at 30 a second, retail's frame rate.

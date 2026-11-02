## netstat: the Help Desk reply mark (2026-11-02)

- Traced in the unpacked FFXiMain.dll (Ghidra decompile + disassembly). FUN_102014f0 per frame: +0x1c the unread mail
  (engine +0xa, <= 200), +0x20 engine +0xe, +0x2c = FUN_10115250 & 0xffff, +0x30 = 2 when +0x2c != 0, else 1 when
  engine +0x10 is set, else 0. FUN_10115250 is 1 while a GM notice is complete (DAT_104e1710 msgId set, +0x16 zero,
  DAT_104e1718 zero, DAT_104e1720 the written gmNotice.fdt size non-zero; FUN_10115020 writes it, FUN_101133a0
  clears it all): ours, `w.gm_reply` complete with text.
- FUN_10201020 (the page object at netstat +0x1c): sprites +0x18 / +0x1c / +0x20 for states 0 / 1 / 2 = windowps 2,
  windowps 418, keytops3 118 (library table DAT_10662e50: 1 windowps, 0 keytops3); +0x24 = windowps 36.
- FUN_10201180, page 0: with +0x2c >= 1, windowps 36 at (+4, +4), the state's sprite (keytops3 118) at (+0x52, +4)
  scaled 1.6, and the count at (+0x4e, +0x10) in 0x80804040; the mail counts are not drawn. Else the state's sprite at
  (x, y), +0x1c at (+0x6c, +0x10) and +0x20 at (+0x55, +0x10) in 0x60808080.
- Page hold: the network page is held while +0x2c < 1 and +0x20 == 0 and +0x1c == 0; the network page lasts
  DAT_10385272 ticks, three times that when +0x20 != 0 and +0x1c == 0.
- Built: hudtop.rs draws state 2 and holds the network page only with no mail and no reply waiting; the modern top
  bar shows "Help Desk reply" (icons/help, icons/notification) on the mail page. Not done: engine +0xe (+0x20), its
  count at +0x55 and the 3 x network page (feature 17 of the modern list).

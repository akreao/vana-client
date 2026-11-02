## netstat: the second count (engine +0xe) never counts (2026-11-02)

- Traced in the unpacked FFXiMain.dll (decompile + disassembly) for the modern list's "Help Desk count and the second
  count". netstat +0x20 is FUN_100f6f00: the u16 at +0xe of the PlayOnline message object (DAT_104df908, set to
  &DAT_104df8a8 by FUN_100f76b0). Beside it: +0xa the unread mail (FUN_100f6f30, capped at 200, `inc word [eax + 0xa]`
  per message in LAB_100f2020), +0xc (FUN_100f6ed0 / cleared by FUN_100f6ef0), +0x10 the system messages
  (FUN_100f6f90, incremented in FUN_100f4bf5..100f657c).
- +0xe is written only twice: 0 by the object's init (FUN_100f1ba0, 0x100f1bb0 `mov word [esi + 0xe], bx`) and 0 by
  FUN_100f6f20 (0x100f6f29), which nothing calls directly. No instruction in the PlayOnline code (0x100e0000..
  0x100fffff) adds to it. So in this build the second count is always 0: its number at (+0x55, +0x10) in
  0x60808080 never draws a non-zero value, and the 3 x DAT_10385272 network page (+0x20 set, no mail) never happens.
- The Help Desk count itself is FUN_10115250 (0 / 1, a complete GM notice), drawn as "1" at (+0x4e, +0x10) by the
  reply mark (docs/modlog/2026-11-02-helpdesk-reply-mark.md). Nothing more to build for feature 17; hudtop.rs's
  step_page notes why.

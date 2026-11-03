## Gamepad: B in a window no longer turns the player (2026-11-03)

From FFXiMain.dll (Steven's unpacked copy, all_decomp.c / full_text.asm).

- Key function 0x9c (keyboard type 2's face-camera: LShift, or the pad's cancel slot 3 = B) is read only by the free
  camera movement, FUN_100a6240, as FUN_10123a70(0x3f, 0x9c, 4 held, -1), at 0x100a66fb and 0x100a70d9. The gates are
  FUN_1025e070 (config +0x44 == 2, keyboard type 2), the player's vt+0x330 (not locked on) and, on the second path, no
  ground-cursor prompt held with Ctrl / Alt (FUN_10158aa0, field +0x22). Its context mask is 0xffff, so every context
  set by FUN_101236e0 (0xffff, 0x3e, 0x42, 0x26) passes; held mode skips the consume flag; the pad tester
  FUN_10120f30 is a plain test of +0x220 bit 8, and nothing clears +0x220 for a window (it is written only by the pad
  read at 0x100071ef and the input object's constructor). So retail, in keyboard type 2, turns the player to the
  camera while B is held even with a window up, and on the press that closes one [C].
- Ours (not retail, Steven 2026-11-03): with a window, a target prompt, a macro palette or outside the field, B goes
  to the window as Escape only; held then, it stays out of the pad word until let go (gamepad::Gamepad::menu_cancel),
  so closing a window with B does not snap the facing either. B in the field and the keyboard's Shift keep retail's
  face-camera.

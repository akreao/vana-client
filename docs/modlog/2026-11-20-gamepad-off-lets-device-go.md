<!-- dll-build: 30261102_0 -->
## Gamepad: turning the pad off lets the DirectInput device go; no new pick after an XInput pad (2026-11-20)

- FUN_10005a40 runs every frame, pad on or off. Its first step, DAT_10456df0 (padguid000, or padmode000's [0] on / [1] force feedback / [5] XInput changed, FUN_10005490 /
  FUN_100055a0): Unacquire (+0x20), the effect DAT_10456dd8 unloaded (+0x2c) and released, the device (+0x90)
  released, +0xc9c = -1, +0xc99 = 0, and FUN_100083d0 only when DAT_10456de4 (the pad on). So the pad turned off
  frees the device, background-exclusive or not, and it is picked again when the pad is turned back on.
- With no device, +0xc99 set (an XInput pad passed during the pick, no padguid000) reads the XInput slot +0xc9c and
  never runs FUN_100083d0 again; only with +0xc99 clear does the 0x5b-frame retry (+0xda4) run. A DirectInput device
  picked after an XInput pad and lost (GetDeviceState 0x8007000C once a foreground device is unacquired on focus loss;
  the release at LAB_10006213 leaves +0xc99) leaves the XInput pad at slot 0 playing until an option changes.
- Ours: gamepad.rs's DirectInput settings check (`di_changed`) now also runs when the pad is off, so the device is
  let go there; `gamepad_dinput::again` is FUN_10005a40's pick-again test. Not retail: the Controller window, waiting
  for a button with the pad off, keeps a device open (retail's FFXiPadConfig is its own program); config
  gamepad_background in the change test (retail's [4] reaches DAT_10456df0 only through FUN_10005600, not traced
  further here).

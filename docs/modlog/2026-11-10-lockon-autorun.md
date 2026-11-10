## Autorun while locked on (2026-11-10)

For lenonk/vana-client#172. Addresses are 30261006_0.

- The locked step FUN_100a7b80 (FUN_100a6240 calls it when vt+0x330 is 0) has its own autorun branch, taken when
  autorun is on (DAT_10487f81) and its vector (DAT_10487f64 / +0x6c) is not zero:
  - the move input is turned by the camera's yaw (FUN_10015250 +0x24 / +0x2c), normalised and dotted with the
    normalised vector; under 0.3 (_DAT_1032b15c) autorun ends (FUN_100a6070(0)). The input does nothing else;
  - the vector keeps its length and is turned at the target every tick (FUN_100272e0 with the unit way from the target,
    scaled by -|vector|). The target is FUN_10157cf0's (the lock), else FUN_10081600's (the follow);
  - gap = horizontal distance less both radii (FUN_100aaef0 for each). gap >= |vector| x ticks (FUN_10014cf0, at
    least 1): the step is the vector; 0 <= gap < that: the unit way x |vector| x 0.3 (_DAT_1032dfa4 is -0.3, against
    the from-target way); gap < 0: the step is DAT_10487f98 (zero). Autorun stays on throughout.
  - No 0.8 / 0.1 floor in this branch (that clamp is the input branch's, _DAT_1032a378).
- With no vector yet, the normal locked step runs, and LAB_100a6d1e in FUN_100a6240 makes its result the vector, as
  unlocked. No input and no vector ends autorun (the test before FUN_10027990).
- Since the vector is rewritten every tick, the lock going off leaves autorun running along the last way to the target.
- Ours: the autorun block now runs while locked (client/mod.rs update_movement), steps straight along the vector, keeps
  a Lua mod's run to a point on its own way (ours), and keeps the run at full tier.

## Effect audit fixes 1: bursts, static gates, weather density, actor sizes, 0x2f, 0x33, 0x50..0x52, 0xba (2026-11-15)

From a full pass over retail's generator (FUN_100496f0, FUN_1004e8d0, FUN_1004e720), element update (FUN_1004a980)
and scheduler (FUN_10057fb0) switches against ours:
- Repeated bursts: ours set `stopping` on the template instance after each burst and never cleared it, so while a
  burst's elements lived every later emission-op-0x3c / end-op-1 burst of that generator emitted nothing (20,419 /
  6,758 generators). End op 1 restarts the resource first (FUN_10053370 clears the stop), and a resource running only
  for bursts has no task, so retail never soft-stops it: `burst` now clears the stop before emitting.
- Static generators (flag 0x400 or op 1 life 0) make their one element on the first update whatever the 0x0a / 0x15
  gate says (0x1004a605..0x1004a628); ours waited for the gate (16,965 + 2,164 zone generators popped in late).
- Weather generators (+0xde bit 0) emit ftol(n x density x 0.3) + 1 elements per emission, n = flags & 0x1ff,
  density = FUN_10002d80 (0x1004a750; 0x1032b15c = 0.3); ours n + 1.
- Actor sizes (vtable +0x98 / +0x9c / +0xa0: FUN_100d08e0 / 09b0 / 0a80) are the model scale x skeleton box 0's
  width (b2 - b3) / 1.7, height (b1 - b0) / 1.9, depth (b4 - b5) / 1.9 (0x10331394 = 1 / 1.7, 0x10331398 = 1 / 1.9);
  1.0 for models without a skeleton (FUN_10081ec0). Hume 0.82 / 1.00, Tarutaru 0.47 / 0.55, Galka 1.18 / 1.32.
  Scale A / B (2019 FUN_10052eb0) take the larger of width / height for a uniform source, width for x / z, height for
  y; ours used the bare model scale. Emission op 0x72 (0x10051d23): bias x ((target depth - 1) x f + 1).
- Element op 0x2f (0x1004c48f): S = (|S|, 0, 0) x euler(rot); ours rotated S again each frame (compounding).
- Element op 0x33 (0x1004b604): the clone's matrix is euler(rot) x the element's parent frame rotation, then its
  world position; scale A = the owner's scale A x the element's scale. Ours dropped the parent rotation and scale.
- Emission ops 0x50..0x52 (0x1004e986): S[s] is the keyframe linked from the 4cc @2; word 1 is 0 in every file.
- Scheduler op 0xba (FUN_100d5430, 657 ops: dsh0, tob0, gek0 ...): actor1's drawobj +0x840 bit 23 = u32 @8 != 0;
  FUN_100c83c0 returns at once while it is set, so the status motion (idle / walk / run) is not switched under a
  scheduler's own motion. Ours: Actor::motion_lock, held in actmotion's status switch.

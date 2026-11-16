## Effect audit fixes 2: scheduler ops 0x62, 0x6b var 0x3e, 0x96, 0x5c, 0x9b / 0xa7 / 0xa8, 0xbe, 0xbc, 0xb6, 0xc0 / 0xc3 (2026-11-15)

- 0x62 (210 ops in monster files, `jh02` / `js02`): a timer task (FUN_10060f80, dur x speed ticks, FUN_10060fd0
  counting down by the frame ticks) holds actor1's +0x86c; +0x870 = |heading - drawn yaw| (heading = -atan2(dz, dx)
  to actor2, (1, 0) when dx < 0.1 and dz < 0.1 as signed tests; wrapped to -pi..pi), +0x874 = f32 @8 degrees
  (x 0.017453, 0x1032a9f4), negated for a negative turn. FUN_100c6290 steps it once a frame (0x100c66bc..): yaw +=
  rate, remaining -= |rate|, an overshoot backed off, while +0x86c and the remaining turn last. Ours: drive::FaceDrive,
  a frame being 2 ticks.
- 0x6b variable 0x3e (FUN_10062900, 70 reads): actor1's entity +0x12c bit 22 (0x00E @0x28 bit 0, the trust mark).
  0x3d (2 reads) is not done (it also wants FUN_10098340 / FUN_1008e130).
- 0x96 (29 NPC `init` schedulers): vt+0xfc FUN_100848c0 = slot 8, the ranged weapon (dat/npc.rs).
- 0x5c (every monster's `cnt0`): moving and not in status 2 / 7, the counter swing follows the move quarter +0x598
  (FUN_1005e5e0): 2 `cnr0`, 3 `cnb0`, 4 `cnl0`, else `cnf0`; standing `cni0`. Ours knows the quarter for us only.
- 0xa7 / 0xa8 / 0x9b (vt+0x324 FUN_100a4a70): the drawn override +0x83e + slot (u32 @8, above 1 -> 0), on / off;
  0x9b holds it on for dur x speed (FUN_10061e00; emotes em00 / em05). FUN_10029320 (0x1002a177): with either byte
  set, slot 0 decides the main hand (point 127) and slot 1 the off hand (point 126), whatever +0x83c says.
- 0xbe (79 `init`s, all 0): entity +0x13c bit 18; FUN_10083220 in op 0x05 skips the scheduler's scope for `ded?` /
  `cor?` unless it is set, so death and corpse motions come from the actor's files.
- 0xbc (FUN_10061c70 / FUN_100d53f0): actor1's footstep counter +0x8ab held up for dur x speed, which opens the step
  gate (client/footsteps.rs).
- 0xb6 (FUN_1005e7e0): drawobj +0x840 bit 22 = u32 @8 bit 0; FUN_100c83c0 then shows wep0 / wep1 on a new status
  motion (client/wepfx.rs status_start).
- 0xc0 / 0xc3 (`@ini` of the scene files 83923.. / 83926..): the zone's +0x1c8 = u32 @8 (FUN_1018bc40, wrapped to
  360 days) and +0x1cc = i16 @8. FUN_10182c40: while online (DAT_10443fe4 +9) and +0x1c8 >= 0, the lighting's time
  is +0x1c8, not the clock. FUN_10180700: with +0x1cc != 0 both the moon (+0xc4) and the sun (+0xa4) vectors turn
  about x by that many degrees (FUN_10026d50, FUN_100273d0). Both reset with the zone (fx switch_zone).
- The rest of the scheduler audit: 0xb9 in "Effect audit fixes 5"; 0xb8, 0xa6, 0x7e, 0xaf, 0x3d, 0xbd, 0xa1, 0xc1 in
  "Effect audit fixes 9" and "Event VM audit fixes 2"; fast-forward of 0x03 / 0x73 children and 0x85 in "Effect audit
  fixes 4".

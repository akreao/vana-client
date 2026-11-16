## Effect audit fixes 9: scheduler ops 0x3d, 0x3f, 0xa1, 0xbd, 0xa6, 0x7e, variable 0x3d, the title loop (2026-11-15)

- Op 0x3d (FUN_1006afb0): the picked op runs in the 0x3d's place and its result is the block's. Ours dropped it, so a
  0x3b call-and-wait inside a random block (318 in the game files) did not hold its parent. A wait now suspends the
  task after the 0x3e, and the countdown takes the 0x3e's wait instead of the 0x3d's.
- Op 0x3f: with no A clone on the pair, C is always deleted (FUN_100543a0 at 0x1005a026; a static B runs its one
  emission first, deleted with C). Ours kept a non-static B's clone emitting.
- Op 0xa1 (0x1005b34c): op 0xa0's colour drive (FUN_1005f6c0, landing on its target) on actor 2. One user: file
  61548 `clor`.
- Op 0xbd (vt+0xc8, FUN_100a4b20 on skeleton actors): +0x8a9 = byte @8 (reset 0x80). FUN_1001ee60 at 0x1001f325, for
  the player: when it is not 0x80 the look height is that point's offset from the position (vt+0x1c8), with no 0.6
  factor and no 2.5 cap, ahead of the riding branch. Users: models 2917 / 2957 (files 53212, 53252), point 0x1f.
- Variable 0x3d (FUN_10062900): 1 when actor 1 has an entity whose model (+0xfc) is not 0x92c and that is a pet or
  trust (index above 0x6ff, FUN_1008e130) or a charmed monster (id high byte set, +0x12c bit 13 = 0x00E u8 @0x27 bit
  3). Its reader is file 0's `nrm0` (started by `nerm` from every race's `cate`): 0 plays `nm00`, the casting ground
  circle; 1 plays `nm10`, empty. Ours read 0, so pets, trusts and charmed monsters cast with the circle.
- Op 0xa6 (0x1005ba2b, FUN_10035e30): with a current stream (DAT_1046391c, set by ops 0x60 / 0x8a, forgotten by a
  failed lookup and the zone reset FUN_100361d0), a kind-8 fade task (FUN_10035a00) from 1.0 toward f32 @8 over int
  @0xc ticks (not x speed); a new one lands the running one first. Each tick (FUN_10035940) level = 1 + (to - 1)(1 -
  left / total); FUN_100360f0 sets the stream voice's volume (level x 127 and the config factors) or stops it at <= 0;
  the end applies the target and forgets the stream. One user: event file 30777 `s12k` (to 0 over 30 ticks).
- Op 0x7e (0x10058469): FUN_10072290 type 0x2f by 4cc @8 in scope, then FUN_1018b010 makes it the env. Each frame
  FUN_101874a0 points the env back at a weather object's own data where one exists, so it holds only where the zone
  has no env for the weather *(inferred from FUN_101874a0)*; ours uses it there, and drops it on a 0x7c or zone-in.
  Users: title file 23 `mov1` and event file 30769 `sg01`.
- The title (file 23): `main` spawns `loop` looping; each pass runs one 0x3d block (mov1..mov8), then each 0xaf block
  (ex1a..c, ex2a..e, ex3a..e) whose bit @8 is set in both DAT_10456a28 +0x14 (the account's content) and +0x10 (the
  installed ROMs), one entry at random each (FUN_1006bf60). Ours played one clock-picked mov and never the ex movies;
  it now plays the pass in turn (before login against the installed mask alone, inferred). The movies' op 0x0e
  feedback blur and op 0x7b's per-zone fog (FUN_1018b3a0, the Clipping factor as in the world) apply too.
- Op 0xb8 (vt+0x114, FUN_100849c0): actor +0x88 bit 7 = @8, read only by FUN_100cd920 (0x100cd9ab): set, every motion
  is sought in the slot-9 whole-model resource (+0x864) first. Its 41 users (models 3005..3117, 3645, 3650 `init`) all
  set 1; ours draws those SubKind 5 / 6 models from the whole model alone (MODLOG "0x00E SubKind 5 / 6"), which is the
  set behaviour, so there is nothing to change until the race base exists.

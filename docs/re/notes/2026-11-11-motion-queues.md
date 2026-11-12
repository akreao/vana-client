# The model's motion queues (queue rows 4 and 9; 2026-11-11)

<!-- dll-build: 30261102_0 -->

FFXiMain 30261102_0, Ghidra decompile (docs/re/dll-index/dll.py) with every branch and offset below checked in Rizin
on the unpacked DLL. This replaces the readings of batch 5 items 4 and 9
([2026-11-03-modlog-gaps-batch5.md](2026-11-03-modlog-gaps-batch5.md)) where they differ:

- The mouth queue FUN_100ce160 fades is region 4 (motion object +0x74 = +0x24 + 4 x 0x14), and "fade" means the
  entries run on for 8 ticks at the weight they have and are then dropped (state 2 never changes the weight).
- The phase carry takes the old entry's position in frames (vtable +0x30 returns +0x24) against a length in
  frames / speed, so the fraction is only a true fraction of the cycle at speed 1.
- Event op 0xd3 (FUN_100ce070) fades regions 3 and 6, not "every motion layer".
- After a counted motion the status motion is queued again in every region it has, from frame 0, blended in over
  the counted motion's out length (FUN_100d4b20); the regions the counted motion did not key are not left alone.

Ours: crates/vana-client/src/motionq.rs (the queues), client/mod.rs pose_actors (what feeds them),
dat/character.rs pose_layers and render/anim.rs (the posers).


## Object
- model +0x674 (+0x44) -> motion object M. Seven region queues at M +0x24 + 0x14 * r (r = 0..6):
  +4 head of a singly linked list (oldest first), +0xc region weight, +0x10 (zeroed each tick, FUN_1001ac40).
- 0x60 = region 3, 0x74 = region 4, 0x9c = region 6.
- Entry (0x40 bytes, vtable 0x1032b1c0, FUN_1001a800): +4 next, +8 state byte, +0xc out timer, +0x10 weight,
  +0x14 blend-in left B, +0x18 weight slope (1 / B), +0x1c share a2, +0x20 out length a6, +0x24 frame, +0x28 rate,
  +0x2c play count (0 loop, n counted, -1 finished), +0x34 resource, +0x38 end frame override a8.

## Enqueue FUN_1001ad00(res, a2, rate, count, B, a6, start a7, end a8) (thiscall on a region)
- appended after last entry L. B == 0 -> 1.0. state = 1, weight = 0, out timer = 0, slope = 1 / B. a6 < 0 -> 1.0.
- With L and both resources: phase carry (lengths in frames / speed):
  oldLen = L.frames / L.speed; newLen = N.frames / N.speed; if oldLen > 0.001: pos = L.frame / oldLen,
  extra = newLen / oldLen * N.speed * 0.5 * rate * B; pos = pos * newLen + extra; newLen <= 0 -> 0 else mod newLen;
  N.frame = N.speed * pos. (oldLen <= 0.001: pos = 0, extra = 0 -> frame 0.)
  Then FUN_1001b520(B) on L.
- Then count entries; if > 3: the (n - 3) oldest get FUN_1001b520(1.0); the entry after them (the oldest kept)
  weight = 1.0. With n <= 3: the HEAD entry's weight = 1.0 (iVar5 is still the head). So the oldest entry of a
  region is always forced to weight 1 on every enqueue.
- FUN_1001acb0 = FUN_1001ad00 then last.frame = a7 (no carry). FUN_1001a660 (model, ..., region, a7, a8) uses acb0;
  FUN_1001a6e0 uses ad00 (carry). Both: region 0 enqueue -> model vt+0x1fc(0) (clears +0x79c).
- FUN_1001b520(x) (start fade): state 0 -> 2, out timer = x. state 1 -> 3, B = out timer = x (slope kept).
  state 2 / 3: only if x <= 1.0: out timer = 0, state 2.
- FUN_1001b170(x): FUN_1001b520(x) on every entry of a region.
- FUN_1001b590 (natural end, regions >= 3): state 0 / 1 -> 3, B = out timer = a6, slope = -(weight / a6) (a6 <= 0: 0).

## Tick FUN_1001a3e0(model): for r = 6..0: FUN_1001b2e0 (fades), +0x10 = 0, FUN_1001b1f0 (frames; skipped when
FUN_10084620() != 0)
- Fade FUN_1001b5d0 per entry (dt in ticks):
  state 1: w += dt * slope; B -= dt; B <= 0 -> state 0. clamp w 0..1. keep.
  state 2: out -= dt; out <= 0 -> remove. (weight untouched)
  state 3: w += dt * slope; B -= dt; out -= dt; out <= 0 -> state 2, remove. else B <= 0 -> state 2. clamp.
  state 0: clamp.
  "remove": if the entry is the last one and r < 3: kept, model vt+0x1fc(1) (and the walk stops); else unlinked.
- Frames FUN_1001a8e0 per entry (model, r): only when count >= 0. end = frames (i16 res +0x34), or +0x38 when > 0.
  If end == 2.0 and speed * 0.5 == 0.5: frame += 1.0; else frame += dt * speed * 0.5 * rate.
  frame >= end: if count > 0 and --count == 0: count = -1, rate = 0, frame = the value before this step;
     r < 3: last entry -> vt+0x1fc(1); r >= 3: FUN_1001b590. Otherwise end <= 0 -> 0, else frame -= end while >= end.
  frame < 0 (reverse): same count rule; end <= 0 -> 1.0, else += end while < 0.
- FUN_1001b360(region): any entry with count < 0.

## Pose FUN_1001a420
- Per bone flag byte (DAT_104601d8): bit 7 usable, bit 6 taken by a higher region, bits 0..5 written in this region.
- Regions 4 down to 0, each non-empty one: FUN_1001b230(r, main buffer), then bones with bits 0..5 != 0 get bit 6.
- FUN_1001b230: entries oldest -> newest; an entry is posed unless its successor's weight == 1.0. Region weight
  +0xc = fold of (share * w + (1 - w) * acc) over ALL entries (r != 0), acc from 0.
- Entry pose FUN_1001ab60 -> FUN_10019a50(region, frame, weight, buffer): weight = +0x10, but 1.0 for the first
  entry of the list in regions 5 / 6. Skipped when weight < 1e-4. Per track (bone usable, not taken):
  written count 0 -> FUN_10019b30 sets the bone (weight not used) and marks it; count 1 -> FUN_10019ee0 blends:
  q = slerp(buffer, new, w), t / s = new * w + (1 - w) * buffer.
  A track whose rot index has bit 31 (regions != 0): b30 leaves the bone unmarked (identity written, r < 5);
  ee0 (bone already written by an older entry of the region, r < 5) blends it toward identity / 0 / 1 by w.
  A track with trans index bit 31 (r != 0) in b30: rot + scale written, trans 0, bone left unmarked.
- Regions 5 then 6, each non-empty: written bits cleared, region posed into a second buffer, then every bone it
  wrote: main = slerp / lerp(main, second, region weight +0xc).

## Callers
- FUN_100cd920(name, a2, rate, count, B, a6, region, carry, start, end, file) -> FUN_100cd540 per file: every chunk
  named name[0..3] + digit; region = the digit unless the region argument >= 5; one chunk per region per call
  (DAT_1048b53c mask); region 2 skipped when +0x881 == 0 and status in {0, 1, 6}; 'jmp*' skips region 1 and then
  queues 'run1' into region 1 at rate * DAT_1032ef40. carry != 0 -> FUN_1001a6e0 else FUN_1001a660.
  Tail: when region 0 was queued, each region 0..2 that held a finished entry (count < 0) before the call and was
  not queued now is emptied (FUN_1001b140).
- Status motion FUN_100c83c0: returns while +0x80 != 0. If +0x79c clear and name == +0x7d8: a changed rate goes
  on the newest entry of all 7 regions (vt+0x2c); else queue: cd920(name, 1.0, rate, 0, B, 1.0, 0, carry, 0, 0, 0)
  with B = 16 and carry = (name != 'jmp') when +0x79c is clear; B = FUN_100d4b20() and carry = 0 when it is set.
  FUN_100d4b20 = max(a6 of region 0's newest entry, a6 of region 1's newest entry) (1.0 with no motion object;
  DAT_1032b3a0 with neither).
- FUN_100ca8c0: cd920(name with '1', 1.0, 1.0, 1, 12.0, 24.0, 0, 0, 0, 0, 0).
- FUN_100cd4e0(res, a2, rate, count, B, a6, region) -> a660 with start 0, end 0: one chunk into a given region.
  FUN_100cd490(res, B) = a660(res, 1.0, 1.0, 1, B, 1.0, region 0, 0, 0).
- Blink FUN_100cdf90: every rand() % 480 ticks: cd4e0(eye3, 1.0, 1.0, 1, 1.0, 1.0, region 3).
- Talk FUN_100ce0e0 (from FUN_10087e80 each time +0x150 runs out, 0x8e ticks apart, +0x14e times):
  the entity's face motion: cd4e0(res, 1.0, 1.0, 1, 8.0, 1.0, region 4); else 'mou6': cd4e0(res, 1.0, 1.0, 1, 16.0,
  8.0, region 6).
- Mouth off FUN_100ce160: FUN_1001b170(8.0) on region 4 only. (evop 0x23 close, unbind / op 0x7c via FUN_10087e40.)
- FUN_100ce070 (evop 0xd3): FUN_1001b170(8.0) on regions 3 and 6.
- FUN_1002aae0: 'btl ' else 'idle' -> a660(res, 1.0, 1.0, 0, B = arg, 1.0, region 0). FUN_1002a9f0 / 1002ab60:
  FUN_1001a760 -> FUN_1001af60 multi-motion entries (0x80 bytes; other entry class) — callers 1002c640 only.

## More call sites
- Scheduler op 0x05 (0x1005cc45..0x1005cd69): cd920(name, a2 = f32 @0x10, rate = 2 x frames / speed / length or 1.0,
  count = byte @0x1e & 0x7f, B = u16 @0x18, a6 = u16 @0x1c, region 0, carry 0, start, end, file). The top nibble of
  byte @0x1f: 1 -> end = (frames + 1) x 0.5 - 1 (first half); 2 -> start = (frames + 1) x 0.5 (second half).
- Op 0x28 (0x1005b12e): cd920(status name, 1.0, rate, 0, B = f32 @8, 1.0, 0, 0, 0, 0, file).
- Reactions FUN_100cdd60 (ops 0x25 / 0x21): cd920(name, amount x level factor, rate, count, B, a6, region 5, 0, 0, 0, 0).
- Guard FUN_100cde70 / parry FUN_100cdf00 (ops 0x5a / 0x5b; call sites push region 5): standing (`gdi?` / `pyi?`,
  +0xf8 clear) the region argument is replaced by 0: the chunks go to their own regions as a counted motion;
  moving (`gdm?` / `pym?`) region 5. a2 = 1.0.
- 0x1005ac91 / 0x10060d25 (knockback): cd920(name, 1.0, 1.0, count, 12.0, 12.0, 0, 0, 0, 0, file).
- Event op 0xd3 (FUN_100b7140): only FUN_100ce070 = FUN_1001b170(8.0) on regions 3 and 6.
- FUN_100cb400 (motion link, event op 0xae 5): per region r, own newest entry and the leader's newest entry of r
  (the leader's region 0 newest when it has none): leader frame <= own frame and leader frame > 0 -> own = leader's.
- FUN_10033220: quaternion blend = lerp toward b or -b (the nearer), no slerp.
- cd540's region 2 gate: +0x881 is body info[9]; FUN_10084400 is the model kind (0 / 1 / 6 = PC looks).

## Seen, not built

- 'jmp*' (the engaged jump): region 1 is skipped and 'run1' queued there at rate x 0.4 (DAT_1032ef40). Ours plays
  the jump's own chunks.
- FUN_1001af60 (0x80-byte entries blending up to four motions, queued by FUN_1002a9f0 / FUN_1002ab60 from
  FUN_1002c640): not reached from the status motion, schedulers or events traced here.
- FUN_10019b30 forces bone 0's scale to 1 and, for a track whose translation index has bit 31 in a region 1..6
  chunk, writes rotation and scale only and leaves the bone open to lower regions. Ours samples the scale and
  treats that translation as a constant, as before.
- FUN_1001a660 / FUN_1001a6e0 refuse an enqueue while FUN_10084620() is set and the model's flag (+0x79c) is clear;
  FUN_1001b1f0 stops the frames under the same global. Not traced (inf: a global motion pause).
- A chunk whose name ends in 0 queued into an overlay region keeps its bit-31 tracks unkeyed in retail (the flag
  is honoured wherever the entry's region is not 0); ours masks the flag for every `xxx0` chunk when it is read.

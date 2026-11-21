# MODLOG gaps, batch 9 (queue items 41..50)

Target and tools as in modlog-gaps-batch1.md (`~/ffxi-pad-re/FFXiMain.unpacked.dll`, sha256 8097dbf8…550c). All
addresses are VAs in this DLL. **(inf)** = inferred. Queue: modlog-gaps-queue.md.

## Item 41: the entity copies (+0x190..+0x1b4) that scheduler ops 0x8c / 0xa4 / 0xa5 also write (gaps L101, MODLOG L11963)

**They make the op's new idle / walk / run names survive every later status-motion recompute, until the entity's
names are reset (entity re-init, or the +0x130 bit 4 path).**

The ops (dispatch table 0x1005dc1c[op − 2]; name = table[u32 @8 of the record]); the entity is drawobj +0x70
(FUN_1005e2e0):

| Op | Code | Drawobj (used now) | Entity fields also written |
|---|---|---|---|
| 0x79 battle | 0x1005c046 | +0x7d4 (FUN_1005e770) | none |
| 0x8c idle | 0x1005c07a, table 0x1032b814 | +0x7c8 (FUN_1005e740) | +0x1a4, +0x1a8, +0x1ac (current idles) and **+0x190** (base idle) |
| 0xa4 run | 0x1005c0e1, table 0x1032b824 | +0x7d0 (FUN_1005e760) | +0x1b4 (current run) and **+0x19c** (base run) |
| 0xa5 walk | 0x1005c138, table 0x1032b834 | +0x7cc (FUN_1005e750) | +0x1b0 (current walk) and **+0x198** (base walk) |

**The entity's name fields:**
- Base: +0x190 idle, +0x194 sit idle, +0x198 walk, +0x19c run, +0x1a0 chair idle. Reset by **FUN_1008a160** to 'idl ',
  'si1 ', 'wlk ', 'run ', 'ci1 ' — called from the entity init FUN_1008a350 (0x1008a67c) and from FUN_1008f740 when
  +0x130 bit 4 is set (0x1008fd34, followed by FUN_1008a1a0 and FUN_1008a2f0).
- Current: +0x1a4 / +0x1a8 / +0x1ac idles, +0x1b0 walk, +0x1b4 run. **FUN_1008a1a0** recomputes them from the base set
  (9 call sites: FUN_1008b460 ×4, FUN_1008c6c0, FUN_1008ee30, FUN_1008f740 ×2, FUN_1008a350 — the status / mount /
  appearance updates):
  - mounted (FUN_10087dc0() == 2): idles 'dft ' / 'dft ' / 'df1 ', walk 'ark ', run 'hsr ' (fixed; the base set is
    ignored);
  - otherwise the idle = +0x194 for status 0x2f (sitting), +0x1a0 / +0x194 for chairs (FUN_10095790), else +0x190
    ('chi ' when a chocobo-riding pose applies, 0x1008a1c6); walk = +0x198, run = +0x19c; for SubKind 2 / 7 with
    model 0x3c0..0x3c9 / 0x979..0x97d / 0x9bf..0x9c3, +0x1ac = +0x190.
- FUN_1008f740 (0x1008fd39.., after FUN_1008a1a0) copies the current set to the drawobj: +0x7c8 = +0x1a4, +0x7cc =
  +0x1b0, +0x7d0 = +0x1b4, and +0x7d8 = "    ".

**So:** after op 0x8c / 0xa4 / 0xa5, the next status change (sit, stand, chair, mount off, appearance update) rebuilds
the drawobj names from the entity — and because the op also wrote the base fields, the new name comes back instead of
'idl' / 'wlk' / 'run'. Exceptions: mounted (fixed set), sitting / chairs (+0x194 / +0x1a0 are never written by the
ops), and op 0x79's battle name (drawobj only, so it is lost on the next model re-init *(inf)*). The change lasts until
FUN_1008a160 resets the base set.

Build: keep per-entity base and current idle / walk / run names; ops 0x8c / 0xa4 / 0xa5 set both (0x79 only the
model's battle name); recompute the current set from the base on each status / mount / appearance update as above,
and reset the base set to idl / si1 / wlk / run / ci1 on entity re-init.

## Item 42: the near-camera fade after the cylinder — vt+0x260, first person, fishing (gaps L104, MODLOG L12129)

The tail of **FUN_100cd040** (this build, same address; decompile lines after the cylinder, asm ~0x100cd1b0..0x100cd3a5),
applied to the cylinder result f, in this order:
1. **vt+0x260 → FUN_100a47b0** (drawobj +0x8ac bit 0): set → f = 0. Its setter vt+0x258 = 0x100a4790 (`|= 1`) and
   clearer vt+0x25c = 0x100a47a0 have **no callers** (no `call [reg+0x258/0x25c]`, no direct call, no pointer use
   outside the 7 vtables), and the actor init (0x100c6252) clears +0x8ac bits 0..2. **So this test never fires in
   this build** *(inf: no computed-slot call found)*.
2. **First person** (DAT_10487f80 != 0) and the actor's entity is a character (+0x1d0 & 0xffffff1f): f = 0 when the
   entity is **the one the player's entity +0x2a2 names** (FUN_10097a40: actor table [+0x2a2], 0 → none). +0x2a2 is
   written only by s2c **0x037 u16 @0x46** (0x1009d846..0x1009d84f) and cleared at entity init (0x1008a65c). It is the
   **forced-follow target**: the player's move step FUN_100a6240 starts with `if (entity && FUN_10097a40())` and walks
   the player toward that actor (MODLOG's "forced-follow path"; item 43). So in first person the actor we are being led
   by is hidden. Then falls into the next step.
3. **Fishing** (FUN_100956c0(0xff): the player's status is fishing) for a character entity other than the player's
   own (FUN_101cc4f0), when the player has a drawobj that isn't this actor: d = horizontal distance to the player;
   d < 1.0 → f = 0; 1.0 ≤ d < 2.5 (DAT_10329ea0) → f ×= (d − 1) × 2/3 (DAT_1032b6d4). (Same in first person, after
   step 2.)
4. **Entity +0x130 bit 18** → f = 0.5 (overrides the above). Written by s2c **0x00E u32 @0x28 bit 31** (0x1009c85a..
   0x1009c880), cleared on the own entity by s2c 0x037 (0x1009d84a `& 0xfffbffff`).
5. drawobj +0xb2 bit 5 cleared, then set when FUN_10087890 (Conflict / team mode) and the entity's team nibble
   (+0x13f & 0xf) is non-zero and differs from the player's: the enemy-team outline flag (no alpha change).
6. +0x13c bit 23 (the party highlight list, FUN_10087ad0's 32 ids): FUN_10085ab0 / FUN_10085ad0 when listed, else
   FUN_10085ac0(0) / FUN_10085b10(0) — highlight state, no alpha change.
7. f = min(f, 1.0) → +0x664.

Build: after the cylinder fade: (skip vt+0x260, it is never set); in first person hide our forced-follow target
(0x037 @0x46); while we fish, fade other characters by horizontal distance (0 under 1.0, linear to full at 2.5);
an 0x00E @0x28 bit 31 character draws at 0.5; cap at 1.0.

## Item 43: the character bump during events; does any actor reach +0x59c ≥ 1? (gaps L105, MODLOG L8858 / L8889)

**1. +0x59c is the actor's spawn fade, and every actor reaches exactly 1.0 about 32 ticks after its files load.**
- Init 0 (actor init 0x100c6110). In the actor frame FUN_100c63d0 (0x100c76aa..0x100c76f0): while the model object
  +0x768 reports ready (vt+0x1e4), **+0x59c += frame ticks × 1/32 (DAT_1032b088)**, clamped at **1.0** (DAT_1032961c).
  When vt+0x36c is non-zero (the actor is going away) it instead falls by ticks × 1/16 (DAT_1032a9f0) to 0
  (0x100c7f09..0x100c7f37).
- Other writers: the entity state machine at 0x1008ba0b (entity +0x1cc 0x96 → 0x97: sets it straight to ebp) and an
  effect op at 0x100477e2 (byte / 128, DAT_1032a778 = 1/128).
- The draw multiplies by it (0x100cbf27), and the bump test (0x100a8898) requires `+0x59c >= 1.0`: so an actor is
  solid only once fully faded in (~0.5 s after load), not while fading in or out.

**2. The bump has no event test of its own; it runs whenever the player's move step runs.**
- FUN_100a8770 (bump) is called from the move step FUN_100a6240 (0x100a6f1d and 0x100a7760, after the delta is built).
  It reads no event state; its gates are GM level (FUN_10084600), the per-actor eligibility (MODLOG L8880), and the
  +0x59c test above.
- The move step returns before any movement (and so before the bump) when **DAT_104568fc +8 != 0, the actor exists
  and FUN_100964f0() == 0** (0x100a6240 decompile lines ~135..139: +0x598 = +0x594 = 0, return). FUN_100964f0 ("can
  move") is false with no player, while FUN_1008b3a0 says no, entity +0x12c bit 5, DAT_10482f18 (the talk watch),
  DAT_10487e49 / e48, statuses 0 / 2 / 7 / 10 / 0x1c in the icon list (FUN_10096270), DAT_10482f43 == 0 (zone-in not
  done), status 0xe or 0x279, fishing, status 0x2c (+0x170 or +0x16c), +0x128 bit 15, or two window tests
  (FUN_1012f1a0, FUN_1014cff0).
- The forced-follow path (entity +0x2a2, at the top of FUN_100a6240) moves the player without the bump.
- So during an event the bump applies exactly when the event leaves the player able to move through the normal move
  step (the input itself is gated elsewhere by the event lock); a script-driven walk (event ops) does not go through
  FUN_100a6240 and has no bump *(inf: event walks move the actor directly, batch 2 §1)*.

Build: give each actor a spawn fade (+1/32 per tick to 1.0 once loaded, −1/16 per tick when leaving) and make it
bump-solid only at 1.0; run the bump inside the player's own move step whenever that step runs (no blanket "not in an
event" rule), and never for scripted moves or the forced follow.

## Item 44: model-file generators — the un-hide restart (gaps L95, MODLOG L7600)

**Already traced and built since: MODLOG L13227 ("Show (FUN_10085a10 [FUN_10084320]) …", 2026-10-25) supersedes
L7600's "not done". Confirmed in this build; the gaps row is stale.**
- **Show FUN_10085a10** (this build's name for MODLOG's FUN_10084320): only when model +0xb2 bit 1 (hidden) was set:
  clear it, FUN_100856f0 (restart the static clones that have no element, clear +0x17a), FUN_100858f0 (resume the
  actor's scheduler tasks), then **vt+0x308**. Otherwise it just clears the bit.
- **vt+0x308 = FUN_100d62c0** in all 7 drawobj vtables: vt+0x1e0 gets the model file; if loaded (FUN_10071060) →
  **FUN_10085060(file)** (MODLOG's FUN_10083970: kill this generator's clones on the actor, clone each 0x1000 generator
  onto (actor, none) and start it, ≤ 128, gated by vt+0x124 = entity +0x130 bit 1 and +0xb2 bit 1 clear); then for
  each mesh entry of the skeleton list +0x674 (FUN_1002b5a0 / FUN_10029e70) the same on that mesh's file.
- Remaining per MODLOG L13238: the skeleton's `fvib` clone made but not started at the show, and the +0x764 / +0x814
  clones — not part of this row.

Build: nothing new (the restart exists as model_generators at the show).

## Item 45: actor ground shadows — +0x66c's source; the ZBIAS overlay order on ATI (gaps L142, MODLOG L18408 / L18428)

**1. +0x66c (the shadow gate / strength) is latched once, when the entity's model is made:**
- Accessors: drawobj vt+0x3c = 0x100a4880 (`+0x66c = arg`), vt+0x40 = 0x100a4890 (get), the same in all 7 vtables.
  Ctor default (0x100c611c) = ebp *(the 0x80 MODLOG reads)*; neighbours: +0x660 model colour (vt+0x30 set / vt+0x2c
  get), +0x663 its alpha (vt+0x44), +0x668.
- The only drawobj caller is **FUN_1008f740** (the entity update, at the model-made step right after the spawn
  scheduler 'pop0' is started, 0x10090a4b..0x10090abe):
  ```
  if (entity +0x12c & 0xc00)  +0x66c = 0            // bit 10 (0x00D / 0x00E @0x27 bit 1) or bit 11 (event op 0xa5): no shadow
  else if (model colour (vt+0x2c) & 0xff000000)  +0x66c = 0x80
  else +0x66c = 0
  ```
  (+0x12c bit 12 → vt+0x3dc(1) just before: the "no blob" flag, MODLOG's op 0xc0.)
- So MODLOG's inference holds, with the no-shadow flags folded in: an actor whose model colour has alpha 0 at that
  moment (a spawn scheduler that starts from alpha 0 and has already applied its first colour op) gets no shadow for
  its lifetime *(inf: whether 'pop0' / 'spop' has written the colour before this check depends on vt+0x298 running
  the scheduler's first ops synchronously — not traced)*. Nothing re-latches it later.

**2. ZBIAS on ATI: the same order, smaller steps.** The values are globals set by the scene init:
- FUN_10177d00 (0x10177eeb..) and its twin 0x101781d0: DAT_10632038 = 0, **DAT_10631f18 = 8**, **DAT_1065c4d0 = 15**,
  **DAT_10631c88 = 15**. Then, when the device flag +0x920 is set, **DAT_10631f18 = 1, DAT_1065c4d0 = 2, DAT_10631c88 =
  2** (0x10178213..0x1017822d).
- Device +0x920 = **the adapter's VendorId == 0x1002 (ATI)** (D3D8 GetAdapterIdentifier, 0x10009466..0x10009484).
- Users (all D3DRS_ZBIAS 0x2f):

  | Value | Normal / ATI | Used by |
  |---|---|---|
  | DAT_10632038 | 0 / 0 | scenery unblended meshes (0x1017c538, 0x1017db54, 0x1017ec19), furniture (FUN_101844c0 0x101848bc), 0x1017b639 |
  | DAT_10631f18 | 8 / 1 | scenery blended (bit-31) meshes (0x1017db22, 0x1017ebf0) |
  | DAT_10631c88 | 15 / 2 | the actor shadow draws (FUN_10184c90, FUN_10184fe0, FUN_10185ca0 *(inf: this build's quad / blob / ground pass, matched by structure and the blob's 0x60 colour term)*) |
  | DAT_1065c4d0 | 15 / 2 | effects (FUN_100467e0 0x10046f6a), FUN_10183420 / FUN_10183760 |
- So on both vendors the stacking is opaque scenery 0 < blended scenery < shadows = effects; ATI only compresses 8 / 15
  to 1 / 2 (its driver scales ZBIAS larger) *(inf: the reason)*. Shadows and effects tie.

Build: latch the shadow gate at model creation (0 for the no-shadow flags or a zero-alpha model colour, else on);
keep one depth-bias ladder 0 / 8 / 15 (no ATI special case is needed with a bias in NDC units; keep the ordering).

## Item 46: engaged state — FUN_1013eff0, DAT_106218ee's source, FUN_1015e6e0's exact effect (gaps L295, MODLOG L6821)

**1. FUN_1013eff0 = "the treasure pool holds items"** (MODLOG's inference confirmed): `return FUN_100ebe60() > 0`;
FUN_100ebe60 counts the 10 pool slots S+0x1951c.. whose state byte == 2 into S+0x19890 and returns it. Callers:
playermo's row filter (0x1012b0d3), 0x1014111b, 0x101416e6, the party window's chest (FUN_102041d0 at 0x10204382).

**2. DAT_106218ee = window manager (0x10621838) +0xb6 = the config 0x9e layout option, forced off on narrow screens.**
- Written only by FUN_101606e0 (manager +0xb6 = arg); its sibling FUN_101606d0 writes +0xb5. Manager init FUN_1015d900
  sets both to 0xff (−1).
- The config page FUN_1019b5e0(id, value): id **0x9d** → page +0x38, id **0x9e** → page +0x3c, each followed by
  **FUN_1019b6d0**: +0xb5 = config 0x9d's value, or −1 when the screen width (DAT_106218b8) < 1024; +0xb6 = config
  0x9e's value, or −1 when the width is < 1024 (with +0xb5 == 0) or < 640 (with +0xb5 == 1 or −1). +0xb5 < 0 also
  writes config 0x9d = 1. The page's rows 0xc / 0xe / 0xf are enabled or disabled to match.
- +0xb5 == 0 selects the long window names (FUN_10160760: "mcr?pall" → "mcr?long" …). +0xb6 is read by the macro
  palettes (0x101a38a0, 0x101a3f20, 0x101a4871, 0x101a4f4b) and the config page (0x1019b74f). So MODLOG's "the palette
  shift also needs DAT_106218ee == 0" = config 0x9e set to 0 on a wide-enough screen *(inf: the option's label — a
  wide-screen menu / palette layout choice — not read from the config text)*.

**3. FUN_1015e6e0(manager, window, state, a4, a5, a6) = the generic hide (not a dim).** For a window not already hidden
(type ≠ 0xe):
- FUN_10118830: the cursor child saved into the layout (cursor memory);
- FUN_10123ea0(0xf): this frame's key 0xf (Escape / No) is consumed;
- window +0x73 = a4, +0x72 = a5; **FUN_101187f0: window type +0x10 = 0xe** (hidden / closing: no keys — FUN_10118260
  returns at once — no focus, skipped by the focus pick FUN_1015e7e0), +0x66 = 0, and **the close zoom starts**
  (+0x64 = DAT_1036c190 when the window animates, +0x68);
- if registry A bit 0 is clear and +0x72 == 0: the target system's +0x6a counter −= window +0x6f, +0x6f = 0;
- manager +0x58 (previous focus) cleared if it was this window; +0x75 = a6 (reopen-later flag);
- **FUN_1011a3b0**: the window's close callback +0x7c(cursor, +0x80) if any.
- An already-hidden window only gets +0x75 = a6.
- **Under a sub-target prompt:** playermo's update FUN_1012b4b0 calls `FUN_1015e6e0(playermo, 1, 1, 0, 0)` every frame
  while the target system +0x68 == 1 → playermo is **hidden** (zoomed out, no keys, not drawn as it finishes) for the
  prompt's duration. When +0x68 leaves 1 (DAT_10578178 holds last frame's value) → **FUN_1015e530(playermo)**:
  re-shown and raised with the focus. Its cursor comes back from the layout (saved by the hide), so the reopened
  menu keeps its cursor *(answers MODLOG's last open point; registry A of playermo has no 0x10000 → cursor restored)*.

Build: the pool test = any pool slot in state 2; read config 0x9e (forced −1 below 1024 / 640 px) for the palette
layout; hide playermo (close zoom, keys off, cursor saved) while a sub-target prompt is up and re-show it with focus
and the same cursor when the prompt ends.

## Item 47: skillchains — the 0x04 MagicBurst info bit (Swipe / Lunge); 67507's purpose (gaps L290, MODLOG L6974 / L6989)

**1. The client never formats anything from the result's info bits; they are only scheduler variables.**
- The battle message printer **FUN_1009e090** (the `mprc` / `wapr` / op 0x71 path) loads the work registers from the
  action header (+0x10 → DAT_10487020) and the record's value (+0x14), proc value (+0x24), react value (+0x2e) and bit
  (+0x1c), and picks the message id from +0x18 / +0x28 / +0x30 by kind (0 / 1 / 2). It never reads the record's info
  (+0x10). The text, including any "Magic Burst!" wording, comes entirely from the message id (FUN_100964a0(5, id)).
- Info is exposed to schedulers through the system-value reader **FUN_10062900**: values 0x28..0x36 read the current
  record's result (FUN_1018cec0 = record + 8): 0x28 miss, 0x29 kind, 0x2a animation, **0x2b info** (u16 record +0x10),
  0x2c scale, 0x2d value, 0x2e message, 0x2f proc kind, 0x30 proc info, 0x31 proc value, 0x32 proc message, 0x33 react
  kind, 0x34 react info, 0x35 react value, 0x36 react message. That is how `crtl` tests info & 2 (MODLOG L3527).
- So info 0x04 can only change what a scheduler in the ability's files does (an effect branch), not the log line
  *(inf: whether Swipe / Lunge's files test value 0x2b & 4 was not scanned)*. For "Magic Burst!" in the log the server
  must send a burst message id; LSB's Swipe / Lunge send the normal message with info 4, which retail would print
  without a prefix.

**2. 67507 is not reachable from the skillchain code.** The burst table **DAT_1035bc3c** (u16 × 0x12, file = 0xaf0 +
entry) holds 0, 479, 480, 475, 476, 477, 478, 467, 469, 474, 471, 472, 473, 470, 468, 1013, 1014, 0 → files 3279, 3280,
3275, 3276, 3277, 3278, 3267, 3269, 3274, 3271, 3272, 3273, 3270, 3268, 3813, 3814 (MODLOG's list). No code constant
0x107b3 exists, and the event ops' file ranges (item 39: op id + 0x7d68 / 0xbfef / 0xdc19 / 0xe95b / 0x10323 by range)
cannot produce 67507 *(inf: it is reached, if at all, through another id table — e.g. a monster ability's file list —
not traced)*. Nothing to build.

Build: print battle lines purely from the message id; expose the result's info word to schedulers as value 0x2b.

## Items 48 + 49: the creator's name filter FUN_101ba240 (3322, random names); the mouse on chmkserv / chmkpass (gaps L55 / L260, MODLOG L5377 / L16013)

**FUN_101ba240(name) is a forbidden-word dictionary match done by PlayOnline: 0 = allowed, 1 = matched (refused),
−1 = no dictionary loaded.**
- Object DAT_10662db8 (FUN_101ba280, 0x20 bytes, vtable 0x10337b2c; made at start-up by FUN_101a8b50).
- Load **FUN_101ba220 → FUN_101ba480**: reads **"entryz.dic"** from PlayOnline file category 0x13 (path = POL's
  directory for the category, DAT_10456a2c +0x1f8, + the name; FUN_101c4b80 size / FUN_101c47d0 read) into a buffer
  ("VulgarName"), then **POL +0x1154(buffer, size)** builds a matcher → +0x14. Unload FUN_101ba260 → FUN_101ba560.
  (The chat profanity filter is the sibling "vulgar2.dic" at +0xc, FUN_101ba390, POL +0x5a8.)
- Test **FUN_101ba520**: no matcher or no name → −1; else **POL +0x1158(matcher, name, strlen) == 1** → 1, else 0.
  The rules live in the dictionary and polcore, not in FFXiMain *(inf: a substring / word list match)*.
- Loaded by the lobby's creator FUN_102435f0 (0x102436d3), unloaded on the creator's exits (0x1023aaac, 0x1023ab2c,
  0x1023ab9c, 0x10243cf8, 0x10243dfd, 0x10243e49). FUN_10094160 does load–test–unload for event op 0x71
  (FUN_100b7e10 at 0x100b7f2b: a name typed in an event *(inf: e.g. naming a chocobo)*) — and treats a failed load as
  **allowed** there.

**Creator world step** (FUN_10239720, 0x1023a2f5..0x1023a30f): name length < 3 → error 0x6e (FFXI-3110); else
**FUN_101ba240(name) != 0 → error 0x142 = FFXI-3322** — so a refused word *or a missing dictionary* gives 3322 and the
name step. Only a 0 continues to the (name, world) comparisons and op 4.

**Random names** (FUN_10238330 at 0x10238420, after the adjective + noun build): the name goes into the text input
only when `strlen < 0x10 && FUN_101ba240(name) == 0`; otherwise nothing is shown (the done flags reset; the next Num+
tries again). Also FUN_10238480 (0x102385f1) tests a name the same way.

**The mouse on the world step** (chmkserv / worldsel / chmkpass) is the window manager's generic mouse, the same for
every focused window (FUN_1015fa00's button tests, all only while the pointer is in window mode, mouse +0x4d == 2):
- left held (0x8b mode 4) → FUN_1015dd90: over a child of the focused window — a different child: move the cursor
  there (FUN_10118db0) and send msg 9 (hover: SE 0001 if the window's +0x85); the same child held more than 30 ticks
  (+0xb0) → msg 0x1d (repeat Enter);
- left release (0x8b mode 5) → FUN_1015de50: on the cursor child → **msg 5 (Enter)**; on no child → SE 0004 (unless
  mouse +0x87);
- **right release (0x8c mode 5) → FUN_1015def0 → msg 6 (Escape)** to the focused window: back (worldsel → name,
  chmkpass → the list); right press does nothing (FUN_1015edb0 is only a test).
- So a click is Enter on the row under the pointer (MODLOG's inference confirmed) and a right click is Escape.
  chmkserv itself has no handler, so it takes no input; worldsel / chmkpass get these messages through their windows.

Build: ship / emulate an allow-all name filter only if the dictionary is truly absent — retail refuses (3322) without
it in the creator but allows in event naming; drop a random name the filter refuses; generic mouse: drag-select,
release = Enter, right release = Escape.

## Item 50: equipment screen — the list's cursor after a grid Enter; the opening set (gaps L201, MODLOG L10452 / L10536)

**Both readings confirmed.**

**The opening** (FUN_10133130, the main menu's Equipment):
1. Refusals first: FUN_101cbe80 → 7 0x4e, FUN_101cbe70 → 7 0x4d, !FUN_100fd6e0(0) → 7 0xa8 (FUN_10129e30), nothing opens.
2. If the equip window isn't open yet (DAT_105781a8 +0x1a == 0): view mask 0xffff on all nine containers
   (FUN_1021c7e0(0xffff, 2, 0x12, 0) ×9), persona mode 2 (DAT_105783e4 +0x1c = 2), then in order:
   `FUN_1015e350("menu    persona", 1, 0)`, `("menu    money", 1, 0)`, the list source mode 0 (vt+0x58(0)) and
   **FUN_101f23a0** (the list: placed by the source at (314, 48), opened with FUN_1015e350(…, 1, 0), then menuwind and
   playermo hidden with FUN_1015e6e0(…, 1, 1, 0, 0) — item 46), and **last `FUN_1015e350("menu    equip", 1, 0)`**.
   Every open asks for the focus, so the last one, the grid, keeps it: the list is up but unfocused.
3. Then FUN_10097fd0(0, 1): c2s 0x061 {1} through the 1 s gate (item 28's sender, no callback).

**Grid Enter in mode 2** (equip vt 0x10333728 +0x18 = FUN_101322b0, case 5, the window's mode +0x1c == 2):
- slot s = FUN_10131680(cursor); s < 0 → nothing; slot encumbered (FUN_100fd350 bit) → SE 0004, nothing else;
- the list's container = the one holding the slot's worn piece (FUN_10132270 → the container's list DAT_10666e2c /
  e4c / e50 / … into list +0x60), source vt+0x58(container); view mask 1 << s on the nine containers;
- **list +0x1e (top row) = 0, FUN_101f66b0** (recompute the visible range +0x1a / +0x1c from the top; it doesn't touch
  the cursor), then **FUN_1015db80** (focus hand-off to the list, next frame; item 31).
- The list window's cursor (+0x4c) is never written on this path, so **the cursor row stays where it was** while the
  rows now start at the top of the filtered list. If that row is past the filtered count, the hand sits on an empty row
  until a key: the list base's Down / Up then pulls it back (FUN_10118db0(count), batch 5 item 10) *(inf: no frame
  clamp found in FUN_101f66b0)*.
- Mode 1 Enter: FUN_10132b90(picked entry or 0) (the equip attempt; failure → SE 0004), FUN_101316a0(0, 0, 0), unpick,
  focus hand-off.

Build: open persona, money, the list (314, 48) and the grid in that order, the grid focused; on a mode-2 grid Enter
filter the list to the slot, reset its top row only, keep its cursor row, and hand it the focus.

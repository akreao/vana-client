# MODLOG gaps, batch 11 (queue items 61..70)

Target and tools as in modlog-gaps-batch1.md (`~/ffxi-pad-re/FFXiMain.unpacked.dll`, sha256 8097dbf8…550c). All
addresses are VAs in this DLL. **(inf)** = inferred. Queue: modlog-gaps-queue.md.

## Item 61: config 0023 "Simple Character Creation" — the forced-on cases (gaps L56, MODLOG L18597)

**Both forced-on cases confirmed in this build: 0023 (DAT_10456aa4) is set to 1 in the top-level state machine's
case 4 (FUN_10011a00, 0x10011f59..0x10011f6c) when either**
1. **DAT_10456950 != 0 — "the dancer has already run in this process".** DAT_10456950 = 1 is set on the two
   return-to-lobby paths of the state machine (0x10011dfe and 0x10011e6b) only when **DAT_10669420 == 1**, and
   DAT_10669420 = 1 only in **FUN_10251610** (called each frame from the state machine at 0x10011caf) when the
   dancer object **DAT_106693fc** (the creator's 3D character renderer; its init prints "Error dancerInit failed")
   has **+4 bit 2** (initialised / running). So: once the 3D creator has been shown, every later creator in the same
   run uses the simple (picture) creator — MODLOG's inference confirmed *(inf: why — likely freeing the dancer's
   resources is not supported)*;
2. **[DAT_103d3cb8] +0x58 != 0 — the fallback device.** FUN_100157e0 creates the D3D device with the configured
   settings (FUN_10002f20); if that fails it retries with the fallback values (DAT_10456c08 / DAT_10456a58, after
   FUN_10003040 / FUN_100164c0(0)) and, when the retry works, sets **+0x58 = 1** (0x100159de..0x100159e4).

Leaving the creator (the tint / music) was not re-traced here; MODLOG L18603's reading stands *(inf)*.

Build: force the simple creator after the first 3D creator of the session, and when the graphics device had to fall
back to its safe settings.

## Item 62: motion-file model kinds for ops 0x5b / 0x66 (gaps L71, MODLOG L1660 / L1664)

**Confirmed in this build; the gate is the actor's entity kind +0xee (FUN_10084400), checked inside each loader.**
(MODLOG's FUN_100d00a0 / FUN_100d01b0 are old-build names for these.)

| Op | Loader (this build) | Bank | Kinds +0xee that load | Others |
|---|---|---|---|---|
| 0x5b (and 0x5f sub 3 / 5) | **FUN_100d2110** | 0: file = val + 0x7d68 / 0xbfef / 0xdc19 / 0xe95b / 0x10323 by range (item 39) | **1, 2, 7, 8** (`switch` at the top) | nothing requested, `*out = 0` |
| 0x66 (and 0x5f sub 4 / 6) | **FUN_100d2220** | 1: per-race body file A + skirt file B (MODLOG's 0x46-blocks) | **0, 1, 6** (`kind >= 0 && (kind < 2 \|\| kind == 6)`) | nothing; val > 0x117 → vt+0xcc name + an error print (DAT_1035f9c0) |

+0xee values (batch 5-build §2, from 0x00E @0x30 & 7 / 0x00D): **0** a PC (0x00D) or a SubKind-1 PC-look NPC without
+0x12c bit 30, **1** a SubKind-1 PC-look NPC with bit 30, **2** SubKind 0 (a model-id NPC / monster), 3 door, 4 lift,
5 ship, **6 / 7 / 8** SubKinds 5 / 6 / 7. So 0x66's per-race files go to PC-look actors (0, 1, 6) and 0x5b's
skeleton banks to monster-look actors (1, 2, 7, 8); kind 1 accepts both.

For a refused kind the op still runs: with nothing requested the readiness test (FUN_100d12d0) passes at once and the
scheduler 4cc is started from whatever files the model already has (item 39's step 4) *(inf: usually "not found",
so nothing plays)*.

Build: gate 0x5b on kinds 1 / 2 / 7 / 8 and 0x66 on 0 / 1 / 6; for other kinds skip the load and still advance.

## Item 63: `7f 80 n` — the whole-text conversions n = 2 / 3 (gaps L76, MODLOG L14706..L14711)

**`7f 80 n` sets a case conversion for the next name the expander appends: 1 = first letter upper, 2 = all upper,
3 = all lower, 4 = alternating upper / lower; 0 = as is. It is used once and reset.**
- The expander FUN_100c1650 case 0x80 (under 0x7f): expander +0xc = n, advance 3 (the same as MODLOG).
- Every name append then goes through **FUN_100c2bf0(dst, src, +0xc)** and clears +0xc (e.g. 0x100c107f..0x100c10a2,
  0x100c2626..0x100c26b0: spell / ability / key item / grammar phrases), so the mode applies to exactly one name.
- FUN_100c2bf0, per character of src (`1e xx` colour codes are copied untouched and don't count):
  - n = 1: the first character upper-cased (FUN_1025bfd0), then plain copy (n reset to 0 inside the loop);
  - **n = 2: every character upper-cased** (FUN_1025bfd0);
  - **n = 3: every character lower-cased** (FUN_1025bf70 → FUN_1011dcc0);
  - n = 4: alternating, starting upper (FUN_1025bfd0 / FUN_1025bf70 by an odd / even counter);
  - other: plain copy by character length (FUN_1011da80).
- **The conversions depend on the client language** (DAT_10456a28 +8; with +0x18's sign set, 2 and 3 take the
  generic path):
  - language 0 (Japanese): no change (plain copy);
  - 1 / 2: FUN_1011dc00 — a..z → A..Z and the accented letters 0x85bf..0x85d5 / 0x85d7..0x85dd (−0x20) to their
    capitals, plus a few pairs (0x855c → 0x854c, 0x855a → …);
  - **3: upper-casing an accented vowel drops the accent** (0x85bf / 0x85c1 → 'A', 0x85c7..0x85ca → 'E', 0x85cd /
    0x85ce → 'I', 0x85d3 → 'O', 0x85d8 / 0x85da / 0x85db → 'U'), other letters as FUN_1011dc00 *(inf: the French
    client's capital style)*;
  - lower-casing (n = 3 / 4) applies FUN_1011dcc0 for languages 1..3, else copies.

Build: implement n = 2 (all caps) and 3 (all lower) — and 4 (alternating) — over the next appended name only; for
English, ASCII a..z / A..Z and the extended accented letters shift by 0x20.

## Item 64: event op 0x71 subs 0x50..0x55 — the "full-screen picture" is the credits roll (gaps L78, MODLOG L5014)

**Subs 0x50..0x55 drive the staff-roll scroller ("Credits Buffer"), not a still picture.** All go through the window
manager's object DAT_10621944 (nothing happens when it is null). From op 0x71 = FUN_100b7e10 (cases 0x50..0x55):

| Sub | Bytes | Call | Effect |
|---|---|---|---|
| 0x50 n | 4 | FUN_10163130(n) | create the scroller (+0xc, FUN_1019e800, 0x58 bytes) if needed and the resource file **0x18891 (100497)** (+0x10); load the text file **0x18892 + n** (100498 + n) into the "Credits Buffer" (FUN_1019e940: FUN_101c4af0 size, FUN_101c4740 read, FUN_1019ea00 parse), sized to the screen × DAT_10329cf0 / DAT_1033530c; **FUN_1019eac0(−2, player name +0x7c)** substitutes the player's name; set the black backdrop (FUN_101633a0(rect, 0xff000000)) from the text block's measured extent |
| 0x51 | 2 | FUN_101632a0 | stop and free the scroller and the resource |
| 0x52 t | 4 | FUN_101632f0(t) → FUN_1019ec00(t, 1) | the scroll duration: speed +0x48 = t × 60 / (lines (+0x34) × 14 px), at least 1.0 |
| 0x53 | 2 | FUN_10163310 | **wait** (no advance) until the roll has finished (scroller +0x54) or there is none |
| 0x54 x y w h | 10 | FUN_10163320 → FUN_101633e0 | the visible rectangle, each operand scaled to the screen (≤ 0 → the full screen's edge) |
| 0x55 v | 4 | FUN_101633f0 | object +8 = v *(inf: a display flag / tint)* |

So scripts run: 0x50 (load credits text n), 0x54 (frame), 0x52 (duration), 0x53 (wait to the end), 0x51 (close).
FUN_1019ec00 without force picks a pacing tier from the ratio (FUN_1019ef60 3 / 4 with 1 / 2) *(inf: slow / fast
scroll modes)*; with force (as 0x52 calls it) it uses the computed speed directly. Seen in zone 291's events (MODLOG).

Build: a credits roll — text file 100498 + n with the player's name inserted, scrolled over a black rect in t seconds;
0x53 waits for the end.

## Item 65: ops 0x7e / 0xac with status 5 / 0x55 — "FUN_10097a20 spawning the mount" (gaps L80, MODLOG L5277)

**Stale row: the later "Riding" entry (MODLOG L12405.., 2026-10-25) built it, and FUN_10097a20 spawns nothing.**
Confirmed in this build:
- **FUN_10097a20(n)** only stores the ChocoboIndex: entity +0x120 bits 24..26 = n & 7. The animal itself is made by
  the riding system when the actor's event status (+0x174) is 5 or 0x55 (MODLOG L12405..).
- **Op 0x7e** (FUN_100b8b80), actor @2, each status write skipped while entity +0x120 bit 2 (op 0xab 3's lock) is set:
  - 0 / 5: +0x174 = 0;
  - 1: +0x174 = 5, ChocoboIndex 1 (the rental chocobo, file 39275);
  - 3: +0x174 = 5, ChocoboIndex 2, the chocobo look race = val@6 + 0x20, body @8, head @10, feet @12, hands @14,
    legs 0 → FUN_1009ac10(look);
  - 6: the same with legs = val@16;
  - 7: +0x174 = 0x55, mount id +0x1db = val@6 + 1;
  - 8: +0x174 = 0, +0x1db = 0;
  - 2: stay on the op while +0x120 bit 10 (a second actor) is set and that actor (+0xa4) isn't loaded (vt+0x1e4);
  - 4: nothing (advance).
- Op 0xac sub 1 passes 5 / 0x55 through as event statuses (MODLOG L12450: "no longer skips 5 / 0x55").

Build: nothing new.

## Item 66: event op 0x34 area swaps — what happens to the zone's files (gaps L81, MODLOG L13549)

**An op 0x34 swap is a full scene reload in the zone's own slot: every actor model is freed, the current scene is
torn down, and the area's map DAT is loaded and parsed like a zone-in — so its auto schedulers do start again. Ours
matches.**
- Op 0x34 → stub 0x100bc42f → **FUN_100b5a90**, a 3-frame sequence on the counter DAT_10489910:
  1. frame 1: **FUN_10092900 on every actor** (0x900 slots: models freed), counter = 1, DAT_104898f9 = 1, yield;
  2. frame 2: counter = 300, yield (one frame for the frees to settle);
  3. frame 3: counter = 0; **FUN_100b5c30** (FUN_10014cb0, **FUN_1018b130** the scene unload — the same call the
     state machine makes before entering a zone, 0x10011c7c — FUN_1001e470, FUN_10081a50), then **FUN_1018b3a0(area =
     operand 1)**; advance 3.
  (DAT_10489918 selects FUN_1018b620 instead, but every writer stores 0 (0x100aedbd at the event start, and the op
  itself), so that branch is dead in this build *(inf)*.)
- **FUN_1018b3a0(area)**: per-area fog / clip constants (areas 0x2a, 0xb9..0xbc, 5, 0xc1, 0x32, 0x2bd..0x2bf special),
  scene +0x1b4 = area, FUN_1018bae0(0), then **FUN_1018b500(area)**: FUN_1018b620 (the old environment cleared: lights,
  fog, sky +0x330 = 1.0, colour 0x80808080), **FUN_10178df0(area) → the file id**, **FUN_1018b6a0(file)** — the zone
  DAT load and parse, the same as a zone-in (the auto schedulers' op 0x52 spawners are created by that parse) — then
  FUN_10015250, FUN_1001e2b0, FUN_1017a3d0.
- So the previous area's generators and auto schedulers die with the scene unload, and the new area's start fresh;
  actors reload their models afterwards (they were freed in frame 1).

Build: nothing new; keep the reload semantics (free models first, unload, load the area DAT as a zone, auto schedulers
fresh).

## Item 67: the call that puts an 0x00E pose on an undriven ship (gaps L83, MODLOG L20101 / L20227)

**Still not found in this pass; behaviour already matches retail (ours holds a mid-drive update until the drive ends).**
Checked, adding to MODLOG's list (all negative for a ship, +0xee 5):
- the 0x00E handler (0x1009b550..0x1009cf00): its only +0xee accesses are the kind writes 0..8 (0x1009c94f..0x1009ce5d);
  its calls into the actor are FUN_10092900 (model free, 0x1009b5f5 / 0x1009b99d), FUN_1009ad20 (+0x130 bits 26..31 /
  +0x134 bit 0, with FUN_10092800 on a change) and FUN_1009afe0 — no position push;
- no `cmp byte [reg + 0xee], 5` anywhere except FUN_1008c6c0 (0x1008c8c5, the status-motion reader's kind gate) and
  the 0x00E writes; none of the 70 FUN_10084400 (kind getter) sites compares with 3 / 4 / 5 directly;
- the entity update FUN_1008cd90's model branch copies drawobj → entity (+0x24 / +0x44 from drawobj +0x34 / +0x44,
  and the mount-rider cases), not the reverse; its door-actor case (statuses 0x1a / 0x1b / 0x1d / 0x1e, XiDoorActor,
  drawobj +0x624) is for doors and lifts.
Remaining candidates for a later pass: the ship actor class's own frame (FUN_100c47f0 → FUN_100a57f0 → FUN_10086e90's
+0xc4 / +0xe4 copy, MODLOG L20227) reading entity +0x24.. when +0x88 bit 0 is clear, and FUN_1009afe0.

Build: keep ours (apply a ship's server pose once no path drive is running).

## Item 68: hit reactions — two at once; the locked camera's CPU pose (gaps L94, MODLOG L4860)

**Retail has no separate "overlay" slot to replace: a reaction is an ordinary motion enqueued in its region's motion
queue, so a second reaction cross-fades over the first (it doesn't cut it).**
- FUN_100cdd60 (the op 0x25 / 0x21 reaction; this build, same address): skipped while FUN_10097a60, for count 0, for the
  statuses FUN_10084350 / FUN_10084370 accept, or while +0x80 (a hold) is set; count is clamped to 1..3 (3 for
  entity-less actors). It picks the 4cc — standing `dfi?` / `dbi?` (front / back by side or position), moving `dfm?` /
  `dbm?` (+0xf8) — and calls **FUN_100cd920(4cc, amount × table[count] ("t123t123" scale table 0x1035d730), rate,
  count, blend-in, fade, …)**.
- FUN_100cd920 is the generic named-motion player (the `?` wildcard, the model's own banks via vt+0x400, kinds 6 / 7
  special-cased) and ends in **FUN_100cd540 → FUN_1001a660 / FUN_1001a6e0 → FUN_1001ad00**, the region enqueue of
  batch 5 item 9. So a new reaction is appended after the playing one in the same region: the old entry gets
  FUN_1001b520(blend) (fade-out over the new one's blend-in) and the new one inherits its phase (the carry), with
  more than 3 entries the older ones fast-faded (1.0) — i.e. a cross-fade, not a replacement. (Regions 5 / 6 per
  MODLOG's FUN_1001a420 reading.)
- The locked-camera point is read from the posed skeleton like every attach point, so in retail it includes whatever
  the region queues hold — reactions too *(inf: no separate camera pose path was found; ours computes a CPU pose
  without the overlay)*.

Build: queue a reaction as a motion in its region with the generic cross-fade (old fades over the new blend-in, phase
carried, ≤ 3 live entries); include reaction regions in the CPU pose the locked camera reads.

## Item 69: ops 0x9c / 0x9d; the s2c 0x038 handler's filters (gaps L98, MODLOG L13172 / L13178)

**Ops 0x9c / 0x9d:** MODLOG's reading stands (a following doll, then FUN_100ac290(f32 @8): y fixed, x / z follow); no
audited file uses them — nothing to build.

**s2c 0x038 (LAB_1009dee0, registered at 0x1009450e; plays a scheduler 4cc on an actor pair) — the filters in order,
any failure drops the packet:**
1. both indices valid: FUN_10086fc0(u16 @0x10) and (u16 @0x12);
2. both actors exist (DAT_10480af0[…]) and their server ids match **@4 (actor 1) / @8 (actor 2)** (+0x78);
3. both have a model (+0x120 bit 9);
4. **(DAT_104808fc | DAT_1047fb40) & 0x800 together with DAT_10487e48** → drop (the "can't move" flag of item 43's
   FUN_100964f0; *(inf: a busy / cutscene state)*);
5. **(DAT_104808fc | DAT_1047fb40) & 0x20** → drop if either actor is bound to an event (+0x120 bit 7);
6. either actor with **+0x124 bit 29** → drop;
7. **4cc @0xc == 'hov1'**: allowed only through the battle-effect display filter **FUN_100adee0(actor1, actor2,
   config 0x92)** — config 0x92 is a mask: 0 → shown; bit 0 → only while the player's status is 1 / 3 (engaged);
   otherwise the pair's relation (FUN_100adf90) must match one of the table rows at 0x10330184.. ({bit, mask a, mask
   b}) whose bit is set in the config *(inf: the "effects for party / alliance / others" options)*;
8. then **actor 1's model vt+0x298(4cc @0xc, actor 2's model, 0)** starts the scheduler.
The globals DAT_104808fc / DAT_1047fb40 are the same flag words FUN_100983f0 (c2s 0x015) tests with & 4.

Build: apply these drops before running a 0x038 scheduler (ids must match, both modelled, not event-bound under flag
0x20, not +0x124 bit 29, `hov1` through the effect filter config 0x92).

## Item 70: model hide / show — the skeleton's `fvib` clone and the +0x764 / +0x814 clones (gaps L103, MODLOG L13236 / L3782)

**+0x814 is the `fvib` step burst and +0x764 the held dust; both are footstep effects, so nothing needs to run them at
the show — the next step does.**
- **Hide** (skeleton vt+0x48 = FUN_100d6370): +0x764 and +0x814 clones are released (FUN_10071ec0 + FUN_100543a0 kill)
  and zeroed, then FUN_10085570 (every non-static clone on the actor deleted), +0x9e8 = 0.
- **Show** (FUN_10085a10 → FUN_100856f0, item 44): for a skeleton without a +0x814 clone, `fvib` is looked up again
  (vt+0x1ec(…, 5, 'fvib')), cloned on (actor, actor) (FUN_100536c0), its +0x64 = the actor, held (FUN_10071e90) into
  +0x814 — **not started**. The same creation happens at the skeleton's model build (FUN_100cf1d0, 0x100cf6bb..0x100cf706).
- **Who starts it**: the skeleton's per-frame footstep code (FUN_100d4bc0, 0x100d5320..0x100d53b2). When the step's
  resource changes (+0x810 ≠ +0x80c: a new step) and the **footstep-effects option DAT_10456a70** is on, it runs
  FUN_100d63f0(step resource) (the dust / footprint) and **restarts the `fvib` clone as one burst**: +0xcc = 0,
  FUN_100531d0 (restart), +0xd0 = 10000.0, +0xc8 = 0, **FUN_100496f0(0)** (the generator's ops run once — the
  emission), +0xc8 = 10001.0. So `fvib` fires once per step (MODLOG L3782's *(inf)* confirmed).
- DAT_10456a70 is written only by FUN_10002e20, from the config switch (0x101aa7ae: value = 1 − the config) and the
  defaults block (0x1001792e) — the footstep-effects option (MODLOG L3763).
- +0x764 (the held dust, MODLOG L19292) is likewise only fed by the step code, so after a show it comes back with the
  next step.

Build: at the show, re-make the `fvib` clone unstarted (or lazily); fire it as a one-shot burst on each new step while
footstep effects are on.

## Addendum (coordinator question): what the callers do when the per-id gate FUN_100fc3c0 refuses

**1. The gate time is stored before the queue allocation.** FUN_100fc3c0 (0x100fc3c0..0x100fc4c2): for force == 0,
if the id's last time is non-zero and now ≤ last + 1000 → return NULL (nothing stored). Otherwise **S+0x41b4[id] =
now (0x100fc47e..0x100fc489) first**, then FUN_100dda60(queue). If that returns NULL the function returns NULL, but
**the time stays set**, so the id is also gated for the next 1000 ms although nothing was sent.

**2. Per caller** (all these call sites pass force = 0, so all are gated):

| Id | Sender | Caller(s) | On NULL |
|---|---|---|---|
| 0x05B (update) | FUN_100a0510 | **event op 0x43** (FUN_100b63a0, 0x100b63e2) and **op 0xa7** (FUN_100baa60, 0x100baae4) | **retries**: the op sets +0x25a = 1 (yield) without advancing (0x100b63d8 / 0x100baab3 with bl = 1), so it re-runs next frame until sent; on success DAT_104855a8 = 1 (op 0x43) and advance 2 |
| 0x05B (end) | FUN_100a0660 | **the event end FUN_100ae960**, called every frame from FUN_10096640 (0x10096af4) while DAT_104804c2 && !DAT_104808f0 | **retries**: the local end FUN_100ae3f0 (which clears DAT_10487e48 / DAT_104804c2 …) runs only when the send succeeded; on NULL the rest of FUN_100ae960 still runs (FUN_100ae9b0, buff show FUN_10221c70, FUN_100852d0, credits close) and the whole thing repeats next frame |
| 0x05C (position) | FUN_100a05e0 | **event op 0x47** (FUN_100b62b0, 0x100b636f) | **retries**: yield (0x100b62f2: +0x25a = 1) without advancing |
| 0x061 (stats) | FUN_10097fd0 | FUN_10147030 (persona) and FUN_10144ca0 (mp_stat): **retry** every 20 frames, ≤ 12 tries (batch 7 item 28); FUN_101d3260: **retries every frame** until sent (+0x811 = 1 on success); FUN_10093db0 (the 0x00C zone-in step), FUN_10133130 (Equipment), FUN_1020dc60 / FUN_1020dfb0, FUN_1022f670: **drop** (result ignored) | see column 3 |
| 0x0DD (check) | FUN_100e5660 | FUN_101347b0 (0x1013491c / 0x1013496a) | **drop**: the result is ignored and DAT_105781c4 = now is stored anyway (the caller's own cooldown) |
| 0x0F1 (cancel status) | FUN_100a0590 | the buff window FUN_10227eb0 | **drop**; on success it also clears that status from the icon list DAT_10485a30 — on NULL the icon stays |
| 0x0E7 (logout) | the /logout handler (0x1007cdde..0x1007cdf4) | the command | **drop**: jumps to the end (0x1007cf77) with no line and nothing changed |
| 0x00C (zone-in) | FUN_10093db0 | FUN_10096640 every frame (0x100966f7: a 0 return ends that frame's player update) and actor ctors (ignored) | **retries**: DAT_104859e4 (pending) stays set until a send succeeds; success clears it, sets DAT_1047d3f0 = 1 and queues 0x061 (FUN_10097fd0(0, 0), dropped if refused) |
| 0x011 (zone-in) | FUN_100fbb40 | the connection state FUN_100fb830 (state 3, while +0xd == 0) | **retries** each call until sent (+0xd = 1 on success) |
| 0x037 (item use) | FUN_100eb5e0 / FUN_100eb660 | the item sender FUN_101f0260 (batch 7 item 29) and /item (FUN_100eb570) | **drop**: returns 0; the cast bar the item sender opened stays up |
| 0x028 (drop) | FUN_100eb6c0 | the linkshell Drop (FUN_101e5d60, FUN_101e5f90), the item Drop FUN_101f2ff0 | **drop**: the item Drop ignores it and closes iuse anyway; the linkshell paths test it (no retry found) |
| 0x029 (move) | FUN_100eb710 | sort / move code (FUN_101daf70, FUN_101dbdb0, FUN_101f0460) | **drop** (results ignored or used only to skip local follow-up; no retry loop found) |
| 0x0B5 / 0x0B6 (chat) | FUN_100e3400 / FUN_100e3570 | the chat line | **drop** (batch 5 item 2 / batch 6 item 12) |

So the **retrying** callers are: event ops 0x43 / 0xa7 / 0x47 (0x05B / 0x05C), the event end (0x05B), the zone-in pair
0x00C / 0x011, and the 0x061 stats requests of persona / mp_stat / FUN_101d3260. Everything else drops silently.

Build note: when gating these ids, the retrying paths must keep their state (stay on the op / keep the pending flag)
and try again next frame; and remember the time is consumed even when the queue itself is full.

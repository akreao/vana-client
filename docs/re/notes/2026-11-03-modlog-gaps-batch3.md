# MODLOG gaps, batch 3 (RE notes)

Target and tools as in [batch 1](2026-11-03-modlog-gaps-batch1.md) (retail `FFXiMain.dll`, unpacked, sha256 8097dbf8…550c). All addresses are VAs in this DLL. **(inf)** = inferred. Old-build (MODLOG / vana-client) addresses are mapped where they differ. Event-op table 0x100bc960. The scheduler dispatcher is FUN_10057fb0, table 0x1005dc1c indexed by op - 2.

## 1. Costume state machine helpers and +0x120 bit 7

**The four predicates** are each `thiscall(actor, u32 status)`. An argument of **0xff means "this actor's current status +0x170"**; any other value is an explicit status.

| Function | Asm | True when status is |
|---|---|---|
| FUN_10095680 | 0x10095680..0x10095697 | 5 |
| FUN_100956a0 | 0x100956a0..0x100956b7 | 0x55 |
| FUN_100956c0 | 0x100956c0..0x100956e4 | 6, 0x26..0x2b, 0x32..0x35, 0x38..0x3e (it ignores its argument and passes 0xff to FUN_100956f0 0x100956f0..0x1009573f and FUN_10095750 0x10095750..0x1009578b) |
| FUN_10095790 | 0x10095790..0x10095811 | 0x3f..0x53 (chairs; already in the jump table) |

At 0x10095ef3 and 0x10095f01 (the calls without `mov ecx`) ecx is still the actor: FUN_10095680 and FUN_100956a0 touch only eax.

**FUN_10095e90 exactly** (asm 0x10095e90..0x10095fd6; S = +0x170; bl = bit 5):
```
want = 1
if bl && S in {2,3,7,0x21,0x2c,0x2f,0x30,0x3f..0x53}: want = 0           (table 0x10095fe0, S-2)
cl = (bl && S in {5,6,0x26..0x2b,0x32..0x35,0x38..0x3e,0x55,0x3f..0x53}) ? 0 : want
if +0x120 bit 7: cl = 0
if bit 4 (pending):
   if u16 +0x166 == 0: FUN_10092900; +0x128 &= ~0x40000000; +0x120 &= ~0x30; +0xee = 0   (costume off)
   elif cl == 1:       FUN_10092900; clear bit 4; set bit 5; +0x128 |= 0x40000000; +0xee = 2  (costume on)
   else:               stay pending
elif bl && cl == 0:    FUN_10092900; +0xee = 0; clear bit 5; set bit 4   (+0x128 bit 30 NOT cleared)
```
- **The status sets only take off a costume that is on. They never block putting one on. Only bit 7 blocks both.**
- (inf) If the status stays in one of those sets with +0x166 != 0, the model alternates apply / remove every frame.
- For a bound actor during an event, +0x170 holds the event status +0x174 (FUN_1008e9a0, 0x1008eb72..0x1008ec5f).
- FUN_10092900 frees the model (+0xa0) and clears +0x120 bit 9; FUN_1008f740 reloads it under the load budget.
- **All writers of bits 4 / 5:**
  - FUN_1008cce0 sets the id and bit 4 when the id changes. Callers: s2c 0x00D @0x30 (0x10099ceb in FUN_10099a40) and s2c 0x037 @0x44 (0x1009d6a4); 0x1008cd10 (id 0, no reference found).
  - Actor reset FUN_1008a350 at 0x1008a3d2 clears both.
  - FUN_10095e90's three exits above.
- **+0x120 bit 7 = bound to the current event.**
  - Set in the per-actor event object constructor FUN_100bd1a0 (0x100bd53e) when the actor's event block lists the current event id DAT_104886dc (set by FUN_100aed00 from char +0x40d92) or 0xfffe. Otherwise bits 7 / 8 are cleared (0x100bd5b9, 0x100bd5d3).
  - Reached via FUN_1008edc0 from FUN_100aeea0 (0x100aefb5); the object goes to +0xd4.
  - Cleared by unbind FUN_1008ee30 (`+0x120 &= 0xfffffe7f`, `+0x124 &= 0x1ffdffff`, frees +0xd4). Callers: 0x1008a7b9, 0x100968ed, FUN_10099a40 ×3, FUN_1009b3f0 ×2, event end FUN_100ae3f0 (0x100ae63e).
  - A bound PC gets +0x174 = 0 (stand) unless +0xee == 1 with status 0x2f / 0x30 / a chair.
- **How often it runs for the player: once per frame.**
  - Main loop 0x100106cc → FUN_10096640. It is skipped unless: FUN_100fc160() & 1, zone-in done, the DAT_10482f43 timer finished, *DAT_10456a28 == 0x60, and not (DAT_10482f44 && [DAT_106631d4+0x18]).
  - Loop 0x10096a3f..0x10096a92: FUN_1008e9a0 buckets every actor; the player is never skipped.
  - FUN_10095900 (0x10095900..0x10095d6a): 5 passes (calls 0x10095a5a, 0x10095aec, 0x10095bad, 0x10095c51, 0x10095d04), each actor once per frame (+0x12c bit 29 marks it done). The load budget only defers model loads.
  - Exceptions:
    - a frame where an event op sets DAT_104898f9 ends early (0x10096c86), and FUN_10095900 doesn't run;
    - an actor with +0x120 bit 23 (hidden / culled) and +0x130 bit 3 clear gets FUN_10092900 instead. For the player, only while hidden (inf).

## 2. 0x7f0b / 0x7f8b and +0x124 bit 25

- **They are event-value operand codes, not constants in the code**, decoded in FUN_100af4c0 (via FUN_100af4b0):
  - 0x7f0b (0x100af6b6..0x100af6c3) = `(actor[event+2]->+0x124 >> 25) & 1`, i.e. the event owner (speaker);
  - 0x7f8b (0x100af75f..0x100af76c) = the same bit of the player.
  - Siblings: 0x7f06/86 job, 0x7f07/87 race +0xef, 0x7f08/88 level, 0x7f0a/8a server id +0x78.
- **+0x124 bit 25 = gender: 1 male, 0 female** (LSB `Gender`; male = Hume / Elvaan / Taru male and Galka). Writers, all copying the server's gender bit:
  - player init FUN_10094190 0x10094c7d..0x10094c9d (char +0x40db0 bit 6, from s2c 0x00A u32 @0x20 bit 15 at 0x100fb260..0x100fb26b);
  - s2c 0x00D @0x20 bit 15 (0x1009a37b..0x1009a3a4);
  - s2c 0x00E @0x20 bit 15 (0x1009bff3..0x1009c020);
  - s2c 0x037 @0x28 bit 8 (0x1009d270..0x1009d286).
- **Readers:**
  - the 0x7f0b / 0x7f8b operands (the ~475 data sites = scripts branching on speaker / player gender);
  - the dialog text expander FUN_100c1650 / FUN_100bdad0, which picks `[a/b]` alternative !bit25, so male gets the first: code `7F 85` player (0x100c1979..0x100c19a5), `7F 90` actor +0xbfe (0x100c1a20..), `7F 91` actor +0xc00;
  - FUN_100c1190 at 0x100c1578..0x100c1593.
- This fills MODLOG line ~4893's open item. MODLOG's "Flags3 bit 25" / +0x12c bit 31 is a different field.
- **FUN_100977f0** (0x100977f0..0x1009783e) does not read bit 25. The PC-specific part is in **FUN_100973f0** (0x100973f0..0x100974f0): only for a model-loaded, unbound PC (+0x78 high byte 0). With +0x128 bit 6 set it makes the model see-through (0x27808080 own player, 0x808080 others) and returns 1, which hides the name. Otherwise it restores 0x80808080. (inf: bit 6 = an Invisible-type flag; its writer was not traced.)

## 3. Event op 0xab subs 9 / 0xa

- Entry 0x100bcc0c = 0x100bc7e7 → FUN_100bac90; sub table 0x100bb108 (subs 0..0x1c). Sub 9 → 0x100bae5f `push 1`, sub 0xa → 0x100bae63 `push 0`; both `call 0x10087120`, then `+0x256 += 2` (0x100bae6d).
- **Both are 2 bytes with no operands**, so they read no Z register. MODLOG's "sites read Z registers" belongs to op 0xb6 0xd / 0xe.
- **Z registers** = zone work words: event operand k 0x1000..0x105f → DAT_104887f8[k - 0x1000] (96 dwords), filled from the server's event-packet params (FUN_1009f7c0, 0x1009f836..0x1009f85c). Other operand ranges (FUN_100af4c0):
  - k >= 0x8000: constant table [ev+0x1c];
  - k < 0x50: event-local [ev+0x260] + k*4;
  - 0x1100..0x113f: Zone_Memorize DAT_10488a00;
  - 0x1700..0x171f: DAT_10488980;
  - 0x7f00 / 0x7f80 and up: actor values.
- **Effect:** FUN_10087120 sets DAT_10487ef5 (sub 9 = 1, sub 0xa = 0). It is cleared at zone / world init (FUN_10093720, 0x10093a00) and read only by FUN_10002d80. That returns **1.0 while set**, else the config value _DAT_10456a68 (config id 0x3b, registered at 0x101a984c; default 1.0; set by FUN_10002d70 from FUN_101aa650).
- It is an effects-amount multiplier, used by effect / generator code:
  - effects with resource +0xde bit 7 are skipped at 0.0 (0x1003fbcd, 0x10040bc7, 0x10043347);
  - a distance limit (0x1004a654);
  - emit count × mult × 0.5 when the count is >= 20 (0x10063c98).
- So **sub 9 forces full effects regardless of the player's setting; sub 0xa restores the setting.** No actor field is touched. (inf: which menu option config 0x3b is.)

## 4. Faded actor rendering (D3D8)

- D3D8 (Direct3DCreate8). Device [DAT_1045666c]+0xc. vtable: 0xc8 SetRenderState, 0xfc SetTextureStageState, 0xf4 SetTexture, 0xa8 SetMaterial, 0x90 Clear, 0x7c SetRenderTarget.
- **Alpha sources** (all write drawobj +0x660 = alpha << 24 | rgb, 0x80 = 1.0):
  - event op 0x6c, FUN_100b7630 (old 0x100b5800): 9 bytes = actor@1, target alpha val@5, frames val@7 (0 → 1);
  - the **kesu fade = scheduler op 0x29** (table entry → 0x1005bd1c; task FUN_1005f6c0, vtable 0x1032ba7c, update FUN_1005f820 lerps the colour to the u32 @8 over the duration);
  - generator actor elements (FUN_10047590, which also sets +0x59c).
- **Effective alpha:**
  - FUN_100cd040 (from the update FUN_100c63d0 at 0x100c7d8d) computes +0x664 = (+0x663/128) × camera-near fade (0.5 with actor +0x130 bit 18, ≤ 1; 1.0 with +0x12c bit 26).
  - +0x59c = the spawn fade (+1/32 per tick; −1/16 in first person).
  - Draw FUN_100cbde0 (0x100cbef6..0x100cbf65): f = (+0x664 < 0.8 ? +0x664 × 1.25 : 1) × +0x59c, ≤ 1; colour alpha = ftol(f × 128). So a byte alpha ≥ ~0x67 already draws opaque, and f < 0.0001 skips the mesh.
- **Draw order:** an actor is "translucent" if +0x59c × +0x664 < 1, or vt+0x70, or +0x710 is non-empty (0x100c7ded..0x100c7ea2).
  - Translucent actors go into the **shared translucent list DAT_1047bc94** (FUN_100553c0; 0x200 depth buckets over 0..512, key = projected z × 512 + +0x8ec). That list is drawn by FUN_10055620 / FUN_10055520 **back to front**, interleaved with other translucent objects (render-queue slot 0xc; where it sits relative to the opaque pass is inf).
  - Opaque actors are drawn in the normal pass (+0x8e8 = 1, FUN_100cbd60).
- **Mesh pass** (FUN_1002b6e0, 0x1002b6e0..0x1002c256). The state block FUN_10009920 sets the defaults: ZENABLE 1, ZWRITE 1, ALPHATEST 1, SRC ONE / DEST ZERO, ALPHABLEND 0, ALPHAREF 0x60, ALPHAFUNC ALWAYS, ZFUNC LESSEQUAL. Then:
  - material ambient = colour / 128;
  - alpha 0 → return;
  - **begin offscreen** FUN_1001d240: redirect to a helper RT (sharing the scene depth, inf) and Clear(TARGET, 0x00000000) — skipped when there is no helper texture, or when alpha == 0x80 with no +0x9c effects;
  - ALPHAFUNC GREATER, ALPHAREF 0x45, ZENABLE 1, **ZWRITE 1**, ALPHABLEND 0, FOG on;
  - stage 0 colour MODULATE2X(TEX, CURRENT), alpha MODULATE2X(TEX, TFACTOR); stage 1 colour MODULATE2X(CURRENT, TFACTOR), alpha MODULATE4X;
  - TEXTUREFACTOR rgb = (0x103515bc × colour) >> 7, **alpha fixed 0x80**. So the mesh is drawn opaque, with alpha test only;
  - after: CULL NONE, ALPHAFUNC ALWAYS, ALPHAREF 0x60, ALPHABLEND 0, FOG 0.
- **Composite** FUN_1001d3b0 (0x1001d3b0..0x1001d7ff, called at 0x1002c1d6, after the RT pop FUN_1001d350): a full-viewport XYZRHW quad (FVF 0x144, 2-tri strip, z 0) textured with the offscreen RT, vertex colour `fadeAlpha << 24 | 0x808080`. States:

  | State | Value | Verified at |
  |---|---|---|
  | ALPHAFUNC | GREATER | 0x1001d531 |
  | ALPHAREF | 0 | |
  | ZWRITEENABLE | 0 | |
  | ZENABLE | 0 | |
  | **ALPHABLENDENABLE** | 1 | 0x1001d583 |
  | **SRCBLEND** | SRCALPHA (5) | 0x1001d599 |
  | **DESTBLEND** | INVSRCALPHA (6) | 0x1001d5af |
  | stage 0 colour | MODULATE2X(TEX, DIFFUSE) | |
  | stage 0 alpha | MODULATE2X(DIFFUSE, TEX) | |
  | stage 1 | off | |
  | sampling | POINT | |
  | alpha test | still enabled from the defaults | |

  Result alpha = 2 × a / 255 × RT.a, so 0x80 = 1.0.
  - Restored after: ALPHABLEND 0, ALPHAREF 0x60, ZWRITE 1, ZENABLE 1, ALPHAFUNC ALWAYS, LINEAR filtering.
- **So a faded actor is drawn opaque (with Z write) into an offscreen target, then blended onto the screen as one SRCALPHA / INVSRCALPHA quad with Z off.** Self-overlap doesn't show through, but its depth is written (inf). Not traced: the DAT_104566e8+0x18 second-RT path, the +0x9c effects.

## 5. Scheduler ops 0x56, 0x6f / 0x70, end callback old 0x100a7440

- Dispatcher FUN_10057fb0 (0x10057fbc..0x10057fd8). Table 0x1005dc1c[code - 2]: 0x56 → 0x1005a3b4; 0x6f and 0x70 → 0x10059da7. Codes 0 / 1 → default 0x1005ac96.
- Lengths from a scan of all type-7 chunks in ROM..ROM9 DATs:
  - **0x56 = 3 dwords** (4003 uses): @4 wait, @6 u16 duration, @8 0.
  - **0x6f / 0x70 = 5 dwords** (753 / 438 uses): @4 wait, @6 duration, @8 generator 4cc, @0xc runtime cache (0 in the file), @0x10 f32 target.
- **Op 0x56 = hold every target** (0x1005a3b4..0x1005a431).
  - With no action (+0x90 == 0) it reuses op 0x20's code (one hold on actor2, resolved through dolls).
  - Otherwise for i < holder +0x1e: target = FUN_1018c980(action, i) (no doll resolving), and a 0x78-byte sub-task FUN_100601f0(task, target, ftol(dur × speed +0x9c)).
  - The sub-task is a child of the scheduler task. Its start does FUN_100a4140: entity +0x11c += 1 (busy), +0x11e = 0x708. Update FUN_100602a0 (vtable 0x1032bb0c): remaining −= ticks, dies when < 0. Its destructor FUN_100602d0 → FUN_100a4160 does busy − 1 (min 0) and +0x11e = 0x708.
  - **Effect:** the status-motion change FUN_1008b460 (0x1008b49b..0x1008b4c1) refuses while busy != 0 (unless forced), so e.g. the death fall waits. Other busy readers: 0x100ad598, 0x100af0c6.
  - **The hold ends on:** its timer; the scheduler task's destruction (children deleted, inf); the actor unlinking (inf); or the entity watchdog at 0x1008ea0d..0x1008ea3c (+0x11e counts down 1800 ticks while busy, then forces busy = 0). The task's own end doesn't wait for holds.
- **Ops 0x6f / 0x70 = point-light ramps** (0x10059da7..0x10059e8d).
  - Resolve the 4cc as a type-5 generator (cached at @0xc). Needs FUN_100539e0 to pass and its live element gen+0xb0 to be **CMoPointLightProgElem** (type 0xb, vtable 0x1032b758).
  - Sub-task FUN_10060990 (0x4c bytes, vtable 0x1032bb80): over dur × speed ticks (float), update FUN_100609f0 lerps linearly from the start to the f32 target. Its destructor FUN_10060960 writes the exact target.
  - **0x6f → element +0x1a4 = the intensity multiplier; 0x70 → +0x1a8 = the range multiplier.** The light's frame FUN_10056400 uses intensity = max(0, +0x1a0) × +0x1a4 × fade and range = +0x1a8 × +0x19c (0x1005646b..0x100564e3).
- **End callback: old 0x100a7440 = 0x100a8db0 here.**
  - Set at 0x100a95d9 (task +0xac, +0xb0 = caster) by FUN_100a9500. That is the wait task (vtable 0x1032e7c0) which, once the caster and targets are ready, starts the caster's type-7 "main" scheduler for a weapon skill / dance / rune (files loaded by FUN_100d25e0 = old FUN_100d0570).
  - Called by the task destructor at 0x10057549..0x1005755d.
  - FUN_100a8db0(task, actor): if another scheduler task on actor +0x68 still has this callback, it does nothing. Otherwise it calls actor → FUN_100d44b0(6), which **unlinks and deletes every kind-6 entry in actor +0x7e4**. Kind 6 = the files FUN_100d25e0 added (0x100d26ce file A, 0x100d2777 file B). **So the skill's motion files are dropped when the last "main" on that caster ends.**
  - (inf) If the wait task dies before starting "main", nothing drops them. The query twin is 0x100a8d60.

## 6. Positional sound volume (old FUN_10035e50 = FUN_10036c70)

- Signature: `int FUN_10036c70(float *pan, int *vol, float *pos, float near, float far, float panScale, u8 keep, int heightMode)`, asm 0x10036c70..0x10036ef6.
- Callers: the element update FUN_100364e0 at 0x10036615 (near +0x1b0, far +0x1b4, panScale +0x1b8, keep +0x1e8, mode +0x1e9); the one-shot FUN_10036fa0 at 0x10036ff1 (mode 0, keep 0); 0x10037256 (result unused). Both real callers then multiply by **DAT_10351704** (the world SFX volume).
- Constants (init 0x1003560a..0x10035628, read as floats):

  | Value | Address | Use |
  |---|---|---|
  | 30.0 | 0x103516e8 | default far |
  | 3.0 | 0x103516ec | default near |
  | 512.0 | 0x103516f0 | default pan scale |
  | 0.65 | 0x103516e4 | behind factor |
  | 3.0 | 0x10329d38 | height factor |
  | 0.1 | 0x1032a378 | inner radius / behind threshold |
  | 64 | 0x1032a98c | pan centre |
  | 127 | 0x1032a34c | pan max |

- Algorithm:
  1. No camera → pan 0, vol 0, return 0.
  2. far 0 → 30; near 0 → 3; panScale 0 → 512, else × 512.
  3. **Listener = the camera eye** (cam +0x60, copied from +0x44 each view build by FUN_10021b70), not the player. d = eye − pos.
  4. dy: mode 1 as is, mode 2 → 0, mode 0 → **× 3**. dist = |d|.
  5. dist < 0.1 → return the input vol (pan untouched).
  6. dist > far → keep ? vol = 1 (pan still computed) : vol 0, return 0.
  7. **Linear:** near < dist ≤ far → vol = trunc(vol × (1 − (dist − near) / (far − near))); ≤ near → full.
  8. View-space v = M × pos (DAT_104568fc+0x94). If v.z > 0.1 (behind the camera) → vol = trunc(vol × 0.65) and a = z, else a = |z|; a < 1 → 1.
  9. **Pan:** p = clamp(focal / a × v.x / panScale, −1, 1), focal = 192 / tan(fovy / 2) (DAT_104568fc+0x2f4, default 350); pan = clamp(p × panInput + 64, 0, 127). With input 64, 0 = left and 127 = right. Input 0 gives a centred 64.
  10. Return vol, or `keep` if vol is 0.
- The element re-sends pan (FUN_10038440) and volume only on change. MODLOG's description matches this build exactly.

## 7. Zone ambient loop changes

- FUN_10036170 has exactly two callers:
  1. **Every frame**, from the main frame FUN_10011a00 (0x10011c97) → FUN_1018b860 → FUN_1018b8b0 (0x1018b8b5) with the zone node's chosen ambient. FUN_101874a0 updates each environment (FUN_1018a340) and copies env +0x2c.
  2. The release hook of the zone's **SeSep (type 0x3d) chunk**, FUN_10037ac0 (vtable 0x1032c530 slot 9; 0x10037af5): it deletes the positional voices of that resource and, if it is the current ambient, calls **FUN_10036170(0)**.
- **FUN_10036170** (0x10036170..0x100361ce):
  - argument 0 → FUN_10036b90 → FUN_10246020 → channel +0x48 = 0, **an immediate key-off, no fade**;
  - a new resource → play it (FUN_10036f00, pan 0x40, ftol(DAT_10351708 × 127)) **without stopping the old one**;
  - the same resource → nothing.
- **The sound engine crossfades.** Ambient files have flag byte +0x29 = 0x20 (checked se001001..1007, 1042, 1056). On play, FUN_1024a7c0 → FUN_1024a1b0 walks the playing flag-0x20 voices:
  - same id: re-targets it (no restart);
  - other id: fades it to table[1] over **0x96 = 150 ticks**, then keys it off (FUN_102489c0, 0x102489e0..0x10248a03);
  - **the new voice ramps in from table[1] over 150 ticks.**
  - A tick is one sound-thread loop (`Sleep(16)`, 0x10245c40) ≈ 16–17 ms, so **≈ 2.5 s** (the client's own conversion FUN_1024abf0 would say 3.0 s, inf).
- **What decides the ambient:** env +0x2c via FUN_1018a220 → FUN_10189ca0, the latest HHMM-named type-0x3d chunk at or before the current time in the weather folder ('indo' when covered, zone+0x1bc; 'extr' excluded). None found → +0x35 = 1 and no further search.
- **Triggers:**

  | Trigger | Path | Result |
  |---|---|---|
  | Time of day | FUN_1018a340 crossing → FUN_1018a220 (0x1018a48f) | crossfade |
  | Weather | FUN_101874f0 makes a new environment head (its +0x2c = 0 → re-pick) | crossfade at once (inf); no ambient for that weather → hard stop |
  | Indoor / covered | FUN_1018b5e0 → FUN_101889e0 → FUN_1018a320 | crossfade |
  | Event time / weather | ops 0x77 / 0x78 (FUN_100b8710 / FUN_100b87b0) | as above |
  | **Zone change** | the old SeSep release → FUN_10036170(0) | **hard cut**; the new zone's loop starts next frame with a 150-tick fade-in (inf: release before start) |

- Logout FUN_100361d0 only clears globals. Level changes (FUN_10035fb0 → FUN_10038470(voice, lvl, 0)) are instant; event ops 0x69 / 0x6a fade them.

## 8. Number windows: UI sounds

- **Generic key layer FUN_10118260** (before every OnKey):
  - Enter → **SE 0002** (FUN_10037b90), or 0004 if the cursor item has state 5;
  - Escape → **SE 0003** (FUN_10037bf0); the window closes unless registry flag bit 2 is set (0x1011844d);
  - cursor moves → SE 0001 only when the cursor moves to another item.
- Only two digit-entry windows exist; every quantity / amount screen reuses them:

  | Window | OnKey | Open function | Registry flags | Users |
  |---|---|---|---|---|
  | "menu    itemctrl" (quantity) | FUN_1013a6c0 (vtable 0x10333d78, 0x1013a6c0..0x1013a816) | FUN_1013a870(1, max, cb, ctx, 0); no window when max == 1 | 0x80002 | delivery, handover, trade, recycle bin, bazaar buy, guild shop, shop |
  | "menu    moneyctr" (gil) | FUN_10140bc0 (vtable 0x10334070, 0x10140bc0..0x10140d35) | FUN_10140d90(value, max or the player's gil, cb, ctx) | 0x2 | delivery gil, handover, trade gil, auction, bazaar price |

- **itemctrl keys:**
  - Down: value −= step (minimum 1); Up: value += step (maximum max);
  - Right: value = 1; Left: value = max;
  - Enter (cursor 1): FUN_1013a8d0(value); Escape: −1;
  - holding a key raises the step ×10 after 60 ticks (FUN_1013a5b0, maximum step 10).
- **moneyctr keys:**
  - Down: value −= step (minimum 0); Up: value += step (maximum max);
  - Right: step /= 10, or value = 0 when the step is already 1; Left: step ×= 10 while max / step ≥ 10, else value = max;
  - Enter: FUN_10140df0(value); Escape: −1.
- **Sounds:**
  - **Up / Down / Left / Right / digit change: none** (no SE helper call in either OnKey). The generic 0001 can't fire, because the numctrl item has no neighbours (−1 in ROM/118/111.DAT at 0x2589f0 / 0x258ba0; inf from the layout).
  - **Hitting a limit: none** (silent clamp, e.g. 0x1013a717 / 0x10140c0b).
  - **OK: 0002. Cancel: 0003** (generic layer).
  - Openers add their own 0002 when opening (0x1012e352, 0x10139832, 0x10151032, 0x101db774, 0x101e065c, auction 0x101d9a00, delivery gil 0x1012ee48).
  - 0004 for a delivery with no free slot (0x1012ede5) and a bazaar row mismatch (0x101de681).
- SE helpers by id:

  | Helper | SE | Helper | SE |
  |---|---|---|---|
  | 0x10037b30 | 0001 | 0x10037dd0 | 0010 |
  | 0x10037b90 | 0002 | 0x10037e30 | 0011 |
  | 0x10037bf0 | 0003 | 0x10037e90 | 0013 |
  | 0x10037c50 | 0004 | 0x10037ef0 | 0014 |
  | 0x10037cb0 | 0006 | 0x10037f50 | 0015 |
  | 0x10037d10 | 0005 | 0x10037fb0 | 0016 |
  | 0x10037d70 | 0009 | 0x10038010 | 0039 |

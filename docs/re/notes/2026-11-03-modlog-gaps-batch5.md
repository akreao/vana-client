# MODLOG gaps, batch 5 (queue items 1..10)

Copied from Steven's `/run/media/SSD/FFXI/re-notes/modlog-gaps-batch5.md` (2026-11-03). Target and tools as in
[batch 1](2026-11-03-modlog-gaps-batch1.md) (`FFXiMain.unpacked.dll`, sha256 8097dbf8…550c). All addresses are VAs in
this DLL. **(inf)** = inferred. Queue: modlog-gaps-queue.md (easiest first).

## Item 1: event text codes 0x1d and 0x13 (gaps L75, MODLOG L15379)

Both are in the text expander FUN_100c1650 (all_decomp.c `//@@FUNC 100c1650`, top-level byte switch).

### 0x1d: the speaker's compass direction (1 byte, no operand)

Asm 0x100c2754..0x100c27c9:
- A = DAT_10480af0[msg +0xbfe] (message actor 1; batch 4 §3). If there is no actor, it appends "" (0x103d3c84).
- Otherwise **k = ftol(wrap2π(A +0x18 + 0.3926875) × 1.27327704) mod 8**:
  - A +0x18 = the actor's yaw (written from 0x00D @0x0B at 0x10099f4b..0x10099f8e);
  - 0x10330274 = π/8; 0x10330270 = 4/π;
  - FUN_100dd9d0 wraps to [0, 2π) using 0x10331818 = 6.283185;
  - the `and 0x80000007` fix-up keeps it 0..7.
- Appends **message group 0xf id k** (`push k; push 0xf; call 0x10152ef0`, then FUN_100c2d00), and advances **1 byte**.
- Group 0xf (file 55659, en): 0 "east", 1 "southeast", 2 "south", 3 "southwest", 4 "west", 5 "northwest",
  6 "north", 7 "northeast" (read with `vana-client msg-dump 15`).
- So it names the 45° sector the speaker faces, centred on each direction: yaw 0 = east, increasing toward south.

### 0x13: not Japanese — the raise / tractor offer (2 bytes, `13 nn`)

Decompile case '\x13':
- nn = 0 and msg actor 2 (+0xc00) == the player (DAT_10485f7a) → FUN_10096620: `DAT_10485ff8 = 2`.
- nn = 1 and actor 2 == the player → FUN_10096630: `DAT_10485ff8 = 3`.
- Always advances 2 bytes and outputs nothing (the `break` path does `+0x1c0 += 2`).

**DAT_10485ff8 is the K.O. offer state** (inf for the names, from its other writers):
- the s2c **0x0F9** handler FUN_100a2540 (registered at 0x10094757): u16 @0x0A 1 → 2, 2 → 3, else 0;
- zone-in 0x10094fde: 1 while char +0x40ddc > 0 (the K.O. timer, minus 360 into DAT_10486004);
- init 0x10093949 / 0x100941d5: 0;
- read each frame by FUN_10096c90(player, &DAT_10485ff8) (0x10096b9c): the K.O. window logic (FUN_1012e0c0 etc.).

So `13 00` / `13 01` in an event line make the line act like a 0x0F9 raise (2) / tractor (3) offer to the player when
the player is the message's second actor. (inf: 2 = raise, 3 = tractor, after MODLOG's "raise / tractor offers (s2c 0x0F9)".)

## Item 2: c2s 0x0B5 / 0x0B6 text at 150 bytes (gaps L167, MODLOG L5986 / L6019 / L6201 / L9110)

Mostly already in MODLOG. This adds the exact copy and the packet size.

- **0x0B5, FUN_100e3400** (asm 0x100e3400..0x100e3541): the packet comes from FUN_100fc3c0(0xb5, 0, 0) (the per-id
  send slot).
  - +4 = kind, +5 = arg 3 byte.
  - len = min(strlen(text), **0x96 = 150**) (0x100e350c..0x100e3513); len bytes are copied to +6, then a NUL at
    `+6 + len` (0x100e352c).
  - Size: FUN_100fc4d0(pkt, 8, len - 1) → FUN_100ddd20: words = ceil((8 + len - 1) / 4), written to header bits 9..15
    (0x100ddd20..0x100ddd5f). So **bytes = 4 × ceil((len + 7) / 4); a 150-byte line gives 160 bytes.**
  - Before that, linkshell kinds 5 / 0x1b / 0x1e map to shell 1 / 2 / 3. Without that shell (FUN_100e9730 != 1) it
    prints system 0x25 and returns 1 without sending. Kind 4 with FUN_100e8f70(5) > 1 prints 0x24 (0x100e3453..0x100e347d).
- **0x0B6, FUN_100e3570** (from 0x100e35ab): name at +6 (length < 0xf, NUL at +6 + n; 0x100e35da / 0x100e35fb), text at
  **+0x15**, len = min(strlen, 0x96) (0x100e360d..0x100e3614). Size FUN_100fc4d0(pkt, 0x16, len - 1): **bytes =
  4 × ceil((len + 0x15) / 4); 150 bytes of text gives 172.**
- Retail sends up to 160 (0x0B5) / 172 (0x0B6) bytes. LSB's GP_CLI_COMMAND_CHAT_STD accepts 6..136 (MODLOG L9110), so
  a full 150-byte 0x0B5 is dropped by LSB. The chat line itself caps typed text at 0x78 = 120 (FUN_101abc50), so in
  practice only an expanded line (auto-translate) can exceed 129 text bytes. Nothing else to trace: implementation only.

## Item 3: the message formatter and a backward jump (gaps L84, MODLOG L20429)

**Retail never stops: the client hangs in FUN_10153310.** (all_decomp.c `//@@FUNC 10153310`; loop head and tail
`while (cVar5 != '\0') … switchD_10153406_default: iVar8 = iVar10; cVar5 = pcVar7[iVar10];`)
- **The only exit is reaching a NUL byte at the read position.** Nothing counts iterations or checks the target.
- Jump codes (control prefix at p, p+1; code at p+2):
  - 0x84: always jumps to `p + 4 + (signed char)p[3]`;
  - 0x85 / 0x87 / 0x88 / 0x8a: jump if the compare flag is equal / greater / less / (less or equal), else `p + 4`;
  - 0x86: jump if not equal;
  - 0x89: jump if greater or equal.
  - 0x83 sets the flags. A negative offset goes backward with no check.
- **A full output buffer does not stop the loop either.** A plain byte with no room sets `local_11c = -1` (and
  multi-byte / string codes do the same), but the loop condition ignores it. The −1 is only returned after the NUL
  (`if (iVar9 < 0) return iVar9;`). Once the buffer is full, nothing more is written, but the loop keeps going.
- So a string whose jump goes back to or before itself (`… 84 xx` with xx ≤ −4) spins for ever in the caller's frame,
  i.e. the client freezes (inf: no watchdog around message formatting was found). Ours stopping at the first repeated
  state is a deliberate deviation; there is no retail behaviour to copy beyond "hang". Nothing to build.

## Item 4: the mouth's 8-tick blend out on op 0x23 and unbind (gaps L69, MODLOG L2440 / L14093)

**One function does it: FUN_100ce160(model)** (0x100ce160..0x100ce17e). It takes the motion pointer at **model +0x6b8**
(`+0x674` plus FUN_1002b5a0's `+0x44`; the mouth region the talk motion plays in) and, if it is non-null, calls
**FUN_1001b170(8.0) on that pointer +0x74**. FUN_1001b170 walks the region's motion list (`+4` next) and calls
FUN_1001b520(8.0) on each entry (0x1001b520..0x1001b56e, by state byte +8):
- state 0 (playing) → state 2 (fading out), fade length +0xc = 8.0;
- state 1 → state 3, +0x14 = +0xc = 8.0;
- state 2 (already fading) → unchanged, unless the new length ≤ 1.0 (then +0xc = 0);
- state 3 → unchanged.

So the lip motion is not cut: every motion in the mouth slot fades out over 8 ticks from its current pose.

**Callers:**
1. **Event op 0x23 (wait for the message to close), FUN_100b2dd0** (0x100b2dd0..0x100b2e94). Message state
   DAT_10488208:
   - 1 → yield;
   - 2 → set DAT_104804c2 / DAT_104808f0 / DAT_10487034 = 1 and yield;
   - **0 (closed) → advance 1 byte, then for the speaker S = actor[body +0x164]:** S +0x14e = 0 (the mouth countdown).
     If S has a model (+0x120 bit 9) and it is a CXiSkeletonActor (FUN_1002c8f0(0x10330ebc)), **FUN_100ce160**. Then
     body +0x164 = 0. (+0x150 is not cleared here.)
2. **Unbind: FUN_100bd5f0** (the per-actor event object teardown; called at 0x1008ee11 and 0x1008ef29) at 0x100bd6b8:
   - clears +0x12c bit 16 (op 0x92's no-name);
   - for a skeleton actor: +0x12c &= ~0x300, +0x148 = 0xffff, **FUN_10087e40** (0x10087e40..0x10087e7e: if the model is
     loaded and a skeleton, FUN_100ce160; then +0x14e = +0x150 = 0);
   - then FUN_100d44b0(2), dropping the kind-2 (event motion) files.
3. FUN_10087e40 is also op 0x7c's "mouth off" path (MODLOG L14093).

Build: on 0x23's close and on unbind (and 0x7c 0), fade the mouth slot's motions to nothing over 8 ticks instead of
dropping them. Also zero the countdowns (0x23: +0x14e only; unbind / 0x7c: +0x14e and +0x150).

## Item 5: op 0x46 sub 1's HUD window (gaps L72; old FUN_10217e90 = FUN_10221c30 here)

**The window is "menu    buff", the status icon window** (handler global DAT_106671b8, vtable 0x1033c218, OnKey
FUN_10227ab0; menu_classes.json). Op 0x46 sub 1 (FUN_100b6440 at 0x100b652f, batch 2 §1) calls
`DAT_106671b8->FUN_10221c30()`.

**Hide, FUN_10221c30** (0x10221c30..0x10221c42 → FUN_10221c90 → FUN_10221cd0):
- window (+8) **+0x69 = 0, +0x6a = 0**. In the generic window draw FUN_101177d0, +0x6a gates the handler's own draw
  (0x10117cf5) and +0x69 gates the item / part draws (0x10117ac8, 0x10117ea1). So the icons stop drawing (inf: the frame
  path is not gated by these two).
- every item disabled (FUN_1011a3d0 → FUN_101192b0(i, 0), i = 1..+0x58);
- window +0x62 = 0 (no cursor / focus);
- buff +0x5c |= 2 (suspended: changes made meanwhile are kept for later).

**Show, FUN_10221c70** (→ FUN_10221cb0 → FUN_10221ce0): +0x69 = +0x6a = 1, items enabled (FUN_1011a400 → FUN_101192e0),
+0x62 = 1. If +0x5c bit 1 was set, FUN_10221ed0 (the icon rebuild), then the bit is cleared.

**Callers:**
- Hide: op 0x46 sub 1; 0x1022abc4; 0x102300f0; 0x102301e9; 0x10233e6a; FUN_102342c0 (0x10234330).
- Show: **the event end wrapper FUN_100ae960 (0x100ae98b, right after FUN_100ae3f0 and the first-person restore)**;
  0x101f7b4a; 0x1022ae09; 0x10231b32; 0x10232cd5; 0x10232ed9; FUN_10234230 (0x10234252). The others are window
  open / close pairs (inf: map / full-screen windows that hide the buffs while up).
- Sub 0 does not show it again; only the event end (or one of those windows closing) does.

Build: op 0x46 sub 1 → hide the status icons (and drop their focus); the event end → show them and rebuild if
anything changed. This answers MODLOG L7359's "whether retail hides the buff window in cutscenes": only when the
script runs 0x46 sub 1 (or cinema mode hides everything).

## Item 6: the creator's child 3, element state 0x10 (gaps L54, MODLOG L15673)

**State 0x10 = "inert": drawn at full colour like a label, but it can never take the cursor, the mouse ignores it, and
keys are dropped while the cursor sits on it.**

- Setter: FUN_10118b60(state, childId) (0x10118b60..0x10118bad) finds the type-1 part with id +0x3a == childId and
  calls **FUN_10119970(state)** (table 0x10119a10 / 0x10119a30). State 0x10 → part **+0x10 = 0x10, +0x18 = 0x10,
  +0x28 = 0** (0x101199f7..0x10119a05). The other states: 0 normal, 1 → +0x10 1 / +0x28 2, 3, 5 (disabled), 0xb, 0xf.
- **Draw** (generic window draw FUN_101177d0, 0x10117927..0x10117a62):
  - state 0x10 → tint **0x80808080**, at the plain position (+0x2a), never the cursor highlight (0x10117936 →
    0x10117a0d);
  - compare state 5: 0x30808080 (dim), or 0x40402810 shifted +1 under the cursor.
- **Keys:** the generic key handler FUN_10118260 returns 0 at once (no SE, no OnKey) if the cursor part's state is
  0x10 (0x1011829d..0x101182a9).
- **Cursor moves:** a part with state 0x10 is skipped as a target (0x10118328 → 0x101184a0).
- **Help:** FUN_10118830 doesn't update the help line for a state-0x10 (or +0x18 == 0xf) cursor part (0x10118848).
- **Mouse:** the hit test FUN_10118cb0 never hits a state-0x10 part (0x10118d54).

In ptc6yesn the open callback FUN_10164580 sets child 3 to state 0x10 for every prompt (MODLOG L15673), so the cursor
only moves between children 1 (Yes) and 2 (No) and child 3 just shows (inf: child 3 is the prompt's text / third
cell). It is hidden outright for a5e0 == 1 / 3. Build: draw child 3 normally and make it unselectable.

## Item 7: /sitchair — trusts ignore chair / sit / heal statuses in 0x00E (gaps L110, MODLOG L18867 / L18930)

The rule is already in MODLOG (L18930); this pins the exact gate. 0x00E handler, inside the **mask @0x0A & 4** block
(0x1009bd9f), asm **0x1009c118..0x1009c16d**:
```
entity +0xec = u8 @0x1e                          // HP %
if (entity +0x120 bit 1) skip                     // status held by event op 0xab 1 / 2 (Entity.status_held, L14200)
elif (u8 @0x28 bit 0) /* TrustFlag */ && (s == 0x21 || s == 0x2f || FUN_10095790(s) /* 0x3f..0x53 chairs */): skip
else entity +0x16c = s                           // s = u8 @0x1f, the server status
```
- So a trust (0x00E @0x28 bit 0; LSB sets `|= 0x45` on trusts) never takes the server's healing (0x21), sitting (0x2f)
  or chair (0x3f..0x53) statuses. Its previous +0x16c stays. Other statuses go through.
- Ordinary NPCs, monsters and pets (bit 0 clear) take every status.
- 0x00D (PCs) has no such rule.

Build: in our 0x00E mask-4 status write, skip statuses 0x21 / 0x2f / 0x3f..=0x53 when @0x28 bit 0 is set (and skip all
while status_held).

## Item 8: dual wield — the 'b' scheduler's vtable +0x250(1) (gaps L111, MODLOG L16315)

**It sets a flag that motion schedulers can read; nothing reads it on its own, which is why no effect was seen.**
- The call is in FUN_100d6850 (the swing scheduler pick; 0x100d68e5..0x100d692e), on **every** swing:
  `if (model +0x8b5 /* dual wield */ && sched && sched name[0] == 'b') vt+0x250(1); else vt+0x250(0)`.
- vt+0x250 = **FUN_100a4770** (the same in all 7 drawobj vtables): drawobj **+0x840 bit 6 = arg**. Its getter vt+0x254 =
  FUN_100a4760 (`(+0x840 >> 6) & 1`).
- The only caller of vt+0x254 is **FUN_10062900** (0x1006297f), the scheduler's system-value reader: **value id 0x3b =
  actor 1's off-hand flag**. Neighbours: 0x3a = vt+0xa4 & 3; 0x3c = entity +0x124 bits 8..11 (& 3 unless 8). FUN_10062900
  is called from the scheduler op dispatcher FUN_10057fb0 (0x1005d2c0, 0x1005d484, 0x1005d50b, 0x1005d592,
  0x1005d619, …: the ops that test / branch on system values).
- So a scheduler in the swing's files can branch on value 0x3b ("this swing is the off-hand one") — e.g. to put a trail
  or effect on the other hand (inf: which files test it not scanned). The flag stays until the next swing resets it
  to 0.

Build: keep a per-model off-hand flag set at each swing (1 only for a dual-wield 'b' scheduler) and return it for
scheduler value 0x3b.

## Item 9: a new motion's start frame carries the old entry's phase (gaps L100, MODLOG L13932)

MODLOG's FUN_10019ed0 is an old-build address (not an instruction start here). The enqueue in this build is
**FUN_1001ad00** (thiscall on a region queue; reached from FUN_100cd540 → FUN_1001a660 / FUN_1001a6e0 → FUN_1001acb0 /
FUN_1001ad00; asm 0x1001ad00..0x1001af2e).
- Args: a1 = motion resource, a2 → entry +0x1c, a3 = rate (via vt+0x2c), a4 (byte) = play count +0x2c, a5 = blend B
  (+0x14 = B, +0x18 = 1/B; 0 → 1.0), a6 → +0x20 (< 0 → 1.0), a7 = start → +0x24, a8 → +0x38.
- It appends a 0x40-byte entry (FUN_1001a800, vtable 0x1032a1c0) after the region's last entry L (FUN_1001b1a0).

**The carry (0x1001adf9..0x1001af18), only when there was a previous entry L and both have a resource:**
```
oldLen = L.res.frames(u16 +0x34) / L.res.speed(f32 +0x36)
newLen = N.res.frames / N.res.speed
extra  = 0
pos    = 0
if oldLen > 0.001:
    k     = 1 / oldLen
    pos   = L.vt+0x30() * k                       // old entry's current position as a fraction of its length
    extra = k * newLen * N.res.speed * 0.5 * a3 * a5   // half the blend, at the new rate (0x10329a08 = 0.5)
pos = pos * newLen + extra
if newLen <= 0: pos = 0 else: pos = pos mod newLen  // subtract newLen while pos >= newLen
N +0x24 (start frame) = N.res.speed * pos           // overrides a7
```
Then **FUN_1001b520(a5) on L** starts its fade-out over B (state 0 → 2 / 1 → 3; batch 5 item 4).
- With no previous entry (empty region), or a missing resource, the start stays a7 (normally 0).
- If the region holds more than 3 entries, the older extras get FUN_1001b520(1.0) (fast fade), and the newest of those
  +0x10 = 1.0 (0x1001af2e..).

So when a status motion replaces another in a region (walk → run, idle → walk), the new one starts at the **same
fraction of its cycle** as the old one had reached, pushed forward by half the blend time at the new rate. Feet stay
in step through the blend. (inf: L.vt+0x30 returns the entry's current frame in the same units as frames / speed.)
Applies to every enqueue with a predecessor in that region: status motions (FUN_100c83c0, blend 16), scheduler op
0x05 and one-shots.

## Item 10: NPC shop list — Up at the list's top (gaps L211, MODLOG L6370)

**Retail does not wrap. Up on the top row scrolls the list up one row (SE 0001); at the very top it does nothing and
plays no sound.** Down is the mirror.
- **Layout** ("menu    shop" in 39542, `vana-client ui-dump --id 39542`): ten rows (children 1..10, 132×16 at (0, 5 + 18k))
  whose links are [up, down, …]: row 1 up = 1, row 10 down = 10; every other row links to its neighbours. So the
  generic key layer FUN_10118260 never wraps the cursor; at an end it doesn't move (no SE).
- **The list's OnKey** is the row-list base **FUN_101f6c70** (vtable 0x1033bba0 slot +0x18; gdecomp/101f6c70.c).
  Fields: +0x1e top, +0x18 visible rows, +0x20 item count, +0x22 max top, +0x24 the cursor before this key.
  - **Up (2):** if cursor == 1 and it was already 1 before the key: top -= 1. If top ≥ 0 → SE 0001 (FUN_10037b30) and
    refresh (FUN_101f66b0). If it goes below 0 → top = 0, refresh, **no SE**. Otherwise (the generic layer moved the
    cursor) → SE 0001.
  - **Down (1):** if cursor == visible rows and it was there before the key: top += 1. Above max top → top = max top,
    refresh, no SE; else SE 0001 and refresh. If the cursor row is past the item count → FUN_10118db0(count) moves it
    back to the last item, no SE. Otherwise SE 0001.
  - The "already there before the key" test means the first press that lands on the edge row only moves the hand; the
    next press scrolls.
  - 0x18 / 0x19 page by the visible count (cursor to row 1 / last row at the ends). 3 / 4 page the same way unless
    +0x47 == 0 and the 0xa8 modifier isn't held (or vt+0x6c takes them). 0x16 / 0x17 scroll by one without moving the
    hand. 0x1b jumps to the scroll-bar position.

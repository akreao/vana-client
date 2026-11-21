# MODLOG gaps, batch 6 (queue items 11..20)

Copied from Steven's `/run/media/SSD/FFXI/re-notes/modlog-gaps-batch6.md` (2026-11-03). Target and tools as in
[batch 1](2026-11-03-modlog-gaps-batch1.md) (`FFXiMain.unpacked.dll`, sha256 8097dbf8…550c). All addresses are VAs in
this DLL. **(inf)** = inferred. Queue: modlog-gaps-queue.md.

## Item 11: blue magic sets, the open order (gaps L205, MODLOG L11116)

**Confirmed: bluequip ends with the focus.** FUN_101df020 (0x101df020..0x101df116) opens, in order:
1. **persona** (`menu    persona` at 0x1036f818; first DAT_105783e4 +0x1c = 2, persona mode 2) via FUN_1015e350(name, 1, 0) (0x101df0b0);
2. **bluehelp**: DAT_106636a0 → FUN_101ec220(-999, -999, 0, 0) → FUN_1015e350("bluehelp", 1, 0) (0x101ec257 / 0x101ec2b2);
3. **bluinven**: DAT_1066369c → FUN_101ebf10(DAT_106634a8, -999, -999, 0x81) → FUN_1015e350("bluinven", 1, 0)
   (0x101ebfac), then window +0x38 = 0x81;
4. **bluepoin** (0x101df0fd), then **bluequip** (0x101df110), both FUN_1015e350(name, 1, 0).

FUN_1015e350(name, focus, c): when `focus` is non-zero (and the `c` stacking test doesn't take over), it calls
FUN_1015e530(window) (bring to front / take the focus) and FUN_1015db80(window, 0). All five pass focus = 1, so each takes
the focus in turn and **the last, bluequip, keeps it**. The draw stacking follows the same order (bluequip on top).

## Item 12: the per-id 1000 ms send refusal, FUN_100fc3c0 (gaps L46, MODLOG L16460 / L16478)

**Every c2s packet id has its own 1000 ms gate. A send that asks for the gate is refused (returns NULL, nothing is
queued) until 1000 ms have passed since that id's last gated send.**

FUN_100fc3c0(id, force, a3) (0x100fc3c0..0x100fc4c9), cdecl:
```
if !charData(DAT_104dfd98) || id >= 0x120: return NULL
if force == 0:
    last = charData +0x41b4 [id]                        // last gated send time, ms
    if last != 0 && now <= last + interval[id]:  return NULL   // wrap-safe compare (0x100fc3fa..0x100fc47c)
    charData +0x41b4 [id] = now
pkt = FUN_100dda60(charData + 0x10, id, force, a3)       // the send queue slot
if !pkt: return NULL
pkt u16 @0 = id & 0x1ff (size bits set later by FUN_100fc4d0); u16 @2 = 0
```
- now = FUN_100dc060() = `[DAT_10492e10+0xc] * 1000 + [DAT_10492e10+0x10]` (seconds × 1000 + ms).
- interval[id] = DAT_104dfd94 +0x43e34 [id]. It is set to **0x3e8 = 1000 for all 0x120 ids** at init
  (0x100f99f8..0x100f9a13), and nothing else writes it.
- force != 0 skips the gate and doesn't update the time.
- A refused send is simply dropped by its caller (e.g. FUN_100e3400 returns 0: the chat line isn't sent). No queueing,
  no retry, no message.

**Which ids are gated** (call sites `push a3; push force; push id; call 0x100fc3c0`, read across full_text.asm):
- **force = 0 (gated, 1 per second):** 0x0C 0x0F 0x11 0x17 0x1B 0x1C 0x28 0x29 0x2A 0x2B 0x2C 0x32 0x34 0x36 0x37 0x38
  0x39 0x3A 0x3D 0x41 0x51 0x52 0x53 0x58 0x59 0x5A 0x5B 0x5C 0x5D 0x60 0x61 0x63 0x64 0x66 0x6E 0x6F 0x70 0x71 0x72
  0x73 0x74 0x75 0x77 0x85 0x8C 0x8D 0x96 0x9B 0xA0 0xA1 0xA2 0xAA 0xAB 0xAC 0xAD **0xB5 0xB6** 0xB7 0xC3 0xC4 0xCD 0xCE
  0xCF 0xD0 0xD2 0xD4 0xD5 0xD8 0xDD 0xDE 0xE0 0xE2 0xE4 0xE7 0xE9 0xEA 0xEB 0xEC 0xF0 0xF1 0xF5 0xF6 0x100 0x102 0x106
  0x109 0x10A 0x10B 0x10C 0x10D 0x10F 0x110 0x112 0x113 0x114 0x115 0x117 0x118 0x119 0x11A 0x11B 0x11C 0x11D.
  Trade 0x032 / 0x033-path, bazaar 0x105 / 0x106 / 0x109 / 0x10A / 0x10B as named in MODLOG fall here (0x105's flag is
  a register; inf).
- **force != 0 (never gated):** 0x0A 0x0B 0x0D 0x1F 0x3C 0x4B 0x4D 0x4E 0x76 0x78 0xBE 0xBF 0xC0 0xC1 0xC2 0xC9 0xCA
  0xCB 0xDB 0xDC 0x101.
- **force from a register (inf, depends on the call):** 0x15 0x16 0x1A 0x35 0x3B 0x42 0x50 0x5E 0x82 0x83 0x84 0xE1
  0xF4 0xFA..0xFF 0x104 0x105 0x111 0x116; **mixed** 0x1E 0x33 0xD1 0xD3 0xE8.

Build: keep a per-id last-send time; for the gated ids drop a send within 1000 ms of the previous one.

## Item 13: scheduler op 0x89 and its head-look suppression (gaps L74, MODLOG L7984 / L4566)

**Correction: op 0x89 does not hold the actor in place. It suppresses actor 1's head-look for `dur × speed` ticks, and
ends early if the actor moves more than 1 unit in x or z.**
- Dispatch: table 0x1005dc1c[0x89 - 2] = **0x1005b14c**. If actor 1 (FUN_10062770) exists, it allocates 0x80 bytes and
  calls **FUN_1005f450(task, actor1, ftol(FUN_1005e590()))** (ticks = dur × speed, truncated). The op doesn't wait.
- **Start, FUN_1005f450:** child of the scheduler task; +0x74 = ticks (float); actor 1 / 2 = the actor. Drawobj
  **+0x840 |= 2**; it records the actor's position x → +0x78 and z → +0x7c (vt+0x1bc → [0] / [8]).
- **Update, vtable 0x1032ba34 slot +0x20 = FUN_1005f540:**
  - moved = |+0x78 − x| > 1.0 or |+0x7c − z| > 1.0 (0x1005f56a..0x1005f61e);
  - +0x74 −= frame ticks (DAT_1047bfa8 +0xeb0);
  - ends when ≤ 0 or moved.
- **End, FUN_1005f660:** drawobj +0x840 &= ~2. It is also destroyed with its scheduler task (child).
- **What bit 1 does, FUN_100d5b10** (the head-look update; asm 0x100d5b49..0x100d5bb0):
  - if +0x840 bit 0 (a forced look) is set and bit 1 is clear → +0x854 = 1.0 (keep looking);
  - otherwise the look target is the actor's target (vt+0x304 ? FUN_10157cf0 : FUN_100845e0). **While bit 1 is set the
    target is forced to none**, so +0x854 = −1.0 and the head eases back to forward.
  - (Costume / bit 5 actors: +0x854 = −1.0 as well.)
- No other reader of bit 1 (the full scan of +0x840 reads found only FUN_100d5b10 testing it).

Build: op 0x89 → for dur × speed ticks (or until the actor moves > 1 unit in x / z) actor 1 doesn't turn its head toward
its target.

## Item 14: event ground-snap skips, +0x128 bit 14 and +0x120 bit 20 (gaps L77, MODLOG L19401 / L19407 / L12720)

**The two bits are server flags: +0x120 bit 20 = GroundFlag, +0x128 bit 14 = HackMove** (LSB names, src/map/packets/
char_update.cpp: Flags0 @0x18 bit 15 GroundFlag; Flags1 @0x20 bit 27 HackMove).

**Writers** (every store found):

| Bit | Source | Code |
|---|---|---|
| +0x120 bit 20 (GroundFlag) | s2c 0x00D u32 @0x18 bit 15 | 0x1009a066..0x1009a092 |
| | s2c 0x00E u32 @0x18 bit 15 | 0x1009bc06..0x1009bc2b |
| | s2c 0x037 u32 @0x28 bit 2 | 0x1009d1b8..0x1009d1d1 |
| | zone-in, own player: char +0x40dac bit 13 | FUN_10094190, 0x10094adc..0x10094af8 |
| | local actor creation FUN_100ab750: `|= 0x100000` | 0x100abbbf |
| | cleared on actor 0 by FUN_10014390 | 0x10014947 |
| +0x128 bit 14 (HackMove) | s2c 0x00D u32 @0x20 bit 27 | 0x1009a526..0x1009a54e |
| | s2c 0x037 u32 @0x2c bit 12 | 0x1009d43c..0x1009d459 |
| | zone-in: char +0x40de4 (signed byte, bit 0) | 0x10094dfe..0x10094e21 |

(The 0x00D / 0x00E writes sit in the update-mask-4 "General" block.)

**Readers:**
- FUN_10084640 = HackMove | +0x120 bit 21 | GroundFlag; when it is true the model gets no ground tilt (FUN_100cbde0,
  MODLOG L12720).
- Event snaps: ops 0x36 / 0x37 / 0xba (FUN_100b3fb0 / FUN_100b4130) and the walks (op 0x1f, batch 2 §1) skip the ground
  snap when +0x128 bit 14, +0x120 bit 20 or bit 21 is set.

Build: parse GroundFlag (0x00D / 0x00E @0x18 bit 15, 0x037 @0x28 bit 2) and HackMove (0x00D @0x20 bit 27, 0x037 @0x2c
bit 12) into the entity and treat either like op 0x33's bit 21: no event ground snap and no tilt.
(The meaning of +0x128 bit 14 for MODLOG L12773's open question is answered too.)

## Item 15: knockback's neighbours in FUN_100a57f0: vt+0x304, +0x103, +0x104 (gaps L291, MODLOG L7210)

FUN_100a57f0 (the CXiControlActor update; gdecomp/100a57f0.c) picks **one** movement mode a frame, in this order:
**+0x103 (FUN_100a9fc0) > +0x104 (FUN_100aa1f0) > +0x102 knockback (FUN_100aab70) > +0x101 second push
(FUN_100aadf0) > normal.**
- **vt+0x304 = FUN_100a4640** (all 7 drawobj vtables): "this is the player's own control actor" (FUN_10081550 on the
  handle 0x1047d600 == this).
  - Not the player: run the chosen mode only.
  - The player with DAT_104568fc +8 == 0: the mode, then FUN_100a5ec0.
  - The player otherwise: by status (FUN_10084390):
    - 0 / 1 / 4 / 0x1c / 0x1f: the modes, else player input FUN_100a5dc0;
    - 2 / 3 (dead): nothing;
    - 7 / 0x21 / 0x2c / 0x2f / 0x30 / chairs: only the knockback;
    - others: FUN_100a5dc0 if FUN_10084350 / FUN_10084370 and no vt+0x34c / +0x358.
- **+0x103 = "close in on an actor"**, set by FUN_100aa380 → **scheduler ops 0x11 and 0x13** (table 0x1005dc1c: 0x11 →
  0x1005883b, 0x13 → 0x1005887b):
  - 0x11 moves actor 1 to actor 2; 0x13 moves actor 2 to actor 1 (dolls resolved);
  - +0x110 = the other actor, +0x108 = ticks (op u16 × task speed +0x9c), +0x10c = speed per tick, +0x104 = 0.
  - **FUN_100a9fc0**, each frame: +0x108 −= ticks. d = |target − pos| − FUN_100d4b90() (the contact distance).
    - Timer out → jump straight to the contact point, clear +0x103.
    - 0 ≤ d < DAT_10329a18, or d < 0 → stop.
    - Else step min(ticks × +0x10c, d) toward the target. (FUN_100aa4e0 gives the target's point.)
- **+0x104 = "go back to its own point"**, set by FUN_100aa3b0 → **scheduler ops 0x12 and 0x14** (0x100588bb /
  0x100588f3; actor 1 / actor 2):
  - +0x110 = itself, +0x108 = ticks, +0x10c = speed, +0x103 = 0.
  - **FUN_100aa1f0**: the same stepping toward the actor's stored position +0xc4..+0xd0 (inf: its server position),
    snapping there when the timer runs out (+0x5c4 / +0x5fc copies, +0x608 = 1.0).
- Both clear at init (FUN_100a8a29 / 0x100a8a2f). The 0x101adexx byte writes to +0x104 are a different (input) object.

So the "other movement modes" are the battle approach (0x11 / 0x13) and return (0x12 / 0x14) of skill and attack
schedulers, and they take priority over a knockback that is running.

## Item 16: s2c 0x017 coded lines, message source 9 (gaps L188, MODLOG L19870-ish "FUN_100964a0's sources")

**Source 9 = the trust (alter ego) lines: language slot 0x22 → en file 55693 (12,201 entries; e.g. 0x4 "Ajido-Marujido,
tell me the truth…"), read through a record cache.**
- FUN_100964a0(src, id) calls the table 0x1032d600[src] (src < 0xb; [0] = none). [9] = **FUN_10096440** → FUN_100bd760() (the
  lazily built singleton DAT_104899b8, ctor FUN_100bd820) → **FUN_100c1620**: `this + 0x858`, id sign-extended from u16 →
  **FUN_101c8450**, the record cache lookup (LRU: 0x1c-byte slots, counters bumped, the hit's reset to 0).
- FUN_100bd820 makes four such caches:
  - +0x788 = slot 0x1c;
  - +0x7f0 = file 0xd8b2 (55474);
  - **+0x858 = slot 0x22** (FUN_1025b580(0x22): DAT_10669464[0x22] for the client language; en and the other languages
    by FUN_1025b510);
  - +0x8c0 = slot 0x1b.
  (`vana-client dmsg-dump 34` lists slot 0x22's file.)
- An id outside 0..count returns 0 (the line is dropped by its caller, as for an empty source). (inf) A record not yet
  loaded behaves as MODLOG L15367 says for point names: the lookup returns nothing until the cache has it.

Build: add source 9 to the coded-chat source table as dat::dialog over slot 0x22 (file 55693 for English), id = the
coded number.

## Item 17: `17 n` moogle motions — S+0x40daa and +0xee (gaps L187, MODLOG L19915)

**S+0x40daa (u16 in char data DAT_104dfd98) = the index of the NPC you last talked to.** FUN_100a0080 ("the moogle
actor") takes it first when FUN_100a0010 accepts it, so with several moogle models in range the one you spoke to plays
the motion.
- Accessors: get FUN_100fc300 (0 with no char data), set FUN_100fc320.
- **Writers:**
  - **Talk:** FUN_1007bf30 (the talk / check action on a target) at 0x1007c189: after the distance test and
    FUN_100780a0 (the request is sent and returns 0), and when the player isn't dead (status not 2 / 3):
    +0x40daa = the target index ([edi]). If the target's +0x128 bit 5 is clear, DAT_10482f18 = 1 ("watch it");
    DAT_10485a70 = 0.
  - **Per frame, FUN_10096640** (0x10096bd7..0x10096c27): while DAT_10482f18 is set and +0x40daa != 0, reset it to 0 when
    that actor no longer exists, or when its +0x12c bit 30 and +0x124 bit 12 are both set; then DAT_10482f18 = 0.
  - Zone-in (s2c 0x00A handler, 0x100faec0: 0) and the event end FUN_100ae3f0 (0x100ae93c).
- **FUN_100a0010 (the moogle test, per MODLOG):** model loaded (+0x120 bit 9), a zone NPC (+0x78 0x01xxxxxx), **+0xee ∈ {1,
  2, 7}** (0x00E SubKind 1 PC-style NPC with +0x12c bit 30, SubKind 0 / costume-style, or SubKind 6; batch 5-build §2's
  table), and model +0xfc ∈ {0x52, 0x3d5, 0x58a, 0x939}.
- Fallback when +0x40daa fails the test: the lowest index 0..0x8ff that passes.

Build: remember the target index of each talk (reset it on zone-in, at the event end, and when that actor goes away),
try it first in moogle_actor, and require +0xee 1 / 2 / 7 alongside the model check.

## Item 18: sound level resets at the event end and on a volume slider change (gaps L153, MODLOG L6334)

Channels (batch 2 §1): kind 1 = UI SFX (DAT_10351700), 2 = world SFX (DAT_10351704), 4 = ambient loop (DAT_10351708),
8 = music stream. Event ops 0x69 / 0x6a drive them.

**One reset per zone, guarded by DAT_104804c3** (cleared by the zone / world init FUN_10093720 at 0x10093892). Whichever
comes first sets it to 1 and resets the levels:
- **Zone-in ready**, FUN_1008abe0 (0x1008ad4e..0x1008ad77, when the player's model is loaded):
  - ambient: FUN_10035f80(1.0) → full at once (any ambient fade task killed);
  - world SFX: FUN_10035d40(1.0, 30) → fade to full over 30 ticks;
  - UI SFX: FUN_10035d90(1.0, 30) → fade to full over 30 ticks.
- **Event end**, FUN_100ae3f0 (0x100ae8a2..0x100ae8df), **only if the flag is still 0** (an event that started before
  the zone-in reset, i.e. a zone-in cutscene):
  - FUN_10035de0(1.0, 30): ambient fade to full over 30;
  - FUN_10035d40(1.0, 30): world SFX fade to full over 30.
  - UI SFX, the stream and the `<call>` mute (DAT_10351710) are not touched here.
- So **after the zone-in reset, a normal event's end does not restore levels an event op lowered**. They stay until
  another 0x69 / 0x6a, a slider change, or the next zone (inf: scripts restore their own levels).

**Slider change:** config 9 / 10 apply (FUN_101aa650, value × 0.01 at 0x101aa663):
- FUN_10035e80(v) stores DAT_103516f8 (called at 0x101aa698, 0x1019dea6);
- FUN_10035ec0(v) stores DAT_103516fc (0x101aa6a5, 0x1019de92, 0x1007f796).
- **Both then set every channel's event level back to 1.0 at once:** FUN_10035fb0(1.0) ambient, FUN_10036050(1.0) world,
  FUN_100360a0(1.0) UI, FUN_100360f0(1.0) stream (0x10035e80..0x10035eb2 / 0x10035ec0..0x10035ef2).
  A slider move during a cutscene undoes the script's mute.

## Item 19: chat-log refusals — invisible (own +0x128 bit 6) and /linkshell (7031 0x25) (gaps L169, MODLOG L5915 / L5917 / L5927)

**+0x128 bit 6 = InvisFlag** (LSB Flags1 bit 29). Writers:

| Source | Code |
|---|---|
| s2c 0x00D u32 @0x20 bit 29 | 0x1009a5a9..0x1009a5cc |
| s2c 0x00E u32 @0x20 bit 29 | 0x1009c1f9..0x1009c21e |
| s2c 0x037 u32 @0x2c bit 15 | 0x1009d578..0x1009d58b |
| zone-in: char +0x40db2 bit 1 | FUN_10094190, 0x10094e56..0x10094e6c |

(Its PC see-through effect is FUN_100973f0, batch 3 §2.)

**Invisible refusal in talk / check, FUN_1007bf30** (0x1007c028..0x1007c077):
- It runs after the mount refusal (7 0x4b) and the model checks, **before** the range test (6.0 → 7031 9).
- If the player's +0x128 bit 6 is set: unless the quiet argument is set, FUN_100a1ab0 prints **7031 0xac "You cannot
  use that command while invisible."** (mode 0x79), with the player's id as both actors. Nothing is sent and it returns
  failure.
- MODLOG L1530 / L12364 list the same check in the other target actions (send a local 0xac instead).

**/linkshell refusal, FUN_100e3400** (batch 5 item 2):
- chat kinds 5 / 0x1b / 0x1e use linkshell 1 / 2 / 3. If FUN_100e9730(0, 0, 0, n) != 1 (no such shell equipped: s2c
  0x0E0, batch 1 §2), it prints **7031 0x25 "You do not have a linkshell item equipped."** (mode 0x7b) and returns 1
  without sending. The caller still echoes the typed line.
- Kind 4 (party) with FUN_100e8f70(5) ≤ 1 prints 7031 0x24 "There are no party members." the same way.

Build: keep InvisFlag from those four sources; gate talk / check (and the other target actions) on it with 7031 0xac.
For /l, /l2 and the third shell, gate on the 0x0E0 slots with 7031 0x25. The echo still prints in every case.

## Item 20: the chat line cut at a control byte, FUN_1018fc40 (gaps L164, MODLOG L6005)

**MODLOG's description is confirmed; this adds the exact loop.** FUN_1018fc40(s) (gdecomp/1018fc40.c):
```
n = strlen(s); out = 0; i = 0
while i < n:
    len  = FUN_1018fa80(s[i], 0)                 // the length of the character at i (multi-byte / FD block)
    rate = FUN_1018fa20(s[i], s[i+1])
    if rate == 2 || rate == 3: break             // cut here
    copy len bytes from s+i to s+out; out += len; i += len
s[out] = 0; return out
```
FUN_1018fa20(b, next) (gdecomp/1018fa20.c):
- 0xFD → 4 (an auto-translate block: kept whole);
- 0x7F → 0 (kept) when next is 0xFB or 0xFC, else 2 (cut);
- 0x1E / 0x1F → 2 (cut);
- 0xEF → 3 (cut);
- 0, 0x0A, 0x0D, or ≥ 0x20 → FUN_1011da30(b) != 0 (0 / 1, kept; inf: the Shift-JIS lead-byte test);
- any other byte < 0x20 → 2 (cut).

So the formatted line is truncated at the first 0x1E, 0x1F, 0xEF, stray 0x7F, or control byte other than NUL / LF / CR.
Bytes before it are copied whole by character. Callers apply it to every chat kind except 0x22 / 0x23 (MODLOG L6005; lines
with an icon sanitise only name and text). Implementation only.

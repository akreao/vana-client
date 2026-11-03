# MODLOG gaps, batch 4 (RE notes)

Target and tools as in [batch 1](2026-11-03-modlog-gaps-batch1.md) (retail `FFXiMain.dll`, unpacked, sha256 8097dbf8…550c). All addresses are VAs in this DLL. **(inf)** = inferred.

## 1. FUN_101cc0d0(mask, quiet): order of the mask checks

Asm 0x101cc0d0..0x101cc1ae. The first match sets esi = line id and jumps to the end:

| Order | Bit | Test | Line (group 7) |
|---|---|---|---|
| 1 | 0x10 | FUN_101cbe90() (0x101cc0d8) | 0x51 |
| 2 | 0x20 | FUN_101cbec0() (0x101cc0f2) | 0xa0 |
| 3 | 0x01 | FUN_101cbe50() or FUN_101cbe60() (0x101cc10a) | 0x4b |
| 4 | 0x02 | FUN_101cbe70() (0x101cc128) | 0x4d |
| 5 | 0x04 | FUN_101cbe80() (0x101cc13d) | 0x4e |
| **6** | **0x40** | **FUN_101cc040() = player +0x120 bit 5, costume (0x101cc152..0x101cc160)** | **0xb1** |
| 7 | 0x08 | FUN_101cc070(0) (0x101cc167..0x101cc178; Mog House) | 0x44 |
| 8 | 0x80 | mask byte sign bit, then FUN_101cc1b0() = DAT_1048098d (Mog Garden) (0x101cc17e..0x101cc189) | 0x44 |

- **So 0x40 sits between 0x04 and 0x08:** 0x10, 0x20, 0x01, 0x02, 0x04, **0x40**, 0x08, 0x80.
- On a match the function returns 0 (bl = 0). If `quiet` (arg 2, [esp+0x10]) is 0 and a line was chosen, it prints `FUN_10129e30(esi)` on DAT_10578158 (0x101cc192..0x101cc1a5). With no match it returns 1.

## 2. Gender bit writes (+0x124 bit 25): guards

**s2c 0x00E** (handler region FUN_1009b3f0 / FUN_1009b550; write at 0x1009bff3..0x1009c020):
- Guarded by the update mask **@0x0A & 4** (`test byte [esi+0xa], 4; je 0x1009c8f5` at 0x1009bd9f).
- Then `test bl, bl; jne 0x1009ce7c` (0x1009bda9). bl = actor **+0x138 bit 31** (0x1009b632..0x1009b641), the event "reserved" flag set / cleared by event op 0xac sub 2 / 3 (FUN_100bb180 at 0x100bb26e..0x100bb28a; old FUN_100b92f0). While it is set, the handler returns there: the mask-4 block and **the rest of the packet are skipped**.
  - (inf) On the despawn path (@0x0A & 0x20) bl is overwritten at 0x1009b896, but that path doesn't reach this block normally.
- No PC / NPC test between 0x1009bdb1 and 0x1009c020 (the only branches are 0x1009be4a / 0x1009be53 / 0x1009be76, which are inside the bit-20 / 21 code). **It runs for every actor 0x00E updates: NPCs and monsters too.**
  - Bit 25 = u32 @0x20 bit 15.
  - The same block also clears b23 / b20 / b21 (0x1009bf30..0x1009bf85) and sets b22 from @0x20 bit 14.

**s2c 0x00D** (FUN_10099a40 / FUN_10099ae0; write at 0x1009a37b..0x1009a3a4):
- Guarded by **@0x0A & 4** (`test byte [esi+0xa], 4; je 0x1009aaa6` at 0x1009a0ed).
- No other gate before the write (branches only at 0x1009a18a / 0x1009a1c9, inside the block).
- Bit 25 = u32 @0x20 bit 15.

**Default:** the actor init FUN_1008a350 stores `+0x124 = 0x880` (0x1008a4e1), so bit 25 = **0 (female)** until an update with mask 4 arrives. An NPC that never got mask 4 reads as female in 0x7f0b / `7F 90/91`.

## 3. Dialog expander codes 7F 90 / 7F 91: message +0xbfe / +0xc00

- The message object (0x1304 bytes) is built by **FUN_100c2e20** (asm 0x100c2e20..0x100c2ee7; thiscall):
  - arg1 = text source (copied, 0x177 dwords at +0x621);
  - arg2 = byte → +0xbfd;
  - **arg3 = actor index 1 → +0xbfe; arg4 = actor index 2 → +0xc00**, each replaced by the player's index (DAT_10485f7a) when >= 0x901 (FUN_10086fc0 = index < 0x901; 0x100c2e6b..0x100c2ebe);
  - arg5 = name string → +0xc10 (or empty);
  - also +0xc0c = DAT_104804cc, +0xc0e = DAT_10482f1a.
- All four creators FUN_100ad5e0 / FUN_100ad660 / FUN_100ad6d0 / FUN_100ad750 pass **arg3 = DAT_10482f40, arg4 = DAT_104855aa** (u16 globals). Those are set by each incoming message handler just before it prints:

  | Writer | Source | 7F 90 (actor 1) | 7F 91 (actor 2) |
  |---|---|---|---|
  | s2c 0x029 (battle message, handler 0x1009e8d0) at 0x1009ea00..0x1009ea10 | packet | u16 @0x14 (actor index) | u16 @0x16 (target index) |
  | s2c 0x02D (handler 0x1009eb10) at 0x1009eba9..0x1009ebb4 | packet | di (inf: actor index computed in the handler) | u16 @0xe |
  | s2c 0x05A (emote, handler 0x100a06e0) at 0x100a07c8..0x100a07d3 | packet | bp (inf: the emoting actor) | u16 @0xe (target) |
  | s2c 0x036 (handler 0x100a0ef0) at 0x100a0f95..0x100a0fa3 | packet | u16 @8 | u16 @8 (both the same) |
  | s2c 0x043 (handler 0x100a1070) at 0x100a115b..0x100a116a | packet | u16 @8 | u16 @8 |
  | s2c 0x02A (handler 0x100a12a0) at 0x100a13b7..0x100a13cb | packet | u16 @0x18 | u16 @0x18 |
  | s2c 0x038 (handler region 0x1009dee0) at 0x1009e0fa / 0x1009e101 | actors | taken from actors' +0x74 / +0x78 (inf: the two actors in the packet) | |
  | FUN_100a1860 (28 callers: local / system lines) at 0x100a18ae / 0x100a18b4 | — | the result of FUN_100a0080() | same value (inf: "the current message actor"; 0 → the line is skipped at 0x100a18a1) |
  | 0x100a1c76 / 0x100a1c85 (inside the 0x03B-area handlers, inf) | | | |

- **So for a battle message (0x029), an action line (0x02D) or an emote (0x05A), 7F 90 = the acting entity and 7F 91 = the target. For an NPC / special message (0x036 / 0x043 / 0x02A) both are the speaking entity.** An index >= 0x901 (none) falls back to the player.
- The expander: `7F 90` (0x100c19cf..0x100c1a2e) and `7F 91` (0x100c1a38..) look up DAT_10480af0[index]. A missing actor gives alternative 0 for 7F 91. Gender bit 25 picks the `[a/b]` alternative (male first).

## 4. The costume id (+0x166)

- **s2c 0x037 @0x44 is u16** (`mov ax, word [edi+0x44]` at 0x1009d694; compared and stored as a word by FUN_1008cce0). 0x00D @0x30 is u16 as well (0x10099cd6).
- **Model lookup:** the costume sets +0xee = 2 (FUN_10095e90). Case 2 of the model loader FUN_100d2f60 (0x100d3fc7..0x100d4001):
  1. if entity +0x130 bit 21 (FUN_10083340): FUN_100c51c0(**2**, look slot 0 (+0xfc));
  2. else if in costume (+0x120 bit 5, FUN_10084670): FUN_100c51c0(**0**, **u16 +0x166**) (FUN_10084690);
  3. else FUN_100c51c0(0, slot 0).
- **FUN_100c51c0 type 0 is the NPC model table**, the same one a 0x00E monster / NPC model id uses: id < 1500 (0x5dc) → file 1300 + id; < 3000 (0xbb8) → 50295 + id; < 3500 (0xdac) → 96907 + id; else 98239 + id (thresholds at 0x100c5212..0x100c523f).
- **So the costume id is an NPC model id** (the same space as 0x00E's 12-bit model), loaded as the whole body.

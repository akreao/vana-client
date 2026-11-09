# MODLOG gaps, batch 1 (RE notes)

Copied from Steven's `/run/media/SSD/FFXI/re-notes/modlog-gaps-batch1.md` (corrected 2026-11-03). Questions came
from [../modlog-gaps.md](../modlog-gaps.md).

Target: `~/ffxi-pad-re/FFXiMain.unpacked.dll` (image base 0x10000000). Sources: `all_decomp.c`, `full_text.asm`,
headless Ghidra, Rizin. All addresses are VAs. **(inf)** = inferred, not read in code. Key-event numbers are the
menu OnKey events (1 Down, 2 Up, 5 Enter, 6 Escape, 9 mouse).

## Build check, and old-build addresses

- `FFXiMain.unpacked.dll` sha256 8097dbf8a79eb35840d350dcc33718aea2fd71afac686d4d95c5f93e4695550c (md5
  42375a87b308f9d07fc9f86ebf63a33d), unpacked from the game's `FFXiMain.dll` (sha256
  042884888eea4fadac183e520f2ae9f8c4fa267860e685048e1880efd358e872, linker time 2026-10-05 10:13:26 UTC). The
  version resource only says "3, 0, 0, 0"; the string "30261006_0" is not in the DLL (it is `client_version` in
  servertest/client/config.json).
- vana-client's FUN_10094190 (zone-in), FUN_100ae3f0 (event end) and FUN_1008abe0 match this DLL.
- **Many MODLOG / vana-client addresses are from an older build.** Only 40 of the 219 handler addresses in
  `dat/event_ops.rs` match this DLL's table 0x100bc960; the other 179 are shifted by +0x1a40 (0x100ae2xx..0x100af7xx),
  +0x1910..0x1c90 (0x100b0xxx..0x100b25xx), +0x1b00..0x1e40 (scheduler stubs), +0x1ca0 (0x100b2fxx..0x100b47xx),
  +0x1dc0, +0x1e30, +0x1e90 (0x100b5dxx..0x100ba1d0). Whether an address matches follows how old the note is.

| MODLOG name | This build | How matched |
|---|---|---|
| FUN_101e8700 (item name colour) | FUN_101f06e0 (0x101f06e0..0x101f080b) | the only code returning 0x80407090 + 0x80509050 + 0x80909040 in that order |
| FUN_100fbc70 (the blue test) | FUN_100febe0 | called from 0x101f0872 / 0x101cc3cf with (item id, extdata) |
| FUN_101671b0 / 101679c0 / 1017ab10 (sweep) | not instruction starts here; the +4 vs +0xbc skip lives in FUN_101693b0, FUN_10169b40, FUN_1016a990, FUN_1016a2d0, FUN_1016b1d0, FUN_1016e930 | by the skip pattern |
| FUN_101db220, FUN_101cc040, FUN_10202d80, FUN_10203570 / 630, FUN_1019b030, FUN_10218340 | same addresses | content matches |
| FUN_102075f0 (yes / no box) | mid-function here (inside FUN_10207510); the prompt is FUN_10210e30 | see 5 |

## 1. The blue item name test (MODLOG FUN_100fbc70 = FUN_100febe0)

FUN_100febe0(itemId, ext) returns `ext != NULL && ext[0] == 1 && (u16 @ ext+2) bit 14` = extdata type byte 1 and
byte 3 bit 0x40. The item id is not read.

- Type 1 is the enchanted / charged item layout: siblings read ext+4 (FUN_100fecd0) and ext+8 (FUN_100fed20), fed
  as times to the charge / reuse line (0x1013bc58 / 0x1013bcba -> FUN_10103a50, `%d/%d` at 0x1013bd3a). Windower's
  extdata.lua calls the bit `usable` (inf, name only).
- Siblings on the same u16: bit 13 (FUN_100feb30, 1 when not type 1), bit 15 (FUN_100fec30), bit 11 (FUN_100fec80).
  FUN_100fec00 adds FUN_100fea90 (item type 4 / 5, excluding ids 0, 0xffff, 0x200..0x203, 0x108c, 0x108d, 0x1525,
  0x1526); the colour code calls FUN_100febe0 directly.
- Item list colour FUN_101f06e0(item, def), first match:
  1. FUN_101cc3e0() (player +0x132 bit 0) -> 0x80404040 grey.
  2. FUN_1015b6f0(def) (equipment) and FUN_101cc3a0(item, 0) -> 0x80407090 blue. FUN_101cc3a0 = FUN_10131f10(this =
     DAT_105781a8, item+2 slot, bag 0) (equipped) and FUN_100febe0(item+0, item+0x10).
  3. FUN_10131f10(slot, 0) -> 0x80509050 green.
  4. Linkshell 1 / 2 equipped from this slot of this list's bag (FUN_100e9730, bag == DAT_10663780) -> green.
  5. !FUN_1015b6c0(def) or type (def+0x14b4) 8, 0x16, 0x17 -> 0x80909040 yellow; else 0x80808080.
- FUN_101f0810 (vtable slot 0x1033a460): an equipped item with max charges (def +0x151c) > 0 is 0x80407090 when
  FUN_100febe0 passes, else 0x80404040 (0x101f085f..0x101f088a) (inf: the server sets the bit when the delay is over).

## 2. Storage row colour FUN_101db220 and the equipped linkshell

In this build FUN_101db220 makes an equipped linkshell row **grey 0x80404040** (it cannot move), not green. The green
rule is in the item lists (FUN_101f06e0 rule 4, FUN_101f08b0).

- FUN_100e9730(n): character data DAT_104dfd98, `slot[n] = byte +0x9808+n`, `bag[n] = byte +0x980f+n`, n = 1 / 2.
  Valid only if slot[n] != 0 and the item there exists (u16 id at +0x9868 + (bag*0x51 + slot)*0x2c != 0). Returns
  slot, bag and the extdata pointer (+0x9878 + idx*0x2c).
- Only the s2c 0x0E0 handler FUN_100ea780 (registered at 0x100e89c5) writes them: @4 n, @5 slot, @6 bag
  (0x100ea78a..0x100ea7ac). If @5 != 0 and state +0x9818+n*4 is 0 / 1 it calls FUN_100eaab0(n, bag, slot, 1|2), then
  state = 2.
- The compare in FUN_101db220 (0x101db46a..0x101db4e6): `slot == row+2` and `bag == the row's container`, n = 1 then 2.

Inputs: `src` = the row's container, `dst` = the other side. Feature bits = `*(PTR_DAT_103601bc)+0xb4`. Rules
(asm 0x101db220..0x101db639), first match wins:

| # | Rule | Colour |
|---|---|---|
| 1 | side 1 and dst == 0x11 (the bin's inventory side) | grey 0x80404040 |
| 2 | FUN_1021cc60(destination list) < 0 (no free slot) | grey |
| 3 | src == 0x11 (bin): row null, id 0, or Rare (def +0x14bc bit 15) and FUN_101dbad0(id) | grey |
| 4 | src or dst 0xb..0x10 (Mog Wardrobe 3..8) without feature bit (container - 9) | grey |
| 5 | src 1 / 9, type 10 / 12 / 14, row +0x11 bit 0x40 (installed furniture) | 0x80404040 if (row+0x11 & 3) == FUN_100f1080() else 0x80602020 |
| 6 | src 0: equipped (FUN_10131f10(slot, 0)) | grey |
| 6b | src 2: worn by a mannequin (FUN_101334c0(slot, -1, 1) or (…, 9)) | grey |
| 6c | src 8 / 10..16: equipped from there (FUN_10131f10(slot, src)) | grey |
| 7 | src 0 / 5 / 6 / 7 and dst not a wardrobe: equipped linkshell or linkshell 2 | grey |
| 8 | dst 8 / 10..16: def +0x14d0 (level) or +0x14d8 (races) null or 0 | grey |
| 9 | dst 2 and type 10 / 12 / 14 | grey |
| 10 | not at the moogle (DAT_10663468+0x70 == 0) and src or dst in 1 / 2 / 4 / 9 | grey |
| 11 | the picked row (this+4 set and this+5 == row slot) | orange 0x80804020 |
| 12 | else | white 0x80808080 |

(Switch on src at 0x101db42d, tables 0x101db63c / 0x101db64c.) The Manual-sort colour FUN_101dbdb0 has no linkshell rule.

## 3. Collision "object id skip" (+4 vs +0xbc)

The compared id is the collision map's own +4, i.e. world object DAT_10631f24 +0x17c (the map is the member at
+0x178): **the sub-area currently loaded** (-1 when none). Entries whose object record +0xbc equals it are skipped.
No caller passes an id.

- Skip sites (`mov eax,[edi+4]; cmp eax,[esi+0xbc]; je`): FUN_101693b0 0x101695d8, FUN_10169b40 0x10169d68,
  FUN_1016a990 0x1016abb8, FUN_1016a2d0 0x1016a519, FUN_1016b1d0 0x1016b3be, FUN_1016e930 0x1016edac / 0x1016ee61
  (inf), the debug draw 0x10168470 (0x1016863d).
- Writers of +0x17c: FUN_10178900(id, flag) 0x101789e6 and FUN_101787c0 0x101788a4 (sub-area number +0x48; its MZB is
  dat id + 100, or + 0x144f7 for >= 600). Reset to -1: 0x10177e4f, 0x1017892e, FUN_10178a20 0x10178a67, FUN_1017b1e0.
- Source: s2c 0x00A @0x60 (u16; stored at char +0x40d7e, handler FUN_100facf0) -> FUN_1018bac0 -> FUN_10178900(n, 0).
  Event FUN_100b8660 loads one (FUN_1018baa0, flag 1); FUN_1009ee60 unloads (FUN_1018ba80).
- The same id drives model visibility: FUN_1017a630 (+0xbc == +0x17c -> 1, +0xb8 != 0 -> 0, else 2); FUN_1017a590
  marks id 999 objects.
- vana-client's `dat/collision.rs` (`Collision::merged`, `link`) already does this.

## 4. Party-list cursor, the fellow row (index -1)

FUN_10202d80(cb, cb2, win, f0, f1, f2, mode): modes 2 / 3 set low bound DAT_106643f4 = -1 and the row count 6 /
0x12; other modes 0 and 6.

- Start row: the target's row (FUN_10203540); else -1 when the low bound is -1, a fellow exists (FUN_101cc550 = actor
  DAT_10480af0[player +0x2a0]) and the target is the fellow (fellow +0x78 == target id); else 0.
- Keys (OnKey FUN_102036f0, table 0x102038f8): Down -> FUN_10203570, Up -> FUN_10203630, with (cursor, low, count,
  FUN_10202c20() = cb2 != 0). Past the last row: -1 when a fellow exists and cb2 == 0, else wrap to 0 skipping empty
  rows; from row 0 Up likewise. From -1: Down to the first used row, Up wraps to the last. The cycle is ... last,
  fellow, 0, .... The second stage of a two-step pick (DAT_106643fc != 0) uses FUN_102035c0 / FUN_10203680, no
  fellow. The mouse can't pick -1. SE 0x10037b30 when the index changes.
- Skipped when: no fellow; cb2 set; mode not 2 / 3.
- Drawing: the party window (vtable 0x1033b078, draw FUN_102041d0) draws the pet row (FUN_10202960) and above it the
  fellow row (FUN_10202a40: name at fellow +0x7c, or +0x214 when +0x210 != 0, font 0x66; HP bar when fellow +0xa0).
  Row y = window top - 0x17, minus 0x10 more with a pet row (0x102044d9..0x10204543). Nothing when no fellow,
  FUN_10087070(3), or cb2; the block is skipped while FUN_1015ed10(8, 0, 0). Then FUN_10202110(0, x, y) (0x10204597).
- The hand: FUN_10202110, cursor < 0 and x >= 0: clears the row cursor (FUN_10118db0(0, 0)) and draws the hand with
  FUN_102022b0(x, y) at the fellow row, tinted by FUN_101505a0 (0xe0d04040 / 0xe0d0d040 / 0xa04040d0). Each window's
  vtbl+0x48 hides its own hand outside its 0..5 / 6..11 / 12..17 range, so -1 hides all three.
- Enter on -1 (FUN_10202fe0): `cb(&{u32 fellow +0x74 index, u32 fellow +0x78 id}, name fellow +0x7c or +0x214)`;
  party rows pass {u16 row +4, u32 row id}, name DAT_10663ade + i*0x7c. Mode 1 refuses (FUN_10037c50) unless the id
  is not ours and in our party (FUN_102031f0(id, 1, 0)).

## 5. conf6win "reset keyboard assignment"

Retail opens **"menu    comyn"** (FUN_10210e30 on DAT_10664520), not "yesno".
- conf6win OnKey FUN_1019b200, Enter on child 10 (0x1019b309..0x1019b339): text = FUN_10152ef0(0xb, 0x8f), then
  `FUN_10210e30(cb 0x10196f30, 0, text, p5 0, p6 0, p7 1, p8 0)`.
- Default No (p5 == 0 -> FUN_10118e90(2), 0x10210ec8); child 1 Yes, 2 No. p7 = 1 keeps Escape on (cb(user, 0)).
- Position: comyn's own layout (frame 252 x 88 at (130, 208), stacked on the log by flag 0x1000).
- Yes (LAB_10196f30): k2assign (DAT_10662774) vtbl+0x48 = FUN_1019ce70, then k1assign (DAT_10662770) vtbl+0x48 =
  FUN_1019d1c0: each resets its DIK table (+0x29..+0x48), with a language-2 variant.

## 6. Help Desk faqmain language row

The state exists, but in this build **the row is never drawn and cannot be selected**.
- Area code: s2c 0x0B4 @0x12 -> char +0x2cff4 (FUN_100e4f20).
- faqmain (DAT_106646a0, OnKey FUN_1021a240): +0x18 row index, +0x1c committed choice, +0x1e edited choice, +0x20
  edit mode; DAT_1066469c page default. Options (table 0x10385cd8): 0 English, 1 French, 2 German.
- Default (0x1021910c): only with page +0x2bc and area code 2, from the client language (DAT_10456a28 +0xc).
- +0x18 is only ever written 0 (0x10217af0, 0x10219112, 0x1021a01e), the one FUN_10218340 call with p7 = 1
  (0x10219d21) is for another row, and nothing reads the names: dormant (inf: left from a build that drew it).
- Edit mode (table 0x1021ad18): 3 / 4 next / previous, 5 commits, 6 leaves and writes `GENERIC.SELLANG` locally
  (FUN_1010d5e0). No packet. Execute (FUN_102197f0) works SELLANG out itself and ignores the row.

## 7. Trade gate bit 0x40 (costume)

FUN_101cc0d0's bit 0x40 tests FUN_101cc040 = player actor +0x120 bit 5, "in costume" (refusal group 7 0xb1). Asm
0x101cc040..0x101cc067, 0x101cc152..0x101cc160.
- Only FUN_10095e90(actor) (the per-frame costume state machine, asm 0x10095e90..0x10095fd6) sets / clears it:
  - bit 4 = "costume changed, apply". If set: costume id +0x166 == 0 -> clear bits 4 and 5, +0x128 bit 30, +0xee =
    0. Else if allowed -> clear bit 4, set bit 5, +0x128 |= 0x40000000, +0xee = 2, after FUN_10092900 (inf: model
    rebuild).
  - bit 4 clear, bit 5 set and not allowed -> clear bit 5, set bit 4, +0xee = 0.
  - Not allowed: bit 5 set and status (+0x170) 2, 3, 7, 0x21, 0x2c, 0x2f, 0x30 or 0x3f..0x53 (table 0x10095fe0); or
    bit 5 set and any of FUN_100956c0 / 10095680 / 100956a0 / 10095790 (0xff); or +0x120 bit 7.
- Bit 4 and the id come from FUN_1008cce0(id) (if +0x166 != id, store it, set bit 4). Callers: s2c 0x00D @0x30
  (FUN_10099ae0, 0x10099cd6..0x10099ceb, @0x0A bit 0x20 clear), s2c 0x037 @0x44 (FUN_1009d0f0, 0x1009d694..0x1009d6a4),
  FUN_1008cd10 (id 0).

## 8. Event op 0xb1 sub 4

The first answer read the wrong table slot (entry 0xb9); corrected here.

**Sub 3 / 4 have no length: the handler does not advance for subs >= 3, so an event reaching them stalls.**
`dat/event_ops.rs` is right on behaviour; its FUN_100b9be0 is the old build's.
- Dispatcher 0x100bc280: `movsx eax, word [esp+4]; cmp eax, 0xda; ja 0x100bc957; jmp [eax*4 + 0x100bc960]`.
- Entry 0xb1 at 0x100bcc24: `17 c8 0b 10` = 0x100bc817 -> `call 0x100bba70; ret 4`.
- FUN_100bba70: sub 0 `FUN_100af310(2, (cfg +0x18 >> 1) & 3)`, +4; sub 1 `(+0x18 & 1)`, +4; sub 2 `1`, +4; else return
  without advancing.

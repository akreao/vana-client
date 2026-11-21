# MODLOG gaps, batch 10 (queue items 51..60)

Target and tools as in modlog-gaps-batch1.md (`~/ffxi-pad-re/FFXiMain.unpacked.dll`, sha256 8097dbf8…550c). All
addresses are VAs in this DLL. **(inf)** = inferred. Queue: modlog-gaps-queue.md.

## Item 51: Adventuring Primer — the 8 empty bars for 10 frames; a drag's release; /primer while up; guide01's arrow sound (gaps L246, MODLOG L15290)

**1. Yes, retail shows the frame with 8 empty row bars and no hand for the first 10 frames.**
- guide00's frame hook (vt+0x28 = **FUN_10233cf0**): while the open counter +0x8c < 10 it only counts; on frame 0 it
  clears the paged list (FUN_10221a30 → FUN_10221900(0, 0, 0, 0): count 0) and sets the row callback +0x1c =
  LAB_10233dc0. From frame 10 it builds the rows when the category count (FUN_10234010) differs from +0x66
  (FUN_10233ef0).
- The paged-list draw FUN_10221f50 (guide00 vt+0x10 jumps to it): with count 0 the hand is hidden (+0x62 = 0) and
  only vt+0x50 runs; the row callback draws nothing. But the eight layout children (rows 1..8, sprite "menu    quest"
  0, the 320 × 18 bars) are ordinary window parts drawn by the generic window draw in state 1, and nothing disables
  them during the wait — so the bars show, empty, until frame 10 *(inf: from the parts' state; not compared with
  footage)*.

**2. A drag's release:** the generic left-release FUN_1015de50 sends Enter (msg 5) only when the pointer is in window
mode (mouse +0x4d == 2), mouse **+0x86 == 0** (no list / bar drag in progress) and the release lands on the focused
window's cursor child. A drag that the mouse state machine took as a scroll (FUN_10125360 state 2 → 3 with +0x86 = 1,
when FUN_102347f0 reports a scroll area) therefore never flips the page; a plain press-move-release that stays on the
same child does *(inf: whether the page view's map-drag sets +0x86 was not traced)*.

**3. /primer while it is up** (/primer → 0x1007fcb0 → FUN_10233b80 → `FUN_1015e350("menu    guide00", 1, 0)`). For an
already-open, visible window FUN_1015e350 restores its cursor (FUN_10118870), **re-runs its open hook vt+4 and vt+0x14**,
then FUN_1015db80 (raise + focus next frame). guide00's open hook **FUN_10233c50** then:
- re-tests the main menu (DAT_1057823c) and social page (DAT_10578468): they are already closed, so **+0x64 / +0x65
  become 0 and neither is reopened when the Primer closes** *(inf: a retail quirk; first open sets them)*;
- clears help slot 2, zeroes +0x68..+0x87, recounts the categories, +0x88 = 0 and **+0x8c = 0** → the 10-frame empty
  wait and the row rebuild happen again;
- vt+0x14 (FUN_10221880 → FUN_10221ed0) re-places the hand.
- With guide01 up, guide00 is still a visible window (guide01 sits over it), so /primer raises guide00 above guide01 and
  gives it the focus *(inf: both are layer 2)*. guide01 is not closed.

**4. guide01's arrows (children 9 / 10):** the paged-list base open FUN_10221ef0 clears only +0x85 (move SE), so the
generic layer still plays **SE 0002** on Enter (+0x86); guide01's OnKey **FUN_10234370** case 5 then re-sends child 9 as
Left (4) and 10 as Right (3): without the 0xa8 modifier → **SE 0001** (FUN_10037b30) and the previous / next category
(FUN_102342a0 / FUN_10234280); with it → FUN_10222150(0x18 / 0x19), a page. So a click on an arrow plays 0002 then
0001 in the same frame; Enter on a topic row plays the generic 0002 and opens the pages (FUN_102342c0); nothing while
+0x68 (a page is up). Hover over an arrow disables the list (FUN_10221cd0), over a row re-enables it (item 25).

Build: draw guide00's 8 bars empty (no hand) for 10 frames after every open; let /primer while up restart that wait,
raise guide00 and forget to reopen the main menu; arrow clicks sound 0002 + 0001.

## Item 52: Help Desk faqsub — the page header against row 0; the hand's frame rate; the keyword box's field frame (gaps L244, MODLOG L10010 / L10025)

**1. The hand: 6 frames of anc_s, each held for 4 draws, a 24-draw cycle.** The marker draw FUN_102167d0 (asm
0x10216893..0x102168c7): `frame = (counter +0x2c >> 2) % 6` picks the anc_s frame drawn at the row; then
`counter = (counter + 1) % 24`. The counter is advanced once per hand draw (the cursor row only), i.e. once per drawn
frame while faqsub is visible; it does not run while hidden. Other rows draw framesus 35 at (x, y + 4) (bullet) or, without
the sprite set, the text markers ">>" / "--" (FUN_102173e0 with 0x10353dfc / 0x103718b8).

**2. The header against row 0** (the menu page builder FUN_102136d0): the header entry (+0xc) is placed at the content
origin (+0x18a, +0x18c) with **height +0x1a = 0**, and each following entry at `previous y + previous height (+0x16 +
+0x1a) + the gap +0x174 (10)`, x = the indent +0x170 (0x14). So on the menu page row 0 (Q&A, +0x10) sits 10 px below the
header's top, and the header contributes no height of its own *(inf: on the result / article pages the header's
height is filled in when its text is measured — FUN_10214980 — so row 0 follows its text there; not traced further)*.

**3. The keyword box** (+0x108) is laid out right after the Q&A row in the same pass: x = 2 × indent (0x28), y = the
Q&A row's y + its height + 10, width = content width +0x192 − 2 × indent − right pad +0x172; its bit 1 (active) is set
only while the cursor is on Q&A and the box is the current input. FUN_102136d0 positions the field but draws no frame
for it, and no frame sprite reference was found next to the box in the faqsub draws *(inf: the field frame, if any,
is the text input's own (FUN_101c5be0 mode 8) — not traced; compare with footage)*.

Build: animate the hand as (counter >> 2) % 6 with a 24-step counter advanced per drawn frame; on the menu page put row 0
10 px below the header's top; place the keyword box at 2 × indent under Q&A. Field frame: leave as is until footage.

## Item 53: NPC shops — DAT_1057876c +0xc4; English width 274; the sell list (gaps L212, MODLOG L6349 / L6373)

**1. DAT_1057876c +0xc4 = the pending target-selection callback** (the sub-target prompt's "what to do with the
pick"). Set by FUN_10159440(…, callback) (0x101594bc), the prompt starter used by commands with a `<st…>` target;
called and cleared by FUN_10158db0 (0x10158e0a: `+0xc4(target, entity)`, then +0xc4 / +200 / +0xcc / +0xd0 = 0) and
cleared by FUN_10158ff0 / 0x1015900e / 0x10159510 (cancel). It is the same "+0xc4 == 0 (no prompt)" the target lock
gate tests (batch 7 item 27). So s2c 0x03E's refusal (LAB_10210ca0: group 7 0x45) fires while a target prompt is
pending (+0xc4) or sub-target mode is on (+0x59).

**2. The width: 256 + 18 = 274 for every non-Japanese client language.** The sources' shared width hook
**FUN_1020f6c0** (source vt+0x18; the buy and sell sources and the bazaar's price source call it):
- modes 1 / 2 (+0xd) add the source's +0xe offset to the x (and +0xf more in mode 2);
- then, whenever the client language (DAT_10456a28 +8) is not 0 (Japanese) — the +0x18 sign / language 2 / 3 test
  in front only leads to the same branch — **width += 0x12** and the column table's 4th entry is rewritten in place:
  `cols[3] = cols[2] + 0x4e` (DAT_10385b82 = DAT_10385b80 + 0x4e; cols {0, 0x12, 0xb4, …} → 0xb4 + 0x4e = 0x102).
  So English: a 274-px list with the 4th column at 258.

**3. The sell list's rows and tint** (sell source vt 0x1033bafc: +0x1c rows **FUN_1020ffd0**, +0x20 tint
**FUN_1020ff40**, +0x24 Enter FUN_102101a0, +0x28 Escape FUN_101ddfd0, shared with the bazaar's source):
- Rows come from the inventory list object **DAT_10666e2c** (the inventory window's own view): nothing until the
  inventory is settled (S+0x3c35c & 0x200000 or & 0x800), then its count byte (+0xc → +0x18); empty → FUN_101f7020(0, 0,
  1). So the order is that view's order (as MODLOG inferred) *(inf: the row loop past 0x10210025 not read)*.
- Tint: ids 0x201..0x203 (linkshell items), lock status 0x19 (bazaar), equipped (FUN_10131f10(slot, 0)) or NoSale
  (record flags & 0x1000) → grey 0x80404040; the slot being priced / sold (+0x10 set and +0x11 == slot) → orange
  0x80804020; else white.

Not settled in this pass: the focus return to shopmain and the buy-row tint helpers FUN_1022b680 / FUN_1022b570 /
FUN_10223360 / FUN_102092b0 — left for a later item. (The list's Escape sound: item 54.)

Build: refuse the shop while a target prompt is pending; widen the shop list to 274 with the 4th column at 258 for
non-Japanese clients; sell rows in the inventory view's order with the tint above.

## Item 54: Bazaar View Wares — FUN_10210880's flag; the Escape sound; shopbuy's registry flags (gaps L218, MODLOG L16503)

**1. FUN_10210880's flag is "the list has been resized for this source", not "the list is live".** FUN_10210880 is the
shop list's per-frame refresh (asm 0x10210880..0x102109be): it reads the slot's item (FUN_100eb490), sets the help
line (source vt+0x14), rebuilds the rows (source vt+0x1c), refreshes item info (vt+0xc), then asks the source's width
hook (vt+0x18; item 53) for the extra width w. Then:
- if the window is missing or w == 0: resize only while **source +0xc == 0**;
- else: resize unless the window's current width already equals `cols[3] (DAT_10385b88) + w` and +0xc is set.
- The resize (0x1021095c..0x102109b4): the tax cell's end `DAT_10385b84 = cols[2 of the widened table] + w (+8 in
  mode 1, else 0)`, FUN_1011a5a0 re-lays the window, 4 columns (+0x16 = 4), the column table 0x10385b7c, and **source
  +0xc = 1**. The source's open clears it, so the widening happens once per open, again only if the width changes.

**2. The Escape sound is the generic layer's SE 0003.** The list base's open (FUN_101f68b0) clears +0x85 / +0x86 but
not +0x84, and the shop list's registry flags (0x00030002) have bit 1, so the generic key layer plays 0003 and passes
Escape on (item 32). The bazaar source's Escape hook FUN_101ddfd0 is just `return 1` (close); the list base itself
plays no Escape sound. (Its Enter: vt+0x5c(row) → SE 0002 accepted / 0004 refused, FUN_101f6c70's tail.)

**3. shopbuy's registry flags:** shopbuy / shopsell are flags **0x2** (MODLOG L6373: DAT_10664510 / 14, "flags 2"):
bit 1 only — no 0x1000, so retail does **not** stack the confirm on the log window (0x1000 is the log-stacking flag,
FUN_1015f790 / MODLOG L8360) and no 0x20000. Their layout puts them at (16, 256) directly.

Not settled: which windows the group walk hides while the list is up (needs the list's group byte against each open
window; left with the group-walk item).

Build: widen the wares list once per open (re-widen only on a width change); Escape plays 0003 from the generic layer;
place shopbuy at its layout position, not on the log.

## Item 55: Linkshell — the Escape path link2.. → link5; c2s 0x0E4; /lsmes's +0x124 bit 27 (gaps L233, MODLOG L9444 / L9586 / L9592)

**1. Escape on link2 / link3 / link4 is the generic close, and link5 simply takes the focus back.** Registry: link2
(entry 127) **A 0x00210000, B 0x00810201** (bit 1 of A clear; group 0x81), link5 (entry 121) A 0x00030000, B 0x225.
- Generic layer (item 32): Escape → SE 0003 (+0x84) and, with A bit 1 clear, **FUN_1015e790(link2, 1) closes it and
  returns without calling link2's OnKey**.
- link5 was never closed (link2.. open beside it; link5's group 0 isn't suspended by group 0x81), so at the end of the
  frame the manager's focus pick FUN_1015e7e0 gives the focus to the top remaining window — link5 (link6 has A bit 0,
  skipped) — through FUN_1015e850, which re-shows link5's hand and row help (list base vt+0x24, item 31). link5's cursor
  row is unchanged *(inf: link6 / the main menu order not traced, but link6 cannot take the focus)*.
- So "back to link5" needs no code of its own: close link2.., SE 0003, focus to link5 with its cursor kept.

**2. c2s 0x0E4** (FUN_100eabc0; `/lsmes level` with no argument): @4 = 0xb0, @0xa = ++S+0x9852 (15 bits, never 0) — a
request for the current access level. The client expects the answer as s2c 0x0CC op 6 (or 3), printed as the level
line 9 0xbc / 0xbd / 0xbe from writeLevel (MODLOG's 0x0CC table) *(inf: retail answers it; LSB only logs 0x0E4, so on
LSB the command prints nothing)*.

**3. /lsmes's +0x124 bit 27 = the "in a linkshell" flag** (the name plate's LinkShellFlag 0x92, MODLOG L10690).
/lsmes (FUN_100805f0) refuses with **7 0x71 "No linkshell items equipped."** unless **both** LS1 is worn
(FUN_100e9730(…, 1)) **and** the player entity's +0x124 bit 27 is set (0x10080629..0x10080634). Writers of +0x124 bit
27:
- s2c **0x00D u32 @0x20 bit 17** (0x1009a3d5..0x1009a401);
- s2c **0x037 u32 @0x28 bit 25** (0x1009d341..0x1009d35d);
- zone-in: S+0x40db1 bit 0 (0x10094ccc..0x10094ce5).
So with the shell worn but the server's flag clear (e.g. right after equipping, before the next 0x037), /lsmes still
says 7 0x71.

Build: Escape on link2..4 = generic close (0003) and focus back to link5 unchanged; /lsmes needs LS1 worn and the
player's LinkShellFlag (0x037 @0x28 bit 25 / 0x00D @0x20 bit 17); "/lsmes level" sends 0x0E4 and waits for 0x0CC op 6.

## Item 56: automaton customization — flow B's double SE 0002; SE 0004 under the pointer (gaps L204, MODLOG L16900)

**1. Flow B's two SE 0002 are two voices started in the same frame, so they sound as one louder 0002, not as two.**
SE 0002 (FUN_10037b90 → FUN_10036f00(0x40, …) → FUN_10036f50 → FUN_10038330) has no de-duplication: the only
"once per frame" latch in FUN_10038330 is for sound ids 0x8bf / 0x80f (DAT_104638f4); otherwise each call takes a new
voice (FUN_10246050) while fewer than the cap (DAT_10456a94, 0x14 in some states) are playing. Both plays start at the
same time with the same sample *(inf: identical, phase-aligned → one sound, ≈ +6 dB)*.

**2. Yes — SE 0004 repeats every frame while something is picked and the pointer rests on the list.** automato's frame
(FUN_10207a40, 0x10207b9f..0x10207bf6): when bluinven exists, the pointer is in window mode (FUN_101262e0) and inside
bluinven's rect (FUN_10118c10), automato sends itself **msg 8 every frame**. Its messages (FUN_10207e80 case 8): with
nothing picked (+0x14 < 0 and the list source +0x10 < 0) → focus to bluinven (FUN_1015db80) + SE 0013 — which ends the
hover once the focus has moved; **with a pick → SE 0004**, and since the focus doesn't move, the same happens on the next
frame: a buzz every frame (30 a second) for as long as the pointer stays there *(inf: no focus or repeat guard found
in that block)*.

Not settled in this pass: how unfocused automato draws its cursor while a slot is picked, and pupsibor's starting
cursor (registry 0, no 0x10000 → the remembered cursor, child 1 the first time, as item 35's rule) — the latter
follows from the generic rule.

Build: play flow B's two 0002 as one; reproduce (or knowingly skip) the per-frame 0004 buzz with a pick under the
pointer; pupsibor opens on its remembered row (row 1 the first time).

## Item 57: inventory / equipment — the job sprite; a slot label under an equipped icon; persona offsets while equipping (gaps L196, MODLOG L1382..L1387)

**1. The job sprite after "Lv%2d "** — already settled later in MODLOG (L9076, the Status window entry): windowps
212 + job, at the text end + 2 (blue for the main job on 0x063 type 2 bit 14). FUN_100fbc70 is settled in batch 1 §1.
Both parts of the row are stale.

**2. Persona while equipping (mode 2) moves to (16, 48)** — persona's open (vt+4 = **FUN_10146ec0**, switch on +0x1c):
- mode 2 (Equipment, item 50): position (0x10, 0x30) through FUN_101602f0 (the manager's layout-to-screen placement),
  FUN_10119110 moves the window, FUN_10118f70 stores the rect in +0x14;
- mode 1 (Status): (0x10, 0x10), window group byte +0x38 |= 0x20, level's +0x1c = 0, then opens "level" (0x10371518)
  and "statcom2" (0x103715ac) — MODLOG's Status entry;
- mode 0: the layout position. Every mode then resets the stats request (FUN_10147000, item 28) and takes the
  windowps sheet's +0x6b8 sprite into +0x24.
So the only "offset" is the window's origin; the fields keep their window-relative positions *(MODLOG's
DAT_103658f0 is an old-build table; no per-mode field offsets exist in this build's persona)*.

**3. Slot labels under equipped icons are not hidden by code.** The equip grid's draw (vt+0x10 = FUN_10132f60) draws,
per slot with an item, the icon (FUN_1015bdb0 / FUN_1015bd70 at scale 1.0, 0x80808080), then for a piece kept in a
wardrobe the container's badge (FUN_10132db0: framesus sheet +0x2f0..+0x2fc / +0x330..+0x33c for containers 8, 10..16,
drawn with FUN_1011fb60) and FUN_10132e60 records that container number in the cell (+0x1ea). It has no label code:
the slot names are the layout children's own sprites, drawn by the generic window draw before the handler draw, so
an icon's transparent pixels show the label through — as the shot shows *(inf: from the draw order, not footage)*.

Build: persona at (16, 48) in equip mode; draw the slot label under the icon (no hide); add the wardrobe badge for
pieces in wardrobes 1..8.

## Item 58: select screen — delete's second confirmation, ptc9dele (gaps L50, MODLOG L2860 / L2888)

**ptc9dele is a hold-D-and-Enter confirmation that starts a 30-second countdown; only when it reaches 0 is the
character deleted. Escape does nothing in it; Cancel (child 2) backs out.** MODLOG's FUN_10233c00 is old-build.

- Registry entry 339 (0x10375d1c): handler **DAT_10669008** (object FUN_10164660, 0x38 bytes, vtable 0x10335d30), flags
  A 0x00010002 (0x10000: opens on child 1 — but see the placed hook), B 0x0308.
- **The flow** (the select screen's delete, FUN_1023b2fe region): ptc6yesn type 4 (FUN_1023e370(4, 0, name, 1); No
  highlighted) → on Yes **FUN_1023e420**: +0x30 = +0x18 = +0x34 = 0, +0x20 = 0, **+0x14 = 2**,
  `FUN_1015e350("menu    ptc9dele", 1, 0)`, then **a modal loop FUN_1023e4d0** (FUN_10245c00 pumps a frame until
  +0x30 == 1; a pump failure → −1), close it, return +0x14. Result 0 → the delete request (FUN_1023b550(0, 1, 0, 2, …)),
  the slot's actor freed, group 8 0x83 shown (FUN_1023e0c0); anything else → back to the list.
- **Open** (vt+4 = 0x101646a0): window +0x5d = 0, the three SE flags +0x84 / +0x85 / +0x86 = 0 (the handler plays its
  own), FUN_1011acc0, child 3 → state 0x10 (inert text). **Placed** (vt+0x14 = 0x101646e0): cursor = +0x14 = **child 2**
  (Cancel) and FUN_10118170 — so it opens on Cancel.
- **Keys** (vt+0x18 = 0x1023d2e0; nothing once +0x30 == 1; jump tables 0x1023d398 / 0x1023d38c):
  - 3 / 4 (Right / Left) and 9 (hover): SE 0001;
  - 6 / 7 / 8 (Escape …): **nothing**;
  - **5 (Enter) on child 1**: only while **key function 0xa6 is held** (context 0x3f, mode 4; 0xa6 = the keyboard **D**,
    keyfn table) — else SE 0004. First time: +0x34 = now (FUN_100dc130, seconds) and SE 0002; while counting, each
    further D+Enter does +0x34 −= 1 (one second less) with SE 0002;
  - **Enter on child 2** (or any child ≠ 1): +0x30 = 1, window +0x5d = 1, +0x14 = child − 1 (1), +0x34 = 0, SE 0002 →
    the loop ends with "no".
- **Draw** (vt+0x10 = 0x1023d1a0): the texts group 8 0xbc, group 0x1b 0xb / 0xc / 0xd and group 8 0xe7 (x + 0x30 for
  non-Japanese clients); while counting, remaining = max(0, +0x34 − now + 30) printed with group 0xa 0x37 (%d seconds);
  **when it reaches 0: +0x30 = 1, +0x5d = 1, +0x14 = 0** → the delete goes ahead.

Build: after ptc6yesn's Yes open ptc9dele on Cancel; Enter on the delete row only counts with D held, starting a 30 s
countdown (each further D+Enter −1 s); delete at 0; Escape ignored; Cancel returns. Our lobby is a different flow, so
the request itself maps onto ours.

## Item 59: select screen — the kept cursor on reopen (MODLOG's DAT_10623ac4); the rest of the row (gaps L51, MODLOG L529 / L2764 / L2864)

**1. The select list keeps its cursor only on the first reopen after leaving the game, and only when config 0xac is
set.** (MODLOG's DAT_10623ac4 / FUN_102371c0 are old-build; this build:)
- The select list object DAT_10669080 (vtable 0x103359d0); its open (vt+4 = **FUN_10241910**): `if (DAT_10669054 != 1)
  DAT_1066908c = 0` — the cursor / camera slot (+0x14 = +0x18 = DAT_1066908c) is reset to slot 0 unless the flag is 1.
  (The same open clears +0x84 / +0x85 / +0x86, sets +0xd0 = 1 and lays the window out at the 640 × 480 design origin.)
- **DAT_10669054 = 1** only in **FUN_10240880**: `if (config 0xac (FUN_10193850(0xac)) != 0)`. It runs as the tail of
  **FUN_10010520** (0x100106ae `jmp`), the leave-the-game cleanup of the top-level state machine (states 0xb / 0xd /
  logout; it also writes the item sort files) — i.e. on returning to the character list from the game.
- The lobby flow FUN_102435f0 consumes it: on entering the select screen (0x1024367a) the flag keeps DAT_1066908c, and
  right after opening chfwin / dbnamese it clears it (0x102436dd..0x102436ed). Other paths also zero DAT_1066908c
  (0x1023b25c, 0x1023b3c2, 0x1023b50e after a delete / error).
- So: log out of the game with config 0xac on → the list opens on the character you played (once); otherwise, and on
  every later reopen, slot 0 *(inf: config 0xac's label not read)*.

**2. The rest of the row is stale or out of reach:** chfwin, the 149-frame (0x95) head-point refresh and the arc-length
camera speed were traced and built later (MODLOG L2763.., L2832..). chswin's "file details" line (group 8 0xee /
0xcb) needs PlayOnline's stored-settings data, which neither LSB nor this client has — nothing to trace without it.
The non-English count lines (numbers + a sprite) were not traced in this pass.

Build: remember the played character's slot when returning from the game and open the list on it once if config
0xac is set; otherwise start on slot 0.

## Item 60: FFXI-3117 — the five failed user-file operations (gaps L52, MODLOG L2832 / L2887)

**3117 means the client could not set up the character's USER folder (USER\<content id in hex>) in five tries.**
The login path (FUN_102435f0's Yes branch, asm 0x10243d16..0x10243da7) and the top-level state 5 (FUN_10011a00 case 5,
the same calls) do, for the cursor slot's content id (list entry +0x64 → FUN_100db790):
```
for (tries = 0; tries < 5; tries++) {
    pump one frame (FUN_1023e4d0(0));
    if (FUN_101c2b60(id)) break;            // the folder exists: FindFirstFile "USER\*" and a name == sprintf("%x", id)
    if (!FUN_101c2c80())                    // fewer than 0x50 (80) entries in USER (".", ".." not counted)?
        FUN_101c2e70();                     //   no: remove the oldest character folder (names parsed as hex,
                                            //   age from each folder's timestamp.dat, FUN_101c4260 / FUN_101c2dc0)
    FUN_101c4ce0(id);                       // create it: "%s\%x\%s" … "timestamp.dat" written (FUN_101c4170)
}
if (tries == 5) { error 0x75 (FFXI-3117, group 0x1b 0) via FUN_1023e4a0; back to the list (FUN_1023e570, FUN_102435b0) }
```
- The USER path is object DAT_10662dcc +500 (with +0xd0 as a parent folder when set). So 3117 comes from a
  read-only / full disk or a path that can't be created, not from the server.
- Ours has no per-character USER folder step in the login (one request to the lobby), hence "no counterpart". If ours
  ever keeps per-character files (macros, mcr*.dat, AUCSORT.DAT, …), the retail rule is: make USER\<id hex> before
  entering, evict the oldest folder past 80, and fail with 3117 after 5 attempts.

Build: nothing needed unless per-character user folders are added; then mirror the loop above.

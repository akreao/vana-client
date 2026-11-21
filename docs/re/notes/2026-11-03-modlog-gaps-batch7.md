# MODLOG gaps, batch 7 (queue items 21..30)

Copied from Steven's `/run/media/SSD/FFXI/re-notes/modlog-gaps-batch7.md` (2026-11-03). Target and tools as in
[batch 1](2026-11-03-modlog-gaps-batch1.md) (`FFXiMain.unpacked.dll`, sha256 8097dbf8…550c). All addresses are VAs in
this DLL. **(inf)** = inferred. Queue: modlog-gaps-queue.md.

## Item 21: the echoes' rank icon (gaps L165, MODLOG L6016)

**Confirmed, and narrowed: it is only on the Assist channel echoes** — /assistj, /aj (FUN_10078be0, "%c%c%s(J) : %s"
at 0x1035806c) and /assiste, /ae (FUN_10078d80, "(E)" at 0x10358088). Other chat echoes have no icon.
- FUN_10078be0 at 0x10078ca4..0x10078d17 (the "(E)" twin at 0x10078e44..):
  - mentor = G+0x40deb, mastery = G+0x40dec (both from s2c 0x01B @0x65 / @0x66, 0x100f0994 / 0x100f099d; cleared at
    0x100fb558 / 0x100fb55f);
  - if both are non-zero and FUN_100e47c0(8) (s2c 0x0B4 byte 7 bit 1, MentorFlg):
    R = (u8)(mentor × 10 + mastery + 0x24); if R < 0x2f or R ≥ 0x4d, R = 0x2f.
    Line = sprintf("%c%c%s(J) : %s", 0xEF, R, ourName (G+0x3c3a4), text);
  - otherwise, the plain "%s(J) : %s" form without the icon pair.
- 0xEF R is the icon escape (fontshp glyph R; MODLOG's rank icons 0x2f..0x4c). Implementation only.

## Item 22: zone lighting record floats @0x50 / @0x54 / @0x68 (gaps L139, MODLOG L478..L479)

**@0x50 and @0x54 are copied to the camera every frame but nothing reads them; @0x68 has no reader found. All three look
unused in this build.**
- FUN_101880c0(E, payload) (gdecomp/101880c0.c) sets E+0x18 = payload and E+0x1c = payload + 0x4c. It then calls
  FUN_10014eb0(FUN_10186ff0()) → cam +0x314 = [E+0x1c]+8 = **@0x54**, and FUN_10014ed0(FUN_10186fe0()) → cam +0x318 =
  [E+0x1c]+4 = **@0x50** (cam = [DAT_104568fc]).
  (The fog end / start come from @0x3c / @0x40 of the group through FUN_10014e80 / FUN_10014e60.)
- Their getters FUN_10014ea0 (+0x314) / FUN_10014ec0 (+0x318) have no call site and no pointer to them anywhere in the
  image. The only other `+0x314` / `+0x318` references are virtual calls on unrelated objects. **No consumer.**
- DAT_1047bf18's override (MODLOG L13309) just feeds different values into the same dead fields.
- @0x68: no load of payload +0x68 (or `[E+0x1c]+0x1c`) was found in the environment / camera code (0x10014000..0x10016000,
  0x10180000..0x101a0000) (inf: unused; a wider search wasn't done).

Nothing to build: the lerp can carry them, but nothing should use them.

## Item 23: do the 3 blended furniture pieces keep Z writes? (gaps L281, MODLOG L15397 / L15427)

**Yes: confirmed from the code.** Furniture (XiModelActor, FUN_100c4c20 → `DAT_10631f24->FUN_101844c0(actor+0x770 /
+0x774 instance)` at 0x100c4cc8) is drawn by **FUN_101844c0** (0x101844c0..0x10184c64), in the actor pass:
- per mesh (0xc0-byte entries, 0x101849fe.. loop): texture (vt+0xf4), then **ALPHABLENDENABLE (0x1b) = the mesh's word
  +0x1c** (0x10184b48..0x10184b52; the bit-31 blend flag), vertex shader / stream, draw (FUN_1000cb20);
- **FUN_101844c0 never sets ZWRITEENABLE (14)**. The states it sets are 0x8b / 0x8c, 0x30, 0x22 / 0x24, ALPHATEST (0xf)
  on / off around one part, CULL (0x16) and 0x1b. So Z writes stay as the actor pass has them (on, from the state block
  FUN_10009920), and the blended meshes write depth;
- no sort: meshes are drawn in file order within the instance, instances in actor order (inf: no depth sort for these).
- Compare the zone scenery draw (FUN_1017c9c0 / FUN_1017de20, 0x1017daec..0x1017db45 and 0x1017ebc3..0x1017ec13):
  mesh word +0x20 set → ALPHABLEND 1 **and ZWRITE 0**; clear → ALPHABLEND 0, ZWRITE 1. So retail's Z behaviour differs
  between scenery (blended meshes don't write Z) and furniture (they do).
- Blend factors (SRC / DEST) are not set in FUN_101844c0 either; they are whatever the pass left (inf).

Build: draw the bit-31 furniture meshes blended with depth writes on, unsorted, in the actor pass (ours uses the scenery
blend pipeline: no Z write, bias −8). This also settles MODLOG L4325's question for scenery: retail blends bit-31 scenery
meshes with Z writes off.

## Item 24: the map's sub-map key and the arrow's heading sign (gaps L271, MODLOG L2571 / L2585)

- **The sub-map key is already decoded** (MODLOG L4598, "Clock grid position and map sub-keys": FUN_101811d0's vertical
  ray, MZB object flags @0xa4 → ((f >> 26) & 3) << 3 | ((f >> 3) & 7); ours Ground::map_key). The gaps row's first half is
  closed.
- **The arrow:** the player marker record lives in the map window at **+0x674** (FUN_101f8ea0, 0x101f8eb6..0x101f8f9b):
  - +0x674 u16 flags = 0x9f; +0x676 = 2 (sprite set keytops3); +0x67a / +0x67c = the player's map pixel
    (FUN_101f8fb0);
  - **+0x688 (= record +0x14) = player vt+0x1c0()[1]**, the rotation vector's y = the yaw in radians (the same value as
    actor +0x18), taken once at open;
  - window +0x98 = &record.
- **Draw:** FUN_101ee650 (record flag bit 1) → `FUN_1011ff00(x, y, angle = record +0x14, 0x80808080)` → per sprite part
  FUN_1011ed70 → FUN_10027c20(angle) (a Z-rotation matrix; inf from use) and FUN_1011ede0. **No negation anywhere:
  screen rotation = +yaw.** With the sprite pointing east at 0 and screen y down, yaw 0 = east, and increasing yaw turns the
  arrow clockwise (east → south), the same convention as the compass words (batch 5 item 1).

Build: draw the arrow rotated by +yaw (screen-space, clockwise with y down), from the yaw captured when the map opened.

## Item 25: the page modifier 0xa8 — the arrows' hover state (gaps L251, MODLOG L13995 / L14016)

**Already fully specified in MODLOG L14016 (implementation only). Confirmed with the helpers read for batch 5 item 5:**
- **FUN_10221cd0** (the paged-list base's "disable"): window +0x62 = 0 (no cursor / hand) and list **+0x5c |= 2**.
  **FUN_10221ce0** ("enable"): window +0x62 = 1; if +0x5c bit 1 was set, FUN_10221ed0 (rebuild), then +0x5c &= ~2.
  (The same base serves the status-icon window, which is why op 0x46 sub 1 reuses them.)
- Hovering an arrow (children 9 / 10, message 9) calls the disable; hovering a row calls the enable; every rebuild
  (FUN_10221920) clears +0x5c. While bit 1 is set the list ignores every message, keyboard pages included.
- A click on an arrow is Enter on child 9 / 10, re-sent as Left / Right through vt+0x18, which tests 0xa8 again
  (FUN_10123a70(0x3e, 0xa8, 4, -1)). With Ctrl / Alt held that is a page on the disabled list → nothing happens. Without
  it, the log / area / week / category changes.
- Exceptions: RoE's arrows and jbpcat's page areas skip the 0xa8 test (a Ctrl-click switches there).

Build: on pointer-over-arrow set the list's disabled bit (hide the hand, drop list keys), clear it on row hover and on
rebuild; route arrow clicks as Left / Right through the same 0xa8 test.

## Item 26: Missions — the held-key header stop (gaps L253, MODLOG L9186)

MODLOG L9186's description is confirmed; addresses added. miss00's OnKey (FUN_1022e760), the Up / Down path at
**0x1022e99c..0x1022ea19**:
1. FUN_10221ce0 (re-enable the list), list +0x5c |= 8 (silent), then **FUN_10222150(dir, repeat)** (the paged-list
   base's cursor move; it stores dir in +0x60).
2. FUN_1022df70(new cursor) classifies the row: −2 = a blank row, −1 = a **header**.
   - **Header on a held repeat** ([esp+0x18] != 0): FUN_10222150(1 /* Down */, …) steps back off it, and
     **+0x60 = 2** (0x1022e9f4).
   - **Header on a fresh press**, or a blank row: FUN_10222150(dir, …) again (skip over it), +0x60 = 0.
3. +0x5c &= ~8; SE 0001 only if the cursor ended on another row (0x1022ea09..0x1022ea14).
- In the base FUN_10222150, +0x60 is the remembered direction: at the top edge (0x10222300) +0x60 == 2 limits the move
  to one page instead of wrapping through the count, and at the bottom (0x10222349) +0x60 == 1 does the mirror; then
  +0x60 = the new direction (0x102223cd). The per-frame FUN_10222110 re-sends a held direction from +0x60. (inf on the
  exact effect: a held Up that reaches the top header stops there, a new press wraps to the last row.)
- Scroll messages 0x16..0x1b skip no headers. Children 9 / 10 (the arrows) disable the list (item 25).

Build: in the mission list, when a held Up lands on a header, bounce back down one row and stop repeating; a fresh Up
from row 1 wraps to the last row.

## Item 27: Mouse — right-button release clearing the target (gaps L257, MODLOG L3251; see L3171, L5768)

MODLOG's FUN_10151f20 is old-build; the current path is the click handler at the **same addresses as MODLOG L5768**
(FUN_101585f0 / FUN_101587f0 / FUN_10158db0 unchanged in this build).
- **Key → result table DAT_10371ee0** (16-byte rows {ctx, key fn, mode, result}, walked by FUN_101587b0, first hit
  wins; terminator key −1): `{0x3b, 0x8b, 5 released} → 0x16`, `{0x3b, 0x8c, 5 released} → 0x17`,
  `{0x3b, 0x92 Esc, 1 pressed} → 0x11`, `{0x3b, 0x91, 1} → 1` (confirm), `{0x3b, 0x97, 1} → 0x10`; full dump in the notes
  below.
- FUN_101585f0 (once per frame) → FUN_101587f0(result). **Lock gate first** (0x101587f0..): if the target window
  +0x5c & 3 (locked), not sub-target mode (+0x59 == 0), no prompt (+0xc4 == 0) and result ∉ {1, 5} → return 1, nothing
  happens. So right release, Escape and the left pick do nothing while locked.
- **Result 0x17 and 0x11 share one case** (switch case 0x16 / 0x10): if FUN_101598a0() (current target) != 0 →
  **FUN_10158ff0** (if +0x6a < 1: FUN_101590c0(0) clears the target, FUN_10158590; +0xc4 = 0; the prompt callback
  +200 is called (0, 0) and zeroed; FUN_10159040) then **FUN_10123e80** (the frame's remaining keys fail). Returns 1.
  So a right release = Escape for the target.
- **No target**: in the mouse state machine FUN_10125360, states 1 / 2 (0x1012564c..0x101256a5): 0x8c released, no
  prompt target (DAT_1057876c → FUN_101598a0 == 0), a player (FUN_10081550) and **autorun on (DAT_10487f81)** →
  FUN_100a6070(0): autorun off (resets 0x10487f64..70 from 0x1035bba8, FUN_10129de0(0x41), FUN_100814f0). DAT_10487f81
  is the autorun flag (also toggled by key 0x1c Num7 in FUN_100a5ef0).
- **State 4** (left-button steer, set by FUN_10125040): a right **press** toggles autorun (0x10125837..0x10125883).
- **Right drag = state 5** (FUN_10125170): right held (or left, in mode FUN_1025e070, DAT_1066276c+0x44 == 2), moved
  ≥ 0x41 px² from the press point or the 25.0 timer (+0x78) run out, +0x54 / +0x55 clear, the cursor kind
  *DAT_10456a28 == 0x60 / 0xd0 (or FUN_102394b0), and +0x88 set. While in state 5:
  - the cursor becomes an 8-way arrow: angle about the press point → +0xa6 = 10..0x11 (0x101258c7..0x10125ab4), read by
    FUN_101245c0 *(inf: the cursor shape; no other reader)*;
  - **the camera axes stop reading the mouse**: key fns 6 / 7 (FUN_101232e0 / FUN_10123450, 0x1012330c / 0x1012347c)
    take the pad axis when state == 5.
  - The release still goes to FUN_101587f0(0x17) (FUN_101585f0 → FUN_10158b40 in state 5 / 4; it only skips the
    prompt check and picks when the result is 0). **No code path makes a drag swallow the release** *(inf: the clear
    still happens after a drag; no FUN_10123e80 / key consume found in FUN_10125170 / FUN_10125360)*.
- **The retail mouse camera is not a right drag.** FUN_10124bc0 sets the camera deltas +0x8c / +0x90 = ±1 when the
  cursor touches the window edge (DAT_106218b4 / b8 ±1), times −1.0 (DAT_1032a3f0) if config 0x8b / 0x8c (invert) is set;
  key fns 6 / 7 read them. The fishing confirm FUN_10088b50 also accepts 0x8b / 0x8c release (ctx 0x3f) — unrelated.

Build: on right-button release in the world (ctx 0x3b, no window focus): if locked and not in a sub-target prompt, do
nothing; else if there is a target, clear it (same as Escape, then drop the frame's other keys); else if autorun is on,
stop it. Remove the right-drag camera: a right drag only turns the cursor into the 8-way arrow (and stops edge-pan);
the camera turns by pushing the cursor against the window edge.

Table DAT_10371ee0 (ctx, key, mode → result): 3b b0 0→18, 3b b1 0→19, 3b b3 0→1a, 3b b2 0→1b, 3b 91 1→1, 3b 27 0→2,
3b 28 0→2, 3b 2a 0→3, 3b 29 0→4, 3b 4c 1→6, 3b 46..4b 1→6..b, 3f 52 1→12, 3b 53 1→d, 3b 54 1→e, 3b 84 1→f, 3b 97 1→10,
3f 4d 1→1c, 3f ab 1→13, 3f ac 1→14, 3f 4e 1→15, 3b 92 1→11, 3b 8b 5→16, 3b 8c 5→17, 3f b8 1→1c.

## Item 28: Status window — the stats request retries (gaps L238, MODLOG L9066)

MODLOG L9066 is confirmed in this build at the same addresses; details added.
- persona's frame **FUN_10147030** (runs every frame; counters are frames, not time-scaled):
  1. If +0x1e (sent) == 0: `--(+0x21)`; when it goes below 0 → **FUN_10097fd0(LAB_10147010, 0)**.
  2. FUN_10097fd0: DAT_10487ea4 = callback; **FUN_100fc3c0(0x61, 0, 0)** allocates c2s 0x061 through the 1000 ms per-id
     gate (item 12) — so it fails if a 0x061 went out in the last second or the queue is full. On success: size 6
     (thunk_FUN_100ddd20(p, 6, 0)), @4 = 0, returns 1.
  3. Sent → +0x1e = 1, and if statcom2 (DAT_105781f8) exists, busy bit 0x40 on (FUN_10221270(0x40)).
  4. Refused → `+0x20++`; while the old count < 11: +0x21 = 0x14 (retry in 20 frames); else +0x1e = 1 (gives up: no
     busy bit). So at most 12 tries, the first on the first frame (+0x21 starts at 0 → −1).
  5. Every frame: +0x1f = 1 once DAT_10485824 bit 0 (the stats-ready bit) is set.
- Init / reset FUN_10147000: +0x1e = +0x1f = +0x20 = +0x21 = 0.
- **The answer**: s2c 0x061 handler FUN_100994a0 (registered at 0x100945f8) copies the stats, then calls DAT_10487ea4
  (0x100995fe) and clears it. LAB_10147010 clears busy bit 0x40 on statcom2 (FUN_10221300 on +0x20) and returns 1.
  0x1009388c also zeroes DAT_10487ea4 (a reset path; the callback is dropped).

Build: on persona open zero the four bytes; each frame, while not sent, count down and try to send c2s 0x061 through
the 1 s per-id gate; on refusal retry 20 frames later, up to 12 tries, then give up silently; on send show the busy
strip until the 0x061 answer runs the stored callback.

## Item 29: Cast bar — Chainspell (status 48) and items (mode 1) (gaps L292, MODLOG L4862 / L5163)

The bar object is DAT_1057817c ("menu    casttime"); **FUN_1012c1d0(mode, ticks)**: +0x20 = mode, +0x14 = +0x18 =
max(ticks, 0) (float), then FUN_1015e350("menu    casttime", 1, 0). Three openers:

| Mode | Opener | Ticks |
|---|---|---|
| 0 | spell: 0x028 category 8 "ca??" on the player → FUN_1022b3d0 (0x1009e60f) | spell record (DAT_10667a74 + id × 100) +0xc byte × 20 |
| 1 | item use: the sender FUN_101f0260 (0x101f03f8) | item record +0x26 (u16) × 20 |
| 4 | event op **0xb4** sub 7 (FUN_100b0a00, 0x100b0d66; table entry 0xb4 → stub 0x100bc82f) | operand 2 (FUN_100af4b0(2)) |

- **Chainspell**: FUN_1022b3d0 opens only if **FUN_1021e670(0x30) == 0**: a scan of the player's status icon list
  **DAT_10485a30..0x10485a6f (32 u16)** for 48. With Chainspell up the bar never opens (nothing else changes; the
  "sp??" close still runs). The same scan helper serves any status id.
- **Items** (FUN_101f0260, asm 0x101f0260..0x101f0450; args (container, slot, target, mode)):
  - r = item record FUN_1015bc40(item id); use time = r+0x26 (0 → no bar); charged = r+0x2d > 0; flag = r+8 bit 3.
  - Charged gear needs FUN_101cc3a0(item, container) (ready) or it fails.
  - A non-charged item on a zone NPC (target id 0x01xxxxxx) without +0x120 bit 13 goes to the c2s 0x036 trade path
    (no bar).
  - Otherwise: if DAT_1048098d (in the Mog Garden, FUN_101cc1b0; MODLOG L15598) is set, only items with r+8 bit 3 pass;
    then **the bar opens (mode 1) before the send**, and the send is FUN_100eb660 (mode 3) or FUN_100eb5e0 (others),
    i.e. the c2s 0x037 path. The bar opens even if that send is then refused (no rollback found).
  - No 0x028 handling opens or closes the bar for items (only category 8 touches it); it runs out on its own
    *(inf: an item interrupt leaves it running)*.
- **Draw by mode** (FUN_1012bf60, 0x1012bf8a): modes 0..3 → the gauge and the percent (MODLOG L4862); **mode 4 →
  FUN_1012bfe0: only the seconds left**, ftol(+0x18 × 1/60), white 0x80808080, at (+0x69, +9); other values draw
  nothing.
- Event op 0xb4: sub 7 opens (if not already open), sub 8 closes ("menu    casttime"), sub 9 stores the ticks left
  (ftol, 0 when closed) into operand 2 (FUN_100af310).

Build: skip the spell bar when status 48 is in the player's icon list; open the bar in mode 1 at item use (before
c2s 0x037) for record +0x26 × 20 ticks when +0x26 > 0; draw mode 4 as a seconds number only.

## Item 30: Set Bazaar — the name's bazaar glyph (0x9c); the row disabled in Monstrosity (gaps L217, MODLOG L9954)

**The glyph** is already decoded in MODLOG L10702 (FUN_10097500's chain: `+0x128 bit 9 → 0x9c`, the last entry before
the default, so any earlier match — away, GM, Campaign, recruiter, party seek, Level Sync, mentor, New Adventurer,
POL, link dead, Monstrosity, team — wins over it). This pins the bit's writers:
- s2c **0x00D u32 @0x20 bit 31** → actor +0x128 bit 9 (0x1009a684..0x1009a6aa; the actor by u16 @8).
- s2c **0x037 u32 @0x2c bit 29** → +0x128 bit 9 (0x1009d53a..0x1009d555). *Correction to MODLOG L9987: 0x037 is @0x2c
  bit 29 (LSB Flags1 of the own char packet), not @0x20 bit 31.*
- Nothing else writes bit 9 in the two handlers *(inf: the 0x00E handler has no bazaar bit; NPCs have none)*.

**The row in Monstrosity:** socialme's open **FUN_1014e720** (after the caption 0x10 and arrows 0xf states):
`if (FUN_10097d50()) FUN_10118b60(5, 5)` — child 5 (Set Bazaar) set to state 5 = disabled (dim tint 0x30808080,
batch 5 item 6).
- **FUN_10097d50 = "in Monstrosity"**: char data +0x40db6 (u16) != 0, or main job DAT_10485640 (0x061 @0xc) == 0x17
  (MON). The same test picks "menu    mp_pmode" over playermo (FUN_101585f0).
- The row's own handler (socialme messages FUN_1014e7b0, child 5) has no check; the disabled state is the only
  gate *(inf: Enter on a disabled child plays SE 0004 through the generic layer, as elsewhere)*.
- Set only at open: the state isn't refreshed while the page is up.

Build: draw 0x9c before the name when +0x128 bit 9 and no earlier glyph matches (bit from 0x00D @0x20 bit 31 and
0x037 @0x2c bit 29); on opening the social page, disable Set Bazaar when +0x40db6 != 0 or the main job is 23.

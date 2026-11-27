## Job Points: jbpcat's page arrows (2026-11-27)

<!-- dll-build: 30261102_0 -->

Modlog-gaps item 192 (split from 182): jbpcat (Job Points' categories / gifts, layout `menu    jbpcat`) has two
children past its ten rows, 11 (2, 5) and 12 (338, 5), 15 x 9, windowps 332, links all 0xff, so no key reaches them;
only the mouse puts the window's cursor there. Ours took the mouse on rows 1..10 only.

- The message path: the window's handler FUN_101196f0 first (arrow keys 1..4 move its cursor by the links through
  FUN_1011c6b0 and FUN_1011a320, only while window +0x61 is set; no SE, the list base FUN_101f7ec0 clears +0x85 /
  +0x86), then the list's win_jbpcat_onkey 0x1014bf50 (vtable +0x18) with the cursor child as its parameter, which
  passes on to win_common_onkey_3 0x101f8280, whose tail (0x101f855b) clamps an index at or past the count to the
  count's child (FUN_1011a240(count - top)) and calls win_jbpcat_v58 0x1014c130 (vtable +0x58).
- win_jbpcat_onkey message 9 (sent by FUN_1011a240 on a new cursor child while the pointer is shown, mouse +0x4e,
  and +0x48 not gliding): mouse +0x52 on 11 / 12 puts the cursor there (FUN_1011a320) and clears window +0x61 /
  +0x62, nothing else (no SE, the last cursor +0x24 kept); elsewhere it sets them and win_common_onkey_3 9 plays
  SE 0001 for a hit row (1..count) other than the last cursor, then the tail's clamp. Messages 3 / 4 with key 0xa8
  (Ctrl / Alt) do nothing while +0x62 is clear, else win_common_onkey_3 0x19 / 0x18.
- win_jbpcat_v58: messages 1 / 2 / 0x18 / 0x19 save FUN_101f8710 (cursor child - 1 + top) into +0x60 / +0x62 by
  page; 3 / 4 save it, turn the page (+0x58), load the other (FUN_1014c410 -> FUN_1014c540 / FUN_1014c770: FUN_101f8630
  sets the last cursor to the window's, then FUN_101f8690(its index, 0): top = min(index, count - 10), cursor child
  index - top + 1 through FUN_1011a240), set +0x61 / +0x62 and play SE 0001 (FUN_10037b30); 5 with the cursor child
  0xb / 0xc does the same without touching +0x61 / +0x62; 5 on a row does nothing while +0x62 is clear.
- So a click on an arrow turns the page and the load's message 9 puts the cursor back on the arrow while the pointer
  holds it. The index the page keeps is FUN_101f8710 on the arrow, top + 10 (top + 11 for 12), unless the tail's
  clamp moved it; page 0 left by its arrow comes back ten rows on.
- Down / Up on an arrow (or on a row with +0x61 clear): the window moves nothing; win_common_onkey_3 1: on row 10
  with the last cursor there the top goes on, else a count below the child puts the cursor on the count's child,
  else SE 0001; 2: on row 1 with the last cursor there the top goes back, else SE 0001. Left / Right turn the page.
- win_jbpcat_v28 0x1014bda0 (every frame): win_jbpcat_v50 0x1014bf20 first (the help: the entry at FUN_101f8710's
  index, FUN_1014c9a0, none at or past the count; so on an arrow the entry below the shown rows), then the tail's
  clamp.

Built: client/jobpoints.rs `List` (the window's cursor child with `arrow`, +0x61 / +0x62 as `keys`, the last cursor,
mouse +0x52 / +0x4e), `List::hover` / `put` / `clamp` / `turn` / `down` / `up` / `rows`; the mouse takes children
1..=12; `jobpoints::key` takes the pointer's shown state (`winhits::shown`). Mouse +0x48 is taken as past its glide
(jbpcat turns no focus). Not checked on screen.

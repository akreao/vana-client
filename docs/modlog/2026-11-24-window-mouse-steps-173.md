## The mouse on windows: hover, held button and release as retail (2026-11-24)

<!-- dll-build: 30261102_0 -->

Modlog-gaps item 173 (split from 161): the hover, the held button and the release of the mouse on windows.
Addresses are 30261102_0, decompiled with Ghidra; the hover tail, the held step and the jbpcat / conf11s handlers
checked in Rizin.

- The hover is the tail of the mouse routine FUN_101267f0 (0x10127220), every frame in mode 2 with the pointer shown
  and a window active (FUN_10160380, not FUN_1015f2b0): the hit test FUN_1011a140 on the active window (no frame
  test), the result kept at mouse +0x52, and the cursor moved (FUN_1011a240(hit, 0), which sends message 9) only when
  the hit is a child and not the hit of the frame before (`jle` / `cmp ax, di; je` at 0x1012722c / 0x10127231).
  In mode 2 with the pointer shown +0x52 is set to -1 before the test, so a frame with no active window or no hit
  (also while the pointer glides: FUN_1011a140 returns -1 while mouse +0x48 > 0) forgets it; ours also forgets it
  while the pointer is outside the game's window.
  A pointer resting on a child does not take the cursor back from the keys; a window opening or a list moving under a
  still pointer does move it.
- The held button, FUN_1015f360 (from FUN_10160fd0 on FUN_10124f00(0x3e, 0x8b, 4)): with the pointer inside the
  window's frame (FUN_1011a0a0: window +0x3a .. +0x40, both ends in) and a child hit, a child other than the cursor's
  takes the cursor every frame (FUN_1011a240, message 9); on the cursor's child the count +0xb0 takes the frame's
  ticks (`fiadd` of FUN_10014cf0, at least 1, `__ftol`) until it is past 0x1e (`cmp word [esi + 0xb0], 0x1e; jle`
  at 0x1015f3e3), and from then every frame sends message 0x1d instead. Only itemctrl / moneyctr (FUN_1013bac0 /
  FUN_10141fc0 case 0x1d: children 4 / 5) and holdtime (FUN_102126d0, not built) take 0x1d.
- The release, FUN_1015f420 (state 5): message 5 when the hit child is the cursor's, nothing on another child; the
  count goes back to 0. With no child hit it plays SE 0004 (FUN_10037c50, "0004") unless mouse +0x87 is set (rem4line
  / rem4li2 windows FUN_10137e70, the map FUN_101fa130); not while a press began on a scroll bar (+0x86).
- The hand (mouse +0x4c = 1 at the end of FUN_101267f0, mode 2) needs FUN_1011a140's out flag: the hit part is not in
  state 5.
- conf11s (object DAT_10663900, vtable 0x103386e8, onkey FUN_1019ba40): its page arrows 15 / 16 are mouse children.
  Message 9 on them puts the cursor there (FUN_1011a320) and clears +0x62, so Down / Up do nothing; message 5 on 15 is
  FUN_1019bc90 (window 1 / 2 back), on 16 FUN_1019bc60 (on). Left / Right (messages 4 / 3) set +0x62 again, put the
  cursor back on +0x4c and switch the window; +0x4c is the cursor as the last Down / Up left it (1 at the open,
  FUN_1019b870), not as the mouse left it.

Ours:
- `winhits::takes(gu, window, child, in_frame)` is the hover and the held step for one window's hit child: true when
  the child is not last frame's hit, or the left button is down with the pointer in the frame. It also sets the hand
  unless the child was drawn in state 5 (`GameUi::disabled_children` records them per frame, `disabled_part`).
  `winhits::frame` starts each frame's hit memory from the pointer input. Ours keys the hit by window as well as child,
  since ours runs more than one window's mouse in a frame where retail runs only the active one's.
- Every window's mouse moves its cursor with `takes` (the generic handler, the lists, the menus, the prompts, the main
  menu, the action menu, the K.O. and raise prompts, the party cursor, the title menu, character select and the
  creator's steps), and a click acts only on the cursor's child (the release). Before, the cursor moved on any pointer
  move over another child or on a click, and a click on another child picked it at once.
- itemctrl / moneyctr (`winhits::ctr`): the button held puts the window's cursor on - / + (ours keeps it as the held
  child), the release on it steps once, and held there past 0x1e ticks it repeats every retail frame (2 ticks at 30
  frames a second, 1 at Movement Rate 60). Ours stepped on the press and repeated after 400 ms every 60 ms.
- conf11s: children 15 / 16 take the mouse, Enter on them switches the log window, Down / Up do nothing there, and
  Left / Right first put the cursor back on the row the last Down / Up left (`Extra::key_row`).
- The list page arrows (miss00 / quest01 / guide01 9 / 10) disable their list on `takes` (no hand), not on any move.
- The raise prompt's own hover memory (`raise_hover`) is gone; it uses `takes`.

Not built (new queue row 182): the release over no child (SE 0004, +0x87); itemctrl / moneyctr's cursor on - / +
(drawn hand, Enter there steps); jbpcat's page arrows 11 / 12 (win_jbpcat_onkey 0x1014bf50: message 9 there sets the
cursor and clears +0x61 / +0x62; win_jbpcat_v58 0x1014c130 message 5 on 0xb / 0xc turns the page, list kept off);
the mouse on iuse (0x101f4970), sortyn (0x10223ec0), bluesibo (0x101e1210) and the map menus map0 / mapv2 / mapv3 /
mapscan; the window frames of character select's and the creator's windows (ours passes the child box); focus moves by
the pointer over another window (message 8, from the loop over FUN_101603f0 at the end of the hover) still follow
ours' pointer moves.

Tests: `client::winhits::tests::hover_takes_only_a_new_hit`, `held_count_repeats_past_thirty_ticks`. Not checked on
screen or live.

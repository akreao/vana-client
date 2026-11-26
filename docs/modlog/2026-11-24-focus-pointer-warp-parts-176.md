## The focus warp's cursor part on the self-drawn windows left over (2026-11-24)

<!-- dll-build: 30261102_0 -->

Modlog-gaps row 176 (split from 163). Most of the row's windows (the config windows, the party menu, the auction,
shop, celestite, blue magic, automaton, Monstrosity and maze lists, the quest and Unity choosers, linkshell naming,
key items ...) were given their cursor part on 2026-11-21 (`GameUi::cursor_on` / `cursor_child`). This audit went
through what was left: every window that draws its own hand or marker and is in the window walk with registry
flag bit 0 clear (so it can take the focus). Decompiled with Ghidra; the evitem row loop (0x1022bf40..0x1022bfc5)
and FUN_1022abd0 checked in Rizin.

Retail:
- Every window starts on child 1: the window load FUN_101193d0 ends with FUN_1011a320(1) (clamped to 1..child count;
  +0x4c stays 0xffff with no child). FUN_1011c080 then looks for the type-1 part (FUN_1011ad60 makes every child
  one) whose id is +0x4c; with none, FUN_10126000 sets +0x48 = 18 and moves nothing.
- faqsub and faqmain (registry 0x20002): their cursor is their own entry index (+0x128 / +0x154). win_faqsub_v14 and
  faqmain's v14 FUN_10219330 clear the window's +0x61 / +0x84 / +0x85 / +0x86, so the generic keys never move +0x4c, and nothing
  in 0x10211000..0x1021e000 moves it for them (no FUN_1011a240 / FUN_10119e10 there; the three FUN_1011a320(2)
  calls are holdtime's, FUN_10212440 / FUN_102126d0 / FUN_10212980). +0x4c is child 1 for good:
  the focus warp goes to child 1's centre (layout (0, 4), 356 x 16), not to the hand on the entry.
- olstat (registry 0): the generic cursor (win_friend_onkey_2 reads the child id), so the warp goes to the child the
  hand is on. Its hand is the small anc2s one.
- evitem's item list (state 2) uses the list base (vtable slots 0x10223500 / 0x10222e90): FUN_102231a0 ->
  FUN_102234e0 puts +0x4c on child +0x54 (1) - top (+0x38) + cursor (+0x30), clamped to the 8 children.
  win_evitem_v10 then sets each shown row's child rect (FUN_10119e10) to (+0x7c, y, w, h) from the entry's size
  (FUN_1022abd0 returns the entry; FUN_10234990 stores w = 0x12d - 0x12 - 0 = 283 and h = the text height, at least
  child 1's 18), y stepping by h + the gap +0x84. +0x7c / +0x7e are child 1's x and y - 1 (FUN_1022b970). So the
  warp and the mouse use 283 x h at (window + 40, row y).
- fep (registry 0x0081000a, bit 0 clear) is a real window (win_fep_v40 opens it with FUN_1015f920("menu    fep")),
  so it takes the focus in retail; ours draws it as a frame outside the window walk. Not built: queue row 184.

Ours now:
- faqsub / faqmain record child 1 (hidden) as their cursor part while focused (`helpdesk::focus_part`).
- olstat records child c with its anc2s hand.
- evitem's item rows: the hand records child 1 + (cursor - top) (at most 8) with the rect 283 x the row height; the
  mouse rows are 283 wide too (they were 319).
- Checked and left: the buff window, the full log and feppalle (registry bit 0, they never take the focus), the
  friend list (the generic hand through `menu_sel`, already recorded), and our own windows outside the window list
  (Settings pages, the updater, the notes, the login form, the two-step code screen, Lua mod windows).

Not done:
- The Unity chooser (quest00 rows drawn by code) records its plate with child 0; quest00 is a list-base window too,
  so its +0x4c is a child, but where retail puts its rows' children was not traced.
- fep as a window that takes the focus: row 184.
- Not checked on screen.

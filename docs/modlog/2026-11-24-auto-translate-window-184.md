## The auto-translate list and the chat line as windows (2026-11-24)

<!-- dll-build: 30261102_0 -->

Modlog-gaps item 184 (split from 176). Decompiled with Ghidra; the open hook's cursor call (0x1019057f..0x10190583),
the draw's FUN_1011a240(sel % 7 + 2, 0) (0x10190c30..0x10190c33) and FUN_1015f0e0's layer walk
(0x1015f0ec..0x1015f122) checked in Rizin. The layouts were read from the menu DAT.

Retail:
- "menu    fep" is a window of the window manager (registry flags 0x0081000a, layer byte +0x29 = 7, group 0). Bit 0
  is clear, so the focus pass (FUN_10160fd0 -> FUN_1015fdb0) gives it the focus while it is up, and the pointer goes
  to its cursor part (FUN_1015fe20 -> FUN_10126000). Row 163's note that it never takes the focus was wrong:
  feppalle (0x0001002b) has bit 0, fep does not.
- It is opened by FUN_1015f920("menu    fep", 0, 0) (menu_fep_fn_3 / _4 / _5 / _6 / _7 / _8) and closed by
  FUN_1015fee0. Its layout: frame 96 x 134, child 1 the "n/m" line at (3, 4), children 2..8 the page's rows at
  (3, 20 + 16 k), 68 x 16. win_fep_v28 sets the frame's width (widest row, at least 90, + 12, at most 200 outside
  Japanese, 140 in it) and leaves the children alone.
- Its open hook win_fep_v04 (0x10190500) places the window at the word and calls FUN_1011a240(2, 1): the cursor to
  the first row with the pointer (18-tick glide), only when the cursor was elsewhere. A new window's cursor is child 1
  (FUN_101193d0), so the first open always moves the pointer. Each draw (win_fep_v10) then sets the cursor to the
  selected row, FUN_1011a240(sel % 7 + 2, 0), with no pointer move.
- FUN_1015f920 on the list while it is open runs the hook again: entering a category (menu_fep_fn_8) and backing out
  of one (menu_fep_fn_6 case 0x1b -> FUN_101b84b0) both do, so the pointer goes to the first row whenever the cursor
  was on another, even when backing out puts the selection back on a lower row.
- The chat line "menu    inline" (flags 0xa, layer 6, group 0) is opened with FUN_1015f920("menu    inline", 1, 0)
  when typing starts (FUN_101b5590) and closed by FUN_101b7050. It takes the focus too. Its only part is its frame,
  so FUN_1011c080 finds no cursor part and the focus moving to it moves no pointer; the focus coming back to the
  window under it when the line closes does.
- FUN_1015f0e0 (raise / insert): the window list's tail is the top. A window whose layer (+0x60, registry +0x29) is
  5 or more goes on top. A lower one is put just above the first window of layer 4 or less found from the top; when
  the walk reaches the bottom on high-layer windows only, it goes just above the lowest of them. Layer 5 and up:
  inline 6, fep / feppalle / the keypads / casttime / eventtim / holdtime 7, itemxinf / itemyinf 6, merityn / comyn /
  ok 8.
- quest00 is a list-base window (vtable 0x1033d7f8). FUN_10222f30 sets its first row child to 1 (+0x54), and
  FUN_102234e0 puts its cursor on 1 + cursor - top. In the Unity modes only children 1 and 2 are shown
  (FUN_10119ff0(0x10, 3 / 4)), with the rects the mode's setup gives them through FUN_10119e10: mode 1
  (FUN_10230eb0) puts their bottoms 0x23 and 0xc above the window's (y 195 / 218 in the 248-tall window), mode 2
  (FUN_10231020) puts them at screen y 100..118 and 130..148 (52 / 82 under the window's top, 48). x and width stay
  the layout's (39, 320).

Ours before: the list was a frame drawn outside the window walk, so the focus stayed on the window under it and its
open moved no pointer; the chat line was outside the walk too; the walk stacked windows by open order only; the Unity
chooser recorded its plate as child 0.

Now:
- `fep::draw_list` (the classic chat line's list; the macro editor's and search comment's lists are the modern
  layer's only, ours, drawn by egui) opens "menu    fep" in
  the walk and records the hand on child 2 + row with that child's rect. Opening the list, entering a category and
  backing out of one ask the open hook's pointer move to the first row (`GameUi::warp_to_part`, paid at the frame's
  end). While typing, "menu    inline" is in the walk (both layers). The modern layer's egui list stays outside the
  walk, as its other egui panels do (no window parts to record or warp to), so the focus stays on the chat line.
- The registry's layer byte is read (`GameUiData.layers`), and a new window of layer 4 or less takes its raise stamp
  as FUN_1015f0e0 places it (`under_high_layers`).
- Several windows may draw a hand in one frame; the focus and the window-code warps take the last hand of their own
  window (`UiBatch.hands`), not the last one drawn.
- The Unity chooser's rows are children 1 / 2 with the retail rects; Ranking Information's rows are drawn at screen
  y 100 / 130.

Not done (new queue rows):
- The mouse on the list (win_fep_onkey 0x10190650): message 9 over a row steps the selection to it, once a frame
  (+0x2201); message 5 passes the child to the list's Enter. Ours has no mouse on the list.
- Typing with the list up: retail asks the dictionary again and opens the list again (menu_fep_fn_7 ->
  FUN_101b84b0 -> FUN_1015f920); ours keeps the old list.
- FUN_1015f920(.., 1, 0) calls FUN_1015fb00 for a new window (state 0xe), which puts the pointer on the focused
  window's cursor before the deferred raise moves the focus (row 185): the chat line's open does that.
- Not checked on screen.

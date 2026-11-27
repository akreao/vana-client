## The mouse on the auto-translate list (2026-11-27)

<!-- dll-build: 30261102_0 -->

Modlog-gaps item 200 (split from 184). Decompiled with Ghidra; win_fep_onkey's message 9 (0x101906ff..0x1019077a),
the FEP object's vtable (0x10338be0) and FUN_101b6990's jump table (0x101b6a14) checked in Rizin.

Retail:
- "menu    fep" is win_fep (vtable 0x10337a38, object DAT_10663f54, FUN_10190410). Its +0x14 is the FEP engine
  (DAT_10663ef8, vtable 0x10338be0): +0xc the command (FUN_101b32d0), +0x1c the selection (FUN_101b4ff0, the
  dictionary list's +0x8c84). Its +0x18 holds the candidates as the update read them, +0x1c their count.
- FUN_10160fd0 runs, each frame: the held step (FUN_1015f360) and the release (FUN_1015f420) on the focused window,
  then the hover (FUN_101267f0, its tail 0x10127220), then every window's update (vtable +0x28, win_fep_v28 for the
  list), and the draw (win_fep_v10) comes after.
- The hover or the held button moving the cursor to child c calls FUN_1011a240(c, 0), which sends message 9 when the
  pointer is seen (mouse +0x48 at 0, +0x4e set). The open hook clears the window's +0x85, so the generic part plays
  no SE.
- win_fep_onkey message 9 on child c, only while +0x2201 is clear: with r = sel % 7 + 2 and s = sel, while r < c and
  s is under the count it sends command 2 and counts r and s up (s does not wrap); then, while r > c, command 1 and
  r down. Then +0x2201 = 1. win_fep_v28 clears it.
- Command 2 (list open, FUN_101b8460) is menu_fep_fn_6(9, 0, 0) -> FUN_101b8670: the next candidate, the first after
  the last. Command 1 is menu_fep_fn_6(9, 1, 0) -> FUN_101b86a0: the previous, the last before the first. So a row past
  the last candidate on the last page steps on past it and the selection wraps to the first page; child 1 (the "n/m"
  line, no caption flag, so not in state 0x10) steps back past the page's first row. The draw hides no child.
- win_fep_v28 plays SE 0001 (FUN_10037b30) when the selection is not the one of its last run (+0x2202).
- win_fep_v10 puts the cursor on the selection's row, FUN_1011a240(sel % 7 + 2, 0). When the cursor was elsewhere (a
  hovered row the steps did not reach) and the pointer is seen, that sends message 9 after the update: it steps
  nothing but sets +0x2201, so the next frame's mouse steps nothing. Held over such a row, the cursor goes back and
  forth each frame and the selection stays.
- The release on the cursor's child is message 5: command 5 -> menu_fep_fn_6(0xd, 1, 0), the list's Enter on the
  selection (the child is read only on the IME path). On another child nothing; on no child SE 0004 (FUN_1015f420).
- A right click is message 6: FUN_101196f0 closes no window with flag 2 set (fep's flags 0x0081000a) and the
  handler has no case 6.
- The held step's frame test reads the frame win_fep_v28 gives the window (FUN_1011ba30: the drawn width, height).
- Commands 3 / 4 / 7 (messages 3 / 4 / 0x12) go to FUN_101b6990, which passes DirectInput codes 0xcb / 0xcd / 0x0f
  to menu_fep_fn_6, whose switch takes virtual-key codes: they do nothing there. Command 8 (message 0xc in Japanese,
  language +8 = 0) is FUN_101b8670 when +0xf0f8 is 1. Outside Japanese message 0xc is message 5 again and 0x11 does
  nothing.

Ours before: the list took no mouse.

Now (`fep::pointer`, `fep::hover_step`, `fep::record`, `fep::ListWin`):
- The classic list records its parts each draw (rows 2..8 at their places, the "n/m" line from the layout, the drawn
  frame). While it has the focus, the hover and the held button (`winhits::takes`) on a child other than the window's
  cursor move the cursor there and step the selection as message 9 does, once between updates; SE 0001 when the
  selection changed. The release on the cursor's child is the list's Enter.
- `ListWin` keeps the window's cursor child (1 at the open, 2 after the open hook, the hovered child, then the
  selection's row at each draw), the step mark (cleared at each draw before the cursor goes back to the row, set
  again when that move sends message 9), and whether the windows saw the pointer this frame.
- The modern layer's egui list and the macro editor's and search comment's lists (ours) take no mouse here.

Not done (new queue rows):
- SE 0001 from win_fep_v28 on any selection change: ours plays it on each list key, also when the selection stays
  (one candidate), and not for the selection a category's list or backing out of one leaves.
- The pad on the list (messages 1 / 2: commands 2 / 1 unless the arrow key itself is held, FUN_10008830(0xd0 / 0xc8);
  0xc outside Japanese: Enter; 0x1e: Escape).
- Not checked on screen.

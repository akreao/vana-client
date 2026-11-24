## The frame pass's raise and the pointer on a window shown (2026-11-24)

<!-- dll-build: 30261102_0 -->

Modlog-gaps item 185 ("the deferred raise", split from 177, with the chat line's open from 184). Decompiled with
Ghidra; the raise loop (0x10161190..0x10161248), FUN_10126000 (0x10126000..0x10126043), FUN_10125b90's glide test
(0x10125baf..0x10125bb5, DAT_1032a5d8 = 0.0) and every call of FUN_1015f150 with its pushes checked in Rizin /
objdump.

Retail:
- raise_focus_next_frame FUN_1015f150(manager, window, take) marks a window: +0x71 = the manager's count +0x60 (then
  +1), +0x98 = take. 162 calls in 129 functions; `take` is 1 at two of them (0x10204572, 0x10204e9d), 0 at the rest.
- FUN_10160fd0 (the frame pass, after the windows' code) goes through the marks in order 1 .. count - 1, then resets the
  count to 1. Each marked window (skipped when +0x6e is set) is taken out of the window list (FUN_1015f1a0) and put back
  by FUN_1015f0e0 (on top, or for layer 4 or less just above the first low window under the high-layer ones on top),
  +0x71 cleared. When its registry bit 0 is clear or +0x98 is set, FUN_1015fe20 gives it the focus and FUN_1011c080 /
  FUN_10125b90(centre, 0.0) put the pointer on its cursor part's centre, also when it had the focus already. After the
  loop FUN_1015fdb0 -> FUN_1015fe20 gives the focus to the top window that can take it, as before.
- FUN_1015fe20 on a change: the old focus's +0x98 cleared, the new one's parts lit (FUN_1011a690(1)), FUN_10126000.
- FUN_10126000: +0x48 = 18.0 first, then, when the focus (+0x54) is up (FUN_10160380: not state 0xe, not suspended),
  the pointer on its cursor part with FUN_10125b90(centre, 0.0).
- FUN_10125b90(at, glide): +4 / +8 = at; +0x48 = glide only when glide is not 0.0 (DAT_1032a5d8). So the raise's own
  warp keeps the glide FUN_10126000 just set on a focus change, and keeps whatever was left when the window had the
  focus already (0 after a finished glide: the pointer jumps).
- FUN_1015f920(name, show, defer), a new window: state 0xe, layer placement (FUN_1015f0e0); with `show`,
  FUN_1015fb00 (state 0, open hooks, FUN_10126000: +0x48 = 18 and the pointer on the cursor of the window that has the
  focus then, the old one), then raise_focus_next_frame(window, 0). An already open window gets the raise only (vtable
  +4 / +0x14, no FUN_10126000).
- The chat line (FUN_101b5590 -> FUN_1015f920("menu    inline", 1, 0)): the pointer goes to the old focus's cursor
  with the glide; the raise gives inline the focus (registry 0xa, bit 0 clear) and finds no cursor part, so nothing more.
- The hand-over (win_handover_onkey 0x1013ae40): Enter on a slot (children 1..8) raises the inventory list
  (DAT_106648f8 + 8); the list's handler for it (FUN_1013abf0) raises the hand-over again on Escape, and itemctrl's
  callback FUN_1013abb0 raises it once the item is in.

Ours had no raise: windows stacked by the order they opened in, a window took the focus only by opening on top, and
a window shown moved the pointer only through the focus change.

Now:
- `GameUi::raise(name, take)` marks a window; `end_frame` raises the marks in order after the windows' code
  (`raise_in`: the walk stamp moved as FUN_1015f0e0 places it, +0x98 kept in `Walk::take`, the focus and the pointer
  on its hand's centre, owed until the hand is drawn on a focus change as before). The walk's focus pick
  (FUN_1015fdb0) counts +0x98.
- A window shown this frame (new to the walk, or opened again in place with `renew`) gives FUN_10126000 on the old
  focus: +0x48 = 18 and the pointer on that focus's cursor when it is still up.
- The frame's pointer moves fold in retail order (shown windows, the window code's cursor warps, the raises, the focus
  step) into one `PtrMove` (where the pointer ends, and whether +0x48 was set to 18 on the way). The app applies it:
  with no position, only the hold (+0x48 = 18 with the pointer where it is); without FUN_10126000, the glide left as
  it was (`PtrWarp::with_glide`). During a drag FUN_10125b90 does nothing, but +0x48 is still set.
- The hand-over raises the inventory list on Enter on a slot and itself on the list's Escape and on an item placed
  (one at once, or the count from itemctrl): the focus and the pointer move between them as in retail.

Not done:
- The other raise sites (queue row 202): 160 calls in the menus that raise one another (equipment, inspect, loot,
  persona, delivery, string input, player trade, macros, item sort / storage, blue magic, abilities, magic, emotes,
  automaton, map, shop, the lobby lists ...). List: re-notes/pl185-notes/callers-named.txt.
- FUN_1015f920 on a window that is already open (a raise with no FUN_10126000) and windows opened hidden (state 0xe)
  and shown later by FUN_1015fb00: ours has no open call and no state 0xe (queue row 203).
- Not checked live.

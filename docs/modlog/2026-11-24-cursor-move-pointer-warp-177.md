## The pointer put on the cursor by window code (2026-11-24)

<!-- dll-build: 30261102_0 -->

Modlog-gaps item 177 ("Cursor moves that warp the pointer from window code", split from 163). Decompiled with
Ghidra; the call sites' arguments, FUN_1011a240's warp tail (0x1011a2ef..0x1011a30b), the title's FUN_10126000 calls
(0x10241cac, 0x102468e1) and the deferred raise (0x101611fa..0x10161234) checked in Rizin.

Retail:
- FUN_1011a240(window, child, warp) is the window's cursor setter with a pointer: clamped to 1..child count, and only
  when the child changes, the old part leaves its hover state and the new one takes it (+0x4c = child); with warp
  set, FUN_1011c080 finds the new cursor part's centre and FUN_10125b90(centre, 18.0) puts the game's pointer there
  with +0x48 = 18 (the glide a focus change gives, row 163). Of its 191 call sites (183 decompiled with arguments,
  the register ones traced to 0) seven pass warp = 1:
  - win_handover_onkey 0x1013ae40 (the NPC hand-over), message 6 (Escape, or the right click through
    FUN_1015f4c0): the cursor to Cancel (10). With the cursor already there nothing happens; the window's registry
    flags 0x10202 have bit 2, so the generic handler (FUN_101196f0 case 6) does not close it either: Escape never
    closes the hand-over, only Enter on Cancel does (case 5 child 10 -> FUN_1013b210).
  - win_trade_onkey 0x10152bb0 (player trade, "menu    trade"): message 6 the cursor to Cancel (10); after our OK
    (+0xc0 = 4, when win_trade_v28 0x10152e80 clears +0x61 so the generic handler moves no cursor) message 1 (Down)
    on OK to Cancel and message 2 (Up) on Cancel to OK, each with 0001, and Enter on any child but OK / Cancel to OK
    (9).
  - win_trade_v28 (its per-frame hook) after our OK: a cursor on neither OK nor Cancel goes to OK.
  - win_fep_v04 0x10190500 (the auto-translate candidate list's open): child 2. That list is not a window in ours
    (queue row 184), so this one stays with it.
- The keys' messages (FUN_101acdf0 -> FUN_1015f2d0 -> FUN_101196f0): key function 0x2c (Down) 1, 0x2b (Up) 2, 0x2d
  (Right) 3, 0x2e (Left) 4; the generic handler moves the cursor by the layout's link at element +0x18 / +0x17 /
  +0x19 / +0x1a (ours links[3] / [2] / [4] / [5]).
- loby2win's handler FUN_10241c00 (the title buttons, holder DAT_1066a208), message 6 (unless a press is on, +0x1c):
  the cursor to Back (index 3, child 4), the help line, FUN_10126000 (+0x48 = 18 and the pointer on the focused
  window's cursor, loby2win being the focused one: registry 0x1002e, bit 0 clear), 0001. Unlike FUN_1011a240 this
  warps with the cursor already on Back too.
- loby1win's FUN_10246840 does the same with its own list (index 2, child 3), but the only function that opens that
  list, FUN_10246920, has no caller in this build (no call, no pointer to it): dead code.

Ours had none of these warps. Now `GameUi::warp_to_cursor(window, child)` owes the pointer to that child, paid at the
frame's end once the window draws its hand there (its rect's centre, as FUN_1011c080), dropped when that window draws
no hand; a focus change in the same frame wins (FUN_10160fd0 runs after the window code). The app applies it as the
focus warp (client/ptrwarp.rs: the glide, the hit test held off, the Windows cursor moved under config 0021).
- The hand-over: Escape puts the cursor and the pointer on Cancel and no longer closes the window when the cursor is
  already there (a deviation: ours closed on the second Escape).
- Player trade: Escape, the Down / Up steps after our OK, and Enter off OK / Cancel after our OK warp as retail. The
  Down / Up steps no longer play 0001 when nothing moves (Down on Cancel, Up on OK). win_trade_v28's step cannot
  happen in ours: our OK is only taken with the cursor on OK, and after it the mouse only reaches OK and Cancel.
- The title: Escape puts the pointer on Back as well as the cursor.

Not done (queue row 185):
- FUN_10160fd0's deferred raise (windows marked by raise_focus_next_frame FUN_1015f150, 186 call sites): the window
  is raised, takes the focus when it can, and FUN_10125b90(centre, 0) puts the pointer on its cursor without a glide,
  also when it already had the focus. Ours has no deferred raise (windows stack by their open order).
- FUN_1015fb00's FUN_10126000 when a window in state 0xe (hidden) is shown again; ours does not model state 0xe.
- Not checked live.

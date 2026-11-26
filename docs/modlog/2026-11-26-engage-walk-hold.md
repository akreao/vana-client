## The walk waits for the draw and the sheathe; the engage waits for the holds (2026-11-26)

<!-- dll-build: 30261102_0 -->

Retail stops our walk for seconds when a monster's spell lands on us around the engage; ours walked on. Addresses
are 30261102_0, decompiled with Ghidra.

- The walk gate FUN_100964f0 (from FUN_100a6240) refuses movement while FUN_1008b3a0 says the entity's machine
  (FUN_1008b460, state byte +0x1bc) is busy: states 1, 2, 4..8, 0xf, 0x10, 0x14..0x1a. For battle that is state 1
  (the applied status became 1; the battle files are asked for and the machine waits for FUN_100d12d0, every file
  of the actor in), state 2 (the frame "in N" starts; vtable +0x31c(1, 0) and state 3 come with the next call),
  state 0x11 / 6 (the status left 1 / 3: `bntg`, then the sheathe next frame) and state 5 (sheathing until
  FUN_100d12d0 and no `waot` on the actor, when FUN_100d44b0(1) takes the files off the list). State 3, engaged and
  drawn, does not hold. FUN_100a6240 returns before the step and the turn, so the keys do nothing meanwhile.
- FUN_1008e9a0 runs the machine twice a frame; at its top (0x1008b4a6) it returns 0 while the entity's busy count
  +0x11c (FUN_10083320) is up, and the applied status +0x170 takes the server's +0x16c only when it returned non-zero.
  Every hold a monster puts on its target raises the count (ops 0x20 / 0x56 through FUN_100601f0 -> FUN_100a4140,
  and the category 3..6 loads), so while a spell's `main` holds us: an engage not yet applied waits (state 0: the walk
  is free), and an engage already in state 1 stays in state 1 (no draw, no walk) until the holds are over.
- Retail capture (Escha, a mob's spell landing within about 3 s of an engage, a move key held): the c2s 0x015
  position and facing stop about 1 s after the engage and stay for 6.1 / 6.5 s; one ends about 1.8 s after the
  disengage (the sheathe). A mob casting at us with no engage: the walk never stops, through the cast and its landing.
- Ours: actmotion::battle_walk_held (Battle.drew for state 2) joins the walk lock like ride::dismounting (not behind
  the move_lock rule: it is the machine, not scheduler op 0x2e); the battle machine's steps (draw, `bntg` -> sheathe,
  the files off the list, the sheathe's end) wait while combat::held is up; and our own engage is parked behind the
  holds like every other status change, which drops the 2026-11-14 exception ("Our engage is never held"): with a
  monster shooting at us our first rounds can land before the draw, as retail's do.
- State 2 goes to 3 on the next machine call that runs, so the draw's own hold (its op 0x1f) keeps the walk held
  through it (Battle::machine_call). The gate's refusal returns FUN_100a6240 before the heading, so Q / E do not turn
  us under the machine's hold either (ride::dismounting included).
- Tests: battle_states_hold_walk, draw_state_waits_out_the_holds.

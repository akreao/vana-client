## Effect tasks: retail's one task list and its walk (2026-11-24)

<!-- dll-build: 30261102_0 -->

Modlog-gaps item 172 (split from 160): a task runs twice in a frame where it starts a scheduler whose first update
makes any task.

- The list. Every task is a node of one doubly linked list (manager DAT_1047e2f0: head +0xd50, running task +0xd4c,
  the walk's next +0xd5c, 16 anchor tasks at +0xd04 + 4 x bucket). The walk FUN_10074c30 takes the running task's
  next (+0xc) into +0xd5c before its update (+4, FUN_10075000 -> vtable +0x20, skipped while +0x22 holds it) and goes
  on from +0xd5c after it.
- Links. The base constructor (FUN_10074740 / FUN_10074790 -> FUN_100749f0 -> FUN_100750b0) puts a new task right
  after the running task and sets +0xd5c to it; with none running, at the front of bucket 9 (FUN_10074f80). A
  scheduler (FUN_10057270 -> FUN_10074a00 -> FUN_10075120) and a generator's task (FUN_10053370, FUN_100531d0) are then
  moved to right before the running task (before anchor 9 with none running); a bucket task (FUN_100749b0 /
  FUN_10075220: elements, op 0xa6's fade FUN_10035e30) to the front of its bucket. The unlink FUN_100751b0 moves +0xd5c
  on to the task's next when it pointed at it, and +0xd4c back to the previous one. A copy (FUN_10074920, ops 0x52 /
  0x64 / 0x65) is only linked after the running task.
- The first update. A scheduler started by an op runs its first update at once (FUN_10056b20 / b80 / bb0 / c70 ->
  FUN_100748c0), which makes it the running task and puts the old one back, but not +0xd5c. A task made in that update
  is linked after the new scheduler, so before the task T the walk is in, and +0xd5c is left on it (a moved one leaves
  +0xd5c on the scheduler's next, T or a task before T). When T returns the walk reaches T again: its update runs a
  second time with the frame's ticks. If T makes a task of its own after the start, +0xd5c is back after T and T runs
  once. Nested starts run every scheduler on the way again (T starts S, S starts S2 whose first update makes a task:
  that task, S, then T).
- Ours (`fx/tasklist.rs`): the schedulers are kept in retail's list order with the anchors; the walk follows the list
  with the running task and the walk's next as retail's. Spawned schedulers, copies and first updates link and run as
  above. The other tasks keep their own passes; an op that makes one adds a node for the rest of the walk, after the
  running task or moved: holds (0x07 / 0x08 / 0x1f / 0x20 / 0x56 / 0x59), drives (0x26 / 0x27, 0x62, 0xa9 / 0xaa),
  door and lift moves (0x0c / 0x0d, 0x1d), sounds (0x0a / 0x0b / 0x4a / 0x53 / 0xb3, 0x78), colour, shine and
  refraction drives (0x29 / 0x2a / 0xa0 / 0xa1, 0x40..0x49 / 0x54 / 0x55), light ramps (0x6f / 0x70), screen ops
  (0x0e / 0x0f / 0x51 / 0x72, 0x10), the colour lock (0x88), after-images (0x2c), blink, look, plate, step and draw holds
  (0xb5, 0x89, 0xb4, 0xbc, 0x9b), weapon timers (0x75), roots and locks (0x2e / 0x2f), pins (0x7a), delayed pushes
  (0xbf), dolls (0x15 / 0x16 / 0x22 / 0x23); generators (0x02, 0x1e / 0x2d, 0x3f), op 0x80's waiting task and op
  0xa6's fade are moved.
- The walk no longer stamps every task it updates; only a first update outside the walk (a start asked for before
  it) still makes that walk pass the task. The dying child no longer steps its parent itself: the parent is after it
  in the list.
- Ops 0x30, 0x5c, 0x76 / 0x77 and 0x84 start their scheduler with FUN_10056bb0, which runs its first update at once;
  ours left it to the walk (a later slot's walk in the same frame, an earlier slot's the next frame). Each now runs it
  at once, op 0x30 once per target in record order.

Not modelled (new queue row 181):

- The entity loop task (vtable 0x1032acfc, +0x20 = FUN_100106c0 -> FUN_10096640) runs every actor's update; a start
  inside it whose first update makes a task runs every actor's update twice that frame. Direct paths: the fade
  schedulers `fdo0` / `fdi0` of file 0x78b8 (FUN_1008abe0 -> FUN_10062e20 -> FUN_10056c70), the weather change
  (FUN_10094190 -> FUN_10188ac0 -> FUN_1018bd70 -> FUN_10056b80), a model's remake (FUN_10095900 -> FUN_1008f740 ->
  FUN_10097a80 -> FUN_100c4730 -> FUN_10056b20); virtual calls not traced. Ours starts those outside the walk.
- Tasks retail's ops make that ours has no object for: 0x04 / 0x81's camera task outside an event (FUN_1003c8a0 ->
  FUN_1003cc80; none when the camera object's +0x70 bit count is 1 and the op's second argument 0; its place in the
  list is modelled for every scheduler, the +0x70 count read from the path since row 181), 0x82 / 0x83
  (FUN_10074790), 0x1a (FUN_10085e40 with DAT_1047e840 set),
  the schedulers 0x2b / 0x71 start for a message (FUN_1009e090) and 0x75 / 0xa3 for a weapon (FUN_100d5f10 /
  FUN_100d60e0), the weather generators of 0x7b / 0x7c / 0x7e / 0xae (ours applies them after the walk), 0x7f
  (FUN_100613d0).

Tests: `fx::tasklist` (the walk, links and unlinks), `parent_runs_twice_when_its_child_makes_a_task`. With DATs the
timings move: `fist_hit_sparks` frame 20 -> 18, `skill_files_dropped_at_main_end` 72 / 101 -> 71 / 100, Selbina's
`pat2` further ahead of the clock (t 2098 at z -84.729), Port Jeuno's `seq0` ends one frame short of the berth point
(-68.001, 10.45, 117.012), `seq7`'s `ptv2` has its last frame at k 1467. Not checked live; a frame-counted retail clip
of a nested effect would confirm.

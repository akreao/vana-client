## Effect tasks: generators' tasks and op 0x75's timers in the walk (2026-11-27)

<!-- dll-build: 30261102_0 -->

Modlog-gaps item 191 (split from 181): a generator's task and op 0x75's timed hide are nodes of retail's one task
list, so where they sit decides whether the walk reaches them in the frame they are made, and in which order.

- A generator's task (vtable 0x1032c654; +0x20 0x10042640 returns 0, so +0x24 FUN_10054220 -> FUN_100496f0(0), the
  emissions) is made by FUN_100531d0 or FUN_10053370. Both delete the generator's old task first (+0xa8, vtable +0x18),
  then build the new one (FUN_10074740: after the running task, the walk's next; with none running, the front of
  bucket 9) and move it: FUN_100531d0 (0x10053318..0x10053348) puts the attached kinds at the front of bucket 9
  (FUN_100749b0(9)) and the unattached ones (attach modes 0, 0xe, 0xf, 0x14) before the running task (FUN_10074a00 ->
  FUN_10075120, before anchor 9 with none running); FUN_10053370 always moves it before the running task.
- Who calls which. FUN_100531d0: the clone constructor FUN_100536c0 when its fourth argument is 1 (op 0x02 at
  0x10059b10, op 0x3f's clone at 0x10059fa0, FUN_100d63f0 / FUN_100d64d0, FUN_10085060), FUN_100856f0 directly (a
  cast's generators, after a clone with the argument 0), op 0x02's restart of an unattached static template (0x10059a0f), and the weather objects' generators
  (FUN_1018bd70 -> FUN_10072540(5, 0, FUN_1018c020), and FUN_1018c0b0 leaving a roof). FUN_10053370: FUN_100538e0 (op
  0x3f's bursts into a target, one per element of the source) and the end stream's op 1 (FUN_1004e720). An element's
  clone (FUN_1004e8d0 0x1004ee86) is made with the fourth argument 0: no task, it runs inside its element's update.
- So a generator a scheduler starts inside the walk is reached in that walk only when it is attached and the scheduler
  is before bucket 9; an unattached one sits before the scheduler, behind the walk, and makes nothing until the next
  walk. From the entity loop's task (bucket 13) both are behind the walk, and the unattached one is past bucket 0xb's
  anchor, so the elements it makes go to the front of bucket 0xb behind the walk: their first update is the next frame.
- Op 0x75's timed hide (FUN_10061960, vtable 0x1032ce08) is a child left after the running scheduler (FUN_10074790).
  Its update FUN_100619d0 takes the frame's ticks (DAT_1047d158 +0xeb0) and below 0.0 (DAT_1032a5d8) deletes itself.
  The destructor FUN_10061920, with the hide argument 0, shows the weapon: FUN_100d5f10(n, 0) clears the hide byte and
  starts the glow (FUN_100d5fe0 while engaged and not invisible, else FUN_100d6180) with the timer still the running
  task, then unlinks it (FUN_100747c0). The scheduler's end deletes its children, the timer with them, inside its own
  update.

Ours:

- `tasklist::Node::Gen` (by the instance's start serial) and `Node::Timer` stay in the list for good
  (`TaskList::new_generator`, `GenLink::Start { attached }` / `GenLink::Before`, `new_timer`). `Effects::start` links
  every instance by FUN_100531d0's rule; `restart` links again; `emit_once` with its count reset (FUN_10053370) moves it
  before the running task; an element's clone and a resource that only bursts have none (`unlink_gen`).
- The walk reaches a generator's node (`visit_gen`): its update is owed from then (`Instance::due`; one linked behind
  the walk makes nothing until the next walk reaches it), its place in the walk is kept (`Instance::rank`), and
  whether it is past bucket 0xb's anchor (`Instance::past_elems`).
- Each element keeps where the walk was when retail made it (`ElemCpu::rank`: its generator's node, the running
  scheduler for an op's emission, bucket 0xb's anchor for one made inside an element's update) and `run_order` sorts by
  it. This replaces the inferred key by attach mode and start order. An element made from a node past bucket 0xb is
  `late` (no life step, no kick, the GPU's first decrement skipped in the frame it is made).
- Op 0x75's timer counts down at its node with the walk's ticks (`walk_timer`, was every tick of ours after the walk)
  and ends there; a scheduler that ends in its update ends its timers there too (`settle` -> `end_timers_of`). The
  show starts the glow inside fx (`glow_in_op`), so the client no longer makes that call (`WeaponOp::glow` is gone).
  A timer whose scheduler was deleted outside an update (client kills carry no actors) ends at the next walk's start,
  with nothing running, as a delete at packet time would.
- Pending starts asked before the walk: retail asks them at packet time, after the walk (FUN_100daf30 after
  menu_chdummy_fn), with nothing running; the links and first updates are the same as ours before the next walk. What
  differs is where the actor pass runs (row 214).

Split off (new queue rows): 223 the elements as nodes of their own (each element's update in its place in bucket 0xb,
against tasks in buckets 12..15 and generators past bucket 0xb; FUN_10053370 inside an element's update puts the task
before that element, ours at the front of bucket 0xb), 224 a generator's cadence (retail once a frame in the walk with
the frame's ticks, ours every tick of ours; a generator deleted later in the walk was updated that frame in retail), 225
start sites whose retail link is not traced (the zone's start-at-load generators, the model generators, footstep dust,
prints and ripples; ours links them by FUN_100531d0's rule).

Tests: `generator_tasks_in_the_walk`, `timed_hide_ends_in_the_walk`, `elements_run_newest_first`. Not checked live.

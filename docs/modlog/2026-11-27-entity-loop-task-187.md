## Effect tasks: the entity loop's task in the walk, and the starts made inside it (2026-11-27)

<!-- dll-build: 30261102_0 -->

Modlog-gaps item 187 (split from 181): retail's actor pass is itself a task of the one task list, so a scheduler it
starts is linked beside it and takes its first update there. Ours started all of them before the walk.

- The task. `menu_chdummy_fn_6` (0x100142b0) makes it when the game is entered: FUN_10028fd0(0x34), base constructor
  FUN_10074740 (FUN_100750b0: after the running task), vtable 0x1032acfc, kept in DAT_10457abc, then FUN_100749b0(0xd)
  -> FUN_10074f80: taken out and put right after anchor 13 (+0xd04 + 4 x 13), the front of bucket 13. Its update
  (+0x20, FUN_100106c0) runs while DAT_10457aac +9 is set (menu_chdummy_fn 0x10011a00 clears it while the connection
  is lost, DAT_103d4db8 +0x28 = 0): FUN_10096640, then FUN_101d5ff0 with DAT_1066438c or FUN_101d6940 with
  DAT_10664390. It is deleted through vtable +0x18 at 0x100108b0, 0x10014350 and 0x10014ad0.
- The frame. FUN_10001b50 calls menu_chdummy_fn (0x10011a00), which walks the list (FUN_10074c30 at 0x10011ca7),
  and only then FUN_100daf30, which dispatches the packets (-> FUN_100fa710, the handler table DAT_104e0f44 +
  0x44364). So a packet's start has no running task (the scheduler goes before anchor 9, FUN_10075120's first
  branch), and a start of the actor pass has the loop's task running: FUN_10056ce0 -> FUN_10057270 links the
  scheduler right before it (FUN_10075120's second branch), in bucket 13, and FUN_10056b20 / b80 / bb0 / c70 run its
  first update at once (FUN_100748c0).
- The second run. FUN_100750b0 leaves the walk's next (+0xd5c) on every task it links, and FUN_100748c0 does not put
  it back. A task made in the new scheduler's first update is linked after that scheduler, so before the loop's
  task: when FUN_100106c0 returns the walk goes there and reaches the loop's task again, and FUN_10096640 runs a
  second time in the frame with the same ticks (FUN_10014cf0: every actor's FUN_1008e9a0, the +0x11e and +0x1fc
  counts, DAT_10480cec). Any task the loop makes afterwards puts +0xd5c back behind it: a later scheduler
  (FUN_10075120's unlink moves +0xd5c on to the loop task's next), a model's task (FUN_1008f740 -> FUN_100c56e0 /
  FUN_100c5880 / FUN_100ac8e0 -> FUN_100749b0), a file's loader (FUN_1008b460 -> FUN_100d14c0 -> FUN_100710f0 /
  FUN_100730e0), a cast's generator (FUN_1008e4d0 -> FUN_10085a10 -> FUN_100856f0 -> FUN_100531d0), the music change
  (FUN_10093e50 -> FUN_10037350).
- The starts inside FUN_10096640, by direct calls (the callers of FUN_10056b20 / b80 / c70 in its call closure):
  - FUN_10094190 (the zone's state): the weather change FUN_10188ac0 -> FUN_1018bd70 (0x1018beea, FUN_10056b80 for
    every self-starting scheduler of the `weat` folder); a model made, FUN_1008f740: FUN_100ad060 / FUN_100ad130
    (a door's `ints` / `inte`), FUN_100c3cf0, FUN_100c41f0 -> FUN_100c46b0 (the model file's self-starting
    schedulers), FUN_1008ef60 (`ini<n>`, vtable +0x298), FUN_10097a80 (a model actor's sequence).
  - FUN_1008abe0: FUN_10062e20 (0x10062e6d), the zone fades `fdo0` / `fdi0`.
  - FUN_1008e9a0 for every actor: FUN_1008c480 (`init`, `inte` on status 0x22, `hen0` through vtable +0x298 =
    FUN_100cee40 -> FUN_10056c70; `!in?` through FUN_100cf060 -> FUN_100cef00); FUN_1008c6c0 (the status change:
    `bton`, `ntob`, the status' name (`btid`, `dead`, `sdep`, `idle`, `fsh?`...), `ntoe` / `eton`, FUN_100ad1e0 /
    FUN_100ad270 for a door, FUN_10097a80 -> FUN_100c4730 for a model actor (the sequence names at 0x1032e42c;
    FUN_10056b20 when it is 600 ticks or more in); FUN_1008b460 (the actor's machine: `res0` / `res2`, `sit0` / `sit2`, `chi2`, `coff`, `bntg`, `klbt`, `hwat`,
    the draw and the sheathe, FUN_100c3d50); FUN_100bcd10 -> FUN_100bc280 (the event's ops for a bound actor).
  - FUN_10095900 (the models due, after the actors): FUN_1008f740 again.
- Ours. `fx/tasklist.rs` has the loop's task as a node at the front of bucket 13. `Effects::in_loop` marks the starts
  asked inside it as the loop's; the walk runs them at that node with it as the running task, in the order asked, so
  each scheduler sits right before the loop's task and is walked after the schedulers of bucket 9 from then on. The
  walk follows its next as before, so the node is reached again after a start whose first update made a task
  (`Effects::loop_runs`, a debug line). The client marks: `init`, `inte` and `ini<n>` (update_movement), the status
  change's schedulers (`combat::status_schedulers`), the machine's `klbt` / `bntg` / `hwat`, draw and sheathe
  (`actmotion`), `res0` / `res2` (`rest`), `chi0` / `chi2` (`chair`), `coff` (`ride`) and a model actor's sequence (`ships`).

Not built (new queue rows):

- 211: the second run itself. The client's actor pass (update_movement) is not run again with the retail frame's
  ticks; it first needs the loop's other tasks as nodes in the order retail makes them (the model tasks of
  FUN_1008f740, the loaders of FUN_100d14c0, a cast's generators, the music change), or ours would run twice where
  retail's later task put the walk's next back. The order of the loop's starts is the order ours asks them in, not
  retail's (FUN_10094190, FUN_1008abe0, the actors by slot, FUN_10095900).
- 212: the loop's starts ours makes elsewhere: the zone fades (`fdo0` / `fdi0` of 30904; ours draws the fade apart),
  the weather change's schedulers (`set_weather` starts them when asked, with no first update), a door's `ints` /
  `inte` and open / close, a model file's self-starting schedulers, `!in?` and the glows of FUN_100cf100 / FUN_100cf160
  (the equipment calls), the event ops' schedulers (the event's ops run inside FUN_1008e9a0), `hen0`, the fishing
  statuses' `fsh?`, FUN_100c3cf0 / FUN_100c3d50.
- 213: tasks that start schedulers from their own update, which ours starts before the walk: the 0x028 wait task
  (vtable 0x1032f7c0, FUN_100a9500: `main` once the caster and targets are in; made by the packet, so at the front
  of bucket 9) and a model's own task (FUN_100c63d0 -> FUN_100d1070: `spop`).
- 214: where ours' actor pass runs. Retail's is inside the walk, after the schedulers of buckets 0 to 12 and before
  the packets; ours (update_movement) runs before the walk, every frame of ours.

Tests: `fx::tasklist` (`entity_loop_starts_go_before_it_in_bucket_13`,
`entity_loop_runs_again_after_a_start_that_makes_a_task`), `entity_loop_starts_run_at_its_task`. Not checked live.

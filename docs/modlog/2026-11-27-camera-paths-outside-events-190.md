## Camera paths outside events: op 0x04 / 0x81 moves the game camera (2026-11-27)

<!-- dll-build: 30261102_0 -->

Modlog-gaps item 190 (split from 181). Ours played an op 0x04 camera path only for an event's tasks; retail's op
makes the one camera task in any scheduler, and that task writes the game camera.

- Op 0x04 / 0x81 (case at 0x1005a03c): the path (kind 6, the 4cc @8) must be found (FUN_10071060, else nothing,
  0x1005a121); FUN_1003c8a0 gets actor1, actor2 and trunc(dur x speed) (`_ftol` at 0x1005a090). It first forces every
  actor's next update (FUN_100852f0 / FUN_10085310 walk DAT_1047e728 calling vtable +0x18c / +0x190 =
  FUN_100a4b30 / FUN_100a4b40: actor +0x9f8, the screen test, and +0xa04, the pose, both read by FUN_100cbde0),
  deletes the last camera task (DAT_1047cb30), then makes the new one (FUN_1003cc80, CMoCameraTask, vtable
  0x1032bb60) as the scheduler's child (FUN_100747d0), unless the path counts one camera and the ticks are 0
  (FUN_1003c970 sets the camera at once).
- The task's update (FUN_1003d220) writes the camera object (DAT_10457aac +0x50, FUN_10015250): eye +0x8c with
  flag +0x88, look +0x98 with +0x89, roll +0xa4 with +0x8a, and the focal through FUN_10015290 (+0x2f8). The frame's
  end (FUN_10015510 from 0x10011a00, after the walk and the draw) applies them (FUN_10021900: FUN_1001e760 eye,
  FUN_1001e780 look only when off the eye, +0x5c roll; +0x2f8 -> +0x2f4 focal) over what the follow camera
  (FUN_1001ee60, in the actor pass) did that frame. Nothing reads DAT_1047cb30 but FUN_1003c8a0 and the task's
  destructor, so nothing else holds the follow camera off: it goes on from the path's last pose the next frame, and
  the focal and roll stay until the zoom keys or a scene load (FUN_1001e4f0: roll 0, focal 350).
- A task's destructor deletes its children (FUN_10074ad0 -> FUN_10074890), so the scheduler's end or kill deletes
  the camera task where it is.
- Op 0xb0 (0x1005a12c -> FUN_1003cbb0) deletes the camera task playing the path it names from any scheduler, not
  only an event's.

Built:

- fx/tasks.rs: op 0x04 / 0x81 asks for the camera task in every task but an event's own scheduler (the runner plays
  that one), only when the path is found, with trunc(dur x speed) ticks, the path's own file and the task's uid
  (`EventCamera::owner`); `Effects::camera_cuts` counts the ops (an event's own scheduler's too), and the client
  forces every actor's next pose (`Actor::anim_force`, +0xa04). `Effects::task_live`.
- fx/tasks.rs: op 0xb0 ends the path in every task but an event's own scheduler, in the walk's order with the starts
  (`CameraOp::Start` / `Stop`, `Screen::fx_stop`).
- fx/tasks.rs: only the event runner's scheduler (`on_runner_scheduler`, client/event.rs's 0x45 family) leaves its
  camera ops and fades to the runner; 0x039 map schedulers, battle-message files, event op 0x7d's file and fishing's
  start through `on_event_scheduler` as any scheduler (their fades had gone nowhere).
- client/event.rs: the runner's own shot ends with the runner's scheduler too (`advance`, by file, tag and actors);
  a written pose carries its focal in first person as well.
- client/event.rs: an event's end keeps the camera task (FUN_100ae3f0 deletes none; only op 0x46 does, FUN_1003d5c0).
- client/event.rs: a shot whose scheduler task is gone ends where it is (`camera_frame`, `Shot::owner`; this also
  holds for an event's sub-schedulers, which ours let run on). `Screen::game_pose` / `game_camera`: outside an event
  the live shot's pose goes over the game camera after the follow camera's step, the last pose once at its time, then
  nothing; a shot that ends inside an event writes nothing after it.
- client/camera.rs: `set_pose` (eye, look unless on the eye, `State::roll`, the focal held while config 0x49 is
  unchanged, `State::held`), `view` draws the roll (`up_of`) and the written pose without a mod's distance scale,
  `scene_reset` at a zone load drops both. The zoom clamps 242..900 only what its keys step, not a path's focal.

Not built: op 0x10's crossfade outside events (FUN_1005f020 in any scheduler; ours only for an event's tasks), new
queue row 222. Actor +0x9f8 (FUN_100cbde0 tests whether an actor is on screen one frame in four; ours tests every frame,
so there is nothing to force). The frame lag of an effect task's path (asked in the walk, started at the next
`advance`) is the one the event's sub-scheduler paths already had.

Tests: `fx::tasks::tests::camera_op_outside_an_event`, `client::event::tests::camera_task_outside_an_event`. Not
checked live.

<!-- dll-build: 30261102_0 -->
## B / Shift held (Compact 2): the body still turns to the stick, eased (2026-11-17)

- Report: on retail with Keyboard Compact 2 (Config > Misc. 2, config 0x42 = 2), holding the pad's B (slot 3, key
  function 0x9c with LShift) keeps the camera's angle, and the character still turns toward the stick. Full and
  Compact 1 do nothing with B. Ours strafed: the facing stayed put and mvl / mvr / mvb played for as long as B was held.
- What 0x9c does is unchanged from "Gamepad: B / Shift held is a strafe lock" (2026-11-04): FUN_100a6240 sets actor
  +0xfb (vt+0x344(1)), saves the camera offset (FUN_1025f6a0) and returns before its turn to the move direction; the
  step's quarter +0x598 (FUN_100a80d0) and FUN_100a8000's speed are taken against the heading +0x39 / +0xe8.
- What that note missed is the turn after it. The actor update (FUN_100c5d60, 0x100c6821 .. 0x100c6c29) for our own
  actor (vt+0x304: handle 0x1047e7b0 is this), free (vt+0x330) and not riding takes the same path with or without
  +0xfb: the drawn rotation +0x61c eases 0.125 a tick (a scheduler's rate when one is set) toward the drawn step's
  direction (vt+0x1bc = +0x5fc easing 1/8 a tick toward the moved place +0x5c4) when it moved 0.01 or more, else the
  heading, and outside first person copies it to the model's +0x44. FUN_100a57f0 starts the next frame with heading =
  +0x44. So under +0xfb the heading follows the step with the drawn ease: from a stand a sideways push sidesteps for
  about 5 ticks and a backward one backpedals for about 10, then the run takes over facing the stick. Riding
  (FUN_10084350 / 70), the drawn rotation eases toward the heading instead, so a chocobo does hold its facing.
- Ours: update_movement's face_cam facing eases toward the step at 0.125 a tick (powf, as World::my_draw_yaw does) and
  is the drawn yaw as well; World::my_strafe under it is the quarter against the facing before this frame's ease
  (lock_dir). The camera hold is unchanged.
- Not retail: the drawn-step target is our logical step, not a separate eased draw position; retail's dot test (a
  draw step against this frame's displacement +0x16d pointing the other way faces the displacement) is not modelled.

<!-- dll-build: 30261102_0 -->
## Our model is drawn at the actor update's eased place, and turns along it (2026-11-18)

- Report: with B held (Compact 2) the body turned toward the stick roughly, with single frames where it swung sideways
  and back; retail turns a little more each frame.
- The actor update is FUN_100c63d0 (the earlier fragments' FUN_100c5d60 is the actor's reset in this build). For an
  actor that is not on a lift (vt+0x19c, +0x5e2 from FUN_100a4ed0), not riding (FUN_10084350 / 70) and without the
  forced-move flags +0x101..+0x104, it eases the drawn position +0x5fc (vt+0x1bc) 0.125 a tick (_DAT_1032b3bc, its
  loop once per whole tick of FUN_10014cf0, at least one) toward the collision position +0x5c4 that FUN_100a4ed0 walks
  (0x100c6591); on a lift it takes the position as it is. The step +0x5fc moved is the turn's aim (2026-11-17), and
  the update adds it to +0x5fc after the attach block (+0x7a4). The skeleton is posed at +0x678 / +0x688, copies of
  +0x5fc / +0x61c made at the end of the update (FUN_100d4560 -> FUN_1002c3e0), and the camera follows vt+0x1bc. So
  retail draws us trailing a steady run by 0.875^2 / (1 - 0.875^2) frames of the move (about half a yalm running) and
  glides onto the spot after a stop.
- The turn is the same code with or without the strafe lock: free (vt+0x330, +0xf9) and not riding, the drawn rotation
  +0x61c eases 0.125 a tick toward the aim (the drawn step, or this frame's displacement when the drawn step points
  against it), else toward the heading +0x39; when it aimed, +0x44 takes it (not in first person, DAT_10489130), which
  FUN_100a57f0 makes the heading next frame. Free, FUN_100a6240 then turns the heading to the move, so the body
  follows the drawn path round a corner rather than snapping to the stick; standing, the heading is what the body last
  aimed at.
- The strafe lock's quarter (FUN_100a6240 0x100a6e57 -> FUN_100a80d0): the point one unit along the heading +0xe8 from
  +0x5fc, less the collision position +0xd4 (FUN_100a57c0, the last frame's walk), normalised and split at +-45
  degrees (0x3f490e56) against the move. That is the heading less how far the model trails, taken each frame before
  its move.
- Ours: World::turn_draw is +0x5fc for all our drawing now (update_movement's step loop, strafe_lock_turn), snapped on
  a lift, a place set from outside (zone-in, keep_pose, a scheduler's pin) and before the first frame; draw_pos runs
  from the last retail frame's +0x5fc to this one's over the frame. The free third-person turn (World::turn_rot, with
  or without B) is the actor update's per retail frame, drawn between the last two frames the same way;
  strafe_lock_quarter is the quarter. Locked on, first person, under the right drag's facing, the forced follow,
  events and a pin keep the ease toward the heading as before.
- Flagged for lenonk: (1) drawn_between_steps (three steps averaged, "retail draws each frame's collision position")
  stays only for riding: retail's rider path (0x100c655c, +0x768, FUN_100c8280) is not ported. (2) "Remote actors keep
  their level" (2026-10-28) removed a 0.125 position ease for remote actors; the same 0x100c6591 ease runs for every
  actor that passes the tests above, so remote actors may need it back (with the level snap kept). (3) Drawing between
  retail frames (position and rotation) is ours: retail draws once per frame of its own.

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
- Remote actors: all three skeleton actor vtables (0x1032e710, 0x10331f40, 0x103323e8) have FUN_100c63d0 at slot
  0x20 and vt+0x1bc = FUN_100a4740 (+0x5fc), so NPCs and other players ease the same way. Their collision position
  comes from the entity slide (+4 -> +0x34, FUN_1008f740; +0x34 -> +0xc4, FUN_10086e90; +0xc4 -> +0xd4, FUN_100a57c0;
  ground snap -> +0x5c4, FUN_100a4ed0). The drawn place is set to it (FUN_10092c20 / FUN_100929d0 / FUN_10092f90 write
  +0x34, +0x5fc, +0x5c4, +0xd4, +0xc4) by FUN_100929a0, which FUN_1008cd90 calls for a slide jump over 12
  (_DAT_1032e6b8 = 144 squared) and the place setters (FUN_100ab830 / 100ab880 / 100c3ff0 / 100c4050) and zone-in
  (FUN_100ae3f0) call; every frame for a status 0x1c rider; and by the push (+0x102) and self-move (+0x101). Not
  ours, the turn aims only at the drawn step (no displacement test). Ours: Entity::drawn (ease_drawn), snapped for the
  cases above, a lift, a fresh spawn and a place set from outside; base_place poses at it; the facing aims through
  drawn_aim at 0.01 a retail frame. Doors, lifts and ship models are not skeleton actors and are not eased.
- Flagged for lenonk: (1) drawn_between_steps (three steps averaged, "retail draws each frame's collision position")
  stays only for riding: retail's rider path (0x100c655c, +0x768, FUN_100c8280) is not ported; remote riders snap.
  (2) Retail's FUN_1008cd90 skips the over-12 jump for an actor on a lift (vt+0x19c); ours still jumps there. (3)
  Drawing between retail frames (position and rotation) is ours: retail draws once per frame of its own; remote
  actors ease continuously per tick (0.875^ticks) rather than once per whole tick.

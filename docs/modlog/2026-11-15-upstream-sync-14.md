## Effects: a burst's parent belongs to one update; an event area keeps the event's effects (2026-11-15)

- The violet stars: Ru'Avitau's zone sparkles `d_be/effe/z_wa/3i01` (argb a880b2, violet) run as a generator
  resource that a zone element E bursts (emission op 0x3c). Ours kept E as the instance's parent for good
  (Effects::emit_once). Once E died its slot went to an element on Yve'noile, and every later emission's op 0x45 copied
  that element's frame and actors: the zone's sparkles spread around her and over the room. Retail passes the parent
  as an argument of that one update (FUN_100496f0; the per-frame task passes 0, 0x1005425e). Now emit_once puts the
  instance's parent back after the update, for scheduler op 0x3f's morph clone too (test
  morph_static_world_in_place checks the emission and no kept parent).
- Op 0x34 / 0x35: Effects::switch_zone killed every instance and task, as for a zone change. Retail's scene swap
  (FUN_1017b1e0 -> FUN_100446a0) removes the clones of the old scene's generators. Ours (Effects::load_area,
  switch_zone(l, true)) now kills the instances of the old zone's and sub-area's libraries and the Zone / SubArea /
  Weather owners, the old scene's tasks, its weathers, wading and footstep folder; the event's tasks and effects go on.
  *(inf: what a model free (FUN_10091150) does to the effects on that actor is not traced.)*
- Live: Yve'noile's appearance shows green-white sparkles and a white glow, no violet, as in the footage.
- The footage's yellow-green orb in the spring is not the cutscene's: triangulated from 20 frames against kn05's
  camera it is one fixed point (-0.66, -30.54, 63.2), 5 cm residual, 1.2 m above where the player's thread
  (event_ffff at 0a6b: op 0x92, `actor_fade self #0 #1`, `set_pos_dir` (-0.589, -29.324, 63.431)) puts the player,
  faded out. The recording's character wields Excalibur 119 III; its model file (Mithra main 545: 65916) has `!in1`
  starting `60mt/aura` fr00 (flame) and gp00 (sparkles, argb 808038): the orb, drawn on the invisible player.
  Model effects ignore an actor's alpha (ours too); before the area swap above, ours killed the aura at op 0x34.
  Not seen live: the test character cannot wield it (a weapon look set by GM command is undone at the zone-in that
  `!in?` waits for).
- The footage reads neutral grey in Ro'Maeve, ours blue-grey: the test and user configs have `grading` 3 (Settings'
  Cool colour grading, ours: warmth -0.035 in the scaling pass), about -9 red / +9 blue. The lighting itself
  (logged live: fog and clear 0.282 at 4:00 in rain, screen multiply, blur and flash neutral) matches the footage.
- The area swap also clears op 0xc0's zone clock and op 0xc3's yaw (a scene's fields).
- s2c 0x00E's log line now prints the entity position.

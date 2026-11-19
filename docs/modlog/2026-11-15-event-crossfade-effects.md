## Event crossfade: the captured frame has the effects (2026-11-15)

- Op 0x10's crossfade (FUN_1005f020) fades from the last frame as drawn. Ours redraws that frame offscreen from its
  camera (client/mod.rs cross_capture) and drew only the scenery and actors, so whatever was an effect blinked out
  at each crossfade cut: in the Zilart ending's Qufim shot (area 126, camera paths cp02 -> cp03) the whole sea,
  which is effect meshes. The capture now draws the effects too (their GPU state is still last frame's).

## Real character shadows: moonlight, the dawn / dusk fade and Shadow Quality (2026-11-04, ours)

- Not retail. Follows "Real character shadows from the sun": switching between those by day and retail's by night was
  jarring, so the real shadows now carry on through the night.
- By night `Lighting::dir` / `diffuse` are the moon's (dat/weather.rs), so the moon casts: fainter by its dimmer colour
  (`sunshadow::strength`) and softer (the PCF taps spaced 2 texels instead of 1; `ZoneRenderer::shadow_soft`).
- `sunshadow::twilight(minute)`: the shadows fade to nothing at 6:00 and 18:00, where the light switches between sun and
  moon, over 45 Vana'diel minutes either side, so the direction change is never seen and retail's do not come back.
  A light too dim to cast (strength 0) still falls back to retail's.
- Settings > Effects > Shadow Quality (config `shadow_map`): Low 1024, Medium 2048 (the default), High 4096 texels over
  the 80 x 80 world units around the camera (about 8, 4 and 2 cm a texel).
- Test: render::sunshadow::light_matrix_frames_the_target (twilight's zeros and full strength).

## Colour grading presets for the 3D scene (2026-11-04, ours)

- Not retail: a deviation, off by default. Settings > Effects > Colour Grading (config `grading`: 0 Off, 1 Vivid,
  2 Warm, 3 Cool, 4 Film).
- render/scale.rs: applied in the scene scaling pass after FXAA (and in 3D LCD Mode), on the gamma-space scene before
  the gamma ramp, so menus, text and the cursor are untouched. Each preset is (saturation, contrast about mid grey,
  warmth: red up and blue down by the same amount, faded in over the darkest quarter of luminance so a faded
  frame stays black): Vivid (1.2, 1.08, 0), Warm (1.05, 1.04, +0.035), Cool (1.0,
  1.04, -0.035), Film (0.85, 1.12, +0.015). Turning it on sends the title and lobby scenes through the pass too.
- Test: render::scale::grading_presets.

## Reflections: a march that holds still (2026-11-09, ours)

- Steven: floor reflections warped and bubbled while moving. The march stepped along the ray in world units (quadratic
  spacing, 20 steps) with a hit thickness tied to the step, so the steps landed on different pixels every frame and
  hits jumped between them.
- render/reflect.wgsl `march` now steps evenly across the screen: the ray's ends are projected (cut just past the near
  plane), walked in at most 32 equal screen steps (no finer than a pixel) with depth interpolated as 1/w, a hit when
  the ray is behind the scene by less than the step's own depth change + 0.1 + 1%, then 5 halvings on a plain
  behind test, and the result turned back into a distance along the ray. Water uses the same march.
- Steven: character reflections came out chopped into streaks. Two causes: rays passing far behind a thin character
  still counted as hits (smears), and with steps up to a dozen pixels apart neighbouring rays hit or skipped the same
  thin part in bands. A hit now allows at most THICK (0.6) + 1% of the distance behind the scene; each pixel starts its
  steps at its own fraction of a step (interleaved gradient noise, 0.25..1, fixed per pixel), so bands become fine
  grain; and floors march 8 units instead of 12, so their steps are finer.
- Limit: a screen-space reflection only has what the camera sees, so a character's reflection is built from its back
  and missing where the reflected ray meets a part hidden from the camera.

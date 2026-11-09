## Reflections: a march that holds still (2026-11-09, ours)

- Steven: floor reflections warped and bubbled while moving. The march stepped along the ray in world units (quadratic
  spacing, 20 steps) with a hit thickness tied to the step, so the steps landed on different pixels every frame and
  hits jumped between them.
- render/reflect.wgsl `march` now steps evenly across the screen: the ray's ends are projected (cut just past the near
  plane), walked in at most 32 equal screen steps (no finer than a pixel) with depth interpolated as 1/w, a hit when
  the ray is behind the scene by less than the step's own depth change + 0.1 + 1%, then 5 halvings on a plain
  behind test, and the result turned back into a distance along the ray. Water uses the same march.

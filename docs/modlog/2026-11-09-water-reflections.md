## Water reflections (2026-11-09, ours)

- Not retail: a deviation, off by default. Settings > Graphics > Reflections (config `reflections`, Off / Water).
- Where (render/reflect.rs `WaterSurface`): the collision's type-9 faces, the ones fishing and wading use, since much
  visible water is an effect mesh (Buburimu's `umi1`) or part of a larger scenery mesh. A face whose object has a water
  height (`water_of`) is a bed and is laid flat at that height (only if some of it is under the surface); the scene's
  depth hides the parts under banks. A face without one is kept as it is when it lies flat (normal within ~18 degrees
  of vertical). Built once per zone on the loader thread (at most 2.4 ms, 28k faces in the zones checked), grouped
  in 32-unit tiles.
- Each frame: the tiles within 300 units and inside the view (box against the frustum planes) are drawn; with none,
  nothing runs. Otherwise one copy of the scene (after the actors and refraction, before the effects) and one draw.
- The shader: a manual depth test against the scene's depth (no depth attachment, so the pass can read it); ripple
  normals from four travelling waves, faded out by 60 units; the reflected ray marched 20 steps (quadratic spacing,
  80 units) through the depth, then 5 halvings on a hit; a hit takes the copy's colour, faded toward the sky at the
  screen edges and the ray's end; a miss takes retail's sky gradient (zone.wgsl fs_sky) in the reflected direction.
  Weight: Schlick Fresnel (F0 0.02) + 0.12, at most 0.7, times the fog factor. Nothing from under the surface.
- With the option on, the scene always draws offscreen (as Anti-aliasing does) so it can be copied.
- Tests: render::reflect::tests (faces kept and laid flat, tile culling, and on the GPU: sky on open water, nothing
  behind scenery); `water_census` (ignored) counts faces per zone.

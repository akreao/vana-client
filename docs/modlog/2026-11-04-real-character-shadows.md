## Real character shadows from the sun (2026-11-04, ours)

- Not retail: a deviation, off by default. Settings > Effects > Character Shadows (config `actor_shadows`: 0 Retail,
  1 Real). Config 0x3a = 1 (shadows Off) turns these off as well.
- render/sunshadow.rs: the drawn skinned actors (alpha 0.5 and up, not refracting) go into one 2048 x 2048 depth map
  through an orthographic light matrix from the zone lighting's direction (`Lighting::dir`, the one the scenery is
  lit by), 80 x 80 world units around the camera's target, its centre snapped to whole texels. Suns lower than 30
  degrees cast at 30. The pass reuses the silhouette's vertex shader (vs_skinned_silhouette), so textures are ignored
  and cut-out cards (hair, fringes) cast solid, the same as retail's silhouette.
- ZoneRenderer::render splits the zone pass after the opaque and cut-out scenery. A full-screen pass rebuilds each
  pixel's world position from the depth, takes a 3 x 3 PCF of the map, and multiplies the frame by 1 - k. k is at most
  0.45, scaled by the sun colour's brightness (0 for a dim light), faded toward the map's edge and on surfaces turning
  away from the sun (their baked lighting already darkens them). Blended scenery (water, glass) and every actor draw
  after it, so characters never shadow themselves or each other and their painted light is untouched.
- Only by day (the Vana'diel clock's sun up, `sun_moon(minute).0.y >= 0`): at night `Lighting` carries the moon, and
  retail's shadows draw. When the pass runs (`ZoneRenderer::sun_shadow_light`), the actors whose box lies wholly
  inside the map (`sunshadow::covers`) are dropped before retail's caster selection, so they take no retail shadow,
  caster slot or silhouette layer; the others keep retail's. The real shadow fades from 0.8 of the map's half side,
  so it is mostly gone before retail's returns at the edge.
- Known limits: every drawn character casts, including ones whose retail shadow is latched off; area-lit interiors
  still take the zone's direction.
- Tests: render::sunshadow::light_matrix_frames_the_target, render::sunshadow::receive_darkens_shadowed_floor.

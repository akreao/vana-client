## Reflections: polished floors (2026-11-09, ours)

- Not retail, off by default: Settings > Graphics > Reflections gains "Water and Floors" (config `reflect_floors`
  with `reflections`).
- render/reflect.rs `floor_textures`: hand-picked floors by zone and texture (the name past its 8-byte prefix) with a
  strength: Ru'Lude Gardens (243) e_yuka / yuka02 0.35, yuka01 0.25, kaka 0.2; Lower Jeuno (245) yuka1 0.2; Port
  Jeuno (246) p 0.25; Aht Urhgan Whitegate (50) yu 0.25. Chosen by looking at each zone's largest upward-facing
  textures; cobbles, grass and Ru'Lude's carpet (yuka03) left out. The loader puts the list on the scene
  (ZoneScene::floors).
- ZoneGeometry::build_with copies each listed model's triangles whose three corner normals point up (y < -0.9 through
  the placement) into world space, for placements at full detail or always drawn, not door parts or a voyage's world.
  Only listed textures are walked (Ru'Lude 2822 faces, Lower Jeuno 1209, Port Jeuno 746, Whitegate 172).
- Drawn in the water's pass from a second tiled buffer (vertex w = strength). A floor matches its own depth within
  0.05 + 0.3% of the distance, has no ripples or shoreline fade, and shows only what the march finds on screen within
  12 units, fading with the ray's length and at the screen's edges; a miss draws nothing (no sky under a roof).
- Tests: render::reflect::tests::floors_face_up_and_keep_their_strength, gpu_floor_reflects_the_scene_only.

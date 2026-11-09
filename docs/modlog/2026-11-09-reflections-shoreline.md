## Water reflections: shoreline fade (2026-11-09, ours)

- Bibiki Bay's type-9 faces mostly lie about half a unit above its sea mesh (`id_umi`, y 0), so where the sea meets a
  cliff a band of reflection showed above the waterline at a distance.
- render/reflect.wgsl now fades the reflection in over the first unit of water depth: the scene's depth at the pixel
  (along the pixel's ray, from the camera's forward, no matrix inverse) gives the point behind the surface, and its
  height below the surface scales the weight (smoothstep 0..1). Shores get a soft edge; a face above the drawn water
  shows nothing where the ground is just behind it.
- Test: render::reflect::tests::gpu_sky_on_open_water_none_behind_scenery adds ground 0.2 under the surface (faint).

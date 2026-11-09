## Reflections: mirrored characters (2026-11-09, ours)

- Steven: character reflections from the screen-space march only show the side the camera sees (their back) and
  break up where the reflected ray meets a hidden part. Characters are now drawn again, mirrored.
- render/reflect.rs `PlaneGrid`: the water's and floors' face heights by 4-unit cell, built with the reflections
  (linear in the faces). `under(feet)`: the nearest height from 0.5 above the feet to 3 below (floors only with the
  floors option).
- ZoneRenderer::mirror_actors, with the reflections, before their pass: each drawn character within 40 units of the
  eye (mips in, posed this frame, not refracting, not invisible) takes the plane under its origin, one cell lookup
  each; at most 4 planes a frame. Each plane's characters are drawn with the skinned pipeline through the camera
  mirrored in it (view_proj x mirror(h), its own uniform buffer, as the portraits' are) into a half-size layer with
  its own depth, cleared transparent. No plane: nothing drawn, the pass binds a transparent texel.
- reflect.wgsl samples the layer (linear) at the pixel: floors blend it over the march's hit (shown even where the
  march misses), water over its colour with the ripples bending it slightly and a weight of at least lift + 0.25.
- Tests: render::reflect::tests::planes_under_feet_and_the_mirror, gpu_mirrored_layer_on_the_floor.

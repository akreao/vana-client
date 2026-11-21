## Real character shadows from the lamps (2026-11-04, ours)

- Not retail. With Character Shadows on Real and Lamp Lights on Real (client/lamps.rs), the up to 4 lamps nearest
  the camera that have a character within 16 yalms each get a 512x512 perspective depth map of the characters
  (render/lampshadow.rs, the sun shadow's depth pipeline, one layer of a depth array each), aimed at those
  characters' centre and widened to take them in (half-angle 5 to 60 degrees).
- The zone shader's per-pixel point lights (zone.wgsl point_lights) look the light's slot up in those maps and scale
  that lamp's term by its 3 x 3 PCF visibility (strength 0.85), so a character cuts only that lamp's own light out
  of the scenery: ambient, sun, baked vertex colour and other lights are untouched, characters are not shaded by
  it, and in Baked lamp mode (lamplight painted into the scenery) nothing is shadowed. Group 0 of the zone layout
  gains bindings 5-7 (uniform, depth array, comparison sampler); they stay bound and empty when off.
- Data lights (kind 0x47, fires and such that retail already lights scenery with) are not shadowed; only our lamps.
- Tests: render::lampshadow (matrix framing, lamp choice), render::faded_tests::zone_renders_with_lamp_shadows
  (globals and a frame with a lamp shadow validate). Not checked in game (no game data in the cloud).

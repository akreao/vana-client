## Water reflections on the drawn water (2026-11-23, ours)

- Not retail: a deviation, off by default (Settings > Graphics > Reflections, unchanged). With it off nothing below
  runs and the frame is the same as before.
- Before: the reflection was a sheet built from the collision's type-9 faces, laid over the frame. Where the collision
  and the drawn water disagreed it showed bands (Bibiki Bay), sat on river beds (faces without a water height) or
  missed water with no wading collision; every zone needed checking. Its weight had a fixed lift (0.12 + Fresnel)
  that put sky colour over every water pixel, so green or dark water went grey-blue.
- Now (render/reflect.rs): the collision only gives the zone's water levels (`water_levels`: the objects' water
  heights, exact, and the heights of flat type-9 faces without one, which may be beds). The reflection is drawn on
  what is drawn as water:
  - Effect meshes (most open water: Buburimu's `umi1`, Windurst Waters' `wt01`..`wt08`, Port San d'Oria's `sea1`):
    a flat mesh (render/effects.rs `is_flat`) of a zone-owned effect is drawn again right after itself in the effects
    draw, through the element's own matrix, texture and colour (reflect.wgsl `vs_fx` / `fs_fx`); a vertex counts only
    on a level (exact within 0.75, or up to 3 above a bed level), a pixel only where the surface is flat.
  - Scenery: a placed model whose flat faces on an exact level make up 90% of it (`is_water`), found at load, with
    its texture's mean colour. Levels without a water height are left out for scenery: their faces are as often the
    bed or a floor as the surface (Bibiki's sand, Windurst's house floors, Port Windurst's sea floor).
- Colour: the weight is Fresnel (F0 0.02, at most 0.7) with no lift, faded only where the water's own alpha fades
  out; the reflection is tinted toward the water's own hue (60%). Mirrored characters show at least 0.3.
- Depth: an opaque effect sheet (op flag 0x1000) wrote its depth with the effects' bias (-15 constant and slope), so
  its pixels are matched with that allowance and take no shallow-water fade.
- FFXI_REFLECT_DEBUG: the water found in flat colours (scenery magenta, effect meshes cyan). zone-shot takes
  FFXI_REFLECTIONS=1/2 with --effects.
- Tests: render::reflect::tests (levels, is_water, colours, plane grid's level fallback, GPU passes),
  render::effects::tests::flat_meshes_for_water; `level_census` (ignored) lists levels and scenery water per zone.

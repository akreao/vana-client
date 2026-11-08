## Map Overlay Opacity moves into the Map Overlay mod (2026-11-08)

- The Map Overlay Opacity is now the Map Overlay mod's setting (`settings.opacity`, 10% to 100% in 5% steps, taken
  from config `map_overlay_opacity` at its first load), on the mod's own settings page. The mod passes it to the new
  client option `map_opacity` (0..1; kept when the mod unloads, as `minimap_span` is; 0.6 until a mod sets it), which
  HUD maps without their own alpha and the minimap's marker use as before. The client's own row and config field are
  gone.
- The modern Settings window's Maps & Timers page is removed. Minimap Position, Minimap Size and Timers Position were
  only ever shown there with the classic interface; they stay on the classic Extras tabs.

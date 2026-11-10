## Map cursor mode: the sprite cursor's mouse step is not zoomed (ours, 2026-11-10)

map.rs `mouse_step`. Retail FUN_101ef0a0 multiplies the sprite cursor's move (config 0021 = 0) by the zoom (+0x58)
unless both axes are under 2 pixels. Zoomed in (up to 4.6) on the conquest maps, a map marker or a Wide Scan narrow,
that sent the cursor across the map faster than the pointer moved, too fast to aim (Steven, in game). Ours keeps the
move one to one with the pointer at every zoom. A deviation from retail; the hardware cursor (config 0021) was already
absolute.

## Lua mods: a changed image file shows after a reload (ours, 2026-11-03)

- `hud.image` and `draw.image` keyed their texture by mod and path only, and the renderer keeps a texture once
  uploaded, so a PNG replaced and the mod reloaded still drew the old pixels. The key now carries a hash of the
  file's bytes (`hud::image_key`), so new contents upload anew. `chat.icon` still keeps the first image for a path
  (its icons are a fixed table of slots). From the Devin review of the upstream merge (PR #112).

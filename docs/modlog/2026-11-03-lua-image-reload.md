## Lua mods: a changed image file shows after a reload (ours, 2026-11-03)

- `hud.image` and `draw.image` keyed their texture by mod and path only, and the renderer keeps a texture once
  uploaded, so a PNG replaced and the mod reloaded still drew the old pixels. The key now carries a hash of the
  file's bytes (`hud::image_key`), so new contents upload anew. `chat.icon` still keeps the first image for a path
  (its icons are a fixed table of slots). From the Devin review of the upstream merge (PR #112).
- A new version of a mod's image marks the same file's older contents stale (`GameUi::image`), and `GameUi::render`
  frees a stale texture (`UiRenderer::drop_image`) once the frame's batch no longer draws it, so editing and
  reloading doesn't pile textures up in GPU memory, while two versions of one file drawn at once (changed on disk
  between two calls without a reload) both keep their pixels (Devin review of #117).

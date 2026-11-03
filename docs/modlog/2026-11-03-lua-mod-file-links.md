## Lua mods: mod-folder files refuse links (ours, 2026-11-03)

- `hud.image`, `chat.icon`, `draw.image` and `audio.play` read the mod's own files through `runtime::mod_file`: the
  path is made relative as `fs` does (no `..`, no drive or root) and a symlink anywhere on the way is refused, so a
  link shipped in a mod's folder can't point the read outside it (Devin review of the upstream merge, PR #112).

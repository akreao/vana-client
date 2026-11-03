## Lua mods: stop_recording needs exec and stops only the mod's own recording (ours, 2026-11-03)

- `client.stop_recording` asked for no permission and ended any recording, so a mod without `exec` could stop another
  mod's capture. It now needs `exec` (as `record` does), and the app ends the recording only when the asking mod
  started it (Devin review of the upstream merge, PR #112). `client.record` still replaces a running recording.

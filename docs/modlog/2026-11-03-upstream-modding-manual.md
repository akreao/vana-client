## Upstream 68dbc73 (modding manual and Lua API fixes) merged (ours, 2026-11-03)

- Upstream's own fixes replace parts of ours from #92 and #93: the /lua gate is now upstream's (grant, revoke, run,
  set and watch only when typed by the player; other /lua lines may come from a mod), so `commands.run` no longer
  refuses every /lua line; packet ids and sizes are checked by upstream's `build_packet` (0..0x1FF, at most 508
  bytes), and `fit` is upstream's again.
- Kept from ours: a mod's lines (`commands.run`, bind lines, action lines) are trimmed before they're sorted
  (`run_line`). Upstream checks `starts_with('/')` on the untrimmed line, and textcmd::run trims, so
  `" /lua grant <mod> exec"` from a mod with `actions` reached /lua as the player's own line.
- Kept from ours: a whole injected packet gets its size field set from its bytes, so a wrong header can't desync the
  batch it goes out in (upstream's manual said a whole packet is used as given; ours now says otherwise).
- docs/lua-api.md is gone upstream (docs/modding replaces it), with its .gitattributes line.

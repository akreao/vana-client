## Lua mods: the lobby reloads only lobby mods, and drops the character's settings (ours, 2026-11-03)

- A watched world mod saved on the lobby screens was loaded there without a world (its load ran with no game state),
  and login then skipped it as already loaded. Before login, `watch_upkeep` now reloads only mods with
  `lobby = true`; a world mod's change is picked up when it loads at login.
- After logout the lobby mods kept the last character's character-scoped settings (`settings.get`). They now go back
  to the account's settings and the defaults, as before login. Both from the Devin review of the upstream merge
  (PR #112).

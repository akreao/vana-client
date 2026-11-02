## Upstream Lua mods merged into the fork (ours, 2026-11-02)

- lenonk/vana-client master up to a3b5561 (the Lua mod layer). Upstream moved /dps into the bundled dps mod and
  dropped client/dps.rs, so our built-in /dps went with it; /uimodes and /favs stay.
- Movement: click-to-move (ours) and a mod's actions.run_to both steer autorun. Starting either clears the other;
  /follow and /automove clear both; actions.stop and actions.run end a click-to-move walk; run_to a point already
  within its radius ends the run (autorun, follow, click-to-move) instead of leaving it going.
- A mod's actions.drop passes the inventory to items::drop_now, which takes the bag since temp items.

## Upstream Lua mods merged into the fork (ours, 2026-11-02)

- lenonk/vana-client master up to a3b5561 (the Lua mod layer). Upstream moved /dps into the bundled dps mod and
  dropped client/dps.rs, so our built-in /dps went with it; /uimodes and /favs stay.
- A mod's actions.drop passes the inventory to items::drop_now, which takes the bag since temp items.

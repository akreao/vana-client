# Command Line

Without a subcommand, `vana-client` opens the game. Run `vana-client --help` for the full list and `vana-client <command> --help` for each command's options.

## For players

| Command | Does |
|---|---|
| `install SETUP.exe` | Install the game data from `FFXIFullSetup_US.part1.exe` (or an extracted copy); `--game-dir` for another folder |
| `update` | Bring the game files up to the current retail version from the PlayOnline patch server; `--dry-run`, `--data-only`, `--connections 1..6` |
| `play` | Open the client |
| `account-create --credentials FILE --user NAME` | Create an account on the server; writes the username and a generated password to FILE |
| `trust-server HOST` | Forget a server's pinned TLS certificate (or pin one with `--port` and `--fingerprint`); see [[Configuration]] |

## For mod authors

| Command | Does |
|---|---|
| `lua-stubs` | Write the LuaLS stubs for the Lua mod API |
| `tex-export`, `tex-pack`, `tex-slim` | Build a texture pack from upscaled PNGs (see [[Mods and HD Textures]]) |
| `mod-install --archive A --root DIR` | Install a pack archive as the Mods page does, to check it before publishing |
| `tex-dump`, `status-dump` | Save the images of game DATs, or the status icons, as PNGs |

## Scripted runs

`play --input '<script>'` drives the client from a script, for test runs, for example:

```sh
vana-client play --input "until:login;user;key:Tab;pass;key:Enter;until:title;shot:a.png"
```

`--input @FILE` reads the script from a file. The steps are in [client/script.rs](https://github.com/akreao/vana-client/blob/master/crates/vana-client/src/client/script.rs). `play` also takes `--credentials`, waypoints, a screenshot path and timing options.

Set `FFXI_CONFIG` to a copy of the config so test runs never write your own config, log or certificates.

## Offscreen renders

These render parts of the client to PNG without opening a window, for checking work by eye:

| Command | Renders |
|---|---|
| `zone-shot` | A zone |
| `npc-shot`, `char-shot` | An NPC / monster, or a character |
| `select-shot`, `creator-shot`, `title-shot` | Character select, the character creator, the title screen |
| `cs-shot`, `flyover-shot` | A cutscene camera, the title flyover |
| `fx-shot` | An action's effects |
| `ui-shot`, `topbar-shot`, `inventory-shot`, `equipment-shot`, `modern-shot` | Retail windows, the HUD, the modern UI |
| `dialogue-shot`, `combat-shot`, `text-shot` | Dialogue, combat lines, text in the game font |

## Decoders and audits

| Command | Does |
|---|---|
| `dat-info`, `ui-dump`, `mmb-dump`, `zone-info`, `dump-msi` | Inspect DATs, menu DATs, models, zones, MSI tables |
| `msg-dump`, `dmsg-dump`, `battle-dump`, `item-dump` | Print client messages, d_msg files, battle messages, items |
| `event-dump`, `event-ops`, `event-run`, `event-sweep` | Disassemble, list, run or sweep event scripts headless |
| `fx-sweep`, `fx-soak` | Run every effect, or spawn every model, headless to find faults and leaks |
| `sound-dump`, `audio-test` | Decode a sound to WAV, play music or an effect |
| `zone-path`, `zone-probe`, `map-grid`, `map-borders` | Walk paths, scenery under a point, map squares, map borders |

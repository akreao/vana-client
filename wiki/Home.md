# vana-client

A native FINAL FANTASY XI client written in Rust, for [LandSandBoat](https://github.com/LandSandBoat/server) (LSB) servers. It runs without Wine or PlayOnline: it reads the game's data files and talks to the server directly.

It aims to be a faithful clone of the retail client first, with its menus, sprites, fonts, motions and cameras rebuilt from the game's files and from how the original executable behaves. Improvements come after that. Retail limits that only existed because of 2002 hardware, like the 30 fps cap, become settings.

Runs on **Linux** and **Windows** (64-bit). macOS is planned.

## Start here

| I want to... | Page |
|---|---|
| Install and play | [[Installation]] |
| Build it myself | [[Building from Source]] |
| Change the server, ports, frame limit... | [[Configuration]] |
| Play with a controller | [[Gamepad]] |
| Pick between the retail look and the new UI | [[Interfaces]] |
| Use HD textures or other mods | [[Mods and HD Textures]] |
| Write a Lua mod | [[Lua Mods]] |
| Use the command-line tools | [[Command Line]] |
| Fix a problem | [[Troubleshooting]] |
| Contribute | [[Contributing]] |

## What works

Playable on a local LSB server: login, lobby and character creation, zones with weather and lighting, NPCs and monsters, dialogue, events and cutscenes, combat effects and skillchains, the full retail key set, gamepads, the HUD and chat, the main menu and its windows, trading and shops, fishing, chocobo racing, Monstrosity, Mog House, game updates from the PlayOnline patch server, a self-updater, a mod manager, music and sound.

Not done yet: cloth simulation, macOS builds. The full list is in the [README](https://github.com/akreao/vana-client/blob/master/README.md#status).

## Disclaimer

This is an unofficial fan project. It is not affiliated with, endorsed by or supported by Square Enix. The repository contains no game data and no code from the original client; you need the game files from Square Enix's installer. vana-client is under the [MIT License](https://github.com/akreao/vana-client/blob/master/LICENSE).

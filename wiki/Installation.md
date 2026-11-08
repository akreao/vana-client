# Installation

## Requirements

- A GPU with Vulkan support
- PipeWire or PulseAudio for sound on Linux
- The game data from Square Enix's official installer: `FFXIFullSetup_US.part1.exe` and its other parts
- A LandSandBoat server to connect to

No game data ships with vana-client.

## 1. Get the client

Download the Linux or Windows build from the [releases page](https://github.com/lenonk/vana-client/releases), or [[Building from Source|build it yourself]]. Releases are signed: the in-app **Update** button installs only a binary whose `.sig` matches the release key built into the client.

## 2. Install the game data

Point the installer at the first part of the official setup, or at an extracted copy of it:

```sh
vana-client install /path/to/FFXIFullSetup_US.part1.exe
```

This unpacks the game to `~/.local/share/vana-client/game/FINAL FANTASY XI` (Windows: `%APPDATA%\vana-client\game`), with the same layout as the Windows installer. Use `--game-dir` to put it elsewhere.

Other options:

- Copy an existing Windows install there, or point `game_dir` in the config at it (see [[Configuration]]).
- Start the client without game files: it offers to download them, or to use an install already on disk ("Use existing game files...").

## 3. Bring the data up to date

The installer's data is years old. Update it from the PlayOnline patch server (no login needed):

```sh
vana-client update
```

`--dry-run` asks the server and checks the files without downloading; `--data-only` leaves executables and libraries alone. The patcher writes an executable or library only when it is on the signed list of pinned code files that the repository publishes.

## 4. Play

```sh
vana-client play
```

Log in on the game's login screen. The server address and ports come from the config file; the default is a server on `127.0.0.1`.

To create an account on the server from the command line:

```sh
vana-client account-create --credentials account.json --user NAME
```

This writes the username and a generated password to `account.json`.

## Linux desktop entry

To add vana-client to your application menu with its icon (Wayland compositors find the icon through this entry):

```sh
install -Dm755 vana-client ~/.local/bin/vana-client
install -Dm644 assets/vana-client.desktop ~/.local/share/applications/vana-client.desktop
install -Dm644 assets/vana-client-512.png ~/.local/share/icons/hicolor/512x512/apps/vana-client.png
```

## Next

- [[Configuration]]: server address, ports, frame limit, GPU and audio device
- [[Interfaces]]: Classic or Modern
- [[Mods and HD Textures]]: the 4x HD texture pack

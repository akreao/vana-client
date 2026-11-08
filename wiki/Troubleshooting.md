# Troubleshooting

Look in the log first: `~/.local/share/vana-client/vana-client.log` on Linux, `%APPDATA%\vana-client\vana-client.log` on Windows. The run before is kept as `vana-client.prev.log`.

## The lobby refuses the client version

`client_version` in the config (default `30261006_0`) must be at least the server's `CLIENT_VER`. Raise it in [[Configuration|config.json]].

## Login fails after the server was reinstalled

The client pins each server's TLS certificate and refuses a different one. If the server really has a new certificate, run `vana-client trust-server SERVER`, or pin its fingerprint (see [[Configuration#server-certificates]]).

## The game files are missing or out of date

Run `vana-client install /path/to/FFXIFullSetup_US.part1.exe`, then `vana-client update`. If the game is somewhere else, set `game_dir` to the folder that holds `FINAL FANTASY XI`.

## The wrong GPU is used

Set `gpu` in the config to the adapter's name. Empty picks the high-performance one.

## No sound, or the wrong device

Set `audio_device` in the config, or pick it in Config. On Linux, sound needs PipeWire or PulseAudio. A build made with `--no-default-features` has no native PipeWire output and goes through PulseAudio (or PipeWire's PulseAudio server) or ALSA.

## Building fails on Linux with PipeWire errors

The system PipeWire headers are too old for cpal's bindings (Ubuntu 22.04, for example). Build with `cargo build --release --no-default-features`. See [[Building from Source]].

## Building fails on Windows

`link.exe` not found, or `cl.exe` errors from ring or unrar_sys, mean the C++ workload is missing: add "Desktop development with C++" in the Visual Studio Build Tools installer. More in [docs/building-windows.md](https://github.com/akreao/vana-client/blob/master/docs/building-windows.md#when-it-fails).

## A Lua mod stopped working

A mod with 20 errors in a minute is suspended. Its errors are in chat and in the log with file:line. Fix the mod and `/lua reload <mod>`. A mod that needs a permission says so when loaded: grant it on the mod's Extras page or with `/lua grant <mod> <permission>`.

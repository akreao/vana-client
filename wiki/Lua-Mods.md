# Lua Mods

vana-client runs Lua 5.4 mods, each in its own sandboxed VM. The API covers what Windower and Ashita add-ons do: game state, events, chat, commands, windows and HUD elements, settings and storage, packets and actions, behind permissions you grant per mod.

The full design and the current state of the API are in [docs/lua-mods.md](https://github.com/akreao/vana-client/blob/master/docs/lua-mods.md).

## A mod's folder

`~/.local/share/vana-client/mods/<id>/` (Windows `%APPDATA%\vana-client\mods\<id>\`) holds:

- `mod.toml`: the manifest
- `init.lua`: the entry point (another file with `entry`)
- any other Lua files, loaded with `require` from the mod's folder

### mod.toml

```toml
id = "hello"
name = "Hello"
version = "1.0.0"
api = 1
authors = ["you"]
description = "Greets you at login."
permissions = []          # e.g. ["actions"], see below

[settings.greeting]
type = "string"           # bool, int, number, enum, string, colour or key
default = "Hello"
label = "Greeting"
help = "The word said at login."
scope = "character"       # account (default), character or server
```

A mod declaring a newer `api` than the client speaks is not loaded. Declared settings appear on an Extras page for the mod, in both interfaces.

### init.lua

```lua
events.on("login", function()
  chat.add(settings.get("greeting") .. ", " .. game.player().name .. "!")
end)

commands.register("hello", function(...)
  chat.add("You said: " .. table.concat({...}, " "))
end)
```

`//hello there` (or `/hello there`) now runs the command.

## Editor support

At login the client writes LuaLS stubs into the mods folder (`.library/vana.lua`, `.luarc.json`). Open the mods folder in an editor with the Lua language server for completion and checks. `vana-client lua-stubs` writes them on demand.

## The /lua command

| Command | Does |
|---|---|
| `/lua list` | Mods and their state |
| `/lua load | unload | reload <mod>` | Load, unload or reload one mod; `/lua reload` alone reloads every mod |
| `/lua watch <mod>` | Reload the mod whenever its files are saved, so you can write it without restarting the game |
| `/lua run <code>` | Run a line of Lua |
| `/lua set`, `/lua settings` | Change and show a mod's settings |
| `/lua grant | revoke <mod> <permission>` | Grant or take back a permission |

Only the player can use `/lua`: a mod's `commands.run` refuses it, so a mod cannot grant itself permissions.

## Permissions

With no permission a mod can read game state, draw its own windows and HUD, keep its own settings and storage, write to chat, register commands and play sounds. Anything more it lists in `permissions`, and you grant it on the mod's Extras page (a row per permission, ON / OFF) or with `/lua grant`. Grants are kept in `mod-data/grants.json`.

| Permission | Grants |
|---|---|
| `actions` | Game actions from the mod's own UI or commands, in response to your input |
| `automation` | Game actions on the mod's own initiative: cast, target, move, trade, lot, NPC menus |
| `packets.read` | Raw packet hooks, read only |
| `packets.write` | Modify, block and inject packets |
| `input.capture` | Swallow keys and mouse, synthesise keys |
| `files.shared` | Read and write in a shared data area outside the mod's folder |
| `net` | HTTP(S), UDP and TCP, open URLs |
| `exec` | Run programs |
| `multibox` | The local bus between clients, window focus |
| `ui.native` | Drive the built-in windows |

No mod can ever read the password, the 2FA secret, `secret.key`, session keys or the Blowfish state.

## Limits

- 64 MB of memory per mod, text-only `load`, restricted standard libraries
- A 100 ms budget per call: a runaway loop is stopped
- Each distinct error is shown once in chat with file:line (full text in `vana-client.log`); 20 errors in a minute suspend the mod
- F12 shows each mod's time per frame, peak, slow calls, memory and errors

## Bundled mods

Some of the client's own features ship as Lua mods (for example `dps`, in [assets/mods](https://github.com/akreao/vana-client/blob/master/crates/vana-client/assets/mods)). They run unless turned off, or replaced by a folder of the same id in your mods folder.

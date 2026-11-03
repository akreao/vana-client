# Configuration

## Files

| What | Linux | Windows |
|---|---|---|
| Config | `~/.config/vana-client/config.json` | `%APPDATA%\vana-client\config.json` |
| Log (and the run before) | `~/.local/share/vana-client/vana-client.log`, `vana-client.prev.log` | `%APPDATA%\vana-client\vana-client.log`, `vana-client.prev.log` |
| Game files | `~/.local/share/vana-client/game` | `%APPDATA%\vana-client\game` |
| Mods | `~/.local/share/vana-client/mods` | `%APPDATA%\vana-client\mods` |
| Pinned server certificates | `~/.local/share/vana-client/known_servers.json` | `%APPDATA%\vana-client\known_servers.json` |

The config file is created on first save. Most options can be changed in the in-game and lobby **Config** menus; the **Extras** page holds additions such as the map overlay, minimap, timers, a skillchain helper and chat timestamps.

The `FFXI_CONFIG` environment variable points the client at another config file. The log, chat history and pinned certificates then live beside that copy, so test runs never touch your own files.

## Fields you may need to set by hand

| Field | Default | Meaning |
|---|---|---|
| `server` | `127.0.0.1` | LSB server address |
| `auth_port`, `data_port`, `view_port` | `54231`, `54230`, `54001` | LSB login and lobby ports |
| `game_dir` | `~/.local/share/vana-client/game` | Folder that holds `FINAL FANTASY XI` |
| `client_version` | `30261006_0` | Version sent to the lobby; must be at least the server's `CLIENT_VER` |
| `frame_limit` | `300` | Frame limit in fps, `0` for unlimited |
| `gpu` | empty | Graphics adapter by name; empty picks the high-performance one |
| `audio_device` | empty | Output device; empty uses the system default |
| `modern_ui` | `false` | Use the Modern interface (see [[Interfaces]]) |
| `gamepad`, `gamepad_padsin`, ... | | See [[Gamepad]] |

Example for a remote server:

```json
{
  "server": "play.example.org",
  "frame_limit": 144
}
```

## Server certificates

The login and profile connections use TLS with the server's self-signed certificate. The client remembers the certificate each server sends the first time and refuses a different one later, since that is what someone intercepting the connection would send.

When a server really gets a new certificate (an LSB reinstall makes one), trust it:

```sh
vana-client trust-server SERVER                                        # forget the pinned one; the next login pins the new one
vana-client trust-server SERVER --port 54231 --fingerprint AB:CD:...   # or pin a known one up front
```

On the server, this prints the login port's fingerprint (`profile.cert` is xi_profile's, on its own port):

```sh
openssl x509 -in login.cert -noout -fingerprint -sha256
```

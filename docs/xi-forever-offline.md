# Playing XI: Forever Offline with vana-client

XI: Forever Offline (Hurami89/xi-forever-offline) runs a LandSandBoat-based server (Phoenix) on this PC with
the standard ports and the same xiloader-3 TLS login, so vana-client connects to it without code changes.
Checked on the `forever` branch on 2026-11-24 against their v0.3.9 Linux install.

## Setup

1. Start the world from their launcher (or its server programs and MariaDB), then close their game client:
   one account can only be signed in once.
2. Give vana-client its own copy of their game folder, with their era DATs and DAT patches already applied
   (`Game/SquareEnix`, holding `FINAL FANTASY XI` and `PlayOnlineViewer`). On btrfs a reflink copy is instant:
   `cp -a --reflink=always "XI Forever Offline/Game/SquareEnix" <somewhere>/game`.
   Never point vana-client at their install itself.
3. In vana-client set the game folder to that copy, server `127.0.0.1`, and sign in with the account their
   launcher made. The server's self-signed certificate is pinned on the first connection; after reinstalling
   their server run `vana-client trust-server 127.0.0.1`.

## What works

Sign-in, the lobby, zoning, NPC talk with the era dialogue, battle, their bot players, `!squad` commands typed
in chat (squad mates join the party and talk), the mode marks their launcher draws into the icon sheet.

## Known gaps

- Squad mates sometimes have no model: their server sends a name over the look in some 0x00E updates.
  Retail shows wrong armor there instead. It needs a server fix.
- Their Ashita addons (squad panel, server menu, Gambits, hardcore icons, journal, minigames) and the
  co-op launcher layer are not in vana-client.
- Their server and a LandSandBoat server of your own use the same ports: stop one before starting the other.

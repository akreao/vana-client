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

## XIFO mods

The `forever` branch adds two standard mods (its build reads the mod catalogue from this branch). Both are our own
code against the server's `!squad` chat commands; none of XIFO's Ashita add-on code is used.

- **XIFO Link** (`xifo-link`): the channel to the server. Other mods hand it commands
  (`mods.call("xifo-link", "send", "!squad panel", { tag = "squadpanel" })`); it says them one at a time, at least
  1.25 s apart and never while the chat line is open, because the client refuses a second chat line within a second
  (retail's gate prints `>> <line>` and "...A command error occurred."). A refused line is said again, up to 3 times.
  It hides the echo of its own lines, the refusal, and the tagged reply lines (with the "SystemMessage" header each
  server line brings) while a reply is due, and hands the reply on as the event
  `xifo_link_reply(tag, lines, complete)`. Lines a player types by hand are left alone.
- **XIFO Squad** (`xifo-squad`, `//squad`): a window with the squad mates (jobs, role, mode, out or away, the call
  cooldown), Call all / Dismiss all / Apply now (dismiss and call again, so new settings count), and for the picked
  mate Call, Dismiss, Profile, Role, Mode, the four action switches, main and support job, and a Camp section (camp
  here, start, pause or resume, lift, the puller). Profile (`//squad sheet <slot>`) shows HP / MP / TP, EXP, stats,
  gear with item icons, jobs and the spells and abilities of a called mate. The panel is asked again every 15 s while
  open, a called mate's profile every 6 s.

Server reply formats used (from their `modules/custom/lua/squad.lua`): `squadpanel:<slot>|<name>|<job>|<level>|<sub or
->|<sub level>|<role>|<mode>|<out 0/1>|<switch mask>`, `jl|<slot>|<JOB:level,...>`, `cd|<slot>|<seconds>`,
`camp|<on>|<puller slot, 0 = you>|<paused>|<pulls>`, `campbots|<names or ->|<pulling bot or ->`, then
`squadpanel:end`; `squadsheet:hdr|job|exp|vital|stat|combat|haste|eq|ls|note|spell|ability|...` then
`squadsheet:end`. Lines the mods do not know are skipped. Item, spell and ability names come from the game's files
(the server sends database names such as `onion_staff`).

## Known gaps

- Squad mates sometimes have no model: their server sends a name over the look in some 0x00E updates.
  Retail shows wrong armor there instead. It needs a server fix.
- Their other Ashita addons (server menu, Gambits, hardcore icons, journal, autobox, minigames) and the co-op
  launcher layer are not in vana-client yet; the squad panel is (XIFO Squad, above).
- Their server and a LandSandBoat server of your own use the same ports: stop one before starting the other.

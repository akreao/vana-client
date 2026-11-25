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

- **XIFO Gambits** (`xifo-gambits`, `//gambits`): the squad mates' Rule Sets (the server's `!gsq` editor protocol,
  version 4 or later). Pick a mate; its job's sets are listed with the one it uses. Open, New, Copy, Delete; Use this
  set / Use job rules (`!squad ruleset <slot> <num|off>`) and Apply now (`!squad reapply`). The open set's rules show
  as target / when / do; Add, Edit, Copy, Delete, Up, Down, and a click opens the rule in its own window: on/off,
  target, a condition and a second one with their values, the kind of action and what (best of a spell family, a
  spell, a job ability, ranged attack, best nuke, magic burst), and a minimum level. The choices are the mate's own
  catalog (`catm`, checked with the server's catalog check); a value it does not list is kept, marked. Save sends
  the set in parts (`x`, at most 90 bytes each); a set that is not yours saves as a new one. Check (`chk`) lists the
  rules the called mate would skip at its level. Rules the wire cannot carry show as "?" and block saving that set.
  For the targets Tank, Melee, Mage and Ranged the rule window also includes or leaves out the other roles
  (version 9 servers). A Weapon skills page holds the mate's setting per main job (`wsg`/`wsl`/`wss`/`wsa`: Squad
  default or Custom, Open / Close / Open + Close / Independent, the TP, Best or one of the called mate's weapon
  skills; Put on now gives it to the called mate at once), and a Party roles page who the role targets mean
  (`prl`/`prs`: Auto by job or a set role, for you and each mate; other party members shown read-only).
  Share codes: Copy code puts the open set on the clipboard as a `GS1...` code, Import code opens one (from the box,
  or the clipboard) as a new unsaved set. The format is XIFO's own (their Gambits add-on reads ours and we read
  theirs; the test holds a code their add-on made), written anew here from its description.

- **Auto Ranged** (`auto-ranged`, `//autora`): their autora as our own mod, and not tied to XIFO (it only uses `/ra`).
  A small edge window picks Manual, Auto or Until hate. Auto keeps shooting while you are engaged with a living
  monster targeted and a ranged weapon or ammo on; it waits for each shot (and spells, items), holds while you move,
  waits `//autora delay <s>` after a shot (0.5 at first) and pauses 3 s when the game refuses a shot (out of range,
  cannot see). Until hate also holds while the monster's last action (within 10 s) was on you.

- **XIFO HC Icons** (`xifo-hc-icons`, `//hcicons`): the hardcore mode's shield (Iron, Steel, Silver, Random; our own
  pictures) left of each hardcore member's row in the modern party and alliance windows; the game already draws
  the shield beside names in the world. On arriving in an area it sends `!hcicons`; the server answers
  `[HC-ICONS] !Name=1,Other=3` and later tells arrivals as `[HC-ICONS] Name=2`, which XIFO Link hides (its `listen`).
  The rows come from `ui.native.party_rows()`. `//hcicons` turns it off or on, `list` names who is known.

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

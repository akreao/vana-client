# Playing XI: Forever Offline with vana-client

XI: Forever Offline (Hurami89/xi-forever-offline) runs a LandSandBoat-based server (Phoenix) on this PC with
the standard ports and the same xiloader-3 TLS login, so vana-client connects to it without code changes.
Checked on the `forever` branch on 2026-11-24 against their v0.3.9 Linux install.

## Setup

Their launcher's PLAY button starts the retail client, so with vana-client their launcher is only the settings
and tools app (RULES, WORLD, SQUADS, the Hardcore characters, updates), used while the world is stopped.
`tools/xifo-play.py` (Python 3) plays instead:

```
tools/xifo-play.py --kit "/path/XI Forever Offline" --client /path/vana-client
```

1. It first does to their game folder what their PLAY does before the game starts: the era DATs a game update
   wrote over go back and their DAT patches (Home Point teleports, items, spells, mode marks) are redone, by their
   own code run through their launcher. The server emulates the era, so the client needs those files.
   Then it gives vana-client its own copy of their game folder (`Game/SquareEnix`, with their era DATs and DAT patches)
   in `~/.local/share/vana-client-xifo/game`: the whole folder on the first run (reflinks where the disk can),
   then only the files their updates changed (everything again after switching to another install).
   `FINAL FANTASY XI/USER` (macros and settings) is copied only into a copy that has none, and then left alone. vana-client never plays from their folder itself, and it does not offer retail game-file updates for this copy
   (its stamp file `.xifo-sync` tells it so); the client's own update still works.
2. It starts their world without its window (`launcher_world.py run`, run through their packaged launcher's
   `--run`): the database, the RULES / WORLD settings, the game servers, the auction house simulator, the co-op
   door. Their launcher must be closed; the world refuses to share the install with it. On a kit with no
   `world.json` it writes one with `"auto_update": false`, because a world left running otherwise installs a new
   release by itself and restarts in the middle of play (`--allow-world-updates` keeps theirs).
3. It starts the local AI their PLAY starts and the world leaves out, when the RULES page has it on: Ollama, squad
   chat (`squad-chat/squad_chat.py`) and the crowd and bot chat helper.
4. It pins their server's TLS key for vana-client (with `openssl`), so a reinstall of their server does not stop
   the sign-in, then opens vana-client with `FFXI_CONFIG` set to `~/.local/share/vana-client-xifo/config.json`
   (server `127.0.0.1`): its settings, pins, log and mod settings stay apart from your usual setup. Sign in
   with the account their launcher made.
5. Closing vana-client stops the helpers and the world the normal way (the servers save, the database stops).

`--profile DIR` puts the vana-client setup elsewhere, `--game DIR` the game copy, `--no-sync` skips the copy,
`--no-ai` the AI helpers, and `--world-only` starts the world until Ctrl+C without the client. `XIFO_KIT`,
`VANA_CLIENT` and `XIFO_PROFILE` can stand in for the first three options. Arguments after `--` go to vana-client.

Not covered without their launcher: PLAY HARDCORE's one-session rules (hardcore characters themselves play
normally), visiting a friend's world in co-op (hosting works), char sync, armory sync and the Ventures relay.

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
  With a v0.3.11 server (its `rs` and `cap` panel lines, `slot` sheet line) the picked mate also shows a Rules line:
  the Rule Set or job rules it runs now, why its next call would run other rules (marked `*` in the list), a fallback
  reason or rules left out; a profile answer for a mate no longer shown is dropped. Older servers just don't show it.
  When the server says it can re-call one mate (`cap|reapply1`), an out mate also gets Apply (`!squad reapply <slot>`):
  only that mate leaves and comes back with its new settings.

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
  or the clipboard) as a new unsaved set. The box keeps 1024 characters, so a longer code is read from the clipboard,
  which needs the mod's `clipboard` permission (granted on a catalogue install; else `/lua grant xifo-gambits
  clipboard`). An imported set saves under its own job, also with no squad mates. The format is XIFO's own (their Gambits add-on reads ours and we read
  theirs; the test holds a code their add-on made), written anew here from its description.
  Version 10 servers (XIFO v0.3.11) also say what each mate really runs: the status line names it (the set, its job
  rules, why a chosen set fell back, rules left out, or that its next call would change them, marked `*` in the mate
  list), and with the `reapply1` word an out mate gets Apply this mate (`!squad reapply <slot>`).

- **Auto Ranged** (`auto-ranged`, `//autora`): their autora as our own mod, and not tied to XIFO (it only uses `/ra`).
  A small edge window picks Manual, Auto or Until hate. Auto keeps shooting while you are engaged with a living
  monster targeted and a ranged weapon with its own kind of ammo (or a thrown weapon) on; it waits for each shot (and spells, items), holds while you move,
  waits `//autora delay <s>` after a shot (0.5 at first) and pauses 3 s when the game refuses a shot (out of range,
  cannot see). Until hate also holds while the monster's last action (within 10 s) was on you.

- **XIFO HC Icons** (`xifo-hc-icons`, `//hcicons`): the hardcore mode's shield (Iron, Steel, Silver, Random; our own
  pictures) left of each hardcore member's row in the modern party and alliance windows; the game already draws
  the shield beside names in the world. On arriving in an area it sends `!hcicons` (in /say, so only for a character
  whose server has answered an XIFO mod before: the first time, open XIFO Squad or `//hcicons refresh`); the server answers
  `[HC-ICONS] !Name=1,Other=3` and later tells arrivals as `[HC-ICONS] Name=2`, which XIFO Link hides (its `listen`).
  The rows come from `ui.native.party_rows()`. `//hcicons` turns it off or on, `list` names who is known.

- **Autobox** (`autobox`, `//autobox`): rules for your own character (their autobox add-on, formerly their Gambits
  2.2, rewritten). One sheet per main job, each rule a target (me, party member, battle target, pet), one or two
  conditions (HP / MP below or above, my TP, lacks or has a status, engaged or not) and an action (a spell, with
  "best tier I can cast", a job ability, a weapon skill or the best one the game takes now, ranged attack, attack).
  Every 0.2 s the first rule that holds and that the game's menus would take is done; it waits while you move, cast,
  talk to an NPC or zone, 1.5 s after each action, and 4 s before the same rule fires again. Party rules pick the
  lowest HP member in casting range; battle target rules only touch a monster you or your party claimed, or any
  monster while you are engaged with it. Spells wait out silence and mute, abilities amnesia; ranged attack needs
  something to shoot. The rules
  start switched off (`//autobox on`); each job starts from a small preset (WHM, RDM, BLM, PLD, WAR, SAM, DRK; the
  rest a weapon skill rule). Not ported: their Parser, EXP Watch and Enemy Info tabs (DPS and XP Tracker cover the
  first two) and the Recruit / Sort / Optimize buttons.

- **Era journal** (their `journal-era` edits for a third-party Journal add-on): the game's own Missions and Quests
  menus already follow the server's expansions (base, Zilart, CoP and ToAU here), so nothing past ToAU shows there.
  Completion Log (`//left`), which lists every quest, does the same now: only the areas the quest menu shows
  (`game.quest_areas`), no unused placeholder entries, and on a server without Wings of the Goddess no Voidwatch,
  Monstrosity, Trust, Records of Eminence or Unity quests in the old areas unless taken. The first two parts are
  generic and also went to `develop`.

Server reply formats used (from their `modules/custom/lua/squad.lua`): `squadpanel:<slot>|<name>|<job>|<level>|<sub or
->|<sub level>|<role>|<mode>|<out 0/1>|<switch mask>`, `jl|<slot>|<JOB:level,...>`, `cd|<slot>|<seconds>`,
`camp|<on>|<puller slot, 0 = you>|<paused>|<pulls>`, `campbots|<names or ->|<pulling bot or ->`, then
`squadpanel:end`; `squadsheet:hdr|job|exp|vital|stat|combat|haste|eq|ls|note|spell|ability|...` then
`squadsheet:end`. Lines the mods do not know are skipped. Item, spell and ability names come from the game's files
(the server sends database names such as `onion_staff`).

## Known gaps

- Squad mates sometimes have no model: their server sends a name over the look in some 0x00E updates.
  Retail shows wrong armor there instead. It needs a server fix.
- Their other Ashita addons (server menu, minigames) and the co-op launcher layer are not in vana-client yet; the
  squad panel, Gambits, hardcore icons, autora, autobox and the era journal are (XIFO Squad, XIFO Gambits, XIFO HC
  Icons, Auto Ranged, Autobox and the era journal, above).
- Their server and a LandSandBoat server of your own use the same ports: stop one before starting the other.

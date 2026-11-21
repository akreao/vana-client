# MODLOG inferred and untraced claims

Generated from `MODLOG.md` and `docs/modlog/*.md` on 2026-11-03. Every line that carries an *(inf)* / *(inferred)* /
*(not traced)* / *(as read)* marker, or says "inferred" or "not traced" in prose, is listed under its entry,
with its line number. These are the claims a check against retail FFXiMain.dll (30261006_0) can settle.
Companion to [modlog-gaps.md](modlog-gaps.md).

826 marked lines in 270 entries (a marker that wraps onto the next line counts once, at its first line).

## Zone session

`MODLOG.md:97`

- L102: … name and ids in the session's list ("We never _see_ the character", login_helpers.cpp createCharacter), so the select screen would show it as race 0 / no job / area unknown *(inf: from LSB, not traced in FFXiMain)*

## Menus / UI (FFXiMain + DATs; title and HUD rendered offscreen and checked by eye)

`MODLOG.md:198`

- L211: Colour bytes read as RGBA (buttons and windows come out blue; inferred, not traced).

## Character creator preview: the "dancer" engine (FFXiMain; all 8 races x 8 faces x 2 hairs rendered, checked by eye)

`MODLOG.md:311`

- L384: File sound/win/music/data/musicNNN.bgw (format "%s\music%3.3u.bgw" FUN_1023bed0; id -> NNN inferred).

## Dancer shading (creator preview), traced to the pixel (creator-shot renders it; before/after checked by eye)

`MODLOG.md:392`

- L453: "_sort" updaters (updaterFlags >> 8 = shape) mark eyelashes and hair; the prototype draws those triangles back to front (the sort updater FUN_10290c90 is not traced).

## NPC talk, event packets, zone dialogue text and the dialogue window

`MODLOG.md:539`

- L542: Incoming handler offsets count from the packet start, including the 4-byte header. Inferred items are marked *(inf)*
- L699: **0x30..0x35 and 0x42..0x44: names through `FUN_1014e180`** *(inf: key items)*

## Event scripts: event DATs, the event VM, cutscene cameras (event-dump, event-ops, cs-shot)

`MODLOG.md:764`

- L838: Ops marked "inferred" in event-ops have a known handler but an unconfirmed meaning: 46 sub 2, 69/6a screen layers, the units of 1f.

## Sound (FFXiMain sound module 0x1023b000..0x10242000 + the install; WAVs and device recordings checked)

`MODLOG.md:843`

- L918: Not traced: positional effect volume (FUN_10035e50 distance curve, used by the creator foley at the hip bone;
- L926: The inputs (+0x218, +0x879, +0x87a) and the step timing are not traced.

## Event VM runtime (event_vm.rs), event-run / event-sweep

`MODLOG.md:928`

- L932: Everything below was decompiled; the parts that are not confirmed are listed under "Inferred".
- L1049: In the client the pc would not advance and nothing would yield, so it would hang. - The data's lengths are {3:4, 4:6}, inferred from the bytes.
- L1110: They loop on state the headless host does not model. ### Inferred / open - Item and key item names in text ({item}, {keyitem}): dialog::expand drops them;

## Item DATs, name tables, item and key-item names in dialogue (dat/items.rs, dat/names.rs)

`MODLOG.md:1118`

- L1159: Fields the parsers map but whose use is not traced are kept raw (`Item::extra`, keyed by object offset, e.g.

## LSB text id mapping (dat/msgshift.rs), event sound volume ops, real event wait times

`MODLOG.md:1190`

- L1229: **Mask bits:** \| Bit \| Channel \| Setter \| \|---\|---\|---\| \| 1 \| world sound effects, DAT_1034656c (scales 3D sounds, FUN_10035590) \| FUN_10035230 \| \| 2 \| interface sounds, DAT_10346568 (inferred from FUN_10036d10, its one use) \| FUN_10035280 \| \| 4 \| music, DAT_10346570 (applied to the current BGM throu
- L1269: It is not traced further.

## Inventory and equipment menus (inventor, equip, persona, money) and their item list

`MODLOG.md:1273`

- L1289: … \| \| `equip` \| DAT_10565794 \| FUN_1012ea90 \| \| `persona` (status panel) \| DAT_105659c4 \| FUN_10141c10 \| \| `money` \| DAT_10565830 \| FUN_1013b8b0 \| \| `iteminfo` \| DAT_105657ec \| not traced \| - **Items menu:** FUN_101e9f10 opens `inventor` for a container source object.
- L1325: … \| \|---\|---\|---\| \| 0x80404040 \| grey \| player +0x132 bit 0 is set \| \| 0x80407090 \| blue \| equipped weapon/armour that also passes FUN_100fbc70 (item id + extdata; meaning not traced) \| \| 0x80509050 \| green \| equipped, or the equipped linkshell / linkshell 2 \| \| 0x80909040 \| yellow \| non

## Combat: battle messages, 0x028 actions, combat motions, the action menu (dat/battle.rs, dat/sched.rs, net/action.rs, client/combat.rs, combat-shot)

`MODLOG.md:1388`

- L1421: Status names: 55725 (1024 entries "KO", "weakness", "sleep"...); its language slot was not found (inferred from content).
- L1471: Where PC battle motions load from is **not traced**.
- L1481: Library 1 (commands 6, 0x10, 0x11, 0x13, 0x14, 0x1a, 0x1b) is not traced.
- L1485: Row rects (21, 5 + 16 n) 88x16 are taken from the PS2 battlemo layout (inferred;
- L1499: Inferred: front / back from the attacker's side of the target;

## Live NPC talk, event playback, server zone text, Items / Equipment (client/event.rs, client/items.rs)

`MODLOG.md:1509`

- L1518: The client sends monsters (0x00E @0x25 bit 0x08) down the menu path (inferred).
- L1519: +0x12c is taken to be u32 @0x25 of 0x00E (inferred from LSB): bit 30 = byte 0x28 bit 0x40, LSB "triggerable".
- L1582: The client cancels an idle event (all threads ended without 0x21) once 0x052 mode 0 has come, without waiting for a key (inferred).
- L1588: The client closes timed lines by itself and waits for a key only on a plain 7f 31 (inferred).
- L1590: Fade schedulers fdi1 / fdo1 (30904) have one op 0x0f each: RGBA (128,128,128,0) / (0,0,0,128) over 60 ticks. They are drawn as an overlay colour with alpha a / 128 (inferred).
- L1609: The client takes the next segment's delta within 8 ids (inferred rule;
- L1631: While equipping, the item list is drawn at x 314 (inferred from the three positions of the list's vfunc +4).
- L1632: While equipping, the item list is drawn at x 314 (inferred from the three positions of the list's vfunc +4). Backspace / Delete removes the slot's piece (inferred key).

## Event motions, actor fades, PC battle motions (dat/motionfile.rs, client/actmotion.rs)

`MODLOG.md:1644`

- L1647: Inferred items are marked *(inf)*
- L1662: Kind 0 is taken to be real players *(inf)*
- L1674: … = FUN_100cce00: scheduler 4cc from the actor's files (FUN_100cc440), else from DAT 0 (FUN_10071510). Subs 5 / 6 also pass val@0x10 to vtable +0x2a8 (a scheduler speed *(inf)*
- L1677: 0x5f sub 7 asks vtable +0x2ac instead (handled as the same wait *(inf)*
- L1685: u16 @0x18 is the blend-in ticks *(inf from its use as +0x14 and 1 / ticks at +0x18)*
- L1702: Client: Actor.alpha (0x80 = opaque) goes into the vertex alpha and the actor pipeline alpha-blends (SRCALPHA / INVSRCALPHA *(inf)*
- L1703: Actors an event faded are made opaque when it ends *(inf)*
- L1708: … battle files (FUN_100cf650, or FUN_100cf900 with dual wield +0x8b5) and calls vtable +0x160; state 1 plays scheduler "in N"; state 2 vtable +0x31c(1, 0) (weapon to the hand *(inf)*
- L1754: … bone (PC Elvaan main 1, file 14745: wep0 -> bone 77 = skeleton point 112 = main info[6] 0x70); points 112..125 are carry bones, 126 (bone 66) / 127 (bone 52) the hand grips *(inf: which is which hand)*
- L1798: The log's punctuation ("(Name)", "<Name>", "Name>>") is inferred.
- L1807: Ours counts in 2-tick frames whatever the frame rate *(inf: at 60 fps retail repeats every 3 ticks;
- L1812: … & 0x41 while typing; then pad callback 0, then the three bindings with their flags (FUN_101201c0 Ctrl = 0x1d / 0x9d held, FUN_101201f0 Alt = 0x38 / 0xb8, FUN_10120220 Shift *(inf: 0x2a / 0x36)*
- L1844: *(inf: which)*
- L1856: Q / E (0x80000000 / bit 32) are read by no callback *(inf: unused)*
- L1874: … face-camera strafe (type 2); 0xa9 / 0xaa Q / E turn (type 2); 0xa4 / 0xa5 the zoom axis (map zoom); 0x35 LShift, a list's Shift selection (FUN_10188060, FUN_101a70d0) *(inf)*
- L1888: … sets); 0x89 / 0x8a pad palette toggles; 0x0e fishing (Enter / Y); 0x0f events and fishing; 0x8b 0x8c 0x96 the mouse buttons; 0x9b Shift+Ctrl+Alt+D and 0xa6 D, lobby hooks *(inf: debug)*
- L1889: … 0x8c 0x96 the mouse buttons; 0x9b Shift+Ctrl+Alt+D and 0xa6 D, lobby hooks *(inf: debug)*; 0x00..0x03 arrows in list windows (Up with Esc held a debug toggle FUN_10077000 *(inf)*
- L1927: … look (5, 0.25, 0) about Z by +pitch, below the eye, which ours matched; the user reports retail's first-person mouse Y the other way (2026-11-02), so ours now follows that *(inf: our reading of the Z rotation's sense or of the mode-5 axis in first person is off somewhere)*
- L1936: Ours uses the posed model's top *(inf: bounds)*
- L1947: Type 2: FUN_10253af0 *(inf: wheel)*
- L1957: Ours: no ceiling, no side rays, no path mode (DAT_10444458: the camera retraces the look's path when the view keeps being blocked *(inf)*
- L1962: Our own model fades out at 0.0625 a tick (actor +0x167 -= ticks x _DAT_1031f860, FUN at all.c ~146770) *(inf: back in at the same rate)*
- L1965: Ours, not retail: face() (zone-in, event end, drives) puts the eye 5 back and 1 up, in front of what the look -> eye ray hits *(inf)*

## Creator screen: script text, help line, backdrop camera, shadow, size, nation flyover, CPU (2026-10-21)

`MODLOG.md:1975`

- L1978: Inferred items are marked *(inf)*
- L1984: Flag 0x20 = no window frame (FUN_101144b0 skips FUN_1022a290 for it); such windows draw their sprite whole *(inf: all cells)*
- L1993: Origin FUN_10159920 mode 0 = (x1 * s/2, y1 * s/2) + p with s = [ECX+0x84]; mode 3 is (1 - s/2) x1 - 512 + p and gives the known W - 512 offset only with s = 0, so the origin is (0, 0) *(inf from that)*
- L2007: Frames counted at 30 per second *(inf: the default frame rate)*
- L2009: XISTRING (file 48 = group 8): the u32 length's high half carries flags (0x10000 on the %-code strings); dat/msg.rs masks it *(inf)*
- L2056: … colours 8-bit), one vertex / index / uniform buffer through a StagingBelt, texture bind groups cached; the creator screen redraws at 30 fps (WaitUntil in about_to_wait) *(inf: 30 fps = the client default)*

## HUD, targeting, lock-on, movement, GPU-side rendering (2026-10-21)

`MODLOG.md:2059`

- L2062: Inferred items are marked *(inf)*
- L2068: Numpad * also binds 0x7e (context 0x04: a recentre, *(inf: the map)*
- L2071: /heal (FUN_1007adb0) also needs FUN_101c2550(0x60, 0) (mask bits 0x20 \| 0x40, not decoded); with FUN_100f9220() & 0x80000 it first clears DAT_10444110+0x18 bit 3 and calls FUN_100f9370(0x80000) *(inf: stops auto-run / follow)*
- L2076: Events: the message window's init FUN_10132230 clears window +0x84 when not cancellable (meaning not traced).
- L2084: Actor +0xf9 = 1 free / 0 locked (vtable +0x330 / +0x334); +0xfa is toggled by key 0x45 (Numpad /, Ctrl / Alt + Z) *(inf: first person)*
- L2092: The free look-at: pos.y - 0.6 h (h the bbox height, capped 2.5 with +0x14c set *(inf: mounted)*
- L2100: Name plates: with DAT_10443fe4+9 == 0 the target's colour loses its low byte (XOR AL,AL; *(inf: white turns yellow)*
- L2103: Inferred, open: the tick unit of DAT_10443fe4+0x28 (60 Hz), who sets lock mode 2, the +0xee kinds and +0x120 bits 9 /
- L2127: Frame height eases 8 px a tick while more than 16 px off, else 2 px *(inf: per 1/60 tick, retail per frame)*
- L2131: msgline: (16, H - 38), (textw + 14) x 22, windowps 37, text at +2 / +2 *(inf: +2 more for the caret gap)*
- L2141: … a server datagram came within 10 s, else -6 (FUN_100f9b20); netmeter sprites in order 12,0..11, base 0/9/5/1, a frame every (11 - pct / 10) draws, taken at 30 draws a second *(inf)*
- L2149: The disk's texture-north alignment and the letter mapping (s4 N red, s1 E, s2 W, s3 S) are *(inf)*
- L2155: s2c 0x058 {UniqueNo @4, AssistNo @8}: our id and a modelled actor -> set the target; lock when it is a monster *(inf: +0x120 bit 13 = the 0x00E monster bit)*
- L2163: Locked movement (FUN_100a6210): side keys circle the target at the same radius, forward / back change it down to 0.8 *(inf: the two radii)*
- L2197: Rates are applied per 1/60 tick with powf so they hold at any frame rate *(inf: retail assumes one tick a frame or more)*

## Cutscene fixes after the 0x1f7 validation, lobby fades, HUD party / target, no egui fallback (2026-10-21)

`MODLOG.md:2255`

- L2257: Inferred items are marked *(inf)*
- L2258: Inferred items are marked *(inf)*
- L2275: FUN_1008cce0 sets the actor's speed / speed base to body +0x16c / +0x168 x 0.8 (_DAT_1031e8a4); the reference unit (units/s vs tenths) is not confirmed *(inf: 4.0 units/s)*
- L2312: Bound actors use status 0 for their motion *(inf: +0x170)*
- L2327: The place goes through FUN_10159920 mode 0, taken as plain pixels (as the creator's origin) *(inf)*
- L2332: … width (DAT_105dc3e4 +0x32), counts its lines (rem4line +0x4e, list +0x14, count +0x24) and writes it to the log ring (FUN_10124050) with entry +0x12 = 1 ("the open message's" *(inf)*
- L2353: Ours: in the GPU pose pass, each of the two bones turns its share (half) of the arc from the body's forward to the point *(inf: retail's per-bone aim is not decoded)*
- L2354: The target point is the rest-pose attach point *(inf: retail reads the animated one)*
- L2361: The standard-model table (0x10352414 by FUN_10082d70) is not traced.
- L2365: … (the one drawn last frame) first; bit 20 appends the player's follow camera (eye = attach point 2 x (-3, 0, 0), target = position lifted by 0.6 x its height or 1.2, focal 350) *(inf: inputs)*
- L2375: Op 0x10 captures the frame before the cut and fades it out over 60 ticks (SRCALPHA); ours renders that frame once offscreen from the last camera with last frame's poses, in its own submit *(inf: the capture source)*
- L2384: UI fade: the UI shader multiplies by the level (colour and alpha) *(inf: op 0x51 as a modulate)*
- L2409: Name plates in cutscenes: ours show over visible NPCs; whether retail hides them during events is not traced (FUN_100ce4d0's vtable checks).
- L2436: 0x46: every sub advances 2 and does nothing while the player has no model; sub 1 calls FUN_10217e90 *(inf: hides a HUD window)*
- L2439: Motion scheduler op 0x28 (32732 'tlk1'): FUN_100c6b70 + FUN_100cb8e0, a 10-tick blend into a chosen motion *(inf: back to the stance)*

## Top bar fade, menu titles, target gauge, map window, log focus, lock-on release, loading indicator (2026-10-21)

`MODLOG.md:2454`

- L2457: Inferred items are marked *(inf)*
- L2472: Manager init sets the timer to 30 and parts start shown: HudTop::enter() on every zone-in *(inf: FUN_10010080 is the in-game start)*
- L2473: The rule runs only in game state 0x60; zoning and the zone load are taken as 0x70 *(inf)*
- L2480: Active also while titlewin's "HN" mode (+0x448, a PlayOnline handle *(inf)*
- L2486: … update (manager 0x105dc2e0, timer +0x78) only while the value at DAT_10444110 is 0x60; 0x70 is set by FUN_100897b0 ('zmrq') and FUN_1009d500 case 8, cases 5 / 9 set 0x60 back *(inf: zoning)*
- L2489: Part 0 (FUN_10114f00): +0x44 state, +0x48 count, +0x0c alpha factor; state 0's factor is 0.58 when part +0x4c is set *(inf: dimmed)*
- L2493: No event, chat-input or cutscene case: in a cutscene the bars go because the bar is empty *(inf: no event text writes helpwind or titlewin;
- L2494: … in a cutscene the bars go because the bar is empty *(inf: no event text writes helpwind or titlewin; not checked site by site)*; whether netstat's content hides in cutscenes is not traced.
- L2502: help), map0. Per-item overrides (+0x4c / +0x50) are not traced.
- L2530: Home Point #2, Cheval River) set neither, so on LSB NPCs show a gauge (live3c_1). Retail's server presumably sets one of them for ordinary NPCs *(inf)*
- L2555: View: the player under (W * 200 / 512, H / 2) clamped to [base, base + max(0, 512 - W / zoom)] (FUN_101e7a60 / FUN_101e6110); retail animates that scroll, ours starts there *(inf)*
- L2564: Not done: the open / close fades, the recentring scroll, pan / zoom keys (input context 0x26, not traced), the sub-map key, Markers (mapv3) / Wide Scan / Change Map, other markers (party, POIs, .MRK files), the Mog House branch (myroom), the group 7 messages 97 / 98, the mouse.
- L2580: Tabs (table 0x1032867c): fep sprites from (x0, y0 - 9), slot s unselected A[2s] / selected A[2s + 1], 30 px after an unselected one; the filter-8 tab only with config 0xb4 (default 0 *(inf)*
- L2587: … log while engaged): a window that takes the keys (main menu, item windows, action menu, confirm, config) takes the focus from the log when it opens, not while it stays open *(inf: the newest window has the focus)*
- L2601: … (6 by default), handed back to the yaw / pitch orbit once settled or on manual input; the height hold and the wall-ray recovery are simplified to a ray from the look point *(inf)*
- L2614: +0x84 / +0x88 screen fractions are 0 *(inf: no other writer found)*
- L2617: Ours: from world entry and NetEvent::Zoning to the end of the zone load; draws are taken at 30 a second *(inf)*

## Player movement collision (2026-10-21)

`MODLOG.md:2627`

- L2644: Type 9 is tested in FUN_1017a6d0 and types 8 / 9 with status 33 (resting), so they look like water *(inf)*
- L2708: Free camera look point (FUN_1001e030, loop after LAB_1001e5b7): eases toward the head 0.25 of the gap a tick (whole ticks, at least one a frame); ours applies it per tick with powf *(inf)*
- L2720: … goto runs): East Ronfaure, holding forward 15 s into a 54 degree stone buttress outside the San d'Oria walls (foot (95.9, -60, 244), top -64.5) stops at its foot, y -60.0 *(inf: the old \|n.y\| >= 0.3 rule took these faces as floor)*

## Select Character screen: camera, name list, info panel, Options, prompts, zone names (2026-10-21)

`MODLOG.md:2724`

- L2727: Inferred items are marked *(inf)*
- L2734: This confirms the earlier *(inf: 30 fps)*
- L2757: FUN_102371c0: window at ((1 - s/2) W - 640, (1 - t/2) H - 480), s = t = 0 *(inf: no writer found)*
- L2760: Hand anc_s at the cursor child *(inf)*
- L2762: … = the fontshp sprite font (not the game font), fitted to the child width 128: pen starts at 8, at the limit the rest becomes an 8 px piece (the ellipsis sprite 0x6c *(inf)*
- L2799: Escape in a prompt answers No *(inf: the result -1 path of FUN_10233cb0 was not traced to a key)*
- L2802: File not found." *(inf: the file-missing path)*
- L2810: Opened by message 8 on a used slot (right button inside the list, FUN_102370d0; numpad + per the caption and key function 0x11, *(inf)*
- L2815: File not found." / 0xf0 "Unable to load ..." *(inf: the -1 path)*
- L2855: With focal 350 (the camera ctor; the lobby does not change it *(inf)*
- L2864: Our "Connecting to the server..." busy line has no retail source found *(not traced)*

## Config: the option store, the in-game Config menu, lobby Config, Config-tool settings, background resolution, target flash (2026-10-21)

`MODLOG.md:2866`

- L2869: … settings, background resolution, target flash (2026-10-21) Decompile: all.c / all.asm (FFXiMain) and the US "FINAL FANTASY XI Config" tool (v5.0, decompiled headless). Inferred items are marked *(inf)*
- L2936: A window under a later one at the same place (k1assign over conf6win, textcol3 over textcol2) is not drawn *(inf)*
- L2945: … style (files 14..21, textures swapped in the UI renderer); 0xb4 the full log's second-window tab; 0xb5 timestamps "[hh:mm]" / "[hh:mm:ss]" (12-hour, local time) on chat lines (*(inf: placement)*
- L2949: … message filters on the client (FUN_100abf20: kind table 0x10350a60, 11 relation tables from 0x10351bbc; party / alliance relations not modelled); 0x3c nearest-N other PCs drawn *(inf: which are dropped)*
- L2951: … is on (aspect = 1 / ((h / 0.75 w) x 0.791), FUN_100147a0); 0x8b / 0x8c invert the mouse camera; 0x8f edge panning (pointer within 2 px of the left / right edge turns 1.5 rad/s *(inf)*
- L2952: … the mouse camera; 0x8f edge panning (pointer within 2 px of the left / right edge turns 1.5 rad/s *(inf)*); 0x91 chase cam (the free camera eases toward the facing while moving *(inf)*
- L3008: A half-size target exists too *(inf: glow)*
- L3034: … draw FUN_1002a8c0; FUN_100c9da0 is reached through FUN_100c9d20 at vtable slot 0x288 of 0x10322540, 0x10325d70 and 0x10326218, so every skeleton actor (PCs, NPCs, monsters) *(inf: which class is which not mapped)*
- L3046: It is strongest where the zone's actor ambient is low (night, dungeons) and almost invisible near 0.86 *(inf: why "some monsters do it")*
- L3052: … whenever a target is set and the player actor exists; the locked camera (FUN_1001e030), locked movement (FUN_100a6210) and "/target %d" (FUN_100a8510) read it too *(inf: normally the same actor)*
- L3059: … FUN_1008dc40: every actor within a radius (FUN_100995f0) and dy^2 <= 64 of a point from entity +0xcc / +0xd0, while DAT_104cd388 != 0 and entity +0x134 >> 20 & 0xf == 2 *(inf: an area-of-effect preview)*
- L3109: *(inf, open)*

## Lock-on on engage, walking hitches: GPU mips, budgeted uploads, shared models (2026-10-21)

`MODLOG.md:3134`

- L3136: Inferred items are marked *(inf)*
- L3137: Inferred items are marked *(inf)*
- L3146: A left click on nothing never clears the target (FUN_10151d20 keeps it and runs FUN_101524e0(0), the confirm key on it): ours keeps it and talks to an NPC target *(inf: monsters take no action on that click)*
- L3151: … toggle; the rest unlock (status leaving 1 / 3, death, +0x22 = cursor moved off the actor to a ground point, invalid target, target change, server-forced position FUN_1008b680 *(inf)*
- L3157: The physical pad button is the 5th entry of the pad list (FUN_10004eb0, local_f0[4] = 10) *(inf)*
- L3160: Ours keeps facing the target every frame while locked (the user's spec; differs from that reading *(inf: retail's schedules may turn the attacker)*
- L3216: The few spikes left are waits before the redraw starts with ~1 ms of work (compositor / driver *(inf)*

## Effect generators: zone, weather, sprites (fx/, render/effects.rs, effects.wgsl) (2026-10-22)

`MODLOG.md:3226`

- L3251: **Inferred / left out**: our random sequence is not retail's MT19937 (only the value mappings are);
- L3285: with names 0b10..0b32 / pb41: bit 1, read in FUN_10045950), meaning inferred.
- L3355: Culled against the scenery draw distance (FUN_1017b170; the planes inferred).
- L3374: Not traced: data 0x80;
- L3383: Header: i16 0x00 h0 (UVs divided by the texture size at init when h0 >= 1 and byte 0x07 is 0) *(inf: texture count)*
- L3400: Fallback: the global texture list, entries from `syst/effe` (flag 0x100) or the current zone (+0x34), compared on name bytes 8..15 only, the most recent first *(inf)*

## Effect schedulers: melee sparks, casting, spells (fx/sched.rs, fx/tasks.rs) (2026-10-22)

`MODLOG.md:3423`

- L3444: Attachment (FUN_1003afd0): modes 1-8, 0xd, 0x10-0x13, 0x16-0x1b with points from the rest pose; 9-0xc (bone matrices) use the point and the actor's rotation *(inf)*
- L3445: Special points 0x30 (actor position *(inf: retail rays to the ground)*
- L3446: Special points 0x30 (actor position *(inf: retail rays to the ground)*), 0x31 / 0x32 (body point 0xd..0x14 nearest the other actor), 0x33 (nearest the eye), the rest point 0 *(inf)*
- L3463: Retail drops them at the task end (+0xac = 0x100a7440); ours stay until the next skill *(not traced)*
- L3491: … actor1 vtable +0xa4 & 3; 0x3b actor +0x840 bit 6 (dual wield and an off-hand 'b' swing, FUN_100d4600); 0x3c entity +0x124 >> 8 (& 0xf if that nibble is 8, else & 3; meaning not traced);
- L3519: Categories 3..6, 0xb, 0xd..0xf are dropped when the caster is us, DAT_10475440 != 0 and (DAT_1046df04 \| DAT_1046d210) & 0x1000 is clear (those globals not traced).
- L3531: Not traced: scheduler op 0x56 (an actor drive task);

## Effect sound (fx/sound.rs, audio voice handles) (2026-10-22)

`MODLOG.md:3533`

- L3539: … point of an Sph capsule chain: points from block u32 @0x10 + 0x10 at 0x20 bytes (x, y, z, r), segment pairs from block u32 @0x14 + 0x10; pulled in by the radius, zero inside *(inf: the clamp, FUN_10054fb0 is garbled)*
- L3543: Scheduler ops 0x0a (actor1), 0x0b (actor2), 0x53 (*(inf)*
- L3559: SeSep chunk (resource type 0x3d): "SeSep " @0 (5 bytes compared, FUN_1023b920); u8 @7 (0, 0x14, 0x1e, 0x28 seen) and u32 @0x10 (2, 3, 4, 6) are not read by the PC paths; @0xc low 31 bits a length *(inf)*
- L3560: … seen) and u32 @0x10 (2, 3, 4, 6) are not read by the PC paths; @0xc low 31 bits a length *(inf)*; the bytes from @0x14 are never read on PC (command 2 passes only the id) *(inf: PS2 sequence data)*
- L3580: … holds 12 other SeSeps (the same one may start again; cleared at the end of the manager pass); -1 when the sound thread's busy voices >= DAT_1044417c (12, set in FUN_10016b00 *(inf: a config value)*
- L3582: The thread's voice stealing (FUN_1023fb90) is unreachable while the client refuses at 12 busy voices *(inf)*

## Lens flares (CMoLfdElem; render/flare.wgsl, vs_flare / vs_glare) (2026-10-22)

`MODLOG.md:3598`

- L3632: The test draws into DAT_10444438 (scene-sized, the screen filters' target) with the main depth buffer; the scene target is device+0x194 (inferred: so the test only touches a scratch copy).
- L3639: FUN_100125c0 (called at 0x1001178c, after the scene; before the UI inferred) draws it when rgb != 0: a = min(255, 2 A), one viewport quad, SRCALPHA / ONE, so dst += rgb / 255.

## D3b morph elements (CMoD3bElem; fx/lib.rs Morph, vs_morph) (2026-10-22)

`MODLOG.md:3644`

- L3670: … obj+0x40, stage 0, elem+0x140, +0x144, +0x148): a plain SetTexture when +0x148 == 0, else a sine-warped copy rendered into DAT_1046a828 (the writer of +0x140..+0x148 not traced).
- L3674: … (K 2, V 30, N 93, T 31), ops 0x33 / 0x34 bind pt01 / pt02 to slots 9 / 6 with repeat 200; update 0x1e / 0x1f -> w0 / w1: the two key meshes crossfade 200+ times over the life (inferred: wing flaps).

## Footsteps (FUN_100d2970; client/footsteps.rs, render/points.rs) (2026-10-22)

`MODLOG.md:3676`

- L3692: Motion gate (inferred): retail tests the base motion against the idle 'idl?' (+0x7d8 != +0x7c8) and a few flags (scheduler op 0xbc's counter, a landing);
- L3701: Retail keeps one clone per actor and restarts it *(inf: a new instance per step)*
- L3714: *(inf: the sign of +0x210: the floor normal turned to +y, so flat ground gives identity, which is what fmrk's flat-lying rotation needs.)*
- L3727: It returns at once, keeping the stored heights, when actor+0x8ac bit 0 is set (invisible, vt+0x260 = FUN_100a2e40) or the entity's +0x128 bit 0 is clear *(inf: shown / active;
- L3738: The alt name (not done): zone flag zone+0x1cf (DAT_106175ac) and surface 3 -> byte 1 = '2' *(inf: a zone whose surface 3 borrows surface 2's sounds;
- L3747: The motion gate as retail has it (ours above is inferred): a step needs actor+0x8ab != 0 (a counter held up by scheduler op 0xbc, FUN_10060cc0 / FUN_10060c80 *(inf: for the scheduler's remaining time, FUN_1005d5e0)*
- L3748: The motion gate as retail has it (ours above is inferred): a step needs actor+0x8ab != 0 (a counter held up by scheduler op 0xbc, FUN_10060cc0 / FUN_10060c80 *(inf: for the scheduler's remaining time, FUN_1005d5e0)*
- L3751: … / scheduler op 0x28, +0x7c8 from op 0x8c / FUN_10088be0), entity+0x128 bit 1, or actor+0x8b6 (set in FUN_100c4390 when FUN_10082fe0() and +0x8bc < 0, which then becomes 32.0 *(inf: a landing)*
- L3756: 'fvib' (not done): actor+0x814 = a clone of the type-5 'fvib' from the actor's files (made at model load, FUN_100cd1d0 / FUN_10084000), restarted as one burst on every step *(inf: a heavy-step effect)*

## Effect area-light tint (emission 0x8f / 0x90, update 0x69) (2026-10-22)

`MODLOG.md:3809`

- L3831: … the zone root's child folder named id, else the root; linked at the tail of the chain DAT_106175ac -> +4 (FUN_101818f0); it takes the previous member's weather (FUN_101804d0) *(inf: arguments garbled in the decompile)*
- L3853: Zones 0xb6 / 0x84 (XiZone +0x1b4) *(inf: Walk of Echoes / Abyssea - La Theine)*
- L3863: Crossfade (FUN_10183dc0): with an older weather at +8, w = (dur - rem) / dur (+0x18, +0x14) and P = old + ftol((new - old) w) per byte (FUN_10182470) *(inf: +0x14 counts down to 0)*
- L3872: … slot area at elem + i16 elem+0x176), a B,G,R,A dword: R from byte 2, G byte 1, B byte 0; alpha (+0xfb) untouched. The slot holds the base colour saved at emission (op 0x91) *(inf: the pairing)*

## Effect spline paths (Vtx 0x3e, CMoSpline; fx/spline.rs, update op 0x34) (2026-10-22)

`MODLOG.md:3881`

- L3915: Elements removed without the expire stream leave the cache built (inferred: harmless).

## Effects filter (config 0x92; FUN_100ac4b0 / FUN_100ac560, fx/tasks.rs effects_allowed) (2026-10-22)

`MODLOG.md:3925`

- L3953: … aura), besides the Config page (load FUN_10190de0, which returns 0x12 rows; store FUN_10190e00) and the registration FUN_1018e330(0x92, 0, 4, 0, 0, -1, 0, 0) at 0x1018dc05 *(inf: default 0)*
- L3959: Message 0xa6 "Special actions started on/by others" belongs to another page *(not traced)*
- L3974: Flags (FUN_100e62e0): & 3 party number, & 4 party leader, & 8 alliance leader, as 0x0DD's member flags *(inf: that 0x0DD / 0x0C8 fill it)*

## Point lights on scenery (CMoPointLightProgElem -> the light table -> zone.wgsl) (2026-10-22)

`MODLOG.md:3981`

- L4043: … one area light (FUN_10184980), without it two (FUN_10181b50: the bg sun and bg moon colours, directions env+0xa4 / env+0xc4; the moon light at the light scale for all hours, inferred).
- L4064: L) x t2 + t3)), alpha = sat(4 x vc.a x tex.a); texcoord clamps t2 / t3 to 0..1. - Constants: c0..c3 transposed world-view-projection (inferred);
- L4078: FUN_10029590 enables 2 lights (DAT_104449a0[0..1]); per material FUN_100216a0 / FUN_1002cdf0 set them again with Diffuse x the material factor f and emissive max(f - 1.5, 0) (inferred: inside FUN_10029590's window).
- L4079: Not traced: c0..c3 and the order of the obj+0x80 inverse (decompile only).

## Effect fixes from the San d'Oria gate lamps; per-pixel point lights and soft alpha (ours) (2026-10-22)

`MODLOG.md:4081`

- L4095: D3DRS_ZBIAS 15 for the effect draw: -15 x 5e-6 in NDC depth in to_clip, as Wine's d3d8 maps it *(inferred: D3D8 left the scale to the driver)*

## Event schedulers run in the effects engine; the level-5 crystal scene (2026-10-22)

`MODLOG.md:4122`

- L4127: Inferred: the camera ops stay with the runner until the two are merged into one task list.

## Game file updates (PlayOnline patch protocol; src/patch.rs) (2026-10-22)

`MODLOG.md:4139`

- L4146: Not yet checked against our own decompile *(inferred until then: unpack polcore.dll 1.18.0.0 and confirm)*

## Tables copied from FFXiMain, ported to 30261006_0 (2026-10-22)

`MODLOG.md:4183`

- L4191: Log window tabs (FUN_10138800, list built by FUN_10138700 in .data 0x1036f940): 11 sprite pairs at 0x10333b4c; pairs 7 / 8 are filters 9 / 10 (Assist J / E, inferred from /assistj 9, /assiste 10 in the channel table 0x10355588), drawn 7 then 8 only in Japanese.
- L4198: 4 (FUN_1019bdd0): children 4 / 5 / 6 set 0xce = 1 / 2 / 0, 1 and 2 kept only while FUN_101bced0() > 0 (inferred: a gamepad count).
- L4203: Chat: /assistj /aj /assiste /ae (list 0x1037ade8), 0x0B5 kinds 0x22 / 0x23 (FUN_10078be0 / FUN_10078d80; the name to handler link inferred from the handler table 0x10356e34 order).
- L4206: Key functions: 0xbb (callbacks 0x1036db18); 0xb7..0xba test a second pad word (inferred: gamepad).
- L4208: … new 0xcb..0xce; defaults changed for 0x0f, 0x13, 0x33 (0x1800), 0x47, 0x86, 0x8b, 0x8c, 0x8f, 0xb7, 0xc0, 0xc2, 0xc3; 0xaf max 8; 0xab default computed from the desktop size (inferred;

## Sub-areas: building interiors (2026-10-22)

`MODLOG.md:4267`

- L4272: Hit test FUN_101760f0: the point in T(-pos) RotY(-yaw) S(1 / \|size\|) inside [-0.5, 0.5]^3 *(inf: the bounds, from the corner table)*
- L4294: … (own buffers, texture slots after the zone's, stand-in instances zeroed on the GPU), Effects::load_sub_area (Owner::SubArea; its auto-start generators start as a zone's *(inf)*

## Scenery and actors lit by their area's environment (2026-10-22)

`MODLOG.md:4301`

- L4321: … 0, an area table (render AREAS = 64, AreaGpu: bg sun / colour / ambient / fog, actor sun / colour / ambient) written per frame from ZoneWeather.areas (the sub-area's first *(inf: lookup order)*
- L4323: An area with no environment is lit as the zone *(inf: FUN_101889c0 finding nothing)*

## Scenery blending and the alpha cutoff (2026-10-22)

`MODLOG.md:4448`

- L4456: … cut out, blended (a blended sub-pass after each slot's opaque draws, close to retail's inline order), blend pipelines without depth writes and a constant depth bias of -8 *(inf: the D3D8 ZBIAS equivalent)*

## Targeting keys (2026-10-22)

`MODLOG.md:4499`

- L4515: Entity kind bits inferred from ours: PC by id, door, monster, else NPC.

## Event special values: race, job, server id (2026-10-23)

`MODLOG.md:4887`

- L4900: Its reader FUN_100977f0 gates something in FUN_10097840 for PCs (not traced);
- L4946: Both live on the actor (getters FUN_10084b40 / FUN_10084b80, vtable slots +0x13c / +0x144 of 0x1032cfe0, 100 when unset) and are read only while FUN_10087060 holds (event running, DAT_10487e48 *(inf)*

## Map window: pan, zoom, fades, event maps (2026-10-23)

`MODLOG.md:4997`

- L5022: A negative key is the one under the player in that zone, else 0 *(inf)*
- L5023: Without the arrow the view starts at the image base *(inf)*
- L5035: map0's Markers opens mapv3 (rows: icon table 0x10382178 at (16, 5 + 16 i), name at (0x1f, 7 + 16 i); font 0x44 taken as fontshp *(inf)*
- L5036: … taken as fontshp *(inf)*); Left / Right step the set skipping empty ones (FUN_101fad70); Enter on set 0 opens mapv2 (View / Place-Remove / Rename; its place beside the row is inferred), set 1 scrolls to the marker, sets 2 / 3 show only that member's label and scroll.

## Item info window: "menu    iteminfo" (2026-10-23)

`MODLOG.md:5049`

- L5104: … item info window, else FUN_1013e6d0 -> FUN_1013e4b0(item, byte 2 ? entry + 4 : 0, jump = byte 3) opens it in mode 4 ((16, 16), stacked on region 3, taken as the choice window *(inf)*
- L5113: Ours: Body::tilt, Pose::tilt, Place::tilt, tilt_matrix in the actor frame (z then x, signs and order inferred;
- L5115: … state 4: FUN_10126100 / FUN_10125fe0): the offset from the anchor over W x 0.047619 gives -1 / 0 / +1 per axis in place of the scroll keys; ours anchors at the press point *(inf)*

## Item use menu: "menu    iuse", Use and Drop (2026-10-23)

`MODLOG.md:5118`

- L5141: Grammar codes in FUN_100bc8a0 (English): items 0x20..0x22 take the index as the id, 0x23..0x25 the table +0xc64, 0x26..0x28 +0xc50 (ours: both the work values *(inf)*
- L5143: A count of 1 (codes 3 / 4) prints nothing and goes to FUN_1024fcd0(0); ours also drops the space after it *(inf)*

## Item sort window: "menu    itmsort2", Auto and Manual (2026-10-23)

`MODLOG.md:5201`

- L5214: The same row splits one off with c2s 0x029 {1, inv, inv, index, 0x52} when the stack holds more than one and stacks (0x52 as the target index is inferred to mean the first free slot).

## Equipment compare: "menu    itemxinf" / "menu    itemyinf" (2026-10-23)

`MODLOG.md:5218`

- L5228: … itemxinf (DAT_10578208, a subclass of the iteminfo object) by FUN_1013e410 mode 5 with that item, or with item 0 / container 0x12 when the slot is empty (an empty frame, inferred from the code path;
- L5237: The compare shows its first page only *(inferred: each window keeps its own page state in retail;

## Event ops 0x7e (stance / mount) and 0xab 9 / 0xa (2026-10-23)

`MODLOG.md:5242`

- L5254: … +0xfe = val@8, feet +0x106 = val@10, hands +0x102 = val@12, legs +0x104 = val@14 (0xe) or 0; face, main, sub and range cleared; +0x120 bit 30 = val@2 in {3, 4, 8} (meaning not traced).
- L5259: … too) and empties the temporary items and the recycle bin at zone-in, so ours resets the inventory (but the container sizes) at ZoneIn; that retail resets its copy on connect is inferred.

## Wide Scan leftovers: the busy strip, the call-for-help pointer, `<ht>`, scanlist paging (2026-10-23)

`MODLOG.md:5264`

- L5270: FUN_10221740 counts a busy bar as active (the bars stay up; inferred from the caller testing AL).

## Recycle Bin (2026-10-23; spec: scratchpad recycle/SPEC.md of that session)

`MODLOG.md:5293`

- L5323: Config 0xab default (FUN_10195460 call after 0xa8): FUN_100156a0(width, height) with the size from DAT_104568fc +0x10 / +0x12 (FUN_10015600 / FUN_10015610; taken as the desktop, inferred): the index of table 0x1034f4a0 (4:3, 15:9, 15:10, 16:9, 16:10, 5:4 as float pairs) with the smallest \|a - r\| / r, -1 for a zero size.

## Random names in the creator (2026-10-23; 30261006_0: FUN_10235310, FUN_10238330, trigger FUN_102384d0)

`MODLOG.md:5335`

- L5346: Name = adjective + noun when shorter than 16 letters (else truncated to 16, then dropped by the < 16 test), then FUN_101ba240 (a validity check, not traced;

## Magic / ability sort windows and the order files (2026-10-23; spec: scratchpad magsort/SPEC.md of that session)

`MODLOG.md:5349`

- L5364: Auto-sort: sortyn 0xe 0x50 "Sort list?" (16 px lower under blusortw, inferred), Yes = FUN_1022c070: every known id by record +0x3e (SE's family order) with the other types zeroed, then the other types' ids refill the holes in their old relative order;
- L5375: Ours skips it while a list, category window, item window or prompt is up (the item windows: inferred).

## Storage views (2026-10-23; spec: scratchpad storage/SPEC.md of that session)

`MODLOG.md:5381`

- L5418: Up / Down in itmstora skip the disabled rows, and it opens on the first enabled one (both inferred).

## Party-list cursor: `<stpt>` / `<stal>` (2026-10-23; spec: scratchpad stpt/SPEC.md of that session)

`MODLOG.md:5421`

- L5444: … x type for list / cast mode, abilities +0x70 + 4 x type, written by the close hooks FUN_1022c990 / FUN_102243d0): the list reopens on the remembered row (scrolled into view: inferred).
- L5451: Retail takes temporary items and a few table counts off the main list\x27s count (not traced);

## Alliance windows: "menu    raid1" / "menu    raid2" (2026-10-23; spec: scratchpad raid/SPEC.md of that session)

`MODLOG.md:5469`

- L5481: Ours maps that to world.menu / win / config / map / magic / ability / palette / event (inferred).

## Moogle menus: s2c 0x02E, "menu    myroom", "menu    mogext", the storage menus (2026-10-23; spec: scratchpad moogle/SPEC.md of that session)

`MODLOG.md:5521`

- L5558: /dig (id 0x37, flags 0x41, FUN_1007aaf0) on a chocobo: c2s 0x01A category 0x11 (LSB ChocoboDig); its place checks (refusal index 8 = group 7 0x4a) are left to the server (inferred).

## The emote list (2026-10-23; spec: scratchpad mount/SPEC.md section B of that session)

`MODLOG.md:5561`

- L5575: /sitchair [n] (id 0x34): c2s 0x113 {u32 mode 0 toggle, u32 chair} (LSB's layout; the handler's checks are not traced).

## Change Jobs: "menu    jobchang" / "menu    jobcselu" (2026-10-23; spec: scratchpad moogle/SPEC.md section 4)

`MODLOG.md:5581`

- L5587: Enter opens jobcselu (16, 48) with +0x14 = 1 / 2; its cursor starts on child 1 *(inf)*

## c2s 0x01A ActionID 0x14 (LSB SendResRdy) (2026-10-23)

`MODLOG.md:5612`

- L5627: Ours (client/mod.rs res_rdy): once the world is shown, in the Mog House after 20 ticks of 1/60 s *(inf: the frame length)*

## Name plates under the screen fade (2026-10-23)

`MODLOG.md:5633`

- L5640: FUN_10160650; the target arrow is a window too *(inf)*
- L5644: Ours draws plates in the HUD batch after ScreenFx; their colour is now scaled by the same multiply (and the zoning fade), which equals retail for an alpha-blended plate *(inf: up to the clamp of MODULATE4X)*

## /jump (2026-10-23)

`MODLOG.md:5706`

- L5716: LSB pushes 0x11E with CHAR_INRANGE_SELF, so the jumper gets it too; by the logic above that would send 0x11D again every time, so ours ignores our own index *(inf: retail's server likely does not echo)*

## Mouse picking in the world (2026-10-23; spec: scratchpad mousepick/SPEC.md)

`MODLOG.md:5720`

- L5736: … / 15). Ours reads the bones back from the GPU pose (render/points.rs) for the actors whose rect held the pointer, a frame or two late; before any came back the rect decides *(inf)*

## Config: Extras and Settings as rows and pages; timestamps on every line; the menu keeps its page (2026-10-23)

`MODLOG.md:5841`

- L5874: The main menu opens on the page it was closed on (the user's account of retail; FUN_1013b3f0 / socialme's open path not traced).

## Chat log fixes, first batch (2026-10-23)

`MODLOG.md:5880`

- L5888: MsgShift (inferred): ids outside the matched segments take the nearest segment's delta (a gap of up to 8 just before one takes the next one's).
- L5897: That the invisible refusal (our +0x128 bit 6, 7031 0xac) means invisibility is inferred.

## The zone's ambient loop (2026-10-23)

`MODLOG.md:5915`

- L5932: Not traced: how retail ends the old voice at a change (we stop it), the event override of the time.

## s2c 0x017 chat lines and our echoes (2026-10-23)

`MODLOG.md:5937`

- L5959: The printed name is 7F FC + name + 7F FB (FUN_100da580, strings 0x10331808 / 0x1033180c); the pair draws nothing *(inf: it lets the log find the name)*
- L5967: … DAT_10456a28) and returns 1 either way. Mode byte 1 is a flag byte: 0x80 the Attr GM bit, 0x40 for the NS and system kinds, 0x20 when a 0x22 / 0x23 line holds 0xEF 0x2f..0x4c *(inf: read by the log;
- L5971: Profanity mask (not done): FUN_101ba200 -> FUN_101ba350 masks words in place from vulgar2.dic ("Vulgar", object DAT_10662db8, FUN_101ba390) while that object's mode (+4) is 0 *(inf: the profanity option)*

## The server message: c2s 0x04B / s2c 0x04D, /servmes (2026-10-23)

`MODLOG.md:6041`

- L6065: … top-level state 5 (lobby -> game, FUN_10011a00) and cleared in states 0xb (logout to the lobby), 0xc (zone change), 0xd (title): "first zone since the lobby" is our name for it (inferred).
- L6082: Header FUN_100a31d0: the name is the world-list record `DAT_104df770 + 0x13840 + world * 0x8c` (FUN_100efcc0; the name at +0 inferred).
- L6085: Each segment keeps its terminating byte; what the log does with it is not traced (ours strips it, inferred).
- L6086: Each segment keeps its terminating byte; what the log does with it is not traced (ours strips it, inferred). "a\r\nb" gives "a\r", "\n", "b": the lone "\n" is probably an empty line (inferred).
- L6088: FUN_10127c00 drops modes 200..201 while DAT_1066318c & 8; the only setter in this build is FUN_101d2260(0x1000, 0) (inferred: a debug switch, never set).

## Auto-translate: the dictionary and the phrases in the log (2026-10-23)

`MODLOG.md:6103`

- L6128: … (count +0x1c), so phrase id N is words[N] (FUN_101bae40 indexes directly); sorted array +0x20 (count +0x28) by key: case-insensitive in English (FUN_10325d40, a strnicmp *(inf)*
- L6129: … indexes directly); sorted array +0x20 (count +0x28) by key: case-insensitive in English (FUN_10325d40, a strnicmp *(inf)*), strncmp in Japanese, FUN_101c88a0 (accent folding *(inf)*
- L6138: The cache holds the item records last looked up (bags, equipment, shops) *(inf: what fills it;
- L6142: Entries from the key item table (FUN_101549e0 count, FUN_101549f0(i) id, FUN_10154550(i) name) that FUN_1022aff0(i) accepts *(inf: owned)*
- L6143: Entries from the key item table (FUN_101549e0 count, FUN_101549f0(i) id, FUN_10154550(i) name) that FUN_1022aff0(i) accepts *(inf: owned)*, stopping at the first id 0x181..0x183 *(inf: why)*
- L6146: … 0) / 0x15 (hi 0 -> FF) / 0x16 (lo 0 -> FF). - The category list (no word, or no match) is every group's title in load order: the 42 file groups, then Items and Key Items *(inf: the order of the two)*
- L6150: The input code never makes one *(inf: from servers / scripts)*
- L6158: … the log add FUN_10127c00 when its param 5 is 1 (every chat line, the tell echo ">> %s : %s", most system chat); FUN_100d9b20 (s2c 0x017, and 0x0CA, the bazaar / inspect message *(inf: LSB's name)*
- L6159: … (s2c 0x017, and 0x0CA, the bazaar / inspect message *(inf: LSB's name)*); FUN_101dd780 (linkshell message lines, chat mode 0xcc, 40 bytes a row, each row expanded alone *(inf: s2c 0x0CC)*
- L6160: … name)*); FUN_101dd780 (linkshell message lines, chat mode 0xcc, 40 bytes a row, each row expanded alone *(inf: s2c 0x0CC)*); FUN_101ddcb0 (41-byte texts in the same module *(inf: the linkshell message window)*
- L6166: No 0xFD handling was found on the event message path: NPC text has no phrases *(inf: absence)*
- L6169: … the runner sends them raw (FUN_10080dc0 -> FUN_101ad4b0). The editor shows them with braces and edits them in the popup FUN_101c5be0 (kind 9, max 60, titles 8; FUN_101a54f0) *(inf: Tab completion works there;

## Auto-translate input: Tab completion and menu fep (2026-10-23)

`MODLOG.md:6178`

- L6191: Ours places it above the chat line at the word *(inf)*
- L6207: Keys reach FUN_101b5d10 as Windows VK codes from DIK through table 0x1037c3e8 (FUN_101aff00); its third argument is a modifier flag (Shift *(inf)*
- L6221: … history (DAT_1066268c, 16-byte entries at +0x8c, count +0x88; FUN_1018d6e0 adds a name when a tell is sent), the name FUN_101598b0 returns (+0x214, or +0x7c when +0x210 is 0; *(inf: the reply target)*

## NPC shops: buy and sell (2026-10-23)

`MODLOG.md:6320`

- L6350: Opening: shopmain +0x14..+0x1c = our position, +0x24 = +0x25 = 0; buy source +0x12 = ShopListNum (low byte); FUN_1021c7a0(0, 1) on DAT_10666e2c *(inf: the inventory view set for the sell list)*
- L6356: c2s 0x085 (FUN_100f8a40): u16 SellFlag 1 @4; its FUN_100fc3c0(0x85, 0, 0) key 0 puts it under the per-id send interval (the 0x083 / 0x084 builders pass the index as key) *(inf)*
- L6373: Open / inferred: DAT_1057876c + 0xc4;
- L6375: … 274 (from the decompile; to check against retail shots); the meanings of the tint helpers FUN_1022b680 / FUN_1022b570 / FUN_10223360 / FUN_102092b0; the Escape / close sound not traced.

## Guild shops (2026-10-23)

`MODLOG.md:6428`

- L6464: … 8 bytes): u16 item @4, u8 slot @6 (0 for a buy), u8 quantity @7. 0x0AB / 0x0AD: FUN_100eb100 / FUN_100eb130, header only; key 0, so the per-id send interval applies *(inf)*
- L6472: … icon by id (FUN_1015bd70); 0x16 an icon from a pointer to an id-led struct (FUN_1015bdb0); 0x17 FUN_101f7120; 0x19 an item name by id drawn by list vfunc 0x68 in (width - 8) *(inf: fitted)*
- L6485: Open / inferred: the cursor reset when the list switches source and the focus back to guildsho;

## NPC trade: the hand-over window (2026-10-24)

`MODLOG.md:6547`

- L6568: On an NPC: FUN_101cc0d0(0x43); its bit 0x40 is the costume refusal, group 7 0xb1 (FUN_101cc040, not traced).
- L6569: A PC on the blacklist (FUN_100e0a50 *(inf)*
- L6570: Range FUN_10152490(e, mode): 0 = the squared 3D distance of the models (+0xa0 -> +0x34..+0x3c) <= 36; 1 = FUN_10084100(a, b, 36.0) *(inf: the same rule, another metric)*
- L6572: … inventory slot, u8 count}, gil +0x3c; the cursor-child table 0x10371d60 (children 1..8 -> slots 1..8, 11 -> 0 gil); the cursor shape FUN_101c8c30(7) on child 11, 0xb elsewhere *(inf: the numctrl vs slot cursor)*
- L6574: The picker lists bag 0 only *(inf)*

## Player trade (2026-10-24)

`MODLOG.md:6581`

- L6615: … +0xc4 the partner's (4 OK); the partner +0x6c / +0x70 / +0x7c, the pending requester +0x74 / +0x78 / +0x9c, times +0x20 request, +0x24 prompt, +0x28 cancel (FUN_100dc130 *(inf: seconds)*
- L6616: An incoming 0x021 counts only while idle or pending; a Start is refused (0x033 Cancel) also while FUN_10160740(8) holds *(inf: the UI locked)*
- L6617: Draws: our name at (+4, +5), orange 0x80804020 in state 4; the gift's gil at the top right *(inf: x + w - 14, y + 8)*
- L6619: The key codes 1 / 2 / 8 and the edge flag +0x5a are inferred.
- L6620: Picker greys (FUN_10150e60): extdata Ex (FUN_101cb960 / FUN_101cba20 & 2, augment tables *(inf)*
- L6622: The action menu shows 0x13 unless our +0x1d0 bit 0x400 (FUN_101cc350(0, 0)) *(inf: a costume or a non-human form)*
- L6629: "menu trddummy" (106 x 14, keytops3 0) sits in another table and neither path uses it *(not traced)*

## Message formatter (FA 40 codes) and /clock (2026-10-24)

`MODLOG.md:6631`

- L6667: … count 1: "a " / "an " (FUN_102589b0: first character in "aieuoAIUEO") + the vt+8 name unless noart; count != 1: vt+0x10 (vt+8 / vt+0x10 = the singular / plural log names, inferred).
- L6681: Both are kind 2 (PlayOnline-wide, keys "LPOL1WDY" / "LPOL1WSC"); where retail gets them is not traced (without PlayOnline they stay 0).
- L6696: … orb (table 0x10385ee8) and the weather badges (0x10385ef8) when it is 0) is made by FUN_102217a0 at each world set-up (FUN_10010d20) and freed by FUN_100108b0: not saved *(inf: once per entry into the game, not per zone)*
- L6706: Not traced: FUN_10080350's message source and FUN_10128ce0's flags;

## Engaged state: the battle set, the gauge window, the lock-on cone (2026-10-24)

`MODLOG.md:6708`

- L6736: Title slot 0 while engaged: retail shows group 0xd 5 "Commands" (seen live twice by the user), although FUN_1012b4b0 reads as 7 "Battle Mode" when +0x170 == 1: that field is not the engaged status (inferred).
- L6768: No sound on this path *(inferred: no SE call in FUN_1015f3e0)*
- L6776: Command table 0x1036f240 (12 bytes: u16 sprite, u8 library 0 keytops3 / 1 windowps, u16 help id, the Right handler @8): 2 / 3 FUN_1012acc0, 4 / 5 FUN_1012aae0 (chatctrl), 8 FUN_1012ab00 (magselec) *(inferred: the ">" is part of keytops3 sprites 7 / 22)*
- L6782: The update runs once a call (a frame), not by ticks *(inferred: steps meant for 2-tick frames)*
- L6784: The cast bar's test is FUN_1011ba40 on DAT_105781ec (FUN_1012c0f0); the palette shift also needs DAT_106218ee == 0 (always when it is 1; its source not traced).
- L6789: Lock modes (target system +0x5c & 3, FUN_10157660): 1 locked, 2 locked with 5.0 (the x1.5 cone, +5 x bounds distance), 0 free; no PC caller passes 2 *(inferred: unused on PC)*
- L6790: Not traced: FUN_1013eff0 *(inferred: the pool holds items)*

## UI sounds and window animations along the menu chain (2026-10-24)

`MODLOG.md:6793`

- L6816: Every SE function looks a type-0x3d SeSep up by its 4-character name (FUN_10072290) and plays it centred (FUN_10036f00(0x40, level)); 0x10037cb0..0x10037d6f have no callers *(inferred: 0005..0008, unused)*
- L6819: … also serves the map's 4-state cycle FUN_101fdbf0 (not identified); FUN_10038010 0039 when the counter DAT_104df908 grows with config 0x48 on (FUN_102010a0 / FUN_102010e0; *inferred: a mail / delivery notice*).
- L6840: The text engine FUN_101b00b0 also plays 0004 for over-long input and 0001 for caret / candidate moves (passinpu uses it; the chat line *inferred*).
- L6841: Not traced: the number windows (quantity / gil;
- L6843: … FUN_100a3ac0 case 0xb, which buzzes 0004 for a message with byte +6 == 0xf6 (the packet not identified); FUN_1007e630 / FUN_1007e7a0's Mog House exit / layout refusals 0004 *(inferred)*

## Skillchains: the burst file, the proc message, load holds by count (2026-10-24)

`MODLOG.md:6902`

- L6936: Speed: that loader's rule (0.7 with several records outside 14 / 15) *(inf)*
- L6958: … 268 / 271 enfeebles; file 7027 0x109 / 0x10d / 0x110, 0x2ed..0x2f1) with the proc empty, so they print at `mdam` like any main line; no client effect keyed on them was found *(inf: from absence)*
- L6959: Where the client adds its prefix for the 0x028 info modifier 0x04 (LSB MagicBurst: Swipe / Lunge) was not traced.

## On-screen damage numbers (config 0x13) (2026-10-24)

`MODLOG.md:6961`

- L6984: The generators: attach word 0x00010003 (mode 0x13), life 0, interval 100 (one element per task *(inf: the task end stops them)*
- L6992: dam2 / cur2 / mis2 are the same with D3m 'hit8' meshes; nothing starts them *(inf: leftovers)*
- L7004: … hidden when view z >= 0 or w < 1, when zi = trunc(z_ndc x 4096) > 4100, on the local player in first person (DAT_10487f80), with actor +0xb2 != 0, or with DAT_104568fc+8 == 0 *(inf: the 2D / world-UI switch)*
- L7010: … glyph size = corners (x1 - x0 - 1, y2 - y0 - 1); x starts at -(total / 2) (integer); quads [x, x + w] x [-h/2, h/2]; FVF 0x144, rhw 1, z = z_ndc; the 0.1 argument is unused *(inf)*
- L7017: Render state (the common element draw): 0x44 SRCALPHA / INVSRCALPHA, 0x48 SRCALPHA / ONE with FOGCOLOR 0; alpha test off; Z test LESSEQUAL without write; fog on *(inf: far numbers fade toward the fog colour, black for the glow)*
- L7020: Sorted in CMoOT by +0x128 - z_view, so the glow (-1.2) draws before the digits (-1.5); in the scene pass, under the 2D windows and the op 0x0f fade *(inf: the exact slot)*
- L7022: No spread or stacking: every printed record starts its own task, so numbers on one target overlap and only their print times separate them; no distance limit; no count limit beyond the pools *(inf)*
- L7023: The Effects filter (0x92) is not applied *(inf: FUN_10056c70 gets 0 for task+0x90)*
- L7027: FUN_1009e090 also needs game state 0x60 (DAT_10456a28) and FUN_1009e370 false *(inf: it hides actions between actors whose team byte +0x200 differs from ours with +0x201 clear: PvP / Ballista;

## Emission op 0x3c and op 0x45; the double "Miss!" (2026-10-24)

`MODLOG.md:7035`

- L7079: *(inf: nothing allocates a never-started resource's +0xa4, so the 0x3c element borrows the mis0 clone's frame through E+0x100.)*

## Per-record reactions from the files: guard, parry, block, counters, added-effect / spike lines (2026-10-24)

`MODLOG.md:7082`

- L7118: `dcnt`'s op 09 `damg` runs on the original attacker, whose op 0x21 uses the original record's scale & 3 *(inferred: LSB sets no hit distortion on a countered swing, so usually none)*

## Knockback: scheduler ops 0x5e / 0xbf (2026-10-24)

`MODLOG.md:7124`

- L7171: … FUN_10060e30, dtor FUN_10060c60) counts delay x task speed (FUN_1005e590); the motion and level are taken at the op; the destructor pushes (also when the task is killed early *(inferred)*
- L7179: Not traced: vt+0x304 / +0x103 / +0x104 in FUN_100a57f0 (other movement modes FUN_100a9fc0 / FUN_100aa1f0;

## Title buttons: sounds and the press animation (2026-10-24)

`MODLOG.md:7183`

- L7201: Window events: 1..4 arrows, 5 Enter (and the left click *(inf: the click path in FUN_10125360 not traced to the end)*
- L7210: Delete with DAT_10456a4c bit 0 gives 0004 and draws 0x80202020 *(inf: the flag is only ever zeroed in this build)*
- L7211: Escape: cursor 3 (Back), window cursor child 4, FUN_10118170, FUN_10124b70 *(inf: puts the pointer on the new child when the mouse is in use)*
- L7214: Create also calls FUN_10037a70(0x1e) when DAT_10456aa4 == 0 *(inf: a sound-system command 9 to the music, FUN_1024c080(9, -1, 0x1e);
- L7217: … [1023ab10]) has the same shape with a list of three items (+0x1c + 4 i enable flags): 1 / 2 / 9 0001; 5 0002 and lock, or 0004 when disabled; 6 the cursor to item 2 with 0001 *(inf: a second button set ours does not show;
- L7224: FUN_10119fb0 [10116c90](window, sprite, x, y, sx, sy, N, c), RET 0x1c: with window +0x99 set, sx / sy become FUN_101601d0's scaled-window factors *(inf: the 0x100000 registry flag;
- L7227: Ring k: alpha = ftol(128 (1 - e / M)), tint alpha << 24 \| 0x805020, centred: x -= ftol((sx s - 1) \|x1 - x0\| / 2), y likewise, with the sprite's +0x20 / +0x22 / +0x24 / +0x26 *(inf: its bounds)*
- L7234: … k0, ring k1): 9: +1, s 1.000 a 128, none; 7: +1, s 1.107 a 82, s 1.017 a 120; 5: +1, s 1.213 a 36, s 1.123 a 75; 3: +1, none, s 1.230 a 29; 1: +1, none, none; 0: -1, none, none *(inf: whether the press frame draws c 9 or already 7 depends on the update / draw order)*

## Actor sounds 1: voices, model init, sub-animations (2026-10-24)

`MODLOG.md:7237`

- L7279: Not traced: ops 0x07 / 0x1f in init / ini1 (locks), the voice priority value, whether 0x00D carries @0x2A for PCs (the reader at 0x1009a194, inf).

## Status icons: "menu    buff" (2026-10-24)

`MODLOG.md:7307`

- L7336: No region bits in A: it never stacks on another window *(inf)*
- L7340: Library sprite 0 = hfr1 240 x 4 at (0, 28)..(240, 32), colour (128, 128, 128, 97), additive; whether the window drawer draws that line was not traced (open: compare with a retail screenshot).

## Status icons: Num+ focus and cancelling (2026-10-24)

`MODLOG.md:7362`

- L7367: … scaled 0.74 at the cell's cursor point - (5, 5); Left / Right one icon, Up / Down a row (SE 0001); title slot 2 = the status name (file 55725 field 0; each word capitalised, inferred for FUN_102588c0(2)), help slot 2 = the entry's description;

## Status icons: party members (2026-10-24)

`MODLOG.md:7385`

- L7393: … x + 200, log y - 8 - rows x 14), above the party window instead when x + 144 reaches it; the name 16 above in fontshp, 0x80808080 in our zone, else 0x80404040 (FUN_10202cc0, inferred).

## Actor sounds 4: sub-animation variants, op 0x53, pop0 / pop1 (2026-10-24)

`MODLOG.md:7396`

- L7403: Left: the hide skip (entity +0x138 bit 31 with model +0xb2 bit 1; +0x138 not traced), `?ead` / `sdep`, fishing.

## Chat call sounds (2026-10-24)

`MODLOG.md:7405`

- L7415: That byte (+0x3c) only sets a blink (DAT_10455c20 +0xdbc, +0xdb4 += 0x3c) while DAT_10351710 is on (FUN_100381d0) *(inf: a window flash)*
- L7417: Tell (mode 0x0c) with config 0x47: SeSep "0p40" (0x30703430 byte-swapped), in no resident DAT (0, 27, 52743), so retail finds nothing either *(inf)*

## Engaged menu: Check through the prompt, Conflict rows, Call for help block (2026-10-24)

`MODLOG.md:7419`

- L7445: *(inf: 2..4 Ballista teams, 5 / 6 Brenner, 8 / 9 Monstrosity Belligerency, 0x28..0x2b another PvP mode, 0x63 used by FUN_101cc220.)*
- L7448: Sprint becomes Scout (0x17) when our +0x1ff is not 5..6 and +0x27c != 0 *(inf: Petra carried)*
- L7450: … +0x27c is printed by the `<petra>` tag and written only at asm 0x100a22e4 from u32 @8 of a packet whose u16 @4 holds kind 1 / sub 2 (code at 0x100a21c0, not decompiled; *(inf: s2c 0x0E6, Ballista)*
- L7453: Nothing happens on the client *(inf: the speed-up is a server status)*
- L7454: Check is also hidden when the target has +0x1d0 bit 0x100 *(inf: an "ally" NPC, +0x12c bit 13)*
- L7460: … window closes. Entity search: slots 0..0x3ff, then 0x400..0x8ff; +0x13c bit 0x40 and not FUN_10133a40 (a Belligerency team test) -> FUN_1007bf30(target, 2, ...) instead *(inf: help 0x327)*

## Text commands: toggles, /ver, /playtime, /chatmode, /localsettings (2026-10-24)

`MODLOG.md:7468`

- L7508: … 0x10f / 0x110); /emotefaith /emotetrust 0x67 / 0x68 FUN_1007dee0 (0x14b linked / 0x14c not; the consumer at asm 0x100a094c skips emotes 0x7a / 0x7c for our trusts and was not traced further);
- L7530: The toggles read the 12-byte mirror at session +0x2cfd8 (FUN_100e47c0 / FUN_100e4520), written by the s2c handler at 0x100e4f20 *(inf: s2c 0x0B4)*
- L7533: … converted by FUN_103114a2) into the file-system object DAT_10662dcc +0x10; without it, "psfiles date[%d:%d:%d]" from bytes +0x14 / +0x13 / +0x12 of "/config/data/winfiles.h" *(inf: a date)*
- L7541: … 5 "/party ", 6 "/linkshell ", 7 "/emote ", 8 "/message ", 9 "/yell ", 10 "//worldmes ", 0xb "/unity ", 0xc "/linkshell2 ", 0xe "/assistj ", 0xf "/assiste ". FUN_1012d2b0 *(inf: the log window's chat-mode menu)*

## Model-file generators: the stop match, var 0x3a, op 0x9a (Raminel's dust, 2026-10-24)

`MODLOG.md:7552`

- L7574: Open: the 1106-type files rely on 0x9a and have no stop in `init`, so their generators would run from load unless the actor is hidden then (not traced).

## Search: the search server client and name lookups (2026-10-24)

`MODLOG.md:7602`

- L7670: The record (0x48): +0 present bits; type 0 name @4; 0xe full char id @0x14 (32; *inferred*;
- L7673: … main / sub job @0x20 / @0x21 (5 + 5; 0x17 MON); 4 levels @0x22 / @0x23 (8 + 8); 7 master level @0x24 (8); 1 zone @0x28 (10); 6 flags low @0x2a (16); 0x17 languages @0x2c (16, *inferred*);
- L7678: Flags (rec +0x30): 0x1 mentor, 0x8 party leader, 0x10 has a comment, 0x20 auto-group, 0x100 away, 0x400 *(inferred: a PlayOnline Friend List link;
- L7679: … Friend List link; LSB never sets it)*, 0x800 linkdead, 0x2000 in a party, 0x4000 anonymous (job / level / nation / race / rank not sent), 0x8000 seeking a party, 0x10000 *(inferred: icon colour 0xfffffffe)*

## Party rows for members in other zones (2026-10-24)

`MODLOG.md:7698`

- L7725: … MoghouseFlg @0x1B is the party flag PARTY_MOGHOUSE 0x200, not a location; a member in a Mog House is in the city's zone; 0x0DF goes to every alliance member with real HP *(inf: also those in other zones)*
- L7729: The away row's name is slot +6 (never the Monstrosity name) through FUN_10202380 -> FUN_101ca180 (the 8-byte ellipsis rule); the gauge anim bytes freeze while away *(inf: the bars ease from the frozen value on return and reset only when the slot's id changes)*
- L7733: Zone names: FUN_101cbc10(zone, 0) = group 4 (file 55466, short names), else group 9 0x40 *(inf: "---------")*

## Search: /search and the results window (2026-10-24)

`MODLOG.md:7742`

- L7769: … W closed; own unity bits FUN_100e3200(S +0x40de6) ((id & 0x3f) << 22, 0x0f800000 none); the LS ids of the equipped linkshells, slots 1..3 (FUN_100e9730; slot 3 *inferred* never equipped);
- L7814: … 40 / x + 47 or "--"; sub job x + 59, its level x + 82 / x + 89; no race; the crest x + 103; the rank x + 115 / x + 121; rows built after the toggle start in the default view *(inferred)*
- L7835: Not traced: rec +0x14 against +0x18 (LSB fills only +0x18);

## Cast tones and the sound grace (2026-10-24)

`MODLOG.md:7845`

- L7860: `switch_zone` clears the stored casts *(inf: retail rebuilds the actors)*
- L7876: FUN_10083d60's other callers all pass 0: the finish loads (0x100a8fe7 / 0x100a924f / 0x100a9543) and the entity re-sync FUN_100aeea0 (0x100aef99 / 0x100af06c, state machine DAT_1048990c; *inferred: the zone-in replay*).
- L7905: FUN_10055cc0's 0.7 test is holder +0x1c > 1, read as the target count *(inferred)*

## Bard songs: ops 0x76 / 0x77 and the range slot (2026-10-24)

`MODLOG.md:7908`

- L7928: a1 +0x7c4 is cleared before and set to the name after (inferred: a record of the last lc / ls;
- L7941: SeSep: @8 the se id, @0xc the length of trailing note bytes (bit 31 clear: no loop) that nothing on PC reads (inferred: a PS2 leftover).
- L7948: No BGM ducking or song special case in the client (none of these 4cc or se ids is an immediate in the code; inferred).
- L7951: Not done: op 0x89 (FUN_1005f450: actor +0x840 bit 1 and its x / z recorded for dur ticks; inferred: holds the actor in place).

## Window groups, region stacking and the raise prompt (2026-10-24)

`MODLOG.md:7992`

- L8023: FUN_10096c90 mode 2 opens it with FUN_10210e30(LAB_10097370, &state[5], 0, 1, 0, 1, 0): the cursor on Accept *(inf: the default child)*
- L8028: Escape (message 6, also the right release): the generic SE 0003 (+0x84; it does not close: registry flags & 2), then the class's SE 0004 (+0x39 == 0) *(inf: whether 0003 is heard too depends on the voice stealing)*
- L8042: … FUN_10087070(0) holds; it floors y at win+0xbe when +0xbe != +0xc2 (both 0 by default) and at 16, moves the window by FUN_10119110 without easing, and sets +0x88 = (x - 8, y) *(inf: the hand)*
- L8054: … / 0x191, 0x167 / 0x192, 0x168, 0x169, 0x16e, 0x16f, 0x247 scale it by 50 / 90 / 120 / 150 / 200 % through FUN_10102030 by the spell type (1, 2, 0x63) and +0x44 bits 0 / 1) *(inf: the SCH arts, Manafont, Manawell, Penury, Parsimony family)*

## /check on a player: "menu    inspect" (2026-10-24)

`MODLOG.md:8058`

- L8106: On View Wares the cursor hook opens "menu comment" (16,240 366x56) without focus (inferred), 3 lines at (x0 + 4, y0 + 4 + 16i);
- L8107: On View Wares the cursor hook opens "menu comment" (16,240 366x56) without focus (inferred), 3 lines at (x0 + 4, y0 + 4 + 16i); on an empty slot iteminfo stays open as an empty frame (inferred).
- L8111: Broadcasts 0x838 close it and reset the store: 0x8 zoning, 0x10 target dropped (gone, hidden, status 2 / 3, beyond 50), 0x20 engaged, 0x800 an event; the registry extra adds 0x200 (game mode 0x40, inferred).

## Chat line history (2026-10-24)

`MODLOG.md:8116`

- L8151: Held keys repeat at Windows' typematic rate *(inferred)*
- L8152: Numpad 8 / 2 with NumLock off arrive as VK_UP / VK_DOWN *(inferred)*

## Search: the /search grammar (2026-10-24)

`MODLOG.md:8191`

- L8207: … parse (STATUS / UNITY may repeat); FRIEND sets +0x4e bit 2 and flag 0x1000; LINKSHELL / LINKSHELL2 take the equipped LS ids (ours: the LS item's exdata u32 @0, LSB GroupId *(inf for retail)*
- L8260: Zone flags (DAT_103669b8, FUN_101070f0, 4 bits a zone, 300 zones): bit 2 searchable, bit 1 listable, bit 4 cross-zone invite *(inferred)*

## Window group walk (2026-10-24)

`MODLOG.md:8263`

- L8282: … the target window, the main menu, the status icons), says it is open each frame; its stamp is the frame it opened (a new top), targetwi's under the windows holding its bits *(inf: gaugewin and playermo taken as new)*
- L8312: Hide FUN_1015e6e0(w, a2, a3, a4, a5): stores the cursor child in the menu template (FUN_10118830 -> FUN_1011b360) *(inf: cursor memory)*
- L8314: … DAT_1036c190 (25, or 0 with the window-animation config) when animated (a2 unused); without flag 0x1 and with a4 == 0 it gives back its +0x6f count from DAT_1057876c +0x6a *(inf: the input lock count)*
- L8315: … back its +0x6f count from DAT_1057876c +0x6a *(inf: the input lock count)*; clears the manager's previous focus (+0x58) if w; +0x75 = a5; calls the window's +0x7c callback *(inf: an on-hide notice)*
- L8322: Show FUN_1015e530 (needs +0x6e == 0, a registry entry and state 0xe): FUN_10117ee0, state 0, the handler's vtable +0x14 *(inf: an on-show refresh;
- L8342: Read from the code, not seen *(inf: the bits look like screen areas)*

## Action menu: the mouse hold, release confirm, party cursor on engaging (2026-10-24)

`MODLOG.md:8357`

- L8368: Confirming the target with the menu up runs its open again (FUN_1012ada0): the rows reload, Call for help resets, the cursor stays *(inf)*
- L8381: A toggle row flips once a second while held *(inf;
- L8382: After Disengage's hold the release confirms the row now shown, so Call for help runs /help *(inf: no guard)*
- L8383: After Disengage's hold the release confirms the row now shown, so Call for help runs /help *(inf: no guard)*; after Magic's the release is judged against magselec *(inf: SE 0004 with the pointer still over playermo)*
- L8386: Middle (0x96) release -> msg 7 when FUN_10123e50(4) (playermo: FUN_1015ee40, the main menu) *(inf: the button)*
- L8389: The party cursor on engaging: cancelled through its callback with 0, its window restored, then FUN_10202d10 / FUN_10157780(0, 0) / FUN_10157860(0, 0) and target-system +0x58 = 0; no sound *(inf)*

## Treasure pool window: loot, lootope, lnowin (2026-10-24)

`MODLOG.md:8391`

- L8444: Closing by itself: engaging (FUN_1015f3e0(0x20)) and an event start (0x800) close the windows whose registry byte +0x28 bit 2 is clear; zoning through 0x200 (bit 3 clear) or 8 (no flag 0x100) *(inf: which game state is the zone change)*
- L8449: … 1 or past the count puts the cursor on the last item, else SE 0001 when it moved onto another row; with lnowin open and the pointer in its rect, loot's update sends message 8 *(inf: while focused)*
- L8465: Packet queue: 0x041 is allocated with key 0, so a second 0x041 within the id's minimum interval is not sent; 0x042 uses the slot as its key (a still-queued pass for the same slot is reused) *(inf: the interval's value)*

## List scroll bars (2026-10-24)

`MODLOG.md:8474`

- L8501: The bar object (window +0x88, 0x2c bytes): +0 x, +4 y, +8 h, +0xc visible, +0x10 total, +0x14 top (the drag writes it), +0x18 colour 0x80808080, +0x1c 0 *(inf: a blend word)*
- L8531: … class, its handler ignores 0x16..0x1b); fulllog (open FUN_101376a0; visible trunc((H - 448) / 16) + 23; update FUN_10137840: h = H - 64, top = total - scrolled - visible, inferred for the main tab;

## Chat mode window: chatctrl (2026-10-24)

`MODLOG.md:8537`

- L8594: Game state 0x40 (FUN_1015f3e0(0x200) -> FUN_101b83e0) sets Say again *(inf: leaving the game)*
- L8597: Tell names DAT_1066268c (FUN_1018d640): +4 count, +8 8 x 16-byte names, loaded from the per-character file acq.dat (FUN_101b9f20 kind 1, 0x80 bytes; names of up to 15 letters) *(inf: saved like cnf.dat)*

## Target flash: the ambient bytes (2026-10-24)

`MODLOG.md:8646`

- L8652: No vertex colour (COLORVERTEX falls back to the material); the light ambient DAT_106174dc is never written *(inf: 0)*

## List scroll bars: the mouse and the wheel (2026-10-24)

`MODLOG.md:8712`

- L8726: The full log: its handler is not traced *(inf: the scroll-only Up / Down, Left / Right pages, 0x1b the bottom-anchored top)*

## Actor collision and lifts (2026-10-24)

`MODLOG.md:8752`

- L8787: FUN_101d5fe0 is true (boxes skipped) while byte +0x40db1 bit 4 is clear; the zone load clears it and the Mog House entry sets it for map 0x121..0x123 or +0x40d82 != 0 *(inf: the rent-a-rooms have no furniture collision)*
- L8814: FUN_10176620 takes the first matching '@' record in list order; its six-face test (table 0x10376bc0, FUN_101768a0) gets the world-space old position and delta, so it is dead for real lifts *(inf: a retail bug;

## Character bump (2026-10-24)

`MODLOG.md:8840`

- L8852: Eligible: main body actors of kind 0/1/2/6/7/8 (not doors, lifts or ships), +0xb2 == 0 (drawn, not culled), not +0xa4 1..4 / 0x54 *(inf: mount get-on / off)*
- L8857: Ours: client/bump.rs (hold, radius from the rest pose *(inf: retail reads the current pose)*
- L8880: … = entity +0x120 bit 6; called nowhere else); kind +0xee (FUN_10084400); +0xb2 == 0; vt+0x34c (FUN_100827a0: +0xa4 not 1..4) and vt+0x358 (FUN_100827f0: +0xa4 != 0x54) *(inf: mount get-on / get-off stages;
- L8889: Entity-less actors read zero defaults (kind DAT_1047d610, status DAT_1047d60c) and pass those tests; model actors (vt+0x1c8 = FUN_10082120) have radius 0 *(not traced: whether one reaches +0x59c >= 1)*

## Party menu (2026-10-24)

`MODLOG.md:8903`

- L8913: A window watching the party state ([7]: prty1 / prty5us 2, the others 3; FUN_10201690) closes when it changes; ours closes the whole stack *(inf)*
- L8952: The Party row is also disabled in Monstrosity (FUN_10097d50; FUN_10140090 at open, FUN_10140270 each frame); menuwind sets +0x86 = 0, so a disabled row is silent *(inferred)*
- L8977: … 3 windowps 258 and help group 9 0xc2 "All treasure obtained will be automatically pooled." when FUN_101cc330 holds (our actor +0x18c != 0, set from 0x00D @0x20 bit 3; meaning not traced);

## Status window (2026-10-24)

`MODLOG.md:8994`

- L9032: Flag 0x200000: FUN_101190f0 puts child +0x58 in state 0x10 at creation *(inf: the caption child)*
- L9041: The cursor starts on child 1 *(inf)*
- L9063: … true picks a random rank instead (not identified, left out); the draw sets the height to (n / 2) x 14 + 0x16, overwritten by the update's n x 14 + 0x16 before the frame draws *(inf)*

## Missions (2026-10-24)

`MODLOG.md:9083`

- L9102: Inferred: row bands beyond the count hidden;
- L9115: Help: missionm 0xe 468; the lists have none, so the help line is empty while one has the focus (inferred).
- L9125: The lists do not rebuild while open (only state 0 polls; inferred: a change shows at the next build).
- L9136: … box FUN_10233130 / FUN_10233190: framesus 35 at (36, 25), the name at (50, 20), the text at (40, 50) wrapped at 327 px; the row bands hidden, hand and bar off, the arrows kept (inferred).
- L9138: Enter on a row that does nothing still plays 0002 (inferred: the generic method runs first).
- L9150: With LSB's values the current CoP mission is listed as completed too (inferred;

## Quests (2026-10-24)

`MODLOG.md:9175`

- L9189: Not done: Enter on the RoE rows 2..4 (their lists, FUN_10232b80, not traced): logged.

## Key Items (2026-10-24)

`MODLOG.md:9213`

- L9230: Inferred: the cursor-follow rule (FUN_10221dc0), Yes / No drawn as plain text at their children, the mouse.

## Search menu, part 1 (2026-10-24)

`MODLOG.md:9256`

- L9289: Ballista's S+0x40dac bit 15 is written by the zone-in handler near 0x100fb07e *(inf: the Ballista state)*

## Linkshell menu: link5 / link6, the item submenus, link11 (2026-10-24)

`MODLOG.md:9370`

- L9402: … +4 is the linkshell number n, not an item slot; FUN_101e3370 settles it when LS n is worn with that GroupId (ActiveFlg 1) or none is worn (ActiveFlg 0), or after 0x3c ticks *(inf: 1 s)*
- L9431: Event 0x800 (FUN_101e43f0) closes link5 in mode 0 *(inf: a close-all / zone event)*
- L9435: The container check FUN_101e3b10 *(inf: always true for real items)*
- L9437: link11: values +0x30..+0x32 (7, 7, 7 at open), backup +0x33, name +0x34; Left / Right are key function 0x3e 3 / 2 *(inf)*
- L9440: LSB's createLinkshell needs item 512, turns it into 513 with the colour and sends 0x020; it does not equip it *(inf: retail the same)*
- L9444: back to link5 *(inf)*

## Search: arealist, scsibori (Refine), the own comment editor, maplist fixes (2026-10-24)

`MODLOG.md:9446`

- L9501: *(inf: an opener running while arealist is open would reset the mode through +8;
- L9522: FUN_1020a9d0 returns at once while a query is out (DAT_10385a04 != 4), but the windows still close, 0002 plays and the condition has changed *(inf)*
- L9541: The first menu reset (FUN_1015d860) clears blocks 2 / 5 and +0x18 *(inf: the comment starts empty each login)*
- L9544: c2s 0x0E0 (0x98 bytes): text[128] @4 (<= 123 bytes), installTime @0x84 (FUN_100e3220: a u32 from "ffxiusr.sys", 8 bytes {time(), -1}, made if missing), "WIN" @0x88 with byte @0x8b = DAT_104b1770 *(inf: from that file's read)*
- L9545: … {time(), -1}, made if missing), "WIN" @0x88 with byte @0x8b = DAT_104b1770 *(inf: from that file's read)*, srvExCode @0x8c (PTR_DAT_103601bc + 0xac) and cliExCode @0x90 (+0xb0) *(inf: the login expansion / feature bits)*
- L9547: Not traced: the generic handler running before the list for arealist (inferred from +0x85 / +0x86 staying set);
- L9549: … set); the editor's FUN_101c6920 code 3 (messages 8 / 0xc) and message 0x12; where ffxiusr.msg lives (FUN_101c4920; probably the character's USER folder, as the .MRK files *(inf)*

## Linkshell message: s2c 0x0CC, the client's own 0x0E1, /lsmes and /ls2mes (2026-10-24)

`MODLOG.md:9551`

- L9570: /lsmes (0x108) / /ls2mes (0x109), FUN_100805f0 / FUN_100807d0: linkshell n worn, else 7 0x71; also actor +0x124 bit 27 (from the zone-in flag word at S+0x40db1 bit 0, FUN_1008xxxx at all.c 114350) *(inf: not modelled)*
- L9588: The ask state S+0x9818 + 4n is set for n 1..3 by FUN_100e8930 (the session's handler setup in FUN_100fa050): 0 when FUN_100da570() is true, else 2 *(inf: true only for the first zone-in after character select)*
- L9592: /lsmes's actor +0x124 bit 27 is not traced;

## Region Info: "menu    region", cnqframe and the conquest maps (2026-10-24)

`MODLOG.md:9609`

- L9700: … S+0x3c17c + 12 off (off = packet +9, < 26); when off + 13 >= 26 the callback (S+0x3c150, LAB_101e8200) takes the buffer: per area entry (mode 2 entries 16..41) bit 0 (battle *(inf)*
- L9710: … "%d" right-aligned at (x1 - 6, y + 0x16 + 16 k). - campsan (dword n = buffer + 4 + 4 i): three sprites at (x + 4, y + 8), (x + 0x20, y + 6), (x + 0x20, y + 0x14) (not traced);
- L9718: Not traced: the beseige2 / campsan sprites and the Besieged forces gauge sprites (in FUN_101e9c40 / FUN_101e9af0 / FUN_101ea150);

## View House: "menu    mogcont" (2026-10-24)

`MODLOG.md:9775`

- L9780: … in the nations' residential zones (230..232, 234..236, 238..241) before our first s2c 0x067 kind 2 of the zone (FUN_100f9680 / S+0x40c41, ours `mh_seen`, cleared at zone-in *(inf)*
- L9789: The read-only list ("menu inventor", group 0) leaves mogcont drawn, unfocused; the two-list view (bank + itmsortw group 0x01) suspends it *(inf: the registry groups)*
- L9813: The cursor starts on child 1 and is not remembered *(inf)*

## Currencies and Currencies 2 (statcom2 rows 6 / 10) (2026-10-24)

`MODLOG.md:9819`

- L9824: c2s 0x10F / 0x115 (header only) is sent again every 31 ticks *(inf: 60 Hz)*

## Unity (statcom2 row 11) (2026-10-24)

`MODLOG.md:9845`

- L9886: Unity Lists = FUN_1020f3a0(1, 1): the search condition reset, all areas, mode 4 *(inf: the reset condition)*
- L9889: ut_point before the stats are ready (s2c 0x119, DAT_10485824 bit 0): retail formats "/%d" with no argument ("/" and garbage); ours "/" *(inf)*

## Set Bazaar: "menu    bazaar", Set Prices, Edit Comment (2026-10-24)

`MODLOG.md:9899`

- L9906: … settled, "N G" (lock 0x19, s2c 0x020 Price @8, now kept per slot) in the row's tint and the count for stackables (columns 0x1037f044 {0, 18, 180, 240}; the count cell's end is *(inf)*

## Help Desk: "menu    faqsub" (2026-10-24)

`MODLOG.md:9960`

- L10024: Mentor Search (FUN_10216500): FUN_1020a600(0), FUN_10107b40, FUN_10107e70(crit, 1), FUN_1020a9d0(1, crit, 1, 0, 0, 0); faqsub stays open under the results *(inf)*

## MacroPalette: Edit Macro (2026-10-24)

`MODLOG.md:10027`

- L10044: "menu mcrbedit" beside them, Num+ (key function 0x11) gives it the focus (SE 0013): Copy / Paste / Delete of the macro (memory), or with Ctrl / Alt held 31 frames *(inf: at 30 fps)*
- L10082: Fields (FUN_101c5be0): the book name mode 4, 15 bytes (+0xc..+0x11 = 0, 0, 0, 1, 0; *(inf: no auto-translate)*
- L10084: The editor blinks +0x24 40 on / 20 off of 60 frames while editing *(inf: the caret)*
- L10085: The flip (not done): on a set change both palettes' element +0x44 = 2 for 20 ticks, then 3 for 20 *(inf: the close zoom, then the open zoom, the contents swapped at once)*
- L10095: mcrselec +0x2c (FUN_101a6560) clears the book and set clipboards (S+0x235e / +0x235d) *(caller not traced)*

## MacroPalette: equipment sets (2026-10-25)

`MODLOG.md:10100`

- L10139: … draw nothing and take no space), frame FUN_101a6d60 (title slot 2 the name, help group 10 0x75 "Equipment Set (#%d)" as the child's help override through FUN_1011ae10 *(inf)*
- L10163: Commands: /equipset (0x9e, flags 0x51) FUN_1007b570 with the gate FUN_101cc0d0(0x46) (fishing, synthesis and bit 0x40 *(inf)*
- L10170: Open: who calls mcres20 vtable +0x2c FUN_101a7010 (it registers the 0x117 / 0x11b / 0x11c callbacks) *(inf: at load)*

## The chat line's caret (2026-10-25)

`MODLOG.md:10173`

- L10186: Drawing: a line wider than the box keeps the caret in view, the text before it cut at the left and the rest cut at the right *(inf: msgline's own scroll not traced;

## Equipment sets: the slot editor (2026-10-25)

`MODLOG.md:10191`

- L10217: … EBX = the cell's value, skipped when 0) would use these values as object pointers, so the source draws the row itself: ours at (x - 18, y + 2) of the icon column *(inf)*
- L10232: FUN_10134d90(the record's slots): the source DAT_105781c8 on the inventory, FUN_101f23a0 opens the list (iteminfo mode 0 *(inf)*

## Help Desk: the GM call questionnaire (faqmain) (2026-10-25)

`MODLOG.md:10260`

- L10269: DISP values ours: GENERIC.PLATFORM "win", GAME.INFIELD "on" *(inf: the evaluator is not traced)*
- L10271: Pages: a select node with visible slots draws the select page (pads 8 / 20 / 10), other select nodes a leaf page (10 / 25 / 15), question nodes the question page (8 / 20 / 15) *(inf: FUN_1010ea10's 1 / 3 not traced)*
- L10285: … 0x20, opt, zone 0, world 0, the actor's position at +4..+0xc and +0x24..+0x2c} (zone / world are never written), version {1, len, opt, u32 dataset, u32 per node of the walk *(inf)*
- L10286: Ours sends the walk's EVAL keys and the fields' parameters *(inf)*
- L10302: … base FUN_1010a290; language 0 Japanese, 1 English, 2 / 3 the European ones, from DAT_10456a28 + 8); a session already in state 4 / 5 -> FUN_1010a550(0), 7 -> FUN_1010a5f0(0) *(inf: restart / retry)*
- L10303: Placed: a non-Japanese client sets +0x2bc = FUN_101addc0('wind', 0x183) *(inf: a resource for the language row)*
- L10306: A slot without a link ends the walk *(inf)*
- L10320: … a row moves the cursor (SE 0001 when it changed) and sets the pointer shape (DAT_104e1d4c + 0x4c = 2); a click on the cursor row posts a click at its centre (FUN_10124700) *(inf: = Enter)*
- L10322: With no session only Escape (FUN_1021bb70) and a debug error-file viewer ("err_%03d.bin", +0x34a; not traced).
- L10332: Open: where "Input:REQUIRE" is enforced (SE 0004 *(inf)*

## Main menu: the Status comparison, the gauge rules, the Mog House row (2026-10-25)

`MODLOG.md:10335`

- L10353: … DAT_1057876c +0x68, +0x59 and +0xc4 are 0, FUN_101b5c90() is false and the myroom mode (DAT_106631d4 +0x1c) is 0 (the blockers are not identified); there is no engaged test (inferred).
- L10357: … (empty), +0x10 draw FUN_1013ff90, +0x18 events FUN_10140400 (3 / 4 / 7 / 0x1c open socialme, page 2; 5 Enter; 6 Escape), +0x24 FUN_10140370 (DAT_1057876c +0x74 bit 0, inferred: holds the world keys), +0x28 per frame FUN_10140270.
- L10358: The open closes the menu at once when FUN_10087070(0) (inferred: an event lock) or the player's +0x170 is 2 / 3 (K.O.).
- L10361: Row 12 per frame: enabled (help 0xe 0x29 "Open the Mog House menu.", +0x25 = 1) out of the Mog House, in Monstrosity, or when FUN_10097b40 && FUN_101cc0a0 (inferred: one's own house).
- L10363: Whether the layout's own "Map" sprite also draws under it is not traced (inferred: hidden).
- L10390: Ours closes the main menu under Status and brings it back without a zoom; retail suspends menuwind (statcom2 is group 0x81) and the walk resumes it with the open zoom (inferred, not checked live).

## Equipment screen as retail: the list, both modes, mnstorag (2026-10-25)

`MODLOG.md:10415`

- L10441: The inferred Backspace / Delete removal is gone (retail removes by Enter on the worn piece).
- L10478: An item arriving while the view is built is appended unsorted (FUN_1021c5b0, which tests mask & cache +0x1c: a quirk) until the next rebuild *(as read)*
- L10483: … draws framesus 188..191 (Wardrobe 1..4) / 204..207 (5..8) at (cell x, y + 2): tinted 16 x 16 itemslot cells with vertex alpha 0 under alpha blending, so they draw nothing *(as read;
- L10495: … returns 0); 0013 Num+ both ways; a mode-1 grid equip that works plays nothing; Escape and the mode-2 grid Enter have no sound in these handlers (the window manager's not traced).
- L10497: Bars: the strip's help is 0, so slot 2 is cleared and slot 1 shows the list child's text, falling back to inventor's 0xe 3 "Select an item." *(inf)*
- L10498: The pointer swap (FUN_10132c30 / FUN_101f27a0; *(inf: FUN_10118c10 = the pointer inside that window)*
- L10502: … the strip by FUN_10135300): mcresed's grid Enter rebuilds view 2 on all 16 containers (FUN_10134f00); at the editor's open no rebuild runs (the view the last user left) *(inf)*
- L10503: Open: the list's cursor row after a grid Enter is kept (only the top row is reset) *(as read)*

## Macros while engaged (2026-10-25)

`MODLOG.md:10514`

- L10518: (+0x170 2 / 3), the unknowns +0x59, +0xc4 and DAT_106631d4 clear; then refused only when a window has the focus (FUN_1015edb0, or DAT_1057876c +0x68 set *(inf: unknown flag)*

## Level up / down effects and the jingle (2026-10-25)

`MODLOG.md:10525`

- L10529: … message queue) treats messages 9, 0xb, 0x32, 0x2cf, 0x2d (only with actor +0x210), 0x32b, 0x32d specially: after a wait (state 0 -> 600 ticks, state 1 on the actor's +0x170, not traced) it calls the actor model's vtable +0x2c4 (FUN_100a9700: prints the text through FUN_10127c00, then FUN_100

## The chat line's channel tab (2026-10-25)

`MODLOG.md:10583`

- L10619: With the Tab completion list open the tab follows FUN_101b7d00's preview text (inferred, not seen).
- L10620: Closing the line leaves the log alone, so its text drops 16 px at once (inferred from the missing call).
- L10622: Visible width: FUN_10190280 with inline +0x34 and the scroll +0x28 (inferred: cuts or scrolls the line to that width).

## Name plate icons, job master stars, invisible players (2026-10-25)

`MODLOG.md:10625`

- L10644: The Petra word (+0x29c) is not read *(inf: LSB has no Ballista)*
- L10647: A party / alliance member stays translucent (0x27) with a plate, as the user knows retail; the client has no party case, so the retail server must send them something else *(inf)*
- L10672: … 1 below 7: 1 0x9e San d'Oria, 2 0x9f Bastok, 3 0xa0 Windurst, 4 0xa3 Wyverns, 5 0xa4 Griffons, 6 +0x29c ? 0xa5 : 0xa3, 7 +0x29c ? 0xa5 : 0xa4 (0xa5 a crystal shard *(inferred: carrying Petra;
- L10673: … : 0xa4 (0xa5 a crystal shard *(inferred: carrying Petra; +0x29c = 0x067 type 2 u32 @0x14)*); allegiance 0x20 / 0x22 / 0x24 / 0x26 -> 0xa6, 0x21 / 0x23 / 0x25 / 0x27 -> 0xa7 *(inferred: Brenner sides)*

## Merit Points and Job Points (2026-10-25)

`MODLOG.md:10715`

- L10758: meritcat colours (FUN_101491a0): the name grey unless in the Mog House (session +0x3c35c bit 0x100; ours myroom *(inf: the same zones)*

## Equipment: pupequip, the not-loaded refusal, the pointer focus swap (2026-10-25)

`MODLOG.md:10791`

- L10801: C.7: the pointer over the list while the grid has the focus, or over the grid while the list has it, sends message 8 as Num+ (FUN_10132c30 / FUN_101f27a0) *(inf: on any pointer move inside)*

## Linkshell: the Link List and member actions (2026-10-25)

`MODLOG.md:10803`

- L10816: Disabled: rank changes and Kick unless through linkshell 1; the rank change unless the member wears it as linkshell 1 *(inf)*
- L10827: link12 shares the list globals with scresult (DAT_106644dc / DAT_106644e4, busy DAT_10385a04) *(inf: opening one replaces the other's list;
- L10836: Kick's prompt also takes FUN_101cc590(race) *(inf: the pronoun)*

## Help Desk: the GM reply page (2026-10-25)

`MODLOG.md:10841`

- L10857: … writes the finished text as "gmNotice.fdt" / "gmNotice.siz"; FUN_10115250 reports the waiting reply to netstat (+0x30 = 2, FUN_102014f0; which sprite its states 1 / 2 draw is not traced).

## The party cursor with the mouse (2026-10-25)

`MODLOG.md:10860`

- L10862: The `<stpc>` / party menu party cursor (FUN_102036f0) takes the party windows' generic mouse events *(inf, as the other windows)*

## Party requests (2026-10-25)

`MODLOG.md:10876`

- L10928: The /search grammar has a type-0x18 branch (FUN_10104f00), but no keyword maps to it *(inf: not reachable by typing)*
- L10930: The Request toggle (FUN_10204ee0): FUN_10210e30(LAB_10204f40, set, 9 0x140, ..) with the cursor on No; LAB_10204f40 clears on Yes (FUN_1020c3e0), then calls FUN_10204c50(set) either way *(inf: Escape answers No)*

## Records of Eminence lists (2026-10-25)

`MODLOG.md:10950`

- L10985: Remove (LAB_10232970): Vana'Bout -> the hold prompt 0xc 0x20d (not done); a count above 0 -> 0xc 0x74 "Your progress will be reset ..." (FUN_10210f10, default No; ours on the log *(inf)*
- L10986: Each send restarts a 30-frame cooldown (DAT_10668edc, also on describe and every answer; ours 1 s *(inf: 30 fps)*
- L10996: FUN_10093a90 at every zone load clears bit 3, but DAT_10487eec only while the login flag (*(PTR_DAT_103601bc + 0x10), game state 5) is set *(inf: once per login)*

## Blue magic sets (2026-10-25)

`MODLOG.md:11034`

- L11068: … calls FUN_1022b3b0(0x3c), recast slot 0x400 (DAT_106679e0, entry 0x400 of the spell recast array DAT_106671e0) = 60 * 60 ticks; 0 resets DAT_10667a88. What the value means is inferred (a change counter or the time of the last change).
- L11084: Open order (FUN_101df020): persona (at 16, 48 as in Equipment), bluehelp, bluinven, bluepoin, bluequip last (it takes the focus; inferred).
- L11094: Only message 5 is handled: Escape closes it through the generic handler, the filter left at 0 (inferred).
- L11095: Mouse (inferred): a focused list whose pointer is over the other list sends itself message 8 (FUN_101262e0, FUN_10118c10 on DAT_104e1d4c +4 / +8), so hovering moves the focus.
- L11097: c2s 0x102: the throttle's intervals sit at player +0x43e34 (all 1000 ms); @5 is 0, @6..7 not written (inferred 0).
- L11098: … sit at player +0x43e34 (all 1000 ms); @5 is 0, @6..7 not written (inferred 0). LSB: a spell set in another slot empties the target slot and stays where it was; retail moves it (inferred).

## Chat line: the auto-translate phrase colour (2026-10-25)

`MODLOG.md:11112`

- L11116: Replaces the inferred 2019 log table entry 0x69 (pink-violet) of "Auto-translate".

## Camera collision as retail (2026-10-25)

`MODLOG.md:11119`

- L11132: Then the tail, shared with the locked branch: (B) when the eye moved more than 0.005 this tick *(inf: retail 0.01 a frame)*
- L11135: … bar (+-0.17 along the camera x: two faces crossing it move the eye until the +x end touches the face nearest the -x end; one face, the same point within 1e-4 from both ends *(inf)*

## Effects audit: fx-sweep (2026-10-25)

`MODLOG.md:11197`

- L11214: 644 item and 14 ability files are effect files with other entry names (4972: goal / kill / open / clos; 56695: str0 / str1 / dis0..2 / trg0): which of their schedulers retail starts is not traced yet.

## Scheduler task ops: holds, locks, loops, flow (2026-10-25)

`MODLOG.md:11269`

- L11303: Readers of +0x80: FUN_100cdd60 (the op 0x25 / 0x21 hit reaction), the function at 0x100cdf00 beside the guard FUN_100cde70 (a reaction, inferred), and FUN_100c63d0 / FUN_100c83c0 (the status / move motion choice).
- L11317: Not traced: whether retail steps a clone in the frame it is made (ours does, up to one frame early).
- L11320: 0x7f (item files 5036, 5069..5071): sub-task FUN_100613d0 drives +0x668 from vt +0x38 to u32 @8 through vt +0x34 (0x80 at construction; no reader on skeleton actors, inferred).
- L11321: 0x86: entity +0x128 \|= 0x80; no test of bit 7 in either build (inferred).

## Weapon meshes hidden and shown: scheduler ops 0x75 / 0xa3 (2026-10-25)

`MODLOG.md:11326`

- L11339: The summary now ranks only inferred attach parts (modes 9..0xc, point 0x35), not every special point.

## Effect stream ops: ground snap, point offsets, emitter spin, no-ops (2026-10-25)

`MODLOG.md:11341`

- L11362: … projection, 141); task ops 0x40..0x47 / 0x54 / 0x55 (the shine overlay), 0x88, 0x3f, 0x2c, 0xb2, screen ops 0x0e / 0x72 / 0x0f, 0x7a; attach modes 9..0xb (the bone rotation, inferred).
- L11379: The nearest-hit rule of FUN_101693b0 (point 0x30, op 0x42) is not traced.
- L11389: Op 0x56's real users besides VuAnm: VuDist 79 files, Disintegrate 4 (5036 yam_/mt00, 5069, 5070, 5071; its use of the particle block is not traced), VuLine 1 (5024;

## The actor shine overlay: scheduler ops 0x40..0x47, 0x54 / 0x55 (2026-10-25)

`MODLOG.md:11414`

- L11430: Fog scales it out *(inf)*
- L11439: … colour (set +0x60, get +0x40c); +0x73c U (+0x410 / +0x414; +0x58 sets U and V); +0x740 V (+0x418 / +0x41c); +0x730 a parameter, 0x48 (+0x7c / +0x80; no reader found, dead, inferred);
- L11444: Whether a child made during the scheduler's update steps in the same frame is not traced (at most one frame).
- L11450: Two drives on one channel do not cancel; the one stepped later wins (inferred).
- L11459: The mesh declaration has no diffuse, so DIFFUSE is white with lighting off (inferred, the D3D8 default).
- L11460: Op 0x05's `ded?` / `cor?` case (0x1005bbd2) skips the scheduler's scope for actors where FUN_10081b30 == 0 (not traced).

## Effect attach modes 9..0xc: the bone frame (2026-10-25)

`MODLOG.md:11463`

- L11477: … times T(doll position), no rotation, the point ignored: a rebuild (op 0x11, spin ops) adds the position again (copied); a doll's 0xb / 0xc heading is the math pool's scratch *(inf: identity)*

## Camera shake and rumble: element update op 0x5f (2026-10-25)

`MODLOG.md:11518`

- L11557: *(inf: whether the pair is read at emission or at instance start;
- L11560: Consumer FUN_10007ae0 from the pad poll FUN_10005a40, once a frame *(inf)*
- L11563: level x 10000 / 256: XInput (pad +0xc9a set, needs +0xc99 and DAT_10455c39) both motors min(level x 4, 0xffff) via FUN_10008ed0 *(inf: XInputSetState)*
- L11566: DAT_10455c39 is the second flag of the gamepad settings string, the external gamepad tool's *(inf: "vibration on")*
- L11567: Also added to +0xdb4: a sound task FUN_100381d0 (60 a frame while its timer bit 0x20 and a flag are set) and FUN_10252ce0 (a sound level) *(inf: which sounds)*
- L11598: … / DAT_10455c39; above 30 fps retail runs n = 1 and so shakes faster, ours keeps n = 2 per 2 ticks; the lobby, creator and title cameras are not the game camera and do not shake *(inf)*

## Distortion meshes: CMoDistModelElem (2026-10-25)

`MODLOG.md:11600`

- L11644: … the copy's fog under vertex fog is driver-defined (ours copies unfogged); whether device +0x194 is the 3D target in every mode (the +0x198 target and the after-image path are not traced);

## Specular effect meshes: CMoD3mSpecularElem (2026-10-25)

`MODLOG.md:11668`

- L11708: … a pre-transformed quad with stage 0 COLOROP MODULATE2X(TEXTURE, TFACTOR), ALPHAOP MODULATE4X, TFACTOR = C, then that target on the stage: S' = sat(2 T C), single mip, 8-bit *(inferred: a half-texel shift, not reproduced)*
- L11736: … footage of 3446 or DAT 109's `cry1`); the effect stages' MIPFILTER is inherited; the copy's half-texel shift and 8-bit quantisation; stage 2 inherited by the no-groups path *(inferred)*

## Scheduler ops 0x3f, 0x88, 0xb1 / 0xb2 (2026-10-25)

`MODLOG.md:11747`

- L11786: … FUN_100543a0 deletes every element it made (asm 0x1005a010..0x1005a027); sound elements start their voice only in the manager pass (FUN_10036230), so nothing is seen or heard *(inferred: no element class acts in its constructor)*

## Effects: the ground projection (D3m + 0x200000, emission op 0x43) (2026-10-25)

`MODLOG.md:11793`

- L11839: *(inf: an MMB resource would leave FUN_10183760's lists, an untextured mesh 0x10352348 / 0x10352648.)*
- L11841: … lists, an untextured mesh 0x10352348 / 0x10352648.)* LIGHTING follows flag 16 (2 generators); without normals in FVF 0x142 the lit colour would be the ambient / emissive terms *(inf)*

## Scheduler op 0x7a: pin actor1 to actor2 (2026-10-25)

`MODLOG.md:11877`

- L11885: Ours: Effects::pins / pinned (computed in step_tasks with who_point), the client's entity loop places a pinned entity there with no ease; afterwards it eases back to its slide *(inf: the dtor's +0x5fc = +0x5c4)*
- L11891: The sub-task (0x78 bytes) writes actor1 +0x7a4 = the point, a reference to actor2 (the field is lost by the decompiler; +0x7a8, inferred) and +0x7b4..+0x7c0 = the offset.

## Effect scheduler screen ops 0x0f / 0x72 / 0x0e (2026-10-25)

`MODLOG.md:11895`

- L11908: f *(inf: FUN_1000a5a0's uv order)*

## Generator op 0x15: the view gate (2026-10-25)

`MODLOG.md:11935`

- L11943: Ours (fx/emit.rs generator_stream): off screen = every corner outside one plane of the scene frustum (fovy 2 atan(192 / focal), Registry::aspect from the client each frame) *(inf: the exact test of FUN_1004a870's vtable +0x8c call;

## Weapon after-image: scheduler op 0x2c, Afb chunks (2026-10-25)

`MODLOG.md:11966`

- L12005: None of the install's Afbs sits in an equipment file *(inferred, by id ranges)*
- L12011: Afb details: bytes 2 / 3 unread *(inferred: exporter data)*
- L12051: Their triangles are taken as not drawn *(inferred: D3D with NaN positions)*
- L12058: … the effect file; FUN_100ce480 searches only the actor's lists, and neither the file's run callback (LAB_100a7640 in the 2019 build) nor op 0x05 (FUN_100cd920 / FUN_100cd4e0 not traced for an append) puts the file there;
- L12061: … deletes itself and the after-image, FUN_10060af0 (+0x14) only itself (leaving the after-image updating with a dead actor handle); which one the actor's removal fires is not traced.

## The near-camera character fade (2026-10-25)

`MODLOG.md:12077`

- L12095: Its meaning was not traced.
- L12097: After the cylinder (not done): vt+0x260 (actor +0x8ac bit 0) sets 0; in first person the actor whose entity FUN_10096250 names on the player sets 0 (not traced further);
- L12100: In the update, entity +0x12c bit 26 (an 0x00E flag, not traced) forces +0x664 = 1.0;

## The block list (blacklist): its readers and /blacklist add / delete (2026-10-25)

`MODLOG.md:12105`

- L12111: … party requests (done before); FUN_100a1a00 (below); and FUN_100f8d30 / the asm-only code at 0x100f8f10 / 0x100f9026, packets 0xa1 / 0xa2 of another session (DAT_104dfd98's) *(not traced: not the map server's)*
- L12116: Ours: partyreq::sender_blocked; for 0x029 / 0x02D only the printed line goes *(inf: the level-up effect of the same handler still runs)*
- L12121: … and answers s2c 0x042 {ID, Name[16]} @4, Mode @0x18 (0 added, 1 removed, 2 failed): ours adds / removes the name and prints 9 0x74 / 0x76, or 0x75 / 0x77 by the edit asked *(inf: the list update on 0x042)*

## The toggles' consumers (2026-10-25)

`MODLOG.md:12154`

- L12200: … 1 (LSB never); +0x120 b13 MonsterFlag = u8 @0x20 bit 0. On LSB, BST-charmed monsters carry b30, so /ignorepet does not skip them; retail's charmed monsters are unknown here *(inf)*

## /autogroup, /recruit, /mentor, /itemsearch (2026-10-25)

`MODLOG.md:12234`

- L12249: /recruit fails while the party object exists and DAT_106643a6 is set *(inf: a party request pending)*
- L12256: Flag != 0 puts the name into the input line buffer DAT_1047d3a8 when DAT_103579ea == 0x11 *(inf: an auto-translate / completion request)*

## Text commands: targets, linkshells, /fish, /automove, /follow, /returntrust, /volunteer (2026-10-25)

`MODLOG.md:12261`

- L12268: /fish (FUN_1007abf0): silent in a Mog House or engaged, the status gate 0x75 prints, else c2s 0x01A category 0x0E on ourselves (the water-in-front test FUN_10177730 is left to the server *(inf)*
- L12269: /automove (FUN_1007e170): autorun on, forward as we face *(inf)*
- L12275: Ours: our trusts are our party rows from index 0x700 *(inf: retail tests the owner index +0x2a4)*
- L12303: The third %s is inferred (the decompile drops the argument).
- L12322: … water in front), the session and zone != 0xd3, else 7 0x49 "You cannot fish here." (refusal index 7); already fishing (DAT_10487e49): done; FUN_1009fb50: FUN_100956c0(0xff) *(inf: rod or bait missing, nothing sent)*
- L12332: … FUN_1007bf30(t, 1, k, 1, 0) is refused mounted (7 0x4b) or with our +0x128 bit 6 (7031 0xac), has no distance test, skips a target in status 2 / 3, and queues with key k *(inf: keeps the per-trust packets apart)*
- L12341: /vote 0x10: argc 2 with a number 1..8 votes for the last proposer (DAT_104df910), else a name (<= 15, starting with a letter) with index 0; FUN_100e0a50(name) must be 0 *(inf: not blacklisted;

## Text command /? (2026-10-25)

`MODLOG.md:12351`

- L12358: Retail sets +0xda of a log object to 1 around the lists (FUN_1018ee20) *(inf: holds the log's scroll)*
- L12370: Id 0x10b (no name, FUN_10078150) is a disabled-command stub printing 9 0x67 "Cannot use yet." *(inf)*

## Riding: the chocobo / mount under the rider (2026-10-25)

`MODLOG.md:12372`

- L12439: 0x00E: ChocoboIndex u32 @0x20 >> 5 & 7, u8 @0x24 -> +0x1db unless +0x12c bit 30 (then +0x294) (inferred: a riding NPC;
- L12440: Our ChocoboIndex at zone-in comes from DAT_104dfd98 + 0x40dae & 7 (inferred: the 0x00A copy);
- L12484: cdam: the riding files and animals carry it (rider `chd?` + `bb1?`; animal `dam?`, `vdam` sounds, a fade); nothing in FFXiMain names it (inferred: a DAT 0 scheduler starts it).
- L12486: The shadow mode DAT_1045b77a is 3 for a rider with a linked animal (inferred).

## The block list window (2026-10-25)

`MODLOG.md:12490`

- L12519: … it closes every 0x40000 window: mcr1pall, mcr2pall, mcr1long, mcr2long, cnttime, prvdungp); blkmain slot 183, layer 2, flags 0x00200000 (read only at creation, FUN_101190f0; not traced).
- L12537: … 0 appends, 1 removes by name (the leading letters, up to 16: FUN_100e0800 / FUN_100e09a0), other modes nothing; then state 1 and changed 1 whatever came before (the list update inferred in "The block list (blacklist): its readers ...");

## Mog House furniture (2026-10-25)

`MODLOG.md:12562`

- L12595: Its 'pop0' / 'init' are not started *(inf: retail plays them as a server NPC's)*
- L12605: No wait on 0x01D: each container's pieces appear once its 0x020s are in (inferred).
- L12643: FUN_100c46b0 also starts type-7 schedulers marked self-starting (none in the sampled files, inferred).

## Ground tilt (2026-10-25)

`MODLOG.md:12656`

- L12672: … morbol, treant, wyrms ...) then take FUN_100a4d10's pitch-only normal: the front feet points 10 / 11 and rear 8 / 9 (ours from the rest pose through last frame's matrix *(inf)*
- L12678: Ours had z then x in the model frame with negated signs (inferred, never seen);
- L12689: … 14 \| +0x120 bit 21 \| +0x120 bit 20 (bit 21: op 0x33 / op 0x59 sub 5, cleared each frame outside events by FUN_1008cd90, which sets it for statuses 0x1a / 0x1b / 0x1d / 0x1e *(inferred: transport)*
- L12706: FUN_100d59a0 negates vt+0x210 for down: the sign MODLOG "Emission op 0x8e" marked inferred is right.
- L12741: Open: the writers of +0x128 bit 14 and +0x120 bit 20, and what statuses 0x1a / 0x1b / 0x1d / 0x1e are; whether vt+0x1c4's bone matrices include the tilted +0x998 *(inferred: likely)*

## Op 0xd4 sub 0: the map preview, and the teleport points on every map (2026-10-25)

`MODLOG.md:12744`

- L12801: The constructor FUN_1014dd30 fills +0x13c..+0x53b with 0xff; FUN_1014e020 (from vtable +8, the window's reset, inferred) clears all 32 and the callback +0x53c.
- L12804: The points' i32 height is not read by these functions (inferred).
- L12812: The image request test in FUN_101f9a20 reads map +0x6c4 as null or *ptr == 0 (inferred: a load handle).
- L12816: Draw order: "menu mapframe" is layer 0 (flags 0x20042), "menu query" layer 2 (0x11012), so the menu draws over the map (inferred).
- L12817: The backdrop's 4 cells were read from an HD-scaled dump: to be re-checked with the stock file 53442; that it darkens what the widened bounds expose is inferred.
- L12822: Markers: flags 0x89 when unnamed; label colour index 2 is 0x606070 (table 0x10382084), times alpha; the named (followed) point is drawn last, so on top (inferred).

## Scheduler op 0xac: the death scheduler pick (2026-10-25)

`MODLOG.md:12825`

- L12849: … FUN_1008a120 / FUN_1008a140, FUN_10099a40 (another slot with the same server id), and the 0x00E handler at asm 0x10099b80 (a new entity in the slot) and 0x10099dec *(inferred: the despawn branch, mask @0xa & 0x1f != 0x1f)*
- L12851: 2461..2469 are models 1161..1169 (folder `guar`, *inferred: the Ghrah forms*);

## Geomancer Indi aura, and an actor's deletion unlinking its effects (2026-10-25)

`MODLOG.md:12856`

- L12920: *Inferred / open*: whether a hidden owner (+0xb2 bits, GM hide) hides the aura (FUN_100c9810 only places it;

## The actor refraction pass: scheduler ops 0x48 / 0x49 (2026-10-25)

`MODLOG.md:12924`

- L12955: Two drives on one actor both write +0x744 every frame, the later-stepped winning *(inf: task-list order)*
- L12978: *(inf: copy.a is about 1 nearly everywhere, opaque scenery writing sat(4 tex.a diffuse.a);

## Weapon glows, `!in?` / `!kl?`, timed hides (2026-10-25)

`MODLOG.md:12987`

- L12994: 3 weapon effect Off *(inf: row 2)*
- L13031: Not modelled: model kind +0xee beyond "a PC look is kind 0" *(inf)*

## Actor flow: skill files, the +0x80 hold, op 0x2e's end, op 0x7a on us, the battle message queue, the dismount stand (2026-10-25)

`MODLOG.md:13056`

- L13106: Children started by `main` carry no +0xac and may outlive the files *(inf: their motions are not from the skill file)*
- L13111: FUN_100c63d0 (asm 0x100c716f) with +0x80 up also skips FUN_100c9810 / FUN_100c9c10 / FUN_100ca310 / FUN_100ca730 (linked-actor syncs, not traced) but keeps the per-status placement (dead, riding, fishing, chair) and FUN_100ca8c0 (knockback).
- L13119: Inferred: the actor -> entity copy for c2s 0x015 (2019 FUN_10096c00) is not traced in cur;
- L13121: End (dtor FUN_10061f50): +0x7a4 = -1, ref cleared, +0x5fc = +0x5c4, +0x608 = 1; the player stays where the pin left it *(inf)*

## Model hide / show and the part files' generators (2026-10-25)

`MODLOG.md:13184`

- L13193: … does not draw it; it still updates); FUN_10085820 pauses the actor's scheduler tasks (FUN_10074a30, +0x22, children too; the file-root flag +0x1a bit 4 exemption not traced).
- L13209: … hidden iff +0x120 bit 17 (the script hide, event ops 0x22 / 0x4e); otherwise hidden iff +0x120 & 0xc000 (bit 14 = 0x00E u32 @0x20 bit 1; bit 15 from FUN_100a17f0, not traced).

## Time-of-day lighting: the eased sun / moon light, retail's key blend, the weather chain (2026-10-25)

`MODLOG.md:13222`

- L13232: next) t; FUN_10032fe0 / FUN_10028200), x 127, chopped *(inf: the sense toward next)*
- L13247: … restarted or cut the blend); 0 minutes deletes the chain; each object blends its child chain's output into its own env by (total - left) / total (the linear weight MODLOG had inferred), and a finished one drops the chain before it, so a change mid-blend starts from the mixed state.
- L13256: Not traced: the light yaw world+0x1cc (a scheduler op at 0x100580a4), DAT_1047bf18's override of @0x50 / @0x54 / @0x5c
- L13273: The light yaw world+0x1cc (i16 degrees x 0.0174528) turns both sun and moon when non-zero (FUN_10027b80, axis not traced);
- L13277: DAT_1047bf18: while set, FUN_10189980 takes @0x50 / @0x54 / @0x5c / @0x5d from DAT_1065ca54 / ca50 / ca4c / ca48 instead of the lerp; written by scheduler-op code around 0x10059273 / 0x100604d7 (not traced).
- L13279: … light (type 3), Diffuse = the eased colour x FUN_10186fd0 (env +0x44) for the background, E+0x19c x FUN_10186f60 (env +0x24) for actors; Ambient = DAT_1065ca34 (source not traced).
- L13287: Inferred: the slerp's sense (toward next) and FUN_10313820 = acos, from use;

## Motion name bases, `sdep`, event-reserved silence, the sound start cap, one-at-a-time loops (2026-10-25)

`MODLOG.md:13290`

- L13331: … / FUN_100b71e0) for a non-PC (id & 0xff000000 != 0), status 0, kind not 3..5, not bound, +0x120 bit 13 clear, outside the special NPC ids: the idle timer +0x1ba runs down *(inferred: by frame ticks)*
- L13352: … sounds through FUN_100370d0 -> FUN_100363b0, generator sounds, footsteps FUN_10036fa0) and is cleared at the end of FUN_10036230, which FUN_10011a00 runs once a frame *(inferred: FUN_10011a00, called from the main loop at 0x10001b74, is the frame)*

## Lit D3b / D3m effect elements, element-update op 0x6e, the CMoDist fan's ZBIAS (2026-10-25)

`MODLOG.md:13364`

- L13401: Env: the zone env DAT_1065cb04, or the area env keyed by generator +0xc4 when non-zero (FUN_101889c0, falling back to the zone; the writer of +0xc4 not traced).
- L13420: D3a (1,301) and CMoDist (104) carry the flag but draw unlit; DistModel 2 (no visible change); one ring (83908 d_pi/effe/iccl/gr00: FVF 0x42 has no normal, ambient only, inferred).
- L13433: Copy texel (i, j) samples u = (i W / 256 - 0.5) / W, v = (H - 0.5) / H - j / 256 (linear, MIRROR); a ring pixel u = (px + 0.5) / W, v = (H - 0.5 - py) / H (inferred from the D3D8 rasterization rules).

## Self-starting schedulers: end loops, the load-time start, op 0x52 time windows, op 0xa2, update op 0x64 (2026-10-25)

`MODLOG.md:13436`

- L13464: Not traced: which emission op fills update op 0x64's slot;
- L13475: Stream @0x10 is op 0 alone in 301009 and 69 6a 00 in 3104; no reader was found (unused, inferred).
- L13487: … m_wa/win1, 51503 m_gi/pian, 51555 m_ev/suzu, 62870..62872 m_ev/bels / belb / belw, 62927 m_bt/show, 62933 m_ev/toke (always 01 a2 52 ...); who starts them with actors is not traced.
- L13513: The emission op that fills the slot is not traced (inferred: the one sharing slot k).
- L13515: … object runs in the zone parse's frame (a weat op-0x52 clone spawned at load would be killed with it unless stepped in between); the sub-area weather (their chained environment, inferred the same);
- L13516: … in between); the sub-area weather (their chained environment, inferred the same); retail's file slots on event op 0x34 area swaps (ours starts the auto schedulers again there; inferred).

## Timers (an Extra): recasts, our effects and the ones we put on others (2026-10-25)

`MODLOG.md:13591`

- L13656: Event flow: LSB runs the whole action before it queues the 0x028, so effects the action consumes (Saboteur, Stymie, Marcato, Elemental / Dark Seal, Divine Emblem; inferred for Presto and Snake Eye) send their 0x029 wear-off before it: take our state at the start, or count an effect lost in the same tick.
- L13761: Not worked out: charmed pets' moves (about 1,790 mob skill scripts), item-script added effects, whether an always-overwrite recast's 0x029 always arrives before its 0x028 *(inf)*

## Ship voyages: the moving world (2026-10-25)

`MODLOG.md:13778`

- L13828: Their routes *(inf, from the endpoints against the docks)*

## Motion regions: a region the new motion lacks keeps playing (2026-10-25)

`MODLOG.md:13878`

- L13896: … still playing where the current one has no chunk, its ticks, once), set at each motion change, the finished-counted clear included; kept regions advance at the status rate *(inf: every frame)*

## Weapon skills from NPCs and trusts: the per-race files (2026-10-25)

`MODLOG.md:14004`

- L14018: Now every actor with a model does, race 0 for model looks. Its body info byte 9 (the skirt file) is taken as unset *(inf)*

## Event op 0xab 0x11: every model built again after the event (2026-10-25)

`MODLOG.md:14080`

- L14088: Furniture and auras are not entities and stay *(inf: retail keeps XiModelActors with +0x778 set)*
- L14096: The loader rebuilds up to 50 a frame (DAT_1035af30), walking the DAT_10486020 bucket lists (inferred: by distance).

## Event toggles, part 2 (2026-10-25)

`MODLOG.md:14164`

- L14169: 0xab 5 / 6 (+0x120 bit 3): Entity.quiet_status; a status change plays no death motion, no status schedulers, and a door takes its new state without its open / close scheduler or box change *(inf: FUN_100ad1e0 / 270 own both)*
- L14197: … leader's region entry (else its region 0 entry), F = ours (FUN_1001b1a0 on motion object +0x674, +0x24 + 0x14 i); when both exist and 0 < L.frame <= F.frame, F.frame = L.frame (inferred: vt +0x30 returns +0x24).

## The control lock: event ops 0x20 / 0x30 (R9, 2026-10-25)

`MODLOG.md:14200`

- L14206: Per frame (FUN_10096640, all.c ~115839): freed when the talk target (FUN_100fc300) is gone, or has +0x12c bit 30 and +0x124 bit 12 (not traced).
- L14211: Freed by: the event end unless carried, s2c 0x052 mode 0, any 0x052 with no event running *(inf: the release after a carried end)*

## Event op 0x46 sub 3: the camera seeded at a point (2026-10-25)

`MODLOG.md:14214`

- L14223: Ours: Effect::Camera's offset, camera::face_at (face() is face_at with (-4, -2, 0), or (-3, -1.5, 0) in the Mog House); the side axis's sense is inferred (no site uses z).

## Mog House layout mode, part 1: entering, selecting, the camera (2026-10-25)

`MODLOG.md:14225`

- L14262: FUN_101d4d90 on a mode change (except 0 -> 2): to non-zero, FUN_100842d0 (inferred: the player camera off), player +0xb2 \|= 0x10 (not drawn), the position DAT_1037eba8[floor], focal saved (+0
- L14277: Link 999 (FUN_1017a590): byte +8 of every instance whose MZB record +0x50 is 999 (instances start at 2; 0 = not drawn, inferred).
- L14283: Not traced: what reads target system +0x74 bit 4 and myroom +0x7e4.

## Mog House layout mode, part 2: the rmlo windows, moving, turning, removing (2026-10-25)

`MODLOG.md:14286`

- L14293: … while Storage (container 2) holds anything; Clear away when Storage's free slots are fewer than the storage slots the piece and those on it give (item DAT +0x10, extra 0xd14 *(inf: FUN_101ce130 sums item info +0x2a)*
- L14322: Dim rows: FUN_101f44f0 / FUN_101f4540 / FUN_101f45d0 (Storage = container 2 view DAT_10666e34; "holds anything" = count > 1, inferred).
- L14324: 0x8e (rot + 1) has no default keyboard binding (inferred: pad only).
- L14329: /layoutctrl (FUN_101cd060): config 0xc1 != 1 and look.z <= eye.z -> dir = {2, 3, 0, 1}[dir]; type 2 never remaps; with the layout camera only view 2 flips (inferred: +x is screen right looking toward +z).

## Auction house (2026-10-26)

`MODLOG.md:14427`

- L14455: Inferred / not compared with retail: the date column's width (the sprite font's "26/10/25" fills its 65 px), the stack
- L14474: Nothing clears it on zoning (inferred: kept with the session;
- L14487: Enter (FUN_101d9850): a charged item (record +0x2d, inferred) whose extdata fails FUN_100feb80 -> 7 0xb0;
- L14493: Not traced: the menus' +0x24 (FUN_101da710, resets the remembered cursor row) runs when the menu is active again (inferred from the bazaar);
- L14494: Not traced: the menus' +0x24 (FUN_101da710, resets the remembered cursor row) runs when the menu is active again (inferred from the bazaar);

## The raise prompt's frame, larger (ours, 2026-10-26)

`MODLOG.md:14499`

- L14504: … edge. Retail's layout and the compass (kept while dead, FUN_1021ed90; closed only by group-4 windows, FUN_1015ed10) were checked; which of the two retail draws on top was not traced.

## Mog House layout mode, part 3: Place (2026-10-26)

`MODLOG.md:14508`

- L14531: List source vt 0x1033a668: open FUN_101f4130 (FUN_1021c7e0(0, 3, 0x12, 0), inferred: view 3 rebuilt), close FUN_101f4160, count FUN_101f4190 (installed pots, type 12, and mannequins, type 14, in both containers on any floor).

## Mog House layout mode, part 4: the grid overlay (2026-10-26)

`MODLOG.md:14537`

- L14544: … 4, 24-byte vertices, closed back to v0) with ZENABLE (7) off around the call; it repacks 0xAABBGGRR to D3D ARGB (byte 0 red). Blend state is inherited *(inferred: alpha blending, as the world's last state)*

## The raise offer during a zone-in event (2026-10-26)

`MODLOG.md:14574`

- L14579: … to 0 when we are not dead, FUN_10097a60 status 2 / 3) is called by FUN_10096640 (c:115815) only outside events (status not 4 / 0x1a..0x1f, +0x1fe clear, and some globals not traced).

## Mog House Remodel (2026-10-26)

`MODLOG.md:14641`

- L14656: +0x50 is sent as Param1 *(inf: 0;

## Dialogue: key items with their own article (grammar 0x36 / 0x45) and `7f 80` (2026-10-26)

`MODLOG.md:14664`

- L14677: … with the next name (spell / ability codes 0x10 / 0x11 / 0x16 / 0x1a, and the grammar block's phrase) and cleared; n = 1 converts the first character (FUN_102518a0 mode 2 *(inf: upper case)*

## Mog House mannequins, part 1: the look and the poses (2026-10-26)

`MODLOG.md:14682`

- L14707: s2c 0x026 (LSB ITEM_SUBCONTAINER): @4 is_used, @5 container (1 / 9), @6 slot, @7 unknown, @8 u16 an item id whose class picks pose 8's motion *(inf: retail's main weapon;
- L14720: Race 0 (LSB's default exdata) reads every per-race table's entry 0, Hume male *(inf)*
- L14734: No furniture box, no camera-ray mesh; the character bump applies (kind 0, id high byte 0x04, status 0x31) *(inf: not seen)*
- L14735: GraphSize 0 makes a Hume 0.95 *(as read;
- L14738: Bank 1's mq0..mq6 are the /hurray poses by weapon class *(inf)*
- L14740: A pose missing from the files plays nothing new *(inf: the rest pose;
- L14745: No client path changes a pose (no text command, nothing writes exdata) *(inf: an NPC event, server side)*

## Mog House mannequins, part 2: dressing (2026-10-26)

`MODLOG.md:14751`

- L14773: Mode 4 (FUN_101ce6a0): the mannequin at y min(y, -0.3), yaw from 0 turning 3° a retail frame (0.02618 a tick *(inf: 2 ticks a frame)*
- L14809: … 250) at +0x7f4 and sets 300 (DAT_1037eb94[4]); 4 -> 1 runs the same branch, saving 300 and setting 250, so leaving layout afterwards restores 300, not the player's focal *(as read;
- L14814: FUN_10133410 checks neither slot nor race, so in mode 1 an Up / Down move can put the picked item on any slot *(as read)*

## Talk refused while invisible (2026-10-26)

`MODLOG.md:14911`

- L14919: The audit's "meaning inferred" is settled.

## MacroPalette: the mouse (2026-10-26)

`MODLOG.md:14932`

- L14940: The palettes: the pointer over the other one moves the focus there at that column *(inf)*

## Fishing (2026-10-26)

`MODLOG.md:15001`

- L15030: … (FUN_100964f0) is refused while the latch is set or our status (or last 0x037 status) is a fishing one; the field's Enter (target / action menu) is not taken while fishing *(inf: retail's menu is busy-gated)*
- L15036: The rod is its own actor (id 0x1000_0000 \| angler; file DAT_1035f598[race] + look, Mithra willow 39477) at the angler's place with the angler's status (its fh1 -> fh10) and scale *(inf)*
- L15045: … 0x38 `!f01` after 136 ticks, 0x3a..0x3e `!f02` after 102 / 40 / 44 / 68 / 0, on the rod; only the rods of looks 14 / 15 (the +1 rods, MId 0x800e / 0x800f) have them *(inf: the others draw nothing)*
- L15049: A status 6 / 0x32 / 0x38 waits in held_status while the model still plays its walk or run motion (FUN_100d65b0(1) *(inf: walking = those motions)*
- L15050: Left: FUN_10127ea0 (drops queued text after a mode-3 send *(inf)*
- L15062: The water test returns true while DAT_1065ca30 is set *(inf: debug)*
- L15077: FUN_10127ea0 after a mode-3 send empties two message lists on DAT_10578158 (FUN_10126a00) and sets DAT_10488208 = 2 *(inf: so the result line is not held behind queued text)*
- L15081: Sub-state in +0x13c bits 8..10, the arrow clear counter in bits 12..17; bit 11 (a forced end) has no setter in this build *(inf: dead)*
- L15104: … status x 4): 0x38 fsh0 (fh0? 136 ticks, then fh1? held), 0x39 fsh7 (fhb?), 0x3a fsh2 (fh3? 168, fh4? 124), 0x3b fsh3, 0x3c fsh4, 0x3d fsh5, 0x3e fsh6; fsh1 is in no table *(inf: unused)*
- L15107: The status motion is `fh1 ` unless DAT_104568fc +9 *(inf: an event pose flag)*
- L15111: … + look, bases {39307, 39307, 39339, 39371, 39403, 39435, 39435, 39467, 39499}; skeleton `sao`, mesh `hf_s`, motions fh00..fhd0 / idl0, fsh0..fsh9, SeSep 8231..8236 / 6095 *(inf: cast / reel)*
- L15115: Leaving fishing drops the files and the rod *(inf: the FUN_1008a2f0 path, as for a mount)*
- L15117: Open: how the plain rods draw their line *(inf: a mesh in `hf_s` animated by the fh motions)*

## /garden and /mutelist (2026-10-26)

`MODLOG.md:15168`

- L15175: Mode 2 itself does not call FUN_101d3f60 (the layout camera); ours has it from the start *(inf: the camera retail shows there)*

## The Adventuring Primer (2026-10-26)

`MODLOG.md:15219`

- L15246: *(inf / to compare with retail footage)*
- L15273: Enter during the first 10 frames reaches FUN_10233ba0 with the default cursor child 1, so it opens Basics *(inferred)*
- L15279: Mode 7 (FUN_101f9a50(-100, 0, 7, ...)): the gate FUN_101f9a20(7) needs no map image up and FUN_10087650 *(inferred: "key items arrived")*
- L15288: guide00 closed: slot 2 cleared by FUN_10234080 *(inferred: faqsub sets it again only in its open hook)*
- L15290: Open: whether the 8 empty row bars show for the first 10 frames *(inferred: yes, about 1/3 s)*

## The generic press on every window (2026-10-26)

`MODLOG.md:15305`

- L15329: Windows that close on the press (ok, ptc6yesn) show no ripple: while closing only the frame draws *(inf)*

## Mog House furniture: the l_01 light (2026-10-26)

`MODLOG.md:15349`

- L15365: Open (below): the scenery ambient may be twice retail's (D3DRS_AMBIENT = ftol(ambient x 128)); the 3 blended pieces keep z writes in retail *(inf)*
- L15383: Callers: the scenery frame mode 0, area 0 (per instance its area, FUN_1017c9c0); the actor update (0x100a5d1c) mode 1 while actor+0xa8 == 0, else 0, area actor+0x5e4 (the writer of +0xa8 not traced;
- L15384: AMBIENT bytes = ftol(min(c, 1) x 128.0) in both modes; the light's Ambient (DAT_1065ca34) is never written (inferred 0).
- L15395: The blended pieces (bit 31: items 113 Cleaning Tool Set, 203 Bomb Lantern, 221 Arcane Flowerpot): retail keeps z writes on and bias 0 (inferred), unsorted, in the actor pass;

## arealist modes 1 / 2: the event's area pick (op 0xcc 0x10 / 0x11) (2026-10-26)

`MODLOG.md:15456`

- L15461: No rows: nothing opens (the pick keeps its value *(inf)*

## The Linkshell Concierge (event 6100, s2c 0x048, link5 mode 1, lscswin) (2026-10-26)

`MODLOG.md:15484`

- L15549: … Escape -> FUN_101e4380 (cb(0, 0), lscswin closed); Num+ FUN_101e4440 (SE 0013 by FUN_10037e90); the pointer over lscswin (FUN_101e4610 with pointer mode 2, FUN_10118c10 *(inferred)*
- L15559: `7f 81 n` (FUN_100c1650 asm 0x100c2164 -> FUN_100bda90 -> FUN_101cbac0(slot + 4)): the decoded name, no colour; a missing slot (n >= 16, Flag 0) makes the expander return 0 (asm 0x100c298c) *(inferred: the line never shows)*
- L15560: Not traced: what a focus change between link5 and lscswin does beyond FUN_1015db80;

## 0x00E SubKind 5 / 6 and event op 0xb6 0x11 (2026-10-26; partly traced)

`MODLOG.md:15586`

- L15593: How FUN_100d0ef0 decodes the 0x9000 word is not traced (for LSB's only SubKind 6 NPC, Qiqirun 0x13fe, neither 44393 = type 1 with the full id nor 40297 = type 1 with 12 bits is a model).

## Creator: the Register prompt and the name check (2026-10-26)

`MODLOG.md:15599`

- L15641: Child 3's element state 0x10 (not done): the open callback FUN_10164580 calls FUN_10118b60(0x10, 3) (-> FUN_10119970) for every prompt; what the state does was not traced.
- L15650: Escape on chmktown: FUN_1023df00(slot) sets the slot's name to " ", FUN_1023dbb0 rebuilds the slot table, FUN_10037a70(0x1e) *(inf: a music fade)*

## Monstrosity, part 1: the look, the predicate, the music (2026-10-26)

`MODLOG.md:15652`

- L15682: Fighting Stance: LSB changes only the displayed species (+0x210 = 0x8000 \| stance species) and keeps the Look *(inf: retail sends the stance form's model id)*
- L15686: … char[32] name; +0x28 u16 record type 0x0d; +0x2a u16 family (FUN_10142770); +0x2c / +0x2e / +0x30 u16 unclear (Rabbit 1 / 1 / 0, Behemoth 5 / 2 / 2, Gnole 0x18 / 0 / 1; +0x2c *(inf: icon or sort)*
- L15691: Size: GraphSize applies to any model through its info chunk; LSB sends the character's own size, not the species' *(inf: retail sends the species')*
- L15702: … @8 (FUN_100fd480), u8 @0xA (FUN_100fd4a0; the instinct cost tier in FUN_100fda60, meaning unclear), 12 instincts u16 @0xC + 2 i (FUN_100fd4c0), u8 @0x24 + i the two prefix ids *(inf)*
- L15711: … 0x0DF's name ids truncated into a u16; 0x00D's Monstrosity fields only in the Model block, so a General-only update carries @0x3E = 0 and observers drop the name and glyph *(inf: +0xee may fall back to 0)*

## Lobby: netwait and netbar (2026-10-26)

`MODLOG.md:15875`

- L15918: Step 1 counts only from sub 14, step 3 only while sub < 10, the others every call; 3600 calls are 2 minutes at 30 fps *(inf)*
- L15954: Unclear: what the PlayOnline calls return under xiloader (step 3 sub 14's flag 0x8000 branch; ours takes 14 -> 15 -> 22 *(inf)*
- L15955: … flag 0x8000 branch; ours takes 14 -> 15 -> 22 *(inf)*); the step rate (one call per frame, taken as 30 fps); netbar above netwait and the input block (open order and focus, not traced);

## Creator: the world step (2026-10-26)

`MODLOG.md:15959`

- L15981: 3118 (unknown world) and 3119 (no world, a pass) cannot come up from a list; the local filter FUN_101ba240 (3322) is not traced.
- L15998: Mode 1 (back from the pass) keeps the rows; the cursor row then is inferred row 1.
- L16001: Paste not traced.
- L16013: worldsel: 14 rows 16 apart from (16, 4); row ref kind 0 = 170 (an invisible cell), kind 1 = 171 (a translucent bar; which row state shows it is not traced);
- L16015: Not traced: FUN_101ba240 (the local name filter, 3322), the right click and the mouse on chmkserv / chmkpass, whether retail sends 0x24 again later.

## Monstrosity, part 10: Customization (2026-10-27)

`MODLOG.md:16018`

- L16037: Commit: c2s 0x102 (flags 1 species, 4 instincts, 8 / 0x10 descriptors) at most once a second *(inf: retail's 30 frames)*

## The full log's tabs and the log filters (2026-10-26)

`MODLOG.md:16100`

- L16116: Channel tabs gather about 50 rows back from the store *(inf: wrapped rows)*
- L16145: Every row also goes to the buffer's history store (FUN_10128740 -> FUN_1018ed80): pages of 50 rows in 20 rotating files "%d_%d.log" *(inf: about 1000 rows)*
- L16149: Tabs have no mouse hit test *(inf: none found)*

## Modern UI sweep fixes (2026-10-26)

`MODLOG.md:16152`

- L16155: … 0x037 carries status 3 (Death) for a character leaving by a zone line (isDead() counts status Disappear) while HP is left; ours keeps the old status for a 2 / 3 with HP > 0 *(inf: retail never sees it)*

## Dual wield (2026-10-27)

`MODLOG.md:16260`

- L16279: With +0x8b5 a 'b' scheduler calls the model's vtable +0x250(1) *(not traced: no effect seen;

## A new scheduler task takes its first step at once (2026-10-27)

`MODLOG.md:16356`

- L16366: … task at once (fx/tasks.rs: ops 0x03 / 0x73, 0x09 / 0x57, 0x3b, 0x3c, 0x24 / 0x5d and run_pending's starts), then marks it `stepped` so the frame's own step passes it by once *(inf: one step a frame, which keeps the hit sparks at T+5 ("Effect schedulers");

## Dolls made before the actor's first pose (Shantotto II's summon sparkle, 2026-10-27)

`MODLOG.md:16370`

- L16378: Ours now: a doll of an actor not yet posed (Actors::posed: no bones yet) keeps no points; who_point reads the live ones until the first pose, and step_tasks fills the doll's points then *(inf: retail's model is posed before its init schedulers run, so its doll never sees the rest pose)*

## Buying from a player's bazaar: View Wares (2026-10-27)

`MODLOG.md:16430`

- L16474: s2c 0x109: u32 buyer UniqueNo @4, u32 count @8, u16 ActIndex @0xc and u16 bazaar ActIndex @0xe (both 0 from LSB), name[16] @0x10, u8 slot @0x20; @0xe as the bazaar owner is inferred from its two comparisons;
- L16501: 7031 0x5f "Bazaar closed due to {name}'s settings change." has no client caller *(inf: a server 0x053 line)*
- L16503: Open / inferred: the flag FUN_10210880 passes to source +0xc, taken as "the list is live";
- L16505: … hidden while the list is up come from the group walk (money 4 and iteminfo 8 take the comment window's bits 0x0c), not watched; the list's Escape sound (the generic close) not traced;

## Scenery culled on the GPU (ours, 2026-10-27)

`MODLOG.md:16561`

- L16604: … in exactly one leaf; an interior box is the union of its children's, a leaf's the union of its placements' world boxes from the record's named MMB (not the _h sibling) *(inferred: a quadtree over x / z built offline)*
- L16626: … / C+0x2f0 (C+0x2f0 = h / (0.75 w) of the aspect entry: 1.0 at 4:3, 0.75 at 16:9); h = u16 C+0x1a, set once at scene init (FUN_10178f90), the 3D target height (default 512) *(inferred: the background resolution's Y)*

## Camera lists (retail's visible sets) in the GPU culling (2026-10-28)

`MODLOG.md:16705`

- L16724: A merged sub-area's objects get vis 0 *(inf: retail would hand slot 0 a list from the sub-area's pool;
- L16726: *(inf: our door boxes name no list)*
- L16747: A list can leave out placements that are in the frustum and in range *(inferred: an offline visible set per floor region)*

## GM prompts: "menu    gmtell" (2026-10-28)

`MODLOG.md:16750`

- L16777: … with 0x84 (retail repeats the open slot's tell each frame while another slot waits, the window already gone); the mouse moves the cursor and a click picks, as in the query *(inf: retail's handler takes no row messages)*

## Automaton customization: automato, statpup, pupsibor (2026-10-28)

`MODLOG.md:16801`

- L16828: … heads / frames (FUN_100fcc50(side, 0x1201, n): bit n = item 0x2000 + n); +0x34..+0x53 owned attachments (0x1202: item 0x2100 + n); +0x54 the name (FUN_100fcdc0; 16 bytes, inferred);
- L16866: … SE 0002 again; Escape on automato with only a slot picked, or on the list with only an item picked, does nothing; Num+ with a pick: automato SE 0004, the list nothing. Mouse (inferred, as BLU and the Equipment grid): hover moves the cursor, a click on it is Enter, the pointer over the other window swaps the focus when nothing 

## Log window 2 (2026-10-28)

`MODLOG.md:16877`

- L16922: Windows update head to tail, so after a reopen or Num+ raise window 2 places itself on window 1's previous-frame rect *(inf: a one-frame lag, not observed)*
- L16927: Global opens: the field entry (FUN_1015f3e0(4)) opens every A & 0x80 window, logwin2 whatever 0xb4 is (closed again by window 1's first update *(inf: not visible)*
- L16930: Registry rows 31 / 32: both A 0x00000093, B 0x00000125 (layer 1, group 0); A 0x01 skipped by the focus walk, A 0x02 not closed by the generic Escape *(inf)*
- L16941: rem4li2 and splmsg2 wrap and size at window 1's +0x32; splmsg2's close (FUN_10136f50) closes "menu spoolmsg", not itself, so the full log's open leaves splmsg2 up *(inf: the visible effect)*
- L16950: … the compass (FUN_1021ee20), the party member icons (FUN_10228440, x0 + 200), the party window's extras (FUN_102041d0: the pet block FUN_10202960 and a second block FUN_10202a40 *(inf: which actor, FUN_101cc560)*

## Mog House gardening (2026-10-28)

`MODLOG.md:16957`

- L17004: The moogle's trip (FUN_101d55d0 / FUN_101d5410, every frame unless furnished on 1F): +0x3c starts at 180 and drops to trunc(t - frame ticks) a frame *(inferred: 3 s at 2 ticks a frame, shorter at high frame rates through the truncation)*
- L17006: Going: DAT_10482f18 = 1 (the control lock), 0x50 on 1F, scheduler 'main' of file 0xbf8 on the moogle (FUN_10062e20) unless it runs; on the first frame FUN_101590a0 (target dropped *(inferred)*
- L17042: Open: what 0xbf8's 'main' shows *(inferred: a vanish effect)*

## Full log: the tab switch (2026-10-28)

`MODLOG.md:17047`

- L17056: Ours: logwin.rs `full_tabs`, `next_tab`, `tab_shows`, `LogWin::prepare_tab` (a channel tab keeps its newest 100 rows *(inf: retail ~50 at the switch, up to 100 while open)*

## charlnk (the full log's menu on a player's line) and acoption (2026-10-28)

`MODLOG.md:17071`

- L17083: Ours draws the band under the text *(inf: the order)*

## The Friend List and PlayOnline messages (2026-10-28)

`MODLOG.md:17126`

- L17144: Header Date = unix seconds *(inf)*
- L17152: … status in force, anc2s hand; its layout anchors it right); titlehan (our handle and id while the list or msglist is up; the id text is polcore's 8 base-36 digits of the nick *(inf: slot 0x1db)*
- L17175: Not done: netstat's Help Desk count (GM replies) and its second count (engine +0xe), deleting / unblocking / reordering (retail has no in-game way: the user's call), Away back to Online at login *(inf, spec 2.5)*
- L17177: … and its second count (engine +0xe), the busy strip bit 0x10, deleting / unblocking / reordering (retail has no in-game way: the user's call), Away back to Online at login *(inf, below)*
- L17186: Which polcore slot is which request is inferred from use.
- L17197: How polcore maps the notice's Status into it is unknown: ours Status - 1 *(inf)*
- L17204: A friend playing FFXI (FUN_101eaec0; not seen live): no search record yet (+0xfc bit 0 clear) or another world: cell 5 "<World>", kind 6 (FUN_100db7b0 gives no string, so nothing is drawn *(inf)*
- L17216: Engine start (DAT_104df908, FUN_100f1f00): every +0xfc bit 1 cleared, folders 7 / 8 / 6 counted (unread / read / sent *(inf)*
- L17223: The handle-id text (slot 0x1db) is unknown too; titlehan draws only while polcore's mode byte *DAT_10456a28 == 0x60 *(inf: signed on)*
- L17230: Our status at login: s2c 0x00A (LAB_100facf0, LoginState @0x80 in {1, 2, 0, 6}) sets Away back to Online with a 0x0DC clear *(inf: only the first 0x00A)*

## rem4line / rem4li2 and spoolmsg / splmsg2: the log's page overflow and Hold (2026-10-28)

`MODLOG.md:17238`

- L17252: Ours runs the test per new row of the frame (a frame's rows come together), and shows the current page *(inf: retail draws from its list head +0x58, stepped 4 rows by the message handler;

## s2c 0x075: the battlefield countdown, objective bars and text (2026-10-28)

`MODLOG.md:17308`

- L17340: A packet without the help flag (LSB's bare objectiveUtility()) leaves it up; only a help of 0 / 0 clears. Ours drops it at the zone load *(inf: retail's helpwind teardown FUN_1013a450 clears it)*

## s2c 0x075: the border (2026-10-28)

`MODLOG.md:17351`

- L17376: A removal stops the looping lp00 with the actor *(inf: retail frees the part's file with it)*

## s2c 0x039 map schedulers, 0x02F chocobo digging, 0x04F equipment clear (2026-10-28)

`MODLOG.md:17382`

- L17405: … rider's riding file (38603.., ride.rs FILES). Ours: ride::on_dig / dig_update / digging (World.digs = +0x12c bits 4 / 5 by id; movement held); FUN_100d65b0 taken as passed *(inf)*

## c2s 0x017: asking again for a partly known entity (2026-10-28)

`MODLOG.md:17445`

- L17457: Ours: client::charreq before every Entity update, World.index_masks (cleared at the zone load *(inf)*

## Chocobo racing (2026-10-28)

`MODLOG.md:17467`

- L17516: and odds are not reset: the last board shows until the first 0x073 *(inf: zero on the session's first open)*
- L17530: FUN_100f9940 (session start) zeroes it *(inf: so per zone)*
- L17551: keytops3 sprites, named from the art only *(inf)*
- L17562: Event 210 polls update option 5 up to 30 times (Z9 -4 one more try, -5 / -6 stop); LSB answers only after the race has run *(inf: else the update never returns)*

## Camera paths framed on actors (2026-10-28)

`MODLOG.md:17587`

- L17602: … factored out of attach_matrix), event::camera_frame after the effects tick (the frame is one frame behind the actors *(ours)*), the blend scaled to retail's frame as 1/30 s *(inf)*

## MonStat idles: dft / df1, ark, hsr (2026-10-28)

`MODLOG.md:17604`

- L17620: A race model (Elvaan F) has no dft / df1 / ark (npc-shot: bind pose), so the motions belong to particular NPC models *(inf: not found yet)*

## Training regime window (an Extra, 2026-10-28)

`MODLOG.md:17635`

- L17661: Id ranges in the book's bytecode (West Ronfaure's actor 0x010642dd, Gustav Tunnel's 0x010d4157): 1..146 FoV, 147..559 hunts, 560..601 unused by LSB *(inf: elite training)*
- L17686: A Hunt Registry (event 1500) end with option 4 cancels: 0x02A REGIME_CANCELED "Current training regime canceled." *(inf for retail)*
- L17688: LSB keeps the state in char vars only ([regime]type / zone / id / repeat / lastReward / needed1..4 / killed1..4), untouched by zoning, logout or a job change; retail cancels on a job change *(inf, from the R + 1 text)*

## Remote actors keep their level (2026-10-28)

`MODLOG.md:17725`

- L17749: Knockback and op 0x7a pins: the slide goes on from the pushed / pinned spot, as retail's slide starts from the actor's position; with no slide time left the actor goes to the goal at once *(inf: LSB sends no position after a push)*

## Moblin Maze Mongers: the Maze Tabula editor (2026-10-28)

`MODLOG.md:17805`

- L17834: Pulse 0x60..0xa0 by 8, glint 0x30..0x7f by 2, stepped 30 times a second *(inf)*
- L17896: Bits 164..170 (d5 >> 4 & 0x7f) are the R iteminfo's "(%d)" *(inferred: server-set)*
- L17901: Keys: message 7 = key function 0x18 (Num-, or - with Ctrl / Alt; a middle-button release *(inferred)*
- L17911: c2s 0x0D8 (FUN_100ec3f0, 0x28 bytes): @4 our ActIndex, @6 the tabula id, @8 the slot, @9..@0xb unwritten *(inferred: 0)*
- L17915: The event end FUN_100ae3f0 (0x100ae931) calls FUN_101fcfb0 whether or not the window is open; the event start's broadcast 0x800 closes prvdungp without saving *(inferred: not re-traced)*
- L17916: Retail's handler is operator_new'd and FUN_101fd000 sets no +0x14..+0x17 *(inferred)*

## Generator clones go with their last element (2026-10-28)

`MODLOG.md:17937`

- L17958: The end-of-frame sweep kills a marked clone still without elements after the generators' emissions *(inf: retail's order of generators and elements within a frame)*

## Ship actors: 0x00E SubKind 4, the XiModelActor (2026-10-29)

`MODLOG.md:17976`

- L18000: A drive made this frame places its actor without running its time *(inf: retail's first update is the frame after)*
- L18056: It first kills every CMoPathDriveActorTask on that actor (FUN_1005e310 / FUN_1005e0e0) and needs the Vtx resolved (FUN_1004e3b0) and actor vt +0x178 *(inferred: "can be driven")*
- L18061: … prev, h = sqrt(dx^2 + dz^2), yaw = wrap(yaw0 - atan2(dz, dx)), pitch = atan2(dy, h) (with face bit 1 set an uninitialised local in retail; no file uses it; ours keeps the pitch *(inferred)*
- L18065: … sin(u pi/2), 2 1 - cos(u pi/2), 3 u <= 0.5 ? 0.5 sin(u pi) : 1 - 0.5 sin(u pi), 4 (1 - cos(u pi)) / 2, 5 without a KeyFrame u. Forward = (cos yaw, 0, -sin yaw); the bow is +x *(inferred)*
- L18089: Draw: FUN_100c4c20 -> FUN_101844c0(record +0x5c8, +0x770): the record zeroed each frame, only the model (+0x5e0), the matrix (+0x608) and +0x69c set; lit as area 0 *(inferred)*
- L18090: No distance test *(inferred: nothing writes +0x8c)*
- L18096: … `sgcm` (late in an arrival), `sglp` (a 1200-tick loop under way); SeSeps 8209..8211 (boats), 8161..8163 (the Jeuno airship), 2195 (overhead); horn or engine not identified *(inferred from the names)*
- L18112: … furniture table with entity +0x130 bit 21, else the NPC table; kind 6 loads NPC model +0x10e & 0xfff and, with a race (+0xef), goes on into the PC race path; kind 7 is no case *(not traced)*

## Actor culling: every +0xb2 reason hides the model and the plate (2026-10-29)

`MODLOG.md:18300`

- L18322: Exempt (bit cleared, +0x8c = +0xbc = 0.1): our actor (vt+0x304) while DAT_10487f80 is set *(inferred: first person)*
- L18326: … == 0 and +0x8c >= 0 (else it sets +0xb0 / +0xb2 bit 0); the queued callback re-checks both; FUN_100cbde0 marks the actor culled and returns when FUN_1002b5a0 yields nothing *(inferred: no skeleton)*

## Actor ground shadows (FUN_10032f80; 2026-10-29)

`MODLOG.md:18364`

- L18376: Not ported: the "nothing drawn on the first frame" rule (+0x728 < 2, inferred) and chairs (they have no actor here).
- L18404: Others: +0x728 (primitives of its last mesh draw) < 2 -> none *(inf: its first drawn frame;
- L18408: +0x66c is 0x80 when the model colour's alpha (+0x663) is non-zero, else 0 (ctor default 0x80) *(inf, not traced: whether a 'spop' actor's alpha-0 start colour is already set then)*
- L18422: TEXTUREFACTOR 0x80FFFFFF, stage 0 SELECTARG2 TFACTOR (cutouts fill solid), lighting, fog and z off; alpha test GREATER 0x60 passes; blending off *(inf)*
- L18428: *(Not traced: whether the overlay passes between the meshes and the shadow leave other stage states.)*

## Config tool 0043 "Screenshot Size" (2026-10-29)

`MODLOG.md:18448`

- L18465: Mode 0: capture at 512 x 382 (FUN_101c7620 -> FUN_101c7680 -> FUN_10016eb0: device +0x194, the world target, resampled through D3DX *(inf)*
- L18466: … the world target, resampled through D3DX *(inf)*), JPEG quality 90 (FUN_101c6ab0(w, h, px, 0x5a)), saved through polcore in folder id 0x21 (FUN_101b7220; created if missing *(inf: the PlayOnline screenshot folder)*

## Config tool 0019 "2-D Mapping" (2026-10-29)

`MODLOG.md:18487`

- L18494: … = 0 (render::MAP_COMPRESSED, set at start and by Settings) a category-10 image goes through texpresso BC2 and back (ClusterFit), the lossy surface retail's D3DX load gives *(inf: D3DX's encoder differs in detail;

## Config tool 0040 "Graphics Stabilization" (2026-10-29)

`MODLOG.md:18508`

- L18527: Whether drivers applied WRAP0 to generated coordinates varied *(inf: the reason for the setting)*

## Config tool 0023 "Simple Character Creation" (2026-10-29)

`MODLOG.md:18538`

- L18552: Not done: the forced-on cases (app state 4 after a lobby return whose dancer had drawn *(inf)*
- L18553: Not done: the forced-on cases (app state 4 after a lobby return whose dancer had drawn *(inf)*, the fallback device) and the screen tint reset on leaving *(inf)*
- L18562: Forced on (not done): app state 4 (FUN_10010d20) sets 0023 = 1 when DAT_10444038 is set (on a lobby return in a session whose dancer had already drawn, DAT_10623e90 *(inf)*
- L18571: Leaving (not done): cancel restarts the title music, confirm plays music 0x78 (both creators); the simple creator frees the picture (FUN_1022eed0) and clears DAT_10443fe4 +0x338 (FUN_10014230(0)) *(inf: the screen tint)*

## Config tool 0011 "Environment Animation" (2026-10-29)

`MODLOG.md:18575`

- L18589: Not modelled: undrawn instances keep advancing (retail advances an instance only when drawn *(inf: call site)*
- L18606: … sqal dust sand wind stom snow bliz thdr bolt aura ligt fogd dark); FUN_10160830 collects up to 32 per tag (the `mode/kfrm` fallback through FUN_100712e0 from the weather node *(inf)*

## Config tool 0030 "3D LCD Mode" (2026-10-29)

`MODLOG.md:18612`

- L18623: The UI draws once over it *(inf, as retail)*
- L18625: … on odd values the bottom 10 window rows get the 1 x 16 strip texture (row 0 black, rows 1..15 (0x48, 0, 0)): one black row, nine dark red, drawn opaque, under our UI *(inf: retail's blend and order for it)*
- L18642: FUN_1000afd0's strip is a pre-transformed quad over (0, H - 10, W, H), Z off *(inf: a signal for the display)*

## Config tool 0017 "Bump Mapping" (2026-10-29)

`MODLOG.md:18644`

- L18649: … + zone from 600) gives its Bp height maps, each made a normal map as FUN_1001d270 (wrapping neighbours; R from the rows above / below, G from the columns, B = 128 + 255 s) *(inf: the rows are taken as stored, no flip)*
- L18670: Enable: FUN_10171360 (cur FUN_10178250), run by FUN_10171c40 on the zone manager's first update (+0x34 == 0; once per process *(inf)*
- L18671: … manager's first update (+0x34 == 0; once per process *(inf)*), sets zone +0x39498 (bump) and +0x3949c (light mode) only with 0017 != 0 and VS / PS 1.1 (device +0x909 / +0x90a *(inf: the cap flags)*
- L18681: … next[x+1], s = 127 / sqrt(gx^2 + gy^2 + 255^2), texel R trunc(gy s) + 128, G trunc(gx s) + 128, B 128 + trunc(255 s), A 0; category 0, so stored DXT3 when the device can *(inf)*

## Config tool 0022 "Opening Movie" (2026-10-29)

`MODLOG.md:18690`

- L18703: Ours takes any key or mouse-button press *(inf: the three counters FUN_10007c70 reads)*
- L18707: … v = 1 at its top corners, so the picture is the bottom 336 rows, upright: 0.142 H..0.862 H, the full width (a 1.85:1 band at 4:3, stretched wider on wide windows, as retail) *(inf: the samples bottom-up, the decoder's usual RGB24)*
- L18709: Colour 0x80808080 taken as 1.0 *(inf)*
- L18724: The path is the working directory + "\mov" when DAT_103c75f8 +0x20 is 1 or 2 (the normal launch *(inf)*
- L18730: off is -110 for English (language 1, or German / French forced to English by +0x18 bit 7) and -160 for subtitled languages, where the picture spans 0.034 H..0.755 H *(inf: bottom-aligned rows)*
- L18732: Subtitles (FUN_10017c90, cur FUN_10018a50; at frame = movie time x 30, FUN_10017d90): language DAT_10444110 +8 (0 JP, 1 EN, 2 DE, 3 FR *(inf: ids)*

## Lamp Lights: sub-area instances and zones with no data light (2026-10-29)

`MODLOG.md:18764`

- L18772: It now takes its glow's colour (the element's readback colour, 0x80 = 1.0) at range 8 and intensity 20, the gate tunnel lights' op 0x49 values *(inf: a stand-in)*

## /sitchair: the chair under the sitter (2026-10-29)

`MODLOG.md:18808`

- L18849: A failed gate still builds an actor with a leftover kind and no type bits, deleted next frame *(inferred: it draws nothing)*
- L18861: FUN_100c8bb0 picks 'ci1 ' (no gate) unless DAT_104568fc +9 is set and actor +0x79c (vt +0x200) clear *(inferred: an event pose flag)*
- L18869: Else -> 0x19 (zoning, FUN_10087060 = DAT_10487e48 *(inferred)*
- L18880: … / 0x1a only mode 2 is sent. In 0x19 with the server still seated LAB_1008c376 re-runs "/sitchair off" every frame until the status arrives *(the send queue may merge them; not traced)*
- L18896: … FUN_100bd1a0): a bound PC-type entity (+0xee 0 / 2 / 6 / 7) gets event status 0 even seated; only +0xee 1 keeps 0x2f / chairs / 0x30 *(so a cutscene stands a seated PC up; not traced further)*
- L18898: *Inferred*: whether retail's other hides (+0xb2 bits 1 / 5) reach the chair (only bit 4 is copied);

## Event cameras: one-key paths in world space (2026-10-30; correction to "Camera paths framed on actors")

`MODLOG.md:18907`

- L18916: FUN_1003be10's default case (mode 0 and above 0x1b, jump table 0x1003c650 entry 0 -> 0x1003c5f0) calls the math object's vtable +0xc4 on the output: the identity *(inf: that slot's meaning)*

## Event areas: their own sub-areas (2026-10-30)

`MODLOG.md:18981`

- L18989: … area keys) and the zone's is kept in AreaSwitch.home_sub until op 0x35 brings the zone back; while an area shows only the event's requests load (no trigger under the player *(inf)*

## Escha - Zi'Tah: Affi and the Register of Deeds (2026-10-30; read for the LSB session)

`MODLOG.md:19047`

- L19059: … next two); chest 7632 / 7751 / 7681 and Domain Invasion 7644-7645 / 7763-7764 / 7693-7694 ("You obtain", colour 0x79). No "can't hold more silt" line exists in any of them *(inf: retail caps silently)*
- L19065: What starts 9702..9704 is not in the script: the server picks the csid *(inf: a trade)*
- L19075: *(inf: bit 3 = DI not open to this player, bit 2 = the DI introduction given.)*
- L19095: Key item shop (prices are in the script, not sent; option: key item, silt): 1 2894 tribulens (1000 while c <= 2, else 1000 + (1000 << c) / 20, c = update 3's num0 *(inf: c decays over time)*

## Wide Scan: a track the server refuses (2026-10-30)

`MODLOG.md:19234`

- L19241: *(inf: the first was out of range or gone when Track was clicked;

## Ru'Aun's sub-areas: textures found in the zone; variant stand-ins and the camera lists past Retail (2026-11-01)

`MODLOG.md:19444`

- L19456: Now ZoneGeometry::build_keyed takes the zone's texture names, and set_sub maps those to the zone's slots *(inf: retail's texture lookup is by name over everything loaded;
- L19457: Now ZoneGeometry::build_keyed takes the zone's texture names, and set_sub maps those to the zone's slots *(inf: retail's texture lookup is by name over everything loaded; not traced)*

## PlayOnline sign-in (net/pol) and FFXiMain's -net 3 lobby path (2026-11-01)

`MODLOG.md:19473`

- L19522: Open *(inf / not verified)*
- L19617: … [1] the gate for [2], [2] slot 0..7, [3] == 1 a flag (sent back inverted at ChangeMyStatus 0x13), [4] u16 activity, [6] presence + 1, [7] a flag, [8] / [0xc] two u32 *(names inferred)*
- L19619: … status 0x72 reads 0x18 bytes (-0x14c2). LoadHandleNameList 0/9 records (0x88): [0] content index, [1] & 0x3f ordinal, [2] & 3, [4..8] u32, [8..0x10] u64 kept to 44 bits *(inferred: the handle's ID)*
- L19648: Open: whether a server checks those stale bytes; the meaning of the status fields beyond content and character; the minimum before FFXiMain's (-2,-1,0,-2,-1,1) *(inferred: the Viewer's (content, -2, 1, 2, 0, -2))*
- L19658: … TCP link to it on -port (FUN_10102340; 0x47DC = 18396 without -port; FFXiMain's developer boot appends "-net 3 -port 54001", the value the PlayOnline launch is taken to pass *(inferred)*
- L19663: … map server (-ip / -worldip / -port, -accunt / -pass), 2 starts at state 4; -net 0 completes every lobby step at once (FUN_100da750); -POLCON sets lobby state +0xa4 (consumer not traced).
- L19703: Every frame FUN_100daf30 stores POLCON_POLL (0x2C1) at lobby state +0xb8; a negative value goes to DAT_10456900, and -0x420 calls FUN_100163e0(1): the PlayOnline connection must stay up *(inferred beyond the stored value)*
- L19720: Subs 0x12 / 0x13: 0xB5 starts StoreHandleNameList 0/8 but FFXiMain polls it with 0xB4 *(inferred effect: normally no traffic)*

## Map session details LSB does not check: header words, 0x00B states, 0x00E gates (2026-11-01)

`MODLOG.md:19728`

- L19781: Clock re-sync FUN_100fbd10 -> FUN_100dc1b0(header @8, header @4 + (+0x4190 >> 1)), at most 20 ms per tick; +0x4190 is the measured receive interval (inferred: a round-trip estimate).
- L19805: … while the name is empty); reserved actors (FFXiMain skips despawn, claim, 0x04 and the look, accepts a partial first update, skips the position while +0x13c bit 0 is set, inferred from op 0xac sub 4);

## Cutscene camera: the mode flag, the task's life, stops, sub-task crossfades (2026-11-01)

`MODLOG.md:20019`

- L20028: A live task shows either way (FUN_10014820 takes its requests, *(inf: the frame order)*

## A mode-1 position moves a bound actor's event body (Mhaura's ferry left-behind) (2026-11-01)

`MODLOG.md:20038`

- L20052: Retail's drive reads the same *(inf)*
- L20066: … FUN_100a57f0 reads it, and the pushes FUN_10092c20 / FUN_10092b20 are kind-gated (not 5); the call that puts the 0x00E pose on an undriven ship is not found *(inf)*

## Op 0x51: the level over the scene and the windows (2026-11-01)

`MODLOG.md:20092`

- L20098: Now Screen::level (from what is shown, as 0x0f), in Screen::multiply (scene and plates) and Screen::ui_level (the 2D layer's level, the mean of its RGB *(inf: as the lobby's modulate)*

## Motion scale and the root turn (2026-11-01)

`MODLOG.md:20117`

- L20140: Normals turn only (our shader normalises them) *(inf: retail's fixed function lighting may see a scaled normal)*

## Plate and arrow points from `init` (scheduler ops 0x9e / 0x9f / 0xad) (2026-11-01)

`MODLOG.md:20169`

- L20176: … (actor1 through its dolls), vtable +0xb0 / +0xb8 / +0xc0 with byte @8 = FUN_100a3150 / FUN_100a3170 / FUN_100a3190: actor +0x8a6 (arrow), +0x8a7 (plate), +0x8a8 (reader not traced).

## Airships: a server pose held until the path lets go (2026-11-02)

`MODLOG.md:20182`

- L20194: The pushes FUN_10092c20 / FUN_100929d0 / FUN_10092b20 skip kind 5. So the call that puts a server pose on a ship is still not found *(inf)*
- L20195: Ours *(inf)*

## Event actor flag ops 0x84 / 0x86 / 0x95 / 0x96 / 0xa4 (2026-11-02)

`docs/modlog/2026-11-02-event-actor-flags.md`

- L22: It shows weather effects even with the Weather setting (config 0x3b) off *(inf: the setter call at 0x101aa703 was not tied to 0x3b)*

## Event op 0xb3: the Fishing Ranking (2026-11-02)

`docs/modlog/2026-11-02-fishing-ranking.md`

- L29: Escape closes it *(inf: the generic window handler)*

## itemctrl / moneyctr: the mouse (2026-11-02)

`docs/modlog/2026-11-02-qty-gil-mouse.md`

- L12: / + step on the press and repeat while held, after 400 ms every 60 ms *(inf: the 0x1d rate is not traced)*


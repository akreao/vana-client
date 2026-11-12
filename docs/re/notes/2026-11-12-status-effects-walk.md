# Status effects: what the client does by effect id (2026-11-12)

<!-- dll-build: 30261102_0 -->

Every reader of our own effect list in `FFXiMain.dll` 30261102_0, and what ours does. The list is `DAT_10486be0`
(32 x u16, 0xffff empty; written by s2c 0x037 in FUN_1009d0f0 as `id & 0x3ff`, reset by FUN_10093720). Its readers
(xrefs to the array): FUN_1021fc80 (has effect), FUN_10101e50 (MP cost), FUN_101021a0 (job rule), FUN_10102430
(valid targets), the buff window FUN_10229200, the cancel FUN_100a0590. FUN_1021fc80 has 9 callers; all are below.
Party members' lists (s2c 0x076) are only drawn.

## The lead: Bind did not stop the player

Bind (11) is not read by the client at all. The server roots a player with Speed 0 (LSB `effects/bind.lua` sets the
base speed 0, so `UpdateSpeed` gives 0 and 0x037 flags1 bits 0..11 carry 0 into actor +0x15c). FUN_100a6240 takes
vt+0x1a0 as it is (x2 riding, at most 30, x1/60), and the move vector is the input times it. At 0 the vector is zero,
which is the no-input path: +0x598 = 0, no step, and FUN_100a8000's locked / strafe speeds scale the same zero
vector. With autorun on, the run's vector is first cut to the tick's move (zero) and the `|vector| <= 0.001` test
ends it. The function's tail still turns a free, unlocked third-person heading to the input by the camera.

Ours: `own_speed` took Speed 0 as "not set" and used 5.0. Fixed: 0 is 0, autorun (and /follow with it) ends, no
step in any mode, and the keys turn us in place.

Live (servertest, Partyb, `!addeffect bind`): free autorun covers 15.2 units in 3 s; bound, the fixed build stays at
the same position for 3 s of autorun; the build before the fix moved 5.3 units into a wall while bound.

## The walk

| Retail | Effects | Ours |
|---|---|---|
| Move input gate FUN_100964f0 (FUN_10096270, then two more) | K.O. 0, sleep 2, petrification 7, stun 10, terror 28; charm 14, 633 | Matches (`effect_held`; K.O. by `dead()`) |
| Speed 0 roots (FUN_100a6240) | none: Bind and the like arrive as Speed 0 | **Fixed here** |
| Log-out / shut-down prompts FUN_10161fc0 / FUN_10161f30 (main menu rows 0xb / 0xc, key functions 0x13 / 0xb7) open only when FUN_10096270 passes | 0, 2, 7, 10, 28 | **Built here** (`prompt_held`); they opened under any effect |
| Merit menu "menu merit2" refused, line 0xbb (win_persona_onkey_2) | 143, 257, 269 | Matches (merits.rs) |
| Cast bar not opened (FUN_1022c9e0) | Chainspell 48 | Matches (batch 7 item 29) |
| Recast at our cast's end set to 0 (FUN_1022c940) | Chainspell 48, Spontaneity 230 | **Built here** (`set_recast`); ours started the DAT recast when the header's was 0 |
| Magic check FUN_1022e540: the job rule FUN_101021a0 | Tabula Rasa 377 (+0x44 bit 3 on SCH), Addendum: White 401 / Black 402 / Enlightenment 416 (bit 2 on SCH, by school), Elemental Seal 79 (bit 4, unless main job 23) | **Built here** (`Actions::can_cast_now`); ours let SCH cast addendum and Tabula Rasa spells and anyone cast bit 4 spells |
| Magic check: MP against FUN_10101e50's cost | see the next row | **Built here** |
| MP cost FUN_10101e50 / FUN_10102050 (magic check, magic list "MP Cost" win_magic_v4c, ability check and list for cost kind 6) | Manafont 47, Manawell 229: 0. White: Light Arts 358 / Addendum: White 401 90 %, Penury 360 50 %, Accession 366 x2 on +0x44 bit 0; without the arts Tabula Rasa 377 90 %, else Dark Arts 359 / Addendum: Black 402 120 %. Black mirrored: Parsimony 361, Manifestation 367 on bit 1. Abilities (type 99): Apogee 583 150 %. Rounded up each step | **Built here** (`dat::actions::mp_cost`, classic and modern lists); was "not modelled" in MODLOG |
| Magic check: Unbridled spells 0x2e0..0x2ff | 485, 505 | Matches |
| Valid targets at the magic list's pick FUN_10102430 | Pianissimo 409: a self-only song takes the party; Entrust 584: a self-only geomancy spell without +0x44 bit 5 too | **Built here** (`dat::actions::spell_targets`) |
| Ability check FUN_10226180: code 0xc, line 7 0xbd | Saber Dance 410 on the row opening list 0xc, Fan Dance 411 on list 0xb | **Built here** |
| Ability check: TP cost (kind 3) skipped | Trance 376 | **Built here** |
| Ability check: MP cost (kind 6) skipped | Manafont 47 | **Built here** |
| Area display for a spell, FUN_1022cef0 (registered by FUN_1022ce60 at the pick; config 0xc0) | Pianissimo 409 (songs with +0x48 2 / 3: none), Convergence 355 (blue), Divine Seal 78 (with FUN_10224970(0x445); spell 0x8f or its family: radius 8), Diffusion 356 (blue, radius 8), Accession 366 (+0x4c bit 23), Manifestation 367 (bit 24), Theurgic Focus 519 (bit 29: radius - 4, at least 1), Elemental Seal 79 (Meteor 0xda), Majesty 621 (spells 1..6, 43..47), Widened Compass 508 (geomancy: +0x53 x3), Entrust 584 (geomancy without +0x44 bit 5: the +0x48 value becomes 6) | **Not built**: ours has no area display at all (only the config row) |
| Area display for an ability, FUN_10224ac0 (FUN_102249f0) | Contradance 582 (ability 0x2c2: radius 8) | **Not built**, same feature |
| Buff window FUN_10229200 | skips 0xffff and 0 | Matches (MODLOG "Status icons") |
| Cancel FUN_100a0590 (c2s 0x0F1, then the id leaves the list) | any | Matches (batch 11) |

Not effect ids, noted in passing: `can_cast` is called with 0 job points everywhere, so the job-point rule's last
branch never passes (not checked whether we hold the points anywhere). Retail also writes our Speed from a 0x00D
with our own index (FUN_10099ae0); ours takes it from 0x00A and 0x037 only.

Not live-checked: everything but Bind (the jobs and effects need a levelled SCH / DNC / SMN / BRD / GEO).

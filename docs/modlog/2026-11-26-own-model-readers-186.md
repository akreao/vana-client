## Our own model under an unload mark: what reads it (2026-11-26)

<!-- dll-build: 30261102_0 -->

Modlog-gaps item 186 (split from 179). Decompiled with Ghidra; every address below checked in Rizin.

Retail:
- FUN_10092900 (the model free FUN_10095900 calls on an unload mark) deletes the model actor, zeroes +0xa0 and clears
  +0x120 bit 9 together, and FUN_1008f740 sets both again the frame the mark is gone. Our own actor is in the frame's
  lists like any other (FUN_1008e9a0 links every slot into DAT_104871d0), so its mark (our hide or sleep bit with no
  event that binds us, or an event's Mode & 0x02 on a player it does not bind) frees our model too. What reads our
  +0xa0 or our bit 9 then finds none. Readers found by our slot (FUN_101cdac0, `(&DAT_10481ca0)[DAT_1048712a]`):
  - FUN_10096640 0x10096676: the zone-in's camera placement (FUN_10021680 on our model, once a zone: DAT_1048909e)
    waits for bit 9.
  - FUN_1008abe0 state 1, 0x1008ad3c: the zone-in's sound reset and fade in (`fdi0` of file 0x78b8) wait for bit 9.
    The state is entered only when the 0x00A carries no event (FUN_10093db0).
  - FUN_100882d0 0x100882e2: c2s 0x01A action 0x14 goes out only with bit 9, tried each frame.
  - FUN_10087230 0x1008723c: "New Challenge" needs bit 9. Its file table is 0x1035c53c in this build.
  - FUN_1007bf30 0x1007bfe3: a talk returns 0 with our bit 9 clear. The rest of the function, in its order: a target
    with no UniqueNo high byte gets system message 9 (0x1007bfbe -> LAB_1007c0ec) and the call returns 1; an index
    past 0x6ff needs +0x12c bit 30; the target's bit 9 (0x1007bfff); a target model with +0xb2 set (hidden) gets
    message 9 (0x1007c01a); our +0x128 bit 6 gets message 0xac (0x1007c028) and returns 1; past 6.0 (double at
    0x1032df50) message 9 and 1; a target in status 2 / 3 returns 0; then the packet, and control locks
    (DAT_104840c8) unless the target has +0x128 bit 5 (0x1007c16d). That bit is written only by the 0x00E handler
    FUN_1009b550 in its mask 0x04 part: Flags1 u32 @0x20 bit 20, or always when @0x28 bit 0 (a trust) is set. The
    caller FUN_1015a380 plays its sound on any return of 1.
  - FUN_1009f430 0x1009f489: s2c 0x030 about us calls FUN_100a6070(0) on our model (autorun and /follow end) only
    with bit 9.
  - FUN_1009fd30 0x1009fe62 (s2c 0x115): `main` of file 0x389 on our model only when +0xa0 is set.
  - FUN_100cd040 0x100cd231: the fade of the others near us while we fish needs our model.
  - Event ops. 0x46 (FUN_100b6440 0x100b6454): with our bit 9 clear no sub runs and the op is passed as 2 bytes
    (0x100b6664), though sub 2 is 4 bytes and sub 3 is 8, so the script goes on inside the operands. 0x7d
    (FUN_100b8b20 0x100b8b35): no scheduler on us, the op is passed. 0xb6 sub 0x10 (FUN_100b1150 0x100b19ef): waits
    for our bit 9.
  - The event's end. FUN_100ae960 -> FUN_100ae9b0(DAT_1035dfa0) puts first person back only with bit 9;
    FUN_100ae3f0 0x100ae776 puts the camera behind us (Mode & 1) only when +0xa0 is set.
  - FUN_100ad550: a debug line (`EventInfo CSTAT ...`), nothing to build.
- Five of the functions item 179 listed read our pet's model, not ours (FUN_101cdae0 is the actor at our +0x1fa,
  FUN_101cdb20 the one at +0x2a0, the adventuring fellow):
  - FUN_10076ff0, the text tags (table 0x10358860): type 0x12 `<pethpp>` (0x1007712a) and 0x13 `<petmpp>`
    (0x10077172) print "%d%%" of the pet's +0xec and of FUN_100875f0 (the pet status packet's MP%, 0 when the
    packet is about another index) only while the pet has a model; 0x14 `<pettp>` (0x100771b7) prints "%d" of
    FUN_10087610 with no test at all; 0x1b `<ft>` is the fellow's name and 0x1c `<fthpp>` (0x100776cf) its HP%
    while it has a model.
  - FUN_1007a1c0 0x1007a30c: `<scan>` as a target needs the tracked actor's model.
  - FUN_10151260 / FUN_10151310 (win_subwindo modes 5 / 6 / 7) and FUN_10203f70 (the party list's pet row): the
    pet's HP bar only with a model.
  - FUN_100adf90 compares an effect's actors with the pet's model; a freed model is no effect's actor.
  - FUN_10220640 (`<ht>`) needs each candidate's model.

Ours kept our model entry under the mark and tested it nowhere. Now:
- `World::my_model_up`: our entry has a model (or failed to load) and no unload mark.
- The zone-in: the fade in and the sound reset wait for it when no event runs or waits, until the reset has run once
  (`sound_reset`); the 0x14 action waits for it; the camera goes behind us again the frame it is up when it was not at
  the settle (`zone_cam_owed`).
- `roe::new_challenge`, `fishing::on_fish`'s keen sense effect and the fishing fade take it; `synth::on_effect` ends
  autorun for us with it (ours ended nothing there before).
- `event::talk_with` follows FUN_1007bf30's order: message 9 for a player character or a hidden target, the calls
  that print a message succeed (so the confirm's sound plays), nothing while our model is gone, and no control lock
  for a target with `Marks::talk_free` (new, from the 0x00E).
- Events: `LiveHost::modelled` is false for a freed model (ours by its mark, the others' in `World::unloaded`), the
  start's walk waits for ours (`has_model`), op 0x46 is passed as 2 bytes without our model, 0x7d starts nothing,
  0xb6 0x10 waits, and the end's first person and camera resets need it.
- Text tags `<pethpp>`, `<petmpp>`, `<pettp>`, `<fthpp>` and, in the classic interface, `<ft>` as the fellow's name
  (the modern interface keeps its own `<ft>`, the focus target). `<scan>` as a target needs a model.
- The pet row already drew its bar only with a model (`petrow::row`).

Still not built (queue rows 205..210):
- 205: FUN_1008f200 (0x1008f21a; s2c 0x05B modes 0 / 1 / 5 / 7 about us): with no model the camera's eye (+0x44) and look (+0x50)
  both go to our place (FUN_1001e780 then FUN_1001e760) in place of FUN_10021680; ours still faces from behind.
- 206: FUN_100aed00 at the event request (0x100aee44): with bit 9 DAT_1035dfa0 takes the first person flag
  (DAT_10489130) and FUN_100a6070(0) ends autorun and /follow; without it DAT_1035dfa0 is 0 and autorun is kept.
  Ours saves first person at the event's begin and ends autorun by its in-event rule.
- 207: the rest of FUN_1008abe0: the zone line's request (DAT_10481b3e 1, 0x1008ad0f) saves the camera's place in
  our model's frame in DAT_1035bf40 with bit 9 (FUN_100215c0), which the next zone's placement uses; state 4
  (DAT_104840f5) and state 1's model test (vtable +0x19c).
- 208: tags `<gatebreach>` (0x19), `<petra>` (0x1a), `<recast=` / `<rc=` (0x1f); `<bt>` (FUN_10077d80) by model and
  status; the modern `<ft>` over retail's.
- 209: win_subwindo modes 5 / 6 / 7 against ours.
- 210: FUN_1007bf30's other callers: /returnfaith (FUN_1007bd80, its fourth argument 1 skips the target's model
  and range tests), FUN_10087480 (fifth argument 1: no messages), FUN_10135bb0 (parameter 2).

Unit test: `follow_camera_op_needs_our_model`. 1337 tests pass. Not checked live.

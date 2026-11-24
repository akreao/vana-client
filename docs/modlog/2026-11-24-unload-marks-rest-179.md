## Unload marks, the rest: the use count, the PvP mark and our flags at the zone-in (2026-11-24)

<!-- dll-build: 30261102_0 -->

Modlog-gaps item 179 (split from 166 / 178). Decompiled with Ghidra, every address and byte search below checked in
Rizin.

- **The use count (+0xf2) is never raised in this build.** FUN_10095900 spares a marked model in use
  (FUN_100961d0: +0xf2 above 0), as does FUN_10095850. Writers of +0xf2: FUN_1008a350 (0 at the actor's init),
  FUN_100833b0 (+1) and FUN_100833c0 (-1, or a log line at 0); nothing else (readers: FUN_100961d0, FUN_10095850,
  FUN_100833f0, the Mog House door FUN_101d5ff0). FUN_100833b0 / FUN_100833c0 have no direct caller: they are the
  +0x170 / +0x174 entries of the 15 model actor vtables (0x1032dfe0 ...). The doll actor class (vtable 0x10330938,
  built by FUN_100abcb0 / FUN_100abea0 / FUN_100abf80) overrides the same two slots with FUN_100abdf0 /
  FUN_100abe10, which jump to the actor it shows (FUN_1003b6d0) at its +0x170 / +0x174. No code calls either slot:
  the byte search for `call` / `jmp [reg + 0x170]` and `[reg + 0x174]` finds only those two jumps (0x100abdff,
  0x100abe1f), and none of the 171 `mov reg, [reg + 0x170 / 0x174]` loads is followed by a `call reg`. So the count
  stays 0 and the exemption never applies; ours has none and needs none.
- **The PvP mark (FUN_1008e4d0 0x1008e582..0x1008e5cc).** For an actor that has not taken part (+0x120 bit 8 clear),
  while our own +0x130 bit 11 is set and our team byte (+0x1ff) is 2..6 or 0x28..0x2b, every player character
  (+0x1d0 bit 0, FUN_1008e290) below GM level 3 (+0x128 bits 11..13 < 3) whose team byte is none of those gets the
  unload mark, ahead of the event-reserved test, so a reservation does not keep it. Our +0x130 bit 11 is 0x037 Flags3
  u32 @0x38 bit 5 (FUN_1009d0f0 0x1009d534); no other packet writes it, and the new entity of a zone-in starts with
  it clear (FUN_1008a350). Ours had no such mark: now `SelfPlate::pvp_only` and `event::pvp_unloads` in
  `HideRule::gone`, so those players' models go and come back like any unload mark's.
- **Our flags at the zone-in.** Ours kept the last 0x037's u32 @0x28 in `World::self_flags` across the zone change,
  where retail's new entity takes the 0x00A's flags (FUN_10094190) and the team byte 0 (0x10094e12). Its readers,
  traced to what retail reads:
  - the bump hold off at GM level 3 (FUN_100a8770 -> FUN_10084600) and acoption / charlnk's Mute
    (win_acoption_v04 FUN_1020e2a0, FUN_10139dc0): our entity's +0x128 bits 11..13, now `my_plate.marks.gm`;
  - /prcmd add's refusal 0xc6 (FUN_1007d9c0): our entity's +0x124 bits 23 / 22 (anonymous / away), now
    `my_plate.marks`. Retail tests them before it resolves or looks up the target (asm 0x1007da8b..0x1007dab7);
    ours tested them after, so an unresolved name was looked up while /anon or /away. Now first.
  - /anon and /invite with no argument (FUN_1007d7e0 / FUN_1007d6a0 -> FUN_100e4590) and conf3win's Show / Hide
    (win_conf3win_v04 -> FUN_100e4830(0)): not our flags at all but the config block, character data +0x2cfd8
    bits 2 / 0 (`World::conf_sys[0]`). The 0x00A sets its first four bytes from u32 @0xF4 (FUN_100fae40
    0x100fb72d), s2c 0x0B4 all twelve; ours read 0x037 and took the 0x00A's bytes nowhere. Now the zone-in copies
    them and Show / Hide follows the block.
  - the mods' `game.player` away / anonymous / seeking (ours): `my_plate.marks`.
  `self_flags` and 0x037's `flags` field are gone; `my_team` is 0 from the zone-in until 0x037.
- Still not built (queue row 186): what reads our own model (+0xa0) while the mark has freed it. Ours keeps our
  model, so these see it: the text codes 0x12 / 0x13 (FUN_10076ff0), the camera's model point (FUN_1008f200,
  FUN_10096640, FUN_1008abe0, FUN_100b6440: the position when there is none), FUN_10151260 / FUN_10151310 /
  FUN_10203f70 (race and id, skipped without a model), FUN_100ad550 / FUN_100b8b20 (+0x120 bit 9 and a model),
  FUN_100cd040 (the hide pass's distance to our model), FUN_100adf90, FUN_100ae3f0, FUN_1009fd30, FUN_1007a1c0 and
  FUN_10220640.
- Unit test: `pvp_mark_takes_outsiders`. 1244 tests pass. Not checked live.

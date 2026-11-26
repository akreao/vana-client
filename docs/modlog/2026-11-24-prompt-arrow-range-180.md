## Prompt arrow range (2026-11-24, queue row 180; 30261102_0)

The prompt arrow's colour (and the party-list cursor hand's) comes from a record in the target object, +0x6c / +0x70,
that FUN_10158d50(bytes, count) fills: up to two 4-byte checks, both zeroed first. Byte 0 is who measures (0 none,
1 us, 2 our pet: FUN_101cdaf0, our +0x1fa, us without one), byte 1 the range in yalms, byte 2 whether height counts,
byte 3 whether both model radii (+0x208) come off. Closing the prompt (FUN_10159b60) and opening the party cursor
(FUN_10204390) clear it; text commands' dry runs (DAT_1047e648, FUN_10081440) set it after the prompt or cursor
opens.

Setters, traced:

- FUN_1022ce60 (magic list, /magic): [1, table 0x103654a8[spell +0x45], 0, 1].
- FUN_102249f0 (ability list via FUN_102257b0; /ja and /pet FUN_1007b900, /bstpet FUN_1007b9d0, /ms FUN_1007bad0),
  count 2: [1, table[ability +0x10], 0, 1], [0, 0, 0, 1]. A record of kind (+2) 6 or 10 looks up ability 0x25b /
  0x2ac (the rows opening sub-lists 6 / 10): the first check takes that row's range, the second becomes [2, the
  move's own range, 0, 1].
- FUN_10224d70 (/weaponskill FUN_1007b820): [1, table[ability +0x10], 0, 1].
- FUN_10158e20 (the menu's Switch Target, command 0x12 of FUN_1012cc80; /attack FUN_1007a7a0): table 0x10336388
  [1, 30, 1, 1].
- FUN_10140a30(1) (the menu's Ranged Attack FUN_10140c80; /shoot FUN_1007a730): [1, 25, 1, 0].
- FUN_10141c50(1) (Trade, FUN_10141270): [1, 5, 0, 1].

The reader is FUN_10151b70 -> FUN_10151be0 (arrow: FUN_10151770, base 0xe04040d0; hand: FUN_102038c0, base
DAT_10386374 = 0xa04040d0 while the cursor is up). Per check: range - 0.5 (0 when the range is 0.5 or less) and
range + 0.5; from the measuring actor to the pick; with the ground-position cursor up (+0x22, FUN_1015b250) the
farther of the pick and the cursor, in 3D, radii 0; otherwise height dropped unless byte 2. Red at once when the
distance less the radii is past range + 0.5 or the distance is 25 or more; yellow past range - 0.5; else the base.

Ours had one range (spells only, from us, flat, radii always off the arrow and never off the hand, the 25 test in
3D, range - 0.5 unclamped). Now `target::Reaches` holds both checks and `target::reach_tint` is FUN_10151be0 for
the arrow and the hand; the ability list and /ja, /pet, /bstpet, /ms, /weaponskill, /attack, /shoot dry runs and
the menu's Switch Target, Ranged Attack and Trade set theirs. /bstpet and /ms also register the area display
callback (FUN_102249f0 -> FUN_10224ac0), which row 170 left out. The DAT ability record's +0x10 range is read
(`dat::actions::Ability::range`); the modern registry shows it for abilities too. Not checked on screen or live.

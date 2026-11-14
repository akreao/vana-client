## Events of the second event file with number 0: Aht Urhgan Whitegate's cutscenes (2026-11-13)

<!-- dll-build: 30261102_0 -->

Symptom: NPCs in Aht Urhgan Whitegate do not play their cutscenes. Seen live on the test server with
`!cs 3000 0 0 0 0 0 0 0 0 0` (Treasures of Aht Urhgan mission 1, Naja Salaheem): the event bound its actors, ran
nothing, and was cancelled after 10 s.

- The event start packets carry two zone numbers (s2c 0x034 EventNum @0x2a and EventNum2 @0x30; 0x032 @0x0a and
  @0x10; the 0x00A login event EventNum @0x62 and EventNo @0x40). FUN_100aed00(EventNum, EventPara, EventNum2,
  our zone) picks the event and dialogue files (0x100aedee..0x100aee24): our zone when it is 1000..0x513, else
  EventNum when the two numbers are equal, else EventNum2 + 1000. There is no test for 0 (`cmp si, ax; je` is the
  only comparison).
- Ours (`event::file_zone`) also took EventNum when EventNum2 was 0. LSB's `text_table = 0` events send EventNum 50
  with EventNum2 0, which in retail is file 1000 (event DAT 57881, 339 blocks). Zone 50's own file (5870) holds
  only one-byte `end` stubs for those ids in the NPC blocks and nothing for the player, so nothing ran. The test
  for 0 is removed.
- LSB uses `text_table = 0` for 35 scripts: Treasures of Aht Urhgan missions, Aht Urhgan quests, Rhapsodies
  missions, the Assault globals, hidden quests and Hugo (Whitegate's map marker NPC). All were broken the same way.
- Checked live after the fix: event 3000 plays (cameras dn00..dn16, Naja Salaheem's lines).

**The whole game, and every start parameter (same day).** The fix is not limited to one NPC:
everything LSB can send with an event start was checked.

- Second event files: of the 300 possible (event DATs 57881..58180, file zones 1000..1299) only file 1000 has
  scripts: 339 blocks, all of zone 50 actors. File 1001 is one empty player block and the rest do not exist. So
  Aht Urhgan Whitegate is the only zone with a second file, 0 is the only meaningful second number, and 203 event
  ids have their script only there (73..75, 150..216, 269, 582..584, 823..978, 3000..3217 and 5000..5093 in part).
- LSB's 490 Whitegate event calls checked against the two files (`re-notes/whitegate-notes`): 488 name the right
  file. Two do not, and are LSB script bugs (branch `claude/whitegate-text-tables` on akreao/lsb): event 3114
  (Seal of the Serpent) is only in file 1000 and is sent without the text table; event 923 (Embers of His Past) is
  only in the zone's file and is sent with it. The old test for 0 hid the second one.
- Mode (s2c 0x032 @0x0e, 0x034 @0x2e, 0x033 @0x0e) is stored as `Mode | 0x2000` by all three handlers
  (FUN_100a0200 / FUN_100a02d0 / FUN_100a03c0); the 0x00A login event keeps its Mode. Ours kept the packet's value,
  so a 0x052 mode 0 during a packet event took the control lock off and allowed cancelling. FUN_100a2050 case 0
  does nothing at all while (DAT_10481aac | Mode) & 0x2000. `net::event_start` sets the bit and `event::release`
  follows the gate.
- Mode bits read by the DLL (every reader of DAT_10480cf0, always or-ed with op 0x38's DAT_10481aac), against ours:
  0x01, 0x02, 0x04, 0x10, 0x80 (login only), 0x200 and EventPara2 bit 0 (FUN_100870e0) were built. Built now:
  0x40 drops s2c 0x036 / 0x02A / 0x027 / 0x043 (FUN_100a0ef0 / FUN_100a12a0 / FUN_100a1dc0 / FUN_100a1070);
  0x800 drops s2c 0x038 / 0x039 / 0x03A through the whole event and 0x20 drops them when either actor is bound
  (FUN_1009dee0 / FUN_1009dd20 / FUN_1009d9d0); 0x400 drops an s2c 0x028 whose actor or first target is bound, and
  our own finishing actions (categories 3..6, 0xb, 0xd..0xf) are dropped while an event runs unless 0x1000
  (FUN_1009e400). 0x08 and 0x4000 have no reader.
- Not built: 0x100 (FUN_100aeea0: without it the start waits until each bound actor is idle and its motion is
  free; ours never waits, which is the 0x100 behaviour for every event) and 0x8000 (FUN_100ae3f0 puts the actors
  back on the floor at the end).
- Params: 0x034 copies num[8] to DAT_104899b0 and 0x033 copies Data[8] only with Mode & 8. 0x032 and FUN_100aed00
  leave them alone, so retail keeps the previous event's numbers; ours zeroes them. Kept on purpose: LSB sends
  0x032 whenever a script passes no parameters, where a retail script would expect zeros.
- Seen in LSB, no client change: 0x032 goes out with Mode 0 where 0x033 / 0x034 default to 8 (no reader of 8),
  and a text table without parameters still picks 0x034, so EventNum2 is always sent when it matters.
- Checked live: `!cs 893` (0x032) and `!cs 3000 ... 0` both run to their end.

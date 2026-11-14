## Events of the second event file with number 0: Aht Urhgan Whitegate's cutscenes (2026-11-13)

<!-- dll-build: 30261102_0 -->

Steven's report: NPCs in Aht Urhgan Whitegate do not play their cutscenes. Seen live on the test server with
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

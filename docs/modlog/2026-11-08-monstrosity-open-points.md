## Monstrosity: the open points (2026-11-08)

The "Unclear" points of "Monstrosity, part 1" and "part 10", read in FFXiMain 30261006:
- A reused party record keeps the old Monstrosity word. The record picker FUN_100ea140 (from 0x0C8 LAB_100e9ee0 at
  0x100ea010 and 0x0E2 LAB_100ea570 at 0x100ea5b3) takes the first record whose id (+0x14) is 0, else the one with
  the oldest +0x1c, and clears only +0x38..+0x4b. A record freed by a 0x0C8 is zeroed whole (rep stosd 0x1c dwords,
  0x100ea11f), so the stale case needs all 20 records taken (0x0E2 creates without a 0x0C8 between). Then +0x4c (the
  word), +0x4e / +0x4f (the prefixes) and the short name +0x50 stay the old member's until the new member's 0x0DF
  (0x100e9dce..0x100e9dde, the only writer); 0x0E2 and 0x0C8 never write them. The 0x0C8 path also clears only
  MemberNumber (+1) and the name's first byte (+2); HP / MP / TP (+0x20..+0x2d) and +0x1a stay. Ours wiped the whole
  record: party.rs `alloc` now keeps it and `table` clears the number and name.
- Job 23 has no special case: /check's job line (FUN_10133a80) and the search comment read group 0x12 by the job
  byte, so MON shows group 0x12's entry 23 like any job; the classic party rows (FUN_10202530) draw no job. Item
  info's job list (FUN_1013ae30) walks bits 1..23 (FUN_10152f00: job 0x16's name +4). Nothing to build.
- Species record words +0x2c / +0x2e / +0x30 are never read. The parser FUN_1015cdb0 points +0x14f4 / +0x1540 /
  +0x1544 at them; their only readers are the item cache copy FUN_10159ff0 (cache +0x1c / +0x42 / +0x44) and, for
  +0x14f4, the automaton attachment lists (FUN_102086d0 / FUN_102087c0 / FUN_102088b0, other record types). No caller
  of the cache getters FUN_1015bc40 / FUN_1015bbd0 reads +0x42 / +0x44, and +0x1c only in the item-use holder
  FUN_1021d2d0 and the automaton list. The species list cell (FUN_101f4bb0 case 0x2a) reads only the family (+0x153c)
  for the level column.
- mp_invnt's +0x5c tag (7 from main menu child 2's species list at 0x10142445, 9 from the instinct list at
  0x10142469) is written by menu_mp_invnt_open only; the class (vtable 0x10336048, object DAT_10578280) and every
  load of DAT_10578280 never read it.
- mp_dead (also listed as unclear): its handler FUN_101419d0 only sets +0x24 (pick: child 1 -> 1, else 2; message 6
  sets both log windows' +0x59, as playermo's). The K.O. state machine FUN_10096c90 in Monstrosity waits in sub 1 for
  +0x24 1 / 2, sends c2s 0x01A 0x0B once (sub 2, StatusId 2 - (+0x24 != 1)), counts 600 ticks (sub 3), and back in
  sub 0 opens mp_dead again while the deadline is ahead (its open hook FUN_10141880 clears +0x24; registry flags
  0x04010002, so the cursor on Retry). Only past the deadline does sub 0 go straight to sending the last answer again.
  Ours sent the pick every 10 s; now ko_step opens mp_dead again 10 s after a pick instead. Whether the window stays
  drawn between the pick and the reopen is not settled (nothing in the handler closes it; ours hides it).

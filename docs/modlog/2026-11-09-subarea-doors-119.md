## Sub-areas: door boxes during a load (2026-11-09)

Modlog-gaps item 119 (MODLOG "Sub-areas" and "Doors": door collision during a sub-area load, FUN_10176dd0). Addresses
are Steven's FFXiMain.dll (FUN_10176dd0 kept its address).
- Door group entries (FUN_10173c10, stride 0xf4, list DAT_10621a08): +0x40 id ('_' first), +0xd4 / +0xd8 the box's
  collision objects (stride 0x1f0), +0xf0 a byte set to 1 at build with every box bit 0x2000 cleared (solid).
- FUN_1017ac40(id, open?): always stores +0xf0 = its argument, and flips the boxes' bit 0x2000 (set = skipped by the
  walks) to +0xf0 == 0 only while FUN_10178780 (zone renderer +0x44, a load running) is false.
- FUN_10176dd0(0) clears bit 0x2000 on every '_' entry's boxes (all solid); FUN_10176dd0(1) sets it where +0xf0 == 0.
  Only FUN_10178a20 calls (0), when a trigger's load starts (+0x44 = 1), and only FUN_101787c0 calls (1), when that
  load ends. FUN_10178a20 is called from the player's trigger (FUN_10014390 -> FUN_10181540) and with 0 by
  FUN_1018ba80 (0x065 mode 7: an unload). The asked loads, FUN_10178900 through FUN_1018bac0 (0x00A SubMapNumber) and
  FUN_1018baa0 (event op 0x75 sub 0), are synchronous: no +0x44, no door hold.
- Ours (was: every load closed the current ground's doors, door changes during it hit the boxes at once, and the end
  re-opened doors by their state, so a quiet event-op 0xab state or an 0x2d swing could open a box retail kept solid):
  Doors::solid is +0xf0 (set_solid always records it); SubArea::doors_held is +0x44 for a trigger's load only
  (subarea::Step::Load's third field says asked), during which set_solid leaves the boxes; the load closes the doors
  of the current and the kept zone ground; doors::reapply (end of load, failure, unload, any new ground) sets every
  box from its flag.

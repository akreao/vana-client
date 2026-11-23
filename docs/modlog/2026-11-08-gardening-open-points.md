## Mog House gardening: the open points (2026-11-08)

The five "Open" points of "Mog House gardening", read in FFXiMain 30261006:
- plnt windows stack. Plant Seeds / Feed Crystal (0x10206cac / 0x10206dff) open bankmenu and leave the plnt window
  open; the prompt answers (FUN_102070c0 seeds, FUN_102071e0 crystals, FUN_102069f0 / FUN_10206930 dry / empty)
  only bring the pot list back (FUN_101f23a0 + FUN_101f7080), so after No the plnt window stays drawn under the list.
  The pot list's Enter (0x102075fb) opens the stage's window with FUN_1015e350(name, 1, 0) and closes no other: an
  open one runs its open hook again and comes to the top; a different one opens over it and the walk suspends the
  old one (all six are group 4). A plnt window's Exit / Escape, Harvest, Dry and Empty Pot close only that window
  (FUN_1015e910 of its own name), and the one under it comes back with its cursor row, acting on whatever pot is
  current now (the list's cursor pot). plnt1..6 all close only on the pot list's Escape (0x10207557), the crystal
  list's Escape (0x10207285) and a stage change of the current pot. Ours: Garden.under under the top window.
- The item list's Escape sound: inventor's open hook (0x101f68b0, vt 0x1033a5d8 +4) clears window +0x85 / +0x86 (no
  generic move / Enter sound) and keeps +0x84 = 1 from FUN_101172c0, so FUN_10118260 plays SE 0003 on Escape before
  the source's handler (FUN_101f2bb0 -> source vt +0x34). Ours already played it.
- Container views. Every container object keeps four views; FUN_1021c4c0 -> FUN_1021c380 syncs them when the item
  change counter (session +0x198c8) moves, slot 0..80 in order: FUN_1021c210 drops a slot whose item id differs from
  the cached one (new or replaced; a count change keeps it) from every view (FUN_1021c680) and appends it at the end
  (FUN_1021c5b0); an emptied slot leaves. FUN_1021c7e0(mask, view, 0x12, 0) rebuilds a view from slots 1..80 in slot
  order (only views 1 and 2 are then qsorted, LAB_1021d220). View 3 is rebuilt by the pot source's open hook for its
  own Safe (0x102073f0: DAT_10666e6c for container 9, else DAT_10666e30), by the seed and crystal sources' open hooks
  for the Mog Safe only (0x101d1be0 / 0x101d1c70, even on Safe 2), and by layout mode's furniture list (FUN_101f4130,
  either Safe). So a Safe 2 seed or crystal list shows Safe 2's view 3 as last rebuilt, with the slots changed since
  appended at the end. Ours: Inventory.view3 (rebuild_view3, viewed3) for the garden and layout lists; the replaced-id
  rule now also applies to Inventory.order (it only appended newly filled slots).
- File 0xbf8 (ROM/12/9) 'main', the moogle's trip: sound 4037; generators go02 / go01 / go00 / un00 / un01 / gu01 /
  gu00 / sy00 / kr00 (smoke, cloud and sparkle meshes un, sy, kr, gu, go, md); op 0x2a at tick 75 drives the moogle's
  colour to 0x00808080 (alpha 0) over 65 ticks; op 3 'mdam' (no such scheduler in the file). A vanishing puff; the
  model is then hidden at t < 0 (FUN_100a17f0(1)). Ours plays it as is (0x2a writes the same alpha the return ramps).
- /garden typed while gardening: bankmenu, inventor, plnt1..6 and comyn all have registry flag 0x20000 (inventor
  0x30002, bankmenu 0x20002, plnt 0x21002, comyn 0x31002), so FUN_10081230's flag 0x10 gate refuses it (window index
  3) before FUN_101d3380 is reached; the mode 3 -> 2 camera quirk cannot happen while a window is up. Ours refuses
  through main_open, and the garden check covers the frames with no window. Not a deviation.
- FUN_101d5fe0 is "no floors": session +0x40db0 bit 0x1000 is FUN_100f9650 (floors) at zone-in (FUN_10094190), and
  FUN_101d5fe0 returns 1 when it is clear. The furnished paths test it == 0 (ours: has_floors), as built.

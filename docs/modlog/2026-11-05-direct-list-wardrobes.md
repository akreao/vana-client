## Action menu Items lists usable gear equipped from the Mog Wardrobes (2026-11-05)

- Retail's direct-use list (action menu > Items) rebuilds in FUN_101f1230: the temporary items, the inventory, then
  each Mog Wardrobe in turn (FUN_101effd0 = 8 bags, DAT_103820b0 = containers 8, 10..16), every wardrobe row passed
  through the source's +0x48 (FUN_101f0e30) and +0x50 (FUN_101f0d20) filter: crystals, Maze Tabulae, non-gear with
  record flag 0x400, and gear that is equipped (FUN_10131f10) with charges. The main Items list's source has
  +0x40 = FUN_1012e4e0 (returns 0), so it stays inventory only.
- We only listed the inventory, so enchanted gear kept in a wardrobe (Chariot Band, warp rings, experience rings)
  could not be used from the menu. `items::listed_at` now appends the wardrobe rows for the direct list; the ready /
  not-ready colour reads the row's own bag's extdata, and Enter sends c2s 0x037 with that bag (LSB accepts
  wardrobes 1..8). The Modern UI list and its right-click Use take the same rows.

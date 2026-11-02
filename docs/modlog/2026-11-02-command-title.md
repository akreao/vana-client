## Long target names in title rows (ours, 2026-11-02)

- Retail's action menu (playermo, draw at 0x1012b3c0 in the 2026-10-05 FFXiMain build) draws only its label sprites:
  no title and no target name. The name is the target window's (targetwi draw 0x101501a0), which breaks a long one with
  FUN_101c9fa0: one line when it fits and has no ':'; else after the first ':' before the fit, else after the last
  ' ', '-', '@', ''' or ':' from fit + 1 down to fit / 4, else at the fit; the rest one line down, cut with an ellipsis
  (FUN_101c9d50) when it does not fit.
- The modern Commands / Battle window shows the target's name as its title row's note. The note was drawn
  right-aligned with no limit, so a long name ran under "Commands" (in-game report). Now (modern/mod.rs
  `title_row_in`, `name_lines`) the note keeps to the room the title leaves and breaks in two as FUN_101c9fa0 does,
  by measured width in chars and with the search from the fit (not fit + 1); the row grows to hold the two lines. A
  title too long for its row (the target panel's name) is cut with an ellipsis and leaves the note at least 40% of
  the row. Every panel's title row gets this (trade partner, equip set names, gil notes).
- `modern-shot --titles`: the Commands window on long names, one with a ':', and a target with a long name.

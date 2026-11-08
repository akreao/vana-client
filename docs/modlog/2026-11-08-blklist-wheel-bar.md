## Block list: the wheel and the scroll bar (2026-11-08)

- blklist now takes the wheel and its bar (`menu    scroll` at (374, 48), 248 tall, drawn while the count is over 15)
  through scrollbar.rs (Owner::BlockList), as the list base FUN_101f6c70 does: 0x16 / 0x17 move the top one row with
  the hand on its screen row (so the selection moves with the rows), the track pages as Left / Right (0x18 / 0x19),
  the thumb drag sets the top (0x1b), SE 0001 when it moved. Only while blklist has the focus (not under blkmain).
- The window groups listed as not done in "The block list window" were already covered: blklist and blkmain draw
  through GameUi's gate, so the group walk (GameUi::want / run_walk, "Window group walk") suspends gaugewin and the
  other 0x1c windows under blklist, and the 0x81 pages under blkmain, as for every other window.
- "Fewer words open blklist" (the blacklist entry) was done in the window's entry: /blacklist with under 3 words.
- Not done: the window manager's SE 0004 on a left release over the window off any row (generic, not per list).

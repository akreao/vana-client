## itemctrl / moneyctr: the mouse (2026-11-02)

- Retail (OnKey FUN_1013a6c0 itemctrl, FUN_10140bc0 moneyctr; message table bytes 0x1013a83c / 0x10140d58, click
  tables 0x1013a85c / 0x10140d78): message 5 (a click) on a child re-sends the window's own message through vtable
  +0x18: child 1 the confirm (FUN_1013a8d0 / FUN_10140df0 with the value), 2 message 4, 3 message 3, 4 message 1,
  5 message 2. Messages 1 / 2 are one step less / more (itemctrl down to 1, moneyctr to 0, both up to the max);
  3 / 4 are itemctrl's 1 / all and moneyctr's step / 10 (at 1: the value 0) and step x 10 (when that would pass the
  max: the value max). Message 0x1d (held) re-sends 1 for child 4 and 2 for child 5; message 6 cancels (-1).
- Ours (winhits::ctr): the same map onto the keys our handlers already take (Enter, Left = 4, Right = 3, Down = 1,
  Up = 2), so every module's rules hold: the Delivery Box, shops, synthesis, the auction house, hand-over and player
  trade, and the storage view's count. - / + step on the press and repeat while held, after 400 ms every 60 ms
  *(inf: the 0x1d rate is not traced)*; a right click on the window is Escape. trade.rs and the storage view now
  record the windows they draw (winhits::record) so the mouse hits them where they are.
- Not tried live (no game data in the build container).

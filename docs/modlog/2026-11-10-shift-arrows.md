## Shift + arrows with a window open: the camera only (2026-11-10)

Issue lenonk/vana-client#174.

- The window cursor ids 0x2b..0x2e are bound to the arrows with flag 0x20, no Shift (DAT_103619f0: 0x20c8 / 0x20d0 /
  0x20cd / 0x20cb), so Shift + arrows never move a window's cursor; with a window active they only drive the camera
  axes (FUN_1011f7b0 / FUN_1011f910, ours Input::axis).
- Ours read egui key events in the windows, not the ids: the world key dispatch now drops Shift + arrows after the
  chat line's keys (the chat line keeps them for its selection). Both UIs.
- Which windows read which ids (2019 FFXiMain, the FUN_1011ff30 calls by id): the macro palette (FUN_101a39a0) reads
  0x2b..0x2e, and layout mode (FUN_101c47d0 / FUN_101cb3a0, context 0x42) 0x31..0x34 (flag 0x80, Shift allowed; ours
  reads those as key functions, outside the dispatch). Two windows read the unflagged 0..3 (Up, Down, Right, Left), so
  Shift steps them too: "menu    itemctrl"'s quantity (FUN_101356c0, vtable 0x103288b0: Up / Down held, x10 past a
  while) and link11's colour channels (FUN_101dd4c0, vtable 0x1032eb10: Left / Right 0..15). Ours lets Shift + arrows
  through to those: the storage view's, a shop's, the Delivery Box's and synthesis's itemctrl when it has the keys,
  and a linkshell colour channel being edited. The generic windows get the cursor as window messages; which ids feed
  those was not traced id by id (the inventory's, the issue's case, matches 0x2b..0x2e).
- The modern right-click menu and list search keep Shift + Up / Down: they count as typing (text_field_open), so the
  camera's arrows are off there (Input::axis) and Shift + arrows only move their selection.
- Seen live (Lnxcsfix, classic): with the Items window open, Shift + Down twice left the cursor on the first row and
  Down moved it. A test drive cannot hold a key for the camera; the camera half was already right.

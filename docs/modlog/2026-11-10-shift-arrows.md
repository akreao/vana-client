## Shift + arrows with a window open: the camera only (2026-11-10)

Issue lenonk/vana-client#174.

- The window cursor ids 0x2b..0x2e are bound to the arrows with flag 0x20, no Shift (DAT_103619f0: 0x20c8 / 0x20d0 /
  0x20cd / 0x20cb), so Shift + arrows never move a window's cursor; with a window active they only drive the camera
  axes (FUN_1011f7b0 / FUN_1011f910, ours Input::axis).
- Ours read egui key events in the windows, not the ids: the world key dispatch now drops Shift + arrows after the
  chat line's keys (the chat line keeps them for its selection). Both UIs.
- The modern right-click menu and list search keep Shift + Up / Down: they count as typing (text_field_open), so the
  camera's arrows are off there (Input::axis) and Shift + arrows only move their selection.
- Seen live (Lnxcsfix, classic): with the Items window open, Shift + Down twice left the cursor on the first row and
  Down moved it. A test drive cannot hold a key for the camera; the camera half was already right.

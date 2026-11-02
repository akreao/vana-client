## Modern storage: drag and drop, split, refusals explained (ours, 2026-11-02)

- Not retail (retail's storage view is keys and itemctrl only): modern/storage.rs. A row dragged onto the other pane
  moves its stack (c2s 0x029 as Enter does); the pane under the pointer is outlined with the pack's
  controls/drag_outline_drop_valid (teal) or _invalid (red) by the same rules as the row colours. Shift on the drop,
  Shift-click or Shift+Enter opens a quantity strip on half the stack (retail's itemctrl opens on 1): - / + and Up /
  Down by one, Left / Right by ten, Move / Enter, Cancel / Escape. Click and Ctrl-click send a stack to the paired
  pane, right click one.
- Refused rows say why, on hover and under the lists after a refused click, drop or key (items.rs judge: tint's rules
  with a reason each). Inventory rows in the bazaar (lock 0x19) are now refused in modern too, as retail's Enter does.
- Nothing moves before the server: a sent move shows as "Moving ..." until its slot changes; unchanged after 5 s the
  view says the server refused it (the reason is in the log).

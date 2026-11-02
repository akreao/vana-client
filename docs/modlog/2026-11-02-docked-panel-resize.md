## Item info: no one-frame jump between a long and a short item (2026-11-02)

- Ours (modern UI). A panel placed by its bottom (the item info on the action menu, anything docked on another
  panel, the bottom-placed windows, a panel risen to fit the screen) was placed by its height last frame. The frame
  its contents grew or shrank it stood that much too low or too high and then jumped back: moving the Items cursor
  from a long description to a short one, or back, flashed the item info at the wrong size and place for one frame.
- Panels::show now moves such a panel's drawn shapes by the difference its new height makes to its top, the same
  frame (egui's transform_layer_shapes on its layer), and docks what sits on it on the moved rect. The panels placed
  by their exact height (chat, party; the chat window fix of 2026-11-02) are unchanged.
- Test panel_tests::docked_panel_keeps_its_bottom_as_it_resizes switches a docked panel between 12, 2, 1 and 7 rows
  and checks its bottom stays on the panel under it every frame (it fails without the fix: bottom 473 on 667).

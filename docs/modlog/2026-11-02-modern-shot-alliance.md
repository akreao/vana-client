## modern-shot: the alliance windows, and the texture limit (ours, 2026-11-02)

- `modern-shot --alliance`: alliance1 / alliance2 with sample rows (mpf None, marks bit 1 the alliance leader's, an
  away member, a trust's row without TP) through hud::party_panel, as the live path does.
- The shot ran no frame before Modern::new loaded the atlases, so egui still assumed a 2048 texture side and the
  2184 x 1776 status atlas panicked. A first empty frame now carries the device's max_texture_dimension_2d (the live
  client passes it to egui_winit already); its texture deltas go with the rest.

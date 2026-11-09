## Settings: Background warns while real shadows are on (2026-11-04, ours)

- Not retail. While Settings > Effects > Character Shadows is Real (config `actor_shadows` 1), the Background row
  (Config 0003 / 0004) shows a warning icon after its heading, and its help line starts with "Real shadows
  affected. Performance will be affected." The real shadows' receive pass is full screen at the Background size,
  so a larger Background makes them cost more.
- The icon is the yes / no window's own warning sprite (menu ptc6yesn child 3, GameUi::warning_icon); the slider
  value's room shrinks by its width. The modern settings view appends U+26A0 to the label instead.
- Test: client::settings::background_warns_with_real_shadows.

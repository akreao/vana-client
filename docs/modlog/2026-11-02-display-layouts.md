## Modern layouts per display (ours, 2026-11-02)

- Config `modern_places` keys: the character's name (its latest layout) and "<name> @<w>x<h> <scale>%" per display
  (modern::display_key). Read: the display's, else the latest; a change writes both. Reset removes every key of the
  character (modern::layout_of).

- Layouts per display (2026-11-02, ours; plan B): config `modern_places` keeps a layout per character and display
  ("Name @2560x1440 60%": the window's pixels and the UI scale) beside the character's latest one (the plain name).
  A display not arranged yet starts from the latest; a change saves both, so a window dragged through sizes stores
  nothing until something is moved there. Panel Places' Reset clears every display's. No other panel resizes (the
  user's call, kept 2026-11-02).
- The user's call on resizing (2026-10-26: chat window only, no lock button) was kept on 2026-11-02, when the
  resizable spell / ability list was dropped.

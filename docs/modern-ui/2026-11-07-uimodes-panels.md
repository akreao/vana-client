- `/uimodes` covers every persistent HUD frame (2026-11-07, ours): Wide Scan, Focus target and Favourites joined the
  panels a mode may hide (they were drawn with `Panels::vis` already, so only modes.rs's list was missing them). Still
  never hidden: the chat window and the command menu (both needed in every mode) and the transient pending chip and
  event / objective bars, as before.
- A mode-hidden Wide Scan stops asking the server for lists and its name filter takes no keys (an invisible filter
  would keep them from the game); while it fades out it is still drawn and takes them, as when folded.

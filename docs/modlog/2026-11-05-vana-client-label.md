## Modern UI says "Vana Client" instead of "ours" (2026-11-05)

- Text shown to players no longer says "ours" for things the retail client doesn't have, since players who
  don't work on the client can't tell what it means. The Settings window's detail panel now reads
  "Retail: No (Vana Client)" (`modern/config.rs`, settings rows and sub-pages), and the GM window's footer starts
  "Vana Client only: not in the retail client." (`modern/gmwin.rs`). Code comments, docs and the Lua API's
  "ours" (meaning the player's own character) are unchanged.

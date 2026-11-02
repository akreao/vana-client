- Favourites (2026-11-02, ours; the plan's "favourites" and "a small optional favourites strip"; client/favs.rs,
  modern/favs.rs): spells, abilities (not traits) and items are starred by the star at their row's right end in the
  magic and ability lists, the Items window and the search's hits, or with Ctrl+F on the cursor's row or the
  highlighted hit. Kept per character in config `modern_favourites` (the search's keys, "m:" / "a:" / "i:"), 24 at
  most. In the list search a favourite ranks above every other match and says "favourite". The favourites strip
  (panel "favourites", movable, over the chat window's right end by default) shows a tile per favourite: its icon, an
  item's count, greyed as its list greys it, its name as the tooltip. A click runs it as a macro line would: /ma,
  /ja, /ws or /pet, /item, with <me> when only we are a valid target and <st> (the target prompt) otherwise. /favs
  [on | off] shows or hides it (config `modern_fav_strip`); it hides while nothing is starred. `modern-shot --favs`.
  Not yet: macros in favourites (the open question "How macros and equipment sets show in search and favourites" stays open for upstream).

- Right-click menus (2026-11-02, ours; the pack's "right click a context menu"): one look for every menu
  (ctxmenu.rs popup, the chat window's Copy / Paste menus now use it too): an optional heading (the name), rows lit
  gold on hover, grey when they can't be picked now, closed by a pick, a press outside or Escape. An Items window row:
  Use, Drop, Sort, Move to... (the action menu's direct list: Use); they set the row's cursor and press iuse's or
  itmsort2's own Enter, so Drop and Sort still ask. Move to... lists the bags open away from the moogle that take the
  item by the storage view's colours (Satchel, Sack, Case, the wardrobes; gear only into wardrobes, nothing equipped
  or in the bazaar) and sends 0x029 with the stack. A party or alliance row: Target, Check, Tell (the chat line on
  "/tell name "), Focus Target (our own party's slots 1..5, as retail's party menu). An actor: a right click on the
  world that did not drag (client/mod.rs, World::actor_menu) opens its menu in place of retail's right click (which
  clears the target); on nothing retail's right click stays. A monster: Target, Attack, Check; a player: Target,
  Check, Tell, Invite to Party, Trade; an NPC or a door: Talk, Target, Trade. They run the text commands (/check,
  /attack, /pcmd add with the server id or name) and trade::start_on, so the refusals are retail's.
  `modern-shot --menus` draws three of them.

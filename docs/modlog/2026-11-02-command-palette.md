## Modern command palette (ours, 2026-11-02)

- Not retail: Ctrl+Space in the field (modern; alive, nothing open, no prompt, not typing, not in Monstrosity) opens
  the command palette (listsearch.rs, World.command_palette; drawn by modern/lists.rs). It is the list search over
  everything the action menu reaches: the magselec spells, the abiselec abilities (job abilities, weapon skills, pet
  commands), the direct-use items (one per item, the total as the note) and the commands Attack, Disengage, Switch
  Target, Check, Call for Help and Heal. An empty field shows the recent picks (they rank first as in the lists).
- A pick goes through the menus as a suggestion's does (listsearch::pick_root): a spell or ability opens its window
  with the cursor on it and presses Enter (checks and the target prompt as ever); an item opens the direct-use list
  and is used from there; a command runs combat::choose (/heal through textcmd). Backspace on the empty field or
  Escape closes it. Ctrl+Space has no retail use we know of (Space alone opens the chat line).

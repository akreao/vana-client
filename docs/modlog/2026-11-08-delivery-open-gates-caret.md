## Delivery Box: the open gates and the name field's caret (2026-11-08)

- The send and receive UIs (FUN_1012f100 / FUN_1014ce80, on s2c 0x04B DeliOpen / PostOpen Result 1) refuse in this
  order: a main-class window up (FUN_1015ecc0(0x20000); inventor carries it, so the other side's list counts), the
  target system's sub-target mode (+0x59) or a pending target prompt (+0xc4) -> refusal index 3 (group 7 0x45); the
  same side already open -> silent; healing (status 0x21 / 7) -> 0x51. Ours had a fixed window list that also counted
  the magic / ability lists (mgcmenu / abimenu have no 0x20000) and missed the target prompt; it now uses
  `World.main_open` and `target::sub_up`, plus the windows we hold as state (items, map, config, mogdoor) for the
  modern UI.
- /mailbox, /deliverybox and /sendpost (0x1007e490 / 0x1007e560) also stop silently while the window manager is busy
  (FUN_10160c80, +0xae of 0x10621838). It is set with the talk watch when the house is being left: the exit prompt's
  yes (comyn callback 0x101d5f90) and a mogdoor pick (FUN_101d6540); cleared by FUN_1015ef80. Ours: `leaving_room`.
  Not mapped: the third writer at 0x1024ce30 (a comyn flow, "%04d").
- The recipient field (stringdl, the shared text input): Left / Right (messages 4 / 3 -> FUN_101c6920(4 / 5)) move the
  caret one character, Home / End are the input's own keys (as the chat line's), typing and pasting go in at the
  caret, Backspace / Delete take the character before / at it; Up (message 1 -> FUN_1014f500), Enter and Escape end
  the editing with the text kept. The caret starts at the end. FUN_101c60e0 draws the '|' (the 7F FF marker) where the
  text after it starts. Ours had appends at the end only.
- Monstrosity: no delivery gate of its own. The text command gate's Monstrosity table (0x10357bb8) holds neither
  command, and ours already matches it entry for entry.

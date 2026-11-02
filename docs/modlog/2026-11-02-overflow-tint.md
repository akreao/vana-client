## rem4line / spoolmsg: the magenta plate is never drawn (2026-11-02)

- Closes "Not done: the frames' magenta tint (windowps 43, spec §8)" under rem4line / spoolmsg (2026-10-28), from the
  game data and the unpacked FFXiMain.dll (30261006_0), without the spec.
- The layouts: rem1line..rem4line, rem4li2, spoolmsg and splmsg2 are one element each (366 x 16 n + 6 at 16, 298;
  rem's anchor 0, spool's 2), whose window sprite is windowps 39 + n (40..43; rem4line, rem4li2, spoolmsg and splmsg2
  all 43). Each of those sprites is a single newtex cell in (128, 65, 128, 169): the magenta. Every other window
  sprite's first cell is the grey backing (128, 128, 128, 192) but polwindo's (97, 97, 97, 192) and a few invisible
  windows' (1, 1, 1, 0) (insay .. inlink, chdummy, targetlo).
- The window draw (0x10117b02..0x10117c0d): colour ftol(alpha x 128) << 24 | 0x808080, then, unless window +0x34
  (the registry flags) has 0x20, the style frame (0x10234ab0, that colour, nothing from the sprite) and the window
  sprite with its first cell skipped (0x1011fc80, last argument 1). With 0x20 the frame is skipped and the sprite is
  drawn whole.
- Registry (0x103728dc..): rem4line flags 0x12, spoolmsg 0x11 (logwindo 0x93): no 0x20, so these windows get the
  plain frame and their one cell, the magenta plate, is skipped. Retail shows no tint; ours (logwin.rs draw_over,
  gu.frame) already draws them that way, so nothing changes.

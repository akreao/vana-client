## Event VM audit fixes 2: the credits roll, ops 0x59 7 / 8, 0xac 0 / 4, 0x9c, 0x60, 0x74, 0x63, 0x9b, 0xaf, 0xb2 1, 0x2c / 0x2d / 0x51 (2026-11-15)

- Op 0x71 subs 0x50..0x55 are the ending credits roll (DAT_10621944, a 0x14-byte object made at UI init; ctor
  FUN_10163090), not a picture window. One user: zone 291, npc 0x01123304 event 9.
  - 0x50 val (FUN_10163130): the scroller (FUN_1019e800, the install-credits class) with the text of file 0x18892 +
    val (ROM/342/14, 1,754 CRLF lines) after 20 empty lines, and file 0x18891 (a `cred` folder with the font
    texture); drawn in the game font (+0x57 = 1, FUN_1011c080 style 0x44) scaled (W / 640, H / 480). `+` becomes the
    player's name (entity +0x7c, at most 31). FUN_1019e150 keeps letters, digits, `/ & . , - '` and space, `{` / `@` as
    spaces, maps 0xa9 to the (c) glyph and `] [ ^ } ~ | \ #` to the font's accented codes (`[` is e acute: Clement,
    Sebastien; `~` u umlaut: Kruger), and drops the rest without advancing (the file's one `!`).
  - 0x51 (FUN_101632a0) closes it; so do the event end (FUN_100ae960) and the zone teardown.
  - 0x52 val (FUN_1019ec00(val, 1)): starts once; ticks per virtual pixel = val x 60 / (lines x 14), so val is the
    roll's length in seconds (420 in the data).
  - 0x53 waits for the end (+0x54) or no roll. 0x54 v2..v8: the panel as screen percentages (x0 = W v2 % or 0 for v2
    <= 0, y0 = H v4 % or 0, x1 = W v6 % or W, y1 = H v8 % but H when v4 <= 0: retail tests v4 there); default x
    294..746, y 0..H. 0x55: the panel's ARGB (default 0xff000000).
  - FUN_1019ead0 each frame: the tick count rises by the frame's ticks (at least one), off = ticks / speed, each 14
    pixels the top line goes; past the last line it is done. Ours counts real 60 Hz ticks (the one-tick floor would run
    5x fast at a 300 fps limit).
  - FUN_101630e0, first in the UI pass: the panel (no blending, opaque), then up to 37 lines at x 310, y 14 i - off,
    colour 0x80808080, leading spaces centring. Ours: client/credits.rs, drawn after the name plates and under every
    window in both UIs (the modern UI's font at the vertical scale), and not dimmed by op 0x51's 2D level (UiBatch
    set_unleveled): the event fades the windows out with it before the roll *(inf: its own quads and colours
    ignore that level, or retail's roll would not show)*. Cinema mode (FUN_101177d0), which the event is in, trims
    only the window loop, so the roll is drawn there too. The modern UI's TrueType face takes the font's Latin row as
    Unicode (0x85 xx is Latin-1 xx + 0x21: 0x85C8 e acute, 0x85D5 o umlaut; 0x87C8 the (c)), seen in the roll's names.
- Op 0x59 subs 7 / 8 (FUN_1008a790 -> vt+0x22c FUN_10082370): model +0xa8 = val (the event actor's val@2, or actor@2's
  val@6). The getter vt+0x230 has two callers, the actor update (0x100a5cf8 -> FUN_1018b9a0) and the draw (0x100cb8cb
  -> FUN_1018b9e0 / FUN_1018ba20), both passing mode (+0xa8 == 0): set, the actor takes the scenery's light, colour and
  ambient instead of the actors'. Scheduler op 0xc1 writes the same field (actor 1 through its dolls, FUN_10062770).
  Users: sub 8 seven times in zone 72 event 0x7d, all resetting to 0; 0xc1 none. Ours: Actor.bg_lit, pose header bit
  0x10000, lit_actor's select *(inf: a rebuilt model keeps the flag; retail's new object starts at 0)*.
- Op 0xac sub 0: the event actor's server status +0x16c = val@2. Sub 4: actor@2's +0x13c |= val@6 & 3 (cleared only by
  the entity reset): bit 0 makes 0x00E skip its position (mask bit 0, 0x1009b9f3); at the event end (FUN_100ae3f0) an
  entity with +0x138 bit 31 (op 0xac 2) takes +0x16c = +0x170 = +0x174 (its event status) and is reseated only with
  bit 0. Bit 1 has no reader. The 4 sites (zone 280) write 0.
- Op 0x9c (FUN_100ba100): L = DAT_10456a28 +8 (0 ja, 1 en, 2 de, 3 fr); with +0x18 bit 7 (set by current retail's
  launcher) de / fr count as en; then 1 -> 2, 2 -> 3, 3 -> 4, else 1. 85 sites in 20 zones.
- Ops 0x60 0 / 1 (+0x124 bit 30) and 0x74 (bit 31): s2c 0x039 (FUN_1009dd20, 0x1009de81) and 0x03A (FUN_1009d9d0,
  0x1009daf7) drop a scheduler naming either actor with the bit; the event end clears both (0x100ae5ad).
- Op 0x63: waits while the event actor's +0x1bc is not 0, then +0x1c0 = val@1 and +0x1bc = 9; FUN_1008b460 then sets
  the emote motion set (FUN_100d1c30: model +0x8a0 = val below 0x40, kinds 0..2 / 6 / 7), loads the race's upper file
  + set and the skirt's (FUN_100d1c80, tables 0x1035d7fc / 0x1035d80e) and plays `emot`. Op 0x9b waits while +0x1bc is
  not 0.
- Op 0xaf subs 0 / 1: set@2 / @6 / @4 = ftol of the game camera's (DAT_104568fc +0x50) eye +0x44 / +0x48 / +0x4c
  (0) or target +0x50 / +0x54 / +0x58 (1), plain world units; other subs stall.
- Op 0xb2 sub 1: on once the Delivery Box send UI (DAT_10578198) exists and FUN_1012f100(0) allows it (no blocking
  window, not healing; post::can_open).
- Ops 0x2c / 0x2d / 0x51 (FUN_100b4c20 / 4f10 / 5000): only when both actors resolve with +0x120 bit 9, which means the
  model object exists, not that its files are in; ours tests it as ops 0x45 / 0x55 do. The MODLOG reason for leaving
  them (background model loading) was wrong.
- Op 0xab subs 0x14..0x18 set bits 4..5, 6, 7, 0 and 1 of DAT_1047d6a8[ActIndex] (also set by 0x00E @0x28 bits 19..20
  and @0x20 bits 17, 18, 11, 12). FUN_100c1190 copies them into the message's fields; the English expander
  FUN_100be760 (also German and French while +0x18 bit 7 is set) reads only bits 2 and 3 (our 0x00E name flags), and
  the rest feed only the German (FUN_100bf3e0) and French (FUN_100c0500) grammar. No event uses the subs, so they
  stay unread.
- Update op 0x46 (0x1004b6e8): the clone's frame is euler x the camera object's +0x294 (FUN_10015240), which
  FUN_10015120 sets to the rigid inverse (FUN_100283c0) of the LookAt view (FUN_10021c10 -> FUN_102d8019), with the
  translation cleared: the inverse view rotation. Ours took c10. Math slot +0x20 is a thunk to +0x24, so 0x33 and 0x46
  multiply the same way.
- Scheduler ops never present in any shipped scheduler (the scheduler audit's census): 0x06, 0x11..0x14, 0x1a, 0x1b /
  0x1c, 0x58, 0x82 / 0x83, 0x87, 0x9c / 0x9d, 0xae, 0xbb, 0xc2, and 0x81 in a root event scheduler. Nothing can run them.

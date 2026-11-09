## Region Info: Campaign's windows and the mouse on cnqframe / map cursor mode (2026-11-08)

- Campaign (cnqframe mode 2, FUN_101e86b0): with the campaign set (DAT_10663644, s2c 0x071 mode 2) one of three
  info windows opens, and allied (the legend, "%10d" allied notes at +0x5f, +0x16) beside it; without the set help slot
  2 says 9 6 and none opens (each draw closes itself, FUN_1015e790, when FUN_101e8880 finds no set).
  - campresu (FUN_101e9ec0): nothing under the cursor, a front (record +2 -2), Jeuno's (name 3) or an area nobody
    holds (entry +3 == 9). Set +4 (controlled areas) as four 5-bit counts "%d" right at the frame's right - 6, y +0x16
    + 0x10 k.
  - campsan (FUN_101ea0d0 open, FUN_101ea150 draw): names 0..2 -> army n = name + 1, 0x2a..0x2d -> name - 0x26 (4..7
    the beastmen); +0x30 = n, the army dword at set + 4 + 4n. The crest (FUN_101ea070's icon set from index 3:
    keytops3 113, 110, 111, 112, 432 by min(n, 4)) at (+4, +8); framesus 153 (nation) / 154 (beastmen) at (+0x20, +6);
    the name framesus 142..144 / 155..158 at (+0x20, +0x14); group 10 0x59 with bits 0..3 right at (right - 0x10,
    +0x40); gauges (FUN_101ea030: framesus 148 at (+1, +2) scaled by f) at (+0x1d, +0x5e) bits 18..24 % and (+0x1d,
    +0x7e) bits 25..31 %; the child frame framesus 146 last at (0, 0).
  - camparea (FUN_101ea2f0 open, FUN_101ea330 draw): any other area, +0x30 = record +7, the record at set + 0x24 + 12
    k. The holder's crest (bits 1..3, at most 4); "%d/%d" fortifications (bits 4..13) / max (dword 2 bits 0..9) right
    at (right - 8, +0x12), resources (bits 14..23) / max (dword 2 bits 10..19) at +0x2e; the four influence bytes 4..7
    x 0.4 % as gauges at +0x56 / +0x6e / +0x86 / +0x9e; framesus 147 last. On the map, at the marker under the cursor
    (FUN_101efb20 / FUN_101efc00): framesus 149 and framesus 150 scaled by (byte 3 >> 1) % (heroism) at (x - 0x10, y +
    0xb), 0x1e higher when the label sits below (flag 0x40, place 1 / 6 / 7; the language picks byte +0xe / +0xf).
  - Ours: region.rs Camp, draw_campaign, info_view's CampView (modern/regioninfo.rs; the labels baked into retail's
    frames are ours there: Controlled Areas by nation, Prosperity / Morale for the two army gauges *(inf)*,
    Fortifications, Resources, Heroism, influence by nation). LSB never sends 0x071 mode 2, so none of this can be seen
    on LSB; it is for retail servers.
- The mouse in cursor mode (FUN_101ef0a0, cnqframe and the map's marker placing / scan narrowing): when the cursor
  axes 0x9d / 0x9e are still (x speed, x frame under 2), the mouse object's +0x4e is set (the pointer shown) and the
  window's argument is true, d = pointer (DAT_104e1d4c +4 / +8) - cursor point (+0x1c / +0x1e). With the game's sprite
  cursor (config 0021 = 0, DAT_10456bf8 0, and DAT_10456a7c 0 or 3) d is times the zoom (+0x58) unless both axes are
  under 2, then times the frame factor (FUN_10014cf0 x 0.5); under 2 an axis is dropped. Then FUN_101edaf0(d) (negated
  when +0xa2 is 0; not for these windows). After it, in cursor mode with the argument, FUN_10124700 puts the pointer on
  the point (SetCursorPos only for the hardware cursor) and any move hides it (FUN_101252f0(0)).
- FUN_101edaf0 in cursor mode (flag 0x100 without 0x200 / 8, arg 4 false): per axis the point becomes trunc(point + d)
  when that is inside the box (+0x44 / +0x4c x, +0x48 / +0x50 y); else it steps by d / zoom rounded half away, at
  least one pixel toward d. Then (+0x8c set) a point past the box is put back and the excess (map pixels) is added to
  the view (+0x24 / +0x28), which is clamped. With flag 0x200 (the fast key 0x9a held, FUN_10123a70(0x26, 0x9a, 4)) or
  8 it takes the normal path instead: the view moves by d / zoom and the point stays.
- Ours: map.rs mouse_step / cursor_move (used by region.rs and map.rs's placing / narrowing; the fast key now scrolls
  the view there, as retail, instead of moving the cursor twice as fast). The sprite cursor counts motion from where
  the pointer was last put (retail puts it on the point every frame); ours asks the app for the warp only after a move,
  so motion under 2 pixels a frame adds up at 60 fps, and a failed warp (Wayland) still works. The hardware cursor
  follows the pointer. Not done: flag 4's FUN_101ed9e0(8.0) under 4.5 px, 0x7e back to the player in cnqframe,
  Japanese label places. Not seen live.

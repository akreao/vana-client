## MMB triangle lists by header byte 4 (2026-11-06)

- An MMB without the `MMB` id is a strip mesh only when header byte 4 is 1; byte 4 = 0 is a triangle list, as
  byte 4 = 2 (the animated, 48-byte-vertex meshes) already was. Read as a strip, Buburimu Peninsula's open sea
  (effect mesh `umi1` in file 218, byte 3 0x05, byte 4 0, the only byte-4-0 mesh there) joined unrelated vertices
  into long dark wedges and thin black seams over the shore water that flickered with the camera. Read as a list it
  draws as one clean sheet. *(inf: from the data; FUN_10168eb0's own test not read.)*

## Effect sort key: flag 0x8000000 wins over 0x400000 (2026-11-06)

- FUN_100440d0 (asm 100450cf..1004516a) works out the key at +0x12c as before (0x400000: bias; else bias - z, or
  (1 - bias) - z for screen elements with |bias| > 0.03), then flag 0x8000000 overwrites it with bias - z + 400
  whatever 0x400000 says. Ours added the 400 only without 0x400000, so Port San d'Oria's black underwater sheet
  `t_sa/effe/wate/down` (flags 0x08401000, colour 0x80000000, below the surface) drew after the water colour `col1`
  and blacked out the harbour wherever the floor lies below it. It now draws first, as retail.

## MMB triangle lists by header byte 4 (2026-11-06)

- An MMB without the `MMB` id is a strip mesh only when header byte 4 is 1; byte 4 = 0 is a triangle list, as
  byte 4 = 2 (the animated, 48-byte-vertex meshes) already was. Read as a strip, Buburimu Peninsula's open sea
  (effect mesh `umi1` in file 218, byte 3 0x05, byte 4 0, the only byte-4-0 mesh there) joined unrelated vertices
  into long dark wedges and thin black seams over the shore water that flickered with the camera. Read as a list it
  draws as one clean sheet. *(inf: from the data; FUN_10168eb0's own test not read.)*

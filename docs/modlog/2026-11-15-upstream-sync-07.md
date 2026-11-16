## Effect audit fixes 7: orientation bit 0, the MMB cull and zone 237 offset, the 0x15 gate (2026-11-15)

- Orientation modes 1 / 2 with op-1 flag bit 0 (FUN_10045190, asm 0x10045307..0x100457b1; 2,151 generators: D3m and
  D3a mostly). d = pos - last position; for attach modes 3 / 6 with a parent frame P, d = ((pos - old) x A) through P's
  rotation, then back through Pn^-1 (P's rows normalised, vt+0xa4; the non-bit-0 yaw / pitch use this d too). v =
  (0, 0, -1, 1) through, by attach mode: 3 / 6, (b50 x Pn)^-1; 0, 0xe, 0xf, 0x14, c10 (no inverse, translation
  kept); else (b90 x Prot)^-1 (TransformCoord, divided by w). The rows of O: X = |d|, Y = |d x v|, Z = (0, 0, 1) x X as
  coded (neither normalised nor orthogonal to Y), and L = L x O. Normalize (vt+0x9c, FUN_1006d630) has no zero guard:
  a still element gets a NaN matrix *(inferred: not drawn; ours hides it)*. The attach mode rides in GEN_BITS bits
  16..20. Seen: zone 46's `sk01` steam (`moku`) by the pipe.
- MMBs in effects (FUN_10040ba0 -> FUN_10040c20 -> FUN_10183760): the 8 corners of the MMB header box through the
  element's model-view and the projection (FUN_10182490, near 0.2) are outcoded by FUN_10182070 (x / y outside |w|,
  view depth under 0.2; no far bit); the draw is skipped when all corners share a bit. Then in zone 237 (the scene
  zone at the chunk's load, DAT_10631f24 +0x3c = 0xed) the chunks FUN_10176ed0 lists at 0x10376c78 (awan, brac, col1,
  sh-u, suim) carry node +0xe0 = (0, -0.8, 0, 0), added to the world matrix's last row: 0.8 down the view. Ours: the box
  centre and half extents in element words 49..53 (z from the radius), GEN_BITS bits 21 / 22, the test and the offset
  in vs_mesh; the zone is checked at emission *(retail decides at the chunk's load)*. Seen: Metalworks' water wheel,
  the channel's water sheet (suim) 0.8 lower and the blade dipping into it.
- Generator op 0x15 (0x100498bc, FUN_1004a870, FUN_1006ed10): the unit cube (word 1 is 0 in all 22,366 users) as
  diag(w3, w4, w5) at the op-1 position through the generator matrix, the view and the projection. Shown when a corner
  is strictly inside the viewport by x / |w|, y / |w| (|w| = 1 at w = 0), so a box larger than the view with every
  corner off it is hidden and there is no near test; word 6 bit 1 (86 generators) also needs every corner with w != 0
  at a depth in near..far (z / w in 0..1). Word 6 bits 0 and 9 are not read. The occlusion query (FUN_1006c280) runs
  only for an element's clone; no 0x15 generator has one (all bit-1 users are zone auto-starts). The "spline offset"
  +0xa0 is op 0x54's keyframe handle; no 0x15 generator keys it. MODLOG "Generator op 0x15" called bit 1 the corner
  choice: it is the depth test.
- D3a with op-1 flag 0x200000 (CMoD3aLightmapElem, ctor FUN_10052c30, draw FUN_1003ee70: the sprite into a 256 x 256
  target from above, then laid on the ground cells like the D3m ground projection): no D3a generator in the install
  sets the flag (0 of 335,702 op-1 records; the flag is on 1,452 D3m, the ground projection, and two D3b, whose factory
  ignores it), so there is nothing to draw.

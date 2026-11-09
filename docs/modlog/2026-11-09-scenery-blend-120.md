## Scenery bit 31 blending checked; '#' placements clamp their UV (2026-11-09)

Modlog-gaps item 120 ("Scenery bit 31: alpha blending in retail, ours alpha-tests", MODLOG L4365). Addresses are
Steven's FFXiMain.dll.
- Already closed by MODLOG "Scenery blending and the alpha cutoff" (L4488): checked again in FUN_1017c9c0 (0x1017dad1..
  0x1017db35). Per model, piece word +0x20 (MMB bit 31) != the cached zone +0x39114: on, ALPHABLENDENABLE (0x1b) 1,
  ZWRITEENABLE (0xe) 0, ZBIAS (0x2f) DAT_10631f18; off, 0 / 1 / DAT_10632038. FUN_101781d0 sets those to 8 and 0 (1
  and 0 when device +0x920, the ATI case). Ours matches (Draw.mode bit 1, blend pipelines without depth writes, bias
  -8). Nothing to build there.
- Correction to that entry: placement +0xd6 is set by FUN_1017a... at 0x1017b0a0 from the placement's name: 1 for '_',
  2 for '#', else 0. FUN_1017c9c0 (0x1017d9cf..0x1017da74): 0 -> ALPHATESTENABLE (0xf) 0 and stage 0 ADDRESSU (13) /
  ADDRESSV (14) WRAP (1); 1 -> alpha test 1, WRAP; 2 -> alpha test 0, CLAMP (3). So '#' does not alpha-test; it
  clamps the texture coordinates.
- Ours: a '#' mesh's vertices carry flag bit 2; vs_main marks VertexOut.area bit 14 (the slot mask is now 0x3fff);
  the scenery fragment paths sample stage 0 through uv0(), which clamps the UV to the edge texels' centres for it (the
  shared sampler repeats, so this keeps the filter off the opposite edge, as CLAMP). Census (light_ref_census pattern, zones 0..299 and 1000..1299): no placement in this
  install is named '#', so nothing changes on screen today; zone-shot 232 renders identical.

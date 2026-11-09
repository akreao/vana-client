## Specular meshes: the tint copy, the stages' MIPFILTER, stage 2 left set (2026-11-09)

Modlog-gaps item 125 ("Specular meshes: the cube's tint copy black on every backend; MIPFILTER inherited; the copy's
half-texel shift; stage 2 in the no-groups path; camera +0xd4 vs +0xb90; FUN_10075700's texture warp", MODLOG
L11734). Addresses are Steven's FFXiMain.dll.

- The tint copy FUN_100759c0 (RS 0x10354874: no blend, test, z or fog; TSS 0x103547d8: stage 0 MODULATE2X(TEXTURE,
  TFACTOR = C), alpha MODULATE4X, linear MAG / MIN, address and MIPFILTER not set; stage 1 off) draws four XYZRHW
  vertices (stride 0x1c) at (0, 0), (W, 0), (0, H), (W, H) with uv 0..1 into DAT_1047d158, one shared target of the
  image's size (FUN_100755a0 remakes it on a size change). Under D3D8's rules pixel i samples u = i / W: half a texel
  left, so linear filtering averages texels i - 1 and i (2 x 2), then 2 C t into 8 bits. The stage then reads that
  single-level target.
- The cube's copy: the same quad samples the cube with 2D coordinates (u, v, 0 or 1 by the driver), whose major axis
  is +X, +Y or +Z, never face -Z (the only one FUN_10039e50 fills): black on every backend. Settled by the data, no
  footage needed.
- MIPFILTER (every setter in the DLL: direct calls and state blocks): the frame runs the scenery task (priority 5,
  FUN_10177d00 / FUN_100749b0(5)), the actors (6) and the effects (0xc, FUN_10074cc0 at 0x1005560e). The scenery pass
  leaves stage 0 NONE (0x1017c5cc), stage 1 as the Mip Mapping setting (0000, 0x1017bac8 / 0x1017bb1d) and, with
  zone+0x3949c == 1, stage 2 the same (0x1017bb43); the actor draw sets stages 0 and 1 to the setting (0x1002bec3,
  0x1002bf29 / 0x1002bf40) and restores nothing. Inside the effects only CMoDist's copy (TSS 0x10353028, FUN_100426f0,
  CMoDist and VuDist with particles) sets stage 0 NONE. So stage 0 and the no-groups specular stage (stage 1) follow the setting
  until a CMoDist copy (then stage 0 NONE for the rest of the frame); the grouped specular stage (stage 2) mips only
  with the setting and bump mapping (config 0017 with shaders, zone 182 never).
- Stage 2: TSS 0x103529c8 (grouped) sets stage 2 MODULATEALPHA_ADDCOLOR(CURRENT, TEXTURE); the draw's cleanup resets
  only stage 0's ops, RESULTARG and the specular stage's TEXCOORDINDEX / TRANSFORMFLAGS, so stage 2 stays on with the
  specular texture bound (the image, or the shared tint copy) and reads uv set 0. Blocks that disable stage 2 in the
  effect draws: D3a (0x10351ec8 / 0x10351fa0), D3b and DistMorph through it (0x10352258), grouped plain D3m and
  DistModel's grouped mesh through FUN_10040670 (0x103524a8 / 0x103523d0 / 0x10352740, and 0x10352590 for 'I'),
  DamValue (0x10352d30); the scenery and actor passes disable it every frame. VuDist (FUN_10043110) only runs
  CMoDist's draw per particle (FUN_10063c80 -> 0x100430d0 -> FUN_100426f0); 0x10353260 sits in FUN_100431b0, which
  nothing references. A VuDist with no particles changes nothing. Two draws enable stage 1 without setting stage 2 and so add what is left there: the no-groups specular path
  (TSS 0x103528f0: rgb + a0 x texel) and the lens flare (FUN_1003ebf0 under FUN_100548d0, TSS 0x103520a8: rgb + a x
  texel). A cube left there adds black.
- The lens flare's own stages (TSS 0x103520a8): alpha MODULATE2X(diffuse, texture), then MODULATE2X(current,
  TFACTOR). Ours lacked the first doubling.
- Camera +0xd4 vs +0xb90: FUN_10015510 sets VIEW from the camera (0x10015569), stores it (FUN_10015120: +0x54, +0x94,
  +0xd4 with the translation zeroed, +0x294 the inverse) and calls FUN_1006b270, which reads the device VIEW back into
  +0xb50 and +0xb90 (translation zeroed). The same rotation in the effects pass. Closed.
- FUN_10075700's warp: dead (nothing writes elem +0x140..+0x148; the earlier entry). Closed.
- Ours: the update pass writes C / 255 to outs 41..43 (-1 for 0x80808080); vs_spec_copy / fs_spec_copy render the
  copy into a target per image size (Rgba8Unorm, never cleared) right before each specular draw; fs_spec reads the
  image, the cube or the copy. EffectsGpu::stage_walk follows the frame's draw order for stage 0's MIPFILTER, each
  specular binding's MIPFILTER and what stage 2 holds; group 3 (leak2d / leakcopy / drawst) feeds fs_spec's
  no-groups variant and fs_flare. Cube images get box-filtered mips (black faces).
- Inferred: the copy's source addressing is WRAP (stage 0's address is the last draw's, not set by the copy); one
  target per size where retail keeps one and remakes it (differs only when a stage-2 binding outlives a size change
  and the same size comes back).
- Seen offscreen: fx-shot 230 category 3 anim 46 (3446, tinted cube, two elements) runs the copy pass; no sheen, as
  before. Tests stage_walk_leak_and_mips, tint_copy_half_texel (GPU, against a CPU reference within 1 level),
  cube_box_mip.

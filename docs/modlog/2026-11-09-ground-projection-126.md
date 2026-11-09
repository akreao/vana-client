## Ground projection: the six open points (2026-11-09)

Modlog-gaps item 126 ("Ground projection: FUN_1003eb30's use of D3a +0x194; whether 'hi17' is D3m or MMB; MZB lists
with bit 0x2000 at load; lit decals' colour; +0x198 default; objects with +0xbc 0xffffffff while no sub-area is
loaded", MODLOG L11851). Addresses are Steven's FFXiMain.dll.

- The draw order of a projection: FUN_10046070 (every class's draw, vtable +0x2c) sets DAT_1047bc0c, runs the
  class prep (vt+0x4c, a bare ret here) and FUN_100467e0(0xffff), then the class draw (vt+0x48 = FUN_10040e70) and
  FUN_100472f0(0xffff). FUN_100467e0 takes a blend code: 0xffff means the element's own +0x16c (0x100467e6); it also
  sets LIGHTING by op-1 flag 16 (elem +0x10e bit 0), the two SetLight calls (FUN_10188b70: directional, diffuse only,
  ambient and specular 0), AMBIENT (FUN_1018b9e0 -> FUN_10188e00) and the material DAT_1047bc28 (diffuse 1, ambient
  0.7, specular and emissive 0, power 50; FUN_10046380). FUN_10040e70 then calls FUN_100472f0(0xffff),
  FUN_100467e0(vt+0x3c()) for the decal and FUN_100472f0(vt+0x3c()) after it. vt+0x38 / +0x3c are the setter and
  getter of +0x198 (FUN_10052eb0 / FUN_10052ec0); emission op 0x43 pushes the u16 @4 zero-extended (0x100501ba).
- D3a +0x194 (CMoD3aLightmapElem, vtable 0x1032b1d8, ctor FUN_10052c30, 0x198 bytes): op 0x43 stores it (vt+0x38 =
  FUN_10052c50) and vt+0x3c returns it (FUN_10052c60), but neither FUN_1003ee70 nor FUN_1003eb30 reads it.
  FUN_1003ee70 = FUN_10054d70 (the 256x256 target and the look-down view), FUN_1003eb30 (the D3a frame through
  FUN_1003e910 / FUN_1003e9a0 with ZWRITE off), then FUN_10054f80 (the decal) at once: no FUN_100467e0, so the decal
  keeps the D3a draw's blend, and no FUN_1006a450 / FUN_1006a4f0 / FUN_1006a0e0, so the WORLD and VIEW stack levels
  leak and the render target stays the 256x256 texture (the 15 callers of FUN_1006a060 / 1006a070 / 1006a0e0 were
  read; none sits on this path). The decal lands in its own image and nothing of the element reaches the screen. No
  generator of this class in any DAT. Ours drew it as a plain sprite; now Draw::None.
- 'hi17': the link step FUN_10049570 looks the name up as a D3m first (FUN_10072290 with type 0x1f, the whole scope
  chain) and only on a miss as an MMB (0x2e, climbing the parents; +0x28 -> elem +0x179 = 1). The op-1 byte @0x28 is
  cleared there first (0x100495b7), so the file's value never counts. DAT 0 `syst/effe` holds a D3m 'hi17' (5216
  bytes) and a D3m 'asi1' (256 bytes), the only two names that do not resolve inside their generators' files (22
  generators in DATs 3019, 3080, 4278, 4305, 56604, 86999, 87262, 87263; 3 in 52616, 52617, 52697). The MMBs named
  'hi17' sit in zone files (368 / 369, and 64-byte stubs elsewhere) and are never reached. So every projection in
  the data is a D3m and FUN_10040c20 (the MMB image, which would draw under WORLD = +0xa0 through FUN_10183760
  instead of S(0.25) L) is dead for this class.
- MZB lists with bit 0x2000 at load: none. Census over every DAT: 657 MZBs (versions 0x17 x 1, 0x1a x 14, 0x1b x
  642), 6,011,705 cell lists, no head with bit 11, 12 or 13. The bit only comes from FUN_1017ac40 (open doors), which
  ours models on the door boxes.
- Objects with +0xbc 0xffffffff: none. 500,382 collision objects: id 0 for 499,676, the sub-area ids 259..636 for the
  stand-ins, 999 for 3 (DATs 389..391), and 141 float-like values in DAT 100 alone (the one version-0x17 MZB, whose
  records are 0x80 bytes, so +0xbc is not the id there; zone 0 is never entered). Nothing is skipped while map+0x17c
  is -1.
- +0x198 default: none. The ctor FUN_10052e90 (base FUN_10044890, vtable 0x1032b3b8, class size 0x19c) never writes
  it and the block comes from FUN_10052a60 -> FUN_100286b0 -> FUN_10028fd0 (pool 3), which does not clear it. Every
  projection in the data has exactly one op 0x43. Ours starts at 0x48 before the ops run, so an op's 0 now stays 0
  (ONE / ZERO; it used to turn into 0x48), and 0xffff takes the element's +0x16c.
- Lit projections (op-1 flag 16): two in the data, DATs 60756 and 60801 `km2_/fls1` (mesh gr01, op 0x43 0x48).
  - The image: LIGHTING is on from FUN_10046070's FUN_100467e0. WORLD = S(0.25) L and VIEW = the look-down view, with
    NORMALIZENORMALS off, so the normal reaches camera space through the inverse transpose 4 times as long (for a
    rigid L); the light, set under the identity VIEW, turns with the look-down view too, so N.L is that of 0.25 L
    alone: Cv x sat(Ga + Ld x max(4 n.(-l), 0)).
  - The decal: FUN_100467e0 turns LIGHTING on again; FUN_10182700's vertices are FVF 0x142 (no normal), so the lights
    add nothing and the colour is AMBIENT x the vertex colour 0x80 (AMBIENTMATERIALSOURCE COLOR1, 0x10009a42;
    emissive from the material, 0), alpha the vertex alpha 0xff. About Ga / 2 instead of 0.5: much darker than unlit.
- FUN_10182700 again, from the listing (Ghidra folded three matrix cells into constants): the per-object matrix is
  the full product M = object x T(-c) S(1 / 10, 1, 1 / 10) T(0.5, 0, 0.5) (DAT_10377338.. = 0.5, 0, 0.5; DAT_10377344..
  = 0.1, 1, 0.1); the face test is n.x M12 + n.y M22 + n.z M32 <= 0 (the object's y column, scale 1), u and v use
  columns 1 and 3 with the translation. Stage 0 ADDRESSU / V = CLAMP (3). As built.
- Data, every DAT this time: 1477 D3m projections in 716 files; op 0x43 = 0x48 x 1117, 0x44 x 327, 0x8068 x 12,
  0x43 x 11, 0x42 x 6, 0x46 x 4 (0x43 is new to the list: REVSUBTRACT like 0x42, already in blend_class).
- Ours: the update pass writes FUN_100467e0's light direction for a lit projection to outs 41..43 (32..40 hold its
  local 3x3); vs_mesh_rt lights the image through 0.25 L (lit_by, shared with fx_lit); vs_decal takes 0x80 x
  d.ambient. fx-shot now passes the zone light to the effects (it never did, so lit elements were unlit there).
- Seen offscreen: fx-shot 230 --file 60756 --caster -100,0,-10 --victim -96,0,-10 --eye -100,-4,-17 --frames 8
  (frames 5..7): the green ground glow under the shell is much dimmer than before. Not checked against retail
  footage.
- Tests: gpu_ground_proj_light (GPU: outs 41..43 and 32..40), ground_proj_lit (the worked numbers); by hand
  ground_proj_census and mzb_list_census (`cargo test --release <name> -- --ignored --nocapture`).
- Not traced: what retail's loader leaves at +0xbc for the 0x80-byte records of a version below 0x1a (DAT 100 only).

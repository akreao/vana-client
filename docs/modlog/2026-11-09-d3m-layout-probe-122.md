## D3m layout probe: format 7 (2026-11-09)

Modlog-gaps item 122 ("Distortion: DistMorph (key mesh 0 only); the D3m loader's format-7 fallback", MODLOG L11684).
Addresses are Steven's FFXiMain.dll.
- DistMorph was built by "Effects: DistMorph (D3b + 0x100000)" (2026-10-25); nothing left there.
- FUN_1003fee0 (the D3m's first use; skipped once res +0x32 has 0x8080) probes formats 5 / 6 (+0x30 & 0xf):
  - no textures (+0x34 A == 0): for the first three 36-byte vertices at the format-5/6 offset (FUN_1003fd70 =
    FUN_1003fcf0 + A x 16), |normal| through the math object's vtable +0x94 (x87 FUN_1006d580 / 3DNow!
    FUN_10070820 / SSE FUN_1006f670: sqrt of the sum of squares) must not be above 2.0 (0x10329990: `and 0x4100;
    je`) nor below 0.1 (0x1032a378: `test ah, 5; jnp`), and pos x, y, z and u, v must pass FUN_1006ca20
    (_fpclass of the float widened to double rejects SNaN, QNaN, -inf, +inf and the double denormals, which no
    float reaches, so only NaN and infinity fail). A NaN length is unordered and passes both compares.
  - with textures: the runs' triangle counts (u16s at +0x38) summing past 3 x N (+0x36) fail.
  - A failure stores format 7 (+0x30 = +0x30 & 0xfff7 | 7). FUN_1003fcf0 then puts the vertices right after the
    header (+0x38) when A == 0; with A != 0 the layout is the same as format 5 / 6.
- The earlier note (4 meshes 6 bytes off: DAT 3559 / 50940 `md`, 51144 `d2`, 3033 `br`) took NaN normals as a
  failure; those keep format 6 in retail. Census (d3m_probe_census, every DAT, 74,033 format-5/6 meshes): 2
  switch, DAT 148 `2cor` (vertex 1's normal is 13.55 long; one generator names it) and DAT 170 `ike2` (vertex 0's
  is 0.099996; none). Retail then reads both six bytes early, so they come out as garbage; ours now does the same.
  zone-shot 48 with 300 ticks of effects renders identical, so `2cor` is not up at load.
- Ours: D3m::parse runs the probe (D3m::probe_fails) before laying out the table. A vertex past the end of the data
  fails the probe *(ours: retail reads whatever follows)*. Tests d3m_format7_probe, d3m_probe_census (ignored).
- The same function's colour pass, now built: while the zone renderer DAT_10631f24 has +0x54 set, every D3m vertex
  colour byte (alpha too) is doubled and capped at 255 (the loop over each vertex word +0x18: `(c & 0xff) x 2`, `> 0xff -> 0xff` per byte).
  +0x54 is 1 only around FUN_101787c0's / FUN_10178df0's FUN_10072fd0 loads of the zone DAT (zone + 100, or
  + 0x144f7 from 600), the bump DAT (zone + 0x9b97 / + 0x14817), file 0x1b8c and the sub-area DATs (FUN_101779b0
  path). FUN_10072fd0 loads and prepares a file at once (FUN_10073430 -> FUN_10073a60, the D3m vtable 0x1032c74c
  +0x20 = FUN_1003fee0 for chunk 0x1f in pass 2), so the zone's and sub-areas' D3m meshes are doubled and DAT 0's
  and the action files' are not. Ours: Library::mark_zone_load from Effects::load_zone / load_sub_area (meshes
  already parsed by prewarm are doubled there, once). zone-shot 50 with effects shows the change (the haze mesh
  around the default camera is brighter); 230, 231, 232, 235, 243, 26 and 102 render identical.

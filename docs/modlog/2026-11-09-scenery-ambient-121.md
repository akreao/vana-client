## Scenery ambient: retail's bytes, about half the env's (2026-11-09)

Modlog-gaps item 121 ("Scenery ambient may be twice retail's", MODLOG L15467). Addresses are Steven's FFXiMain.dll.
- The env's ambient is bytes / 255 (FUN_10188190 at 0x10188190: env+0x104 = key byte 0x34 x 0x10329e94 = 1 / 255;
  the actors' env+0x84 the same). FUN_10188e00 (mode 0 scenery and furniture, mode 1 actors) takes it through the
  windowed boost (DAT_10456a7c with every part below 0.8 (0x10329a34): times FUN_10017100's factors), caps it at 1
  (0x1032961c) and writes the D3DRS_AMBIENT bytes ftol(a x 128.0) (0x1032ae60; 0x10188f4e..0x10188f8c).
- The scenery pass sets them at 0x1017bd24 (SetRenderState 0x8b from FUN_1018b9e0(zone+0x39250, 0, 0)); the
  per-model area switch in FUN_1017c9c0 sets them again (0x1017d7a9). AMBIENTMATERIALSOURCE is COLOR1 from device
  init (0x10009a42) and is never changed, so the scenery's ambient term is the vertex colour x bytes / 255: about
  half the env's. Ours added the env's ambient unhalved (lit_zone), so shadow sides of scenery and furniture were
  about twice retail's.
- The config 0017 vertex-shader path (type-3 bumped meshes) takes its ambient constant from FUN_10188fa0 instead:
  the same boost and cap, then x 0.5 (0x10329a08) as a float, not rounded to bytes (zone+0x394e0, constants at
  0x1017d707..).
- Ours: zone.wgsl ambient_rs (lit_zone; lit_actor now goes through it too, so the actors' ambient also takes the
  windowed boost FUN_10188e00 gives it), fs_bump's t3 min(boost(a), 1) x 0.5. Lit effect elements (D3b / D3m) now
  take the boosted light colour and ambient (render::boosted, FUN_1000bf30 / FUN_10188e00) before ambient_ref.
- Two-light mode: already built for all scenery by "Config tool 0017" (Lighting::two, Uniforms modes.x), and
  furniture draws through the same scenery fragment paths, so FUN_101844c0's two-light mode is covered. The
  point-light falloff half of the item was built by item 118 (akreao/vana-client#321).
- Footage check: bg-wiki's retail Southern San d'Oria auction house shot (dusk, cloudy) has the north wall at about
  RGB 36 and the plaza at 86; zone-shot 230 at 17:30 and 18:30 from the same spot is darker on the shadowed faces
  after the change and closer to that look. The shot's hour and weather are unknown, so it is a sanity check only;
  the DLL settles the factor.
- Test boost_tests::boosted_ambient.

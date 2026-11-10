## DistRing: retail's 256x256 squeezed copy (2026-11-09)

Modlog-gaps item 123 ("Distortion: DistRing (retail squeezes the viewport into the 256x256 copy)", MODLOG L13487).
Addresses are Steven's FFXiMain.dll.
- Checked FUN_10042d00 again: the vertex lookup u = (sx + 0.5) / W, v = (H - 0.5 - sy) / H (box -0.5..W - 0.5 and
  H - 0.5..-0.5, 0x10329a08 = 0.5); the copy pass is four XYZRHW vertices (stride 0x1c) at (0, 0), (256, 0),
  (0, 256), (256, 256) (0x43800000 = 256.0) with uv (-0.5 / W, (H - 0.5) / H), ((W - 0.5) / W, (H - 0.5) / H),
  (-0.5 / W, -0.5 / H), ((W - 0.5) / W, -0.5 / H), W and H from FUN_10009e00() +8 / +0xc, into DAT_1047bb88's
  256x256 target, RS 0x1035322c / TSS 0x103531c0 (colour TEXTURE, alpha TFACTOR 0x80, MIRROR). So copy texel
  (i, j), its centre at (i, j) under D3D8's rules, samples u = (i W / 256 - 0.5) / W, v = (H - 0.5) / H - j / 256,
  and a ring pixel (px, py) reads the copy at ((px + 0.5) / W, (H - 0.5 - py) / H): the background blurred over
  about W / 256 px and shifted about (-0.5 - W / 512, -0.5 + H / 512) px, as the earlier entry worked out.
- Ours: DistRing alone now samples a 256x256 copy (EffectsGpu ring_copy) made from the full-size scene copy by
  render/squeeze.wgsl (the copy pass's mapping, linear MIRROR, alpha 0x80); vs_distring flips v for it. The other
  distortion draws keep the full-size copy (retail's soft look for them stays the open item 124).
- Seen offscreen: zone-shot 121 (Sanctuary of Zi'Tah) `--start izc1` from (-100, -2, 78): the ring's band over the
  crystals is softer and slightly shifted (differences up to 44 levels in a 185x110 box); nothing else changes.
  Zone DATs with DistRing generators: 221 `izum` izc1..izc4 (auto-start flag, not up in our runs), 222, 270; plus
  3483 `evw_`.
- Cost: one 256x256 pass per DistRing draw, only while one draws.
- Test squeeze_matches_retail_mapping (GPU: a 320x200 gradient squeezed, texels against a CPU bilinear MIRROR
  reference within 2 levels, alpha 128).

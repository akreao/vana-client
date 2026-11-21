## CMoVuDist repeats in compute (2026-11-18)

- After "Distortion reads the scene in place", a fire spell's impact still split the effect pass about 600 times a
  frame. Fire III (file 2946, `fair/g004`) has up to 20 live CMoVuDist elements at n = 10, each repeat ending the pass,
  running the 256x256 copy pass and opening the pass again. Live: about 4 ms of CPU encode a frame in the impact.
- Ours now runs an element's repeats in compute (render/effects.rs DistIter, effects.wgsl "repeats in compute"):
  - dist_region: the screen region both passes touch (the diamond's box and the shifted fan, 2 pixels wider), the
    copy's texel rect, the shared geometry, and the indirect group counts.
  - dist_load: the region's colour (as stored) and depth, once.
  - Per repeat: dist_copy (vs_dist_copy / fs_dist_copy) and dist_fan (vs_dist + fragment variant 9 + SRCALPHA /
    INVSRCALPHA colour-only blend + LESSEQUAL depth, written by flag 12). They sample by hand as the copy sampler does
    (linear with 8-bit sub-texel weights, MIRROR), cover as the rasteriser does (pixel centres, 1/256 snapping,
    top-left rule), and encode / decode sRGB in linearize mode.
  - vs_dist + fs_dist_put write the region back once, under the fan, with the fan's depth state.
  - So one break of the effect pass per element, not one per repeat; the repeat cap goes from 64 to retail's 512.
  - Used when the scene and its depth can be bound; else the passes as before. The offscreen target (render_pngs_lit)
    can be bound now, so the offscreen tools take the compute path too.
- Checked offscreen against the passes (fx-shot 232, Fire III, every 20 ticks): identical outside the VuDist frames.
  Within them, at most 8 levels on a few dozen pixels and 1..2 on the rest (sampling precision over the 10 repeats).
- Live (Lnxcsfix BLM 75, Fire III on a Snipper, no frame limit, back to back in the same weather): the impact cost
  +2.0 ms a frame with the passes (CPU encode +2.3 ms) and +1.1 ms with the compute repeats (CPU +0.5, GPU +1.0).
- Not modelled: a fan triangle with a point behind the eye is dropped where the rasteriser would clip it. The
  write-back (vs_dist_put) drops the same triangles, so it never writes a pixel dist_load did not read (review). Not
  seen with Fire III: its VuDist elements face the camera, so their five points share one depth.

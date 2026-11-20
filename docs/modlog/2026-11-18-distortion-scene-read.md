## Distortion reads the scene in place (2026-11-18)

- User report: many effects, fire spells most, looked like they played at a low frame rate.
- They did. The cost was in the frame itself, not in effect timing:
  - Offscreen, Fire's elements change every 0.2-tick step: fx-shot with a fractional step, a temporary
    FFXI_FX_DT not kept.
  - Live, FFXI_PERF on Lnxcsfix (BLM 75, Fire III on a Snipper, frame limit 144): 41.7 fps for the 2 s of the impact,
    GPU 16.96 ms a frame of which fx draw was 13.42 ms (0.3 before), worst gap 62.5 ms.
- Each CMoDist / CMoVuDist element copied the whole colour target before its 256x256 copy pass (section "Distortion
  / heat haze"), and so did the squeeze of the DistModel / DistRing / DistMorph variants. A fire spell's dozens of
  haze elements made dozens of full-size copies a frame. Retail copies only the diamond's screen box.
- The copy pass draws into the 256x256 target and only reads the scene, so it now samples the colour target itself,
  through a view in the copy's format, when the target can be bound. The swapchain image cannot, and still takes the
  copy. What it reads is the same: the scene as drawn up to that element, the earlier distortions included.
- Seen live (same test): 89..92 fps during the impact, fx draw 2.0..2.4 ms, worst gap 31.7 ms. That run's baseline
  was about 105 fps.
- Left: each distortion element still ends the effect pass and opens two (CPU encode up to 3.9 ms a frame in the
  impact). Batching elements whose boxes do not overlap would need their screen boxes on the CPU.

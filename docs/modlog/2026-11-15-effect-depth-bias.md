## Effect ZBIAS as Wine's polygon offset (2026-11-15)

- D3DRS_ZBIAS 15 for the effect draw (FUN_10045950; FUN_1017c840 for MMB elements via FUN_101705e0(DAT_10616f78))
  was ours as a fixed -7.5e-5 in NDC depth (to_clip, "as Wine maps it", 2026-10-22). With near 0.1 that is no small
  bias far away: depth 1 - 0.1 / z moves a surface at 600 to look like 414. Wine's d3d8 sets
  WINED3D_LEGACY_DEPTH_BIAS and draws ZBIAS as glPolygonOffset(-bias, -bias): a slope term plus units of the depth
  buffer's resolution. Ours is now the pipelines' depth bias (render/effects.rs ZBIAS: constant -15, slope -15),
  same semantics in Vulkan, on every effect pipeline but the CMoDist fan (vs_dist, which draws after ZBIAS 0).
- Seen in the Zilart ending (zone 163 event 7, area 519, scheduler `sha0`): the cloud domes cld1 / cld2 (MMB
  `suny`, 0x2440004 camera-relative, scale (8, 5, 8) / (7, 5, 7)) were pulled in front of the tree rows (420..780
  away; the dome's cap is about 150 high there), so the trees showed through the clouds. The user's retail
  recording has the trees solid in front of the clouds and the explosion glow; ours does too now (offscreen and
  live). The same dome is every zone's weather cloud layer (e.g. Konschtat `weat/suny/cld1`, (9, 7, 9)).
- The case the old bias was for (zone 101's gate tunnel at night, the `gin1` lamplight MMB over the walls) does not
  z-fight with the polygon offset; the far lamp glows (MMB `ltla`, 40..60 away) now sit partly behind their lamp
  housings, which the old bias pulled them in front of *(not compared with retail)*.

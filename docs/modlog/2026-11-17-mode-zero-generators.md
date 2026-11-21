## Mode-0 generators keep their matrix: Nightmare Scythe's discs (2026-11-17)

- User report: Nightmare Scythe was missing its two spinning discs that slice through the target.
- Mithra file 37131 (37067 + animation 64), its `main`:
  - At t95, op 0x02 starts `g0a0`..`g0a2` (attach 0xc402; elements of life 15; update op 0x33).
  - Each of their elements clones `g0b0`..`g0b2` (emission op 0x53).
  - The `g0b` clones draw D3m 4441, a quad textured with the green crescent blades (texture 4441, 256x128).
  - They run along the Vtx `line` with keyframe `k004` (emission op 0x54, update op 0x34), each disc living 90 ticks.
- FUN_10052eb0 (the per-frame generator attach) returns at once for attach mode 0, so the matrix stays.
- Ours reset a mode-0 instance's matrix to none every frame. That was written for zone generators, before element
  sub-clones existed.
  - An element sub-clone (attach `&= 0xfffefff0`: mode 0) kept the matrix ops 0x53 / 0x6a gave it only while the
    driving element re-placed it every frame (update op 0x33 / 0x46).
  - Once the parent element died, or at once without those ops, the clone's discs were drawn about the world origin.
- Ours now leaves a mode-0 instance's matrix as it is. Test fx::tasks::tests::element_clone_keeps_its_matrix.
- Seen offscreen (`fx-shot 232 --file 37131`): the two discs come in above the target, grow and slice down through it
  (t108..t174), where before nothing was drawn there.

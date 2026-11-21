## Element update ops 0x25 / 0x33 / 0x46: linked clones per frame, checked against the DLL (2026-11-08)

fx/mod.rs (`drive`, `clone_follow`, the op 0x53 / 0x6a clone start). Closes the "element ops 0x25 / 0x33 / 0x46
(per-frame child emission)" gap (MODLOG L3490, modlog-gaps item 113). Addresses are Steven's FFXiMain.dll: update
stream FUN_1004a980 (jump table 0x1004e03c, index op - 1), emission stream FUN_1004e8d0.
- Op 0x25 (0x1004b5d0): when the element's life (+0x110) is not 0 and it has a clone (+0x164), FUN_100496f0(clone,
  element): the clone's generator update and emissions, nothing else. As ours already did.
- Ops 0x33 / 0x46 (0x1004b604): the clone's matrix (vtable +0x28) = Euler(element +0xe0) (math vt+0x5c); clone +0x78 =
  the element generator's +0x78 x the element's scale (+0xec); +0x84 left alone. 0x33: when FUN_100473f0 (the element's
  frame: +0x108 if set, else its generator's +0xa4 attach matrix) is not 0, M = M x that frame with its translation
  zeroed. 0x46: no frame; M = M x FUN_10015240 (DAT_104568fc + 0x294) with its translation zeroed. +0x294 is the rigid
  inverse (FUN_100283c0) of the view FUN_10015120 stores from the camera's look-at (FUN_100151a0 -> FUN_10021c10), so
  0x46 turns the clone to face the camera. Then M's translation += FUN_10044c40 (the element's world position), and
  the clone runs while the element's life or the clone's own (+0xd0) is not 0. Math vt+0x20 jumps to +0x24, a plain 4x4
  multiply (0x1006d270 in the FPU table 0x1032c368).
- Op 0x53 / 0x6a start (0x1004eef5): the same Euler and frame (FUN_100473f0), then c10 (DAT_1047bfa8 + 0xc10) for 0x6a;
  clone +0x78 = the element's scale (no generator factor at start), +0x84 = FUN_10047490 (the frame's +0x50, else the
  generator's +0x84). 0x44 skips the placement; all three run the clone once (FUN_100496f0).
- Ours before: 0x33 never took the frame's rotation (a clone on a turned actor kept the world's axes), 0x46 used c10
  (yaw-only camera turn) instead of the inverse view, the per-frame scale missed the generator's scale A, and the clone
  start used the emitter's matrix / scale B even for an element on a borrowed (op 0x45) or snapshot frame. Now each op
  in the element's update stream places (0x33 / 0x46) and runs the clone in stream order, as retail; `parent_frame` gives
  FUN_100473f0 / FUN_10047490.
- Census of all 82,947 DATs on Steven's install (generator update streams): 0x25 in 5,871 generators, 0x33 in 819,
  0x46 in 221 (e.g. 1314 / 1742 `dm00` / `dm01`, 2848 `mah3`); no stream has more than one of them.
- Checks: tests `clone_follow_faces_camera`, `clone_follow_frame_rotation`. Not compared on screen yet (in-game test).

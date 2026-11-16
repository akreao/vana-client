## Effect audit fixes 5: model elements (kind 1), the camera rig (scheduler op 0xb9), the mesh radius (2026-11-15)

- Element kind 1, CMoSkeletonElem (FUN_1004e8d0 kind 1): its resource is a folder holding a skeleton (0x29), meshes
  (0x2a) and motions (0x2b); it owns a CXiSkeletonActor made from them (ours: an actor id from 0xfe000000 up, loaded
  from the folder's byte range by dat::npc::load_folder_model). FUN_10047830: once the model is in, the scheduler
  named like the element's generator plays on it (FUN_100724c0(7, name) -> FUN_10056c70), and while that runs (actor
  +0x84) the element's life is held at frame + 1. FUN_10047590, each frame: the model takes the element's world
  position, rotation (the draw matrix +0x5f8 is the element's whole matrix, the frame's yaw on top) and scale, its
  colour (vt+0x30) and alpha +0x59c = A / 128. It goes with the element. Seen live: Titan's crystals (mob ability 549,
  file 50196; bti_ / tit_ iw00).
- Scheduler op 0xb9 (vt+0x150 FUN_100d6f50): actor 1's +0x8ac bit 1 = u32 @8 != 0. While set, FUN_100cce60 (at the end
  of the actor's pose, FUN_100cbde0) drives the camera: eye = point 0x17, target = point 0x18, both through the
  actor's posed bones; point 0x1c's world matrix gives the roll, _13 x 60 degrees (camera +0xa4 -> +0x5c, FUN_10021900,
  turned in by FUN_10020de0), and the focal length, _22 x 200 (FUN_10015290). The status-motion switch and the heading
  ease stay off for the rig (FUN_100c83c0 is skipped). Its users: the 'cmon' / 'cmof' motions of the rig files 69769..
  (event motion values 3430..3443) in zones 126, 273 and 291. Seen live: zone 273 event 8, the tiger's rig shots
  between 'cmon' and 'cmof', then the event cameras.
- Ops 0x2e / 0x48 on a D3m or ground projection (FUN_10041a40) add |(r, 0, 0) x L| to |view position|: r x |scale x|
  (element word 63; the parent's scale B when it is size-scaled). r is the D3m header's u16 >> 4 (FUN_1003fcd0), or
  an MMB's half box diagonal (its scene node +0xd4, FUN_1016fcd0).
- Event op 0x5b / 0x66 (FUN_100b5210) starts the 4cc through vt+0x298 unless it is 0 or 'xxxx' (the whole u32). The
  thread scripts of zone 273's event 8 pass 0xfefefe00 to only load files: vt+0x298 (FUN_10084c30 / FUN_100cee40) finds
  no scheduler of that name and returns 0, as ours does.

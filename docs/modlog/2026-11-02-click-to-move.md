## Click-to-move (ours, 2026-11-02)

- Retail has none. client/clickmove.rs: Ctrl + left click on the ground with the modern UI (mouse_button's world
  click, no actor under the pointer) unprojects the pointer through the camera, finds the mesh with Ground::raycast,
  and searches a one-yalm grid from the character (Ground::floor within 0.8 up / 1.5 down, Ground::raycast at chest
  height for walls, 25000 cells, 60 yalms). The walk sets `autorun` and points `autorun_dir` at each point in turn, as
  /follow (FUN_100a6070's users) does; World.click_move ends at a move or turn key, the autorun key, arrival (0.6),
  or no progress (0.5 in 1.5 s, "Can't go any further that way.").

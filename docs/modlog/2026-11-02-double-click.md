## Double click on the world (ours, 2026-11-02)

- Retail has no double click on the world: a left click picks, and a click that leaves the target as it was confirms
  it (FUN_101585f0: talk, or the action menu for a player or monster). The modern UI's mouse contract adds one
  (target::click `double`): with config `modern_ui`, a second plain left click on the same actor within 500 ms (the
  Windows default) runs Ctrl+C's FUN_10130280 instead (talk to an NPC, check anyone else, c2s 0x0DD) and closes the
  action menu the first click may have opened. Locked targets, the sub-target prompt, drags and the classic UI keep
  retail's rule.

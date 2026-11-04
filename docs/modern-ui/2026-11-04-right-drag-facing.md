- Right drag (user, 2026-11-04; supersedes "a right drag turns the camera and the character faces where it looks" in
  the mouse bullet): a right drag turns only the camera. The facing follows the move as retail (FUN_100a6240), so a
  drag while moving never strafes or backpedals, and a release after a camera drag (any look motion, or the drag
  threshold) does not stop autorun (ours: retail has no right-drag camera, so its release always cancels).

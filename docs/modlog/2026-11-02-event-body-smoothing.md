## Cutscene actors drawn between VM steps (ours, 2026-11-02)

- NPCs shook during cutscenes. The event VM moves its bodies in 60 Hz ticks, as retail does, but we draw at up to the
  frame limit (300 by default). The bodies stood still for several frames and then jumped, while the scheduler camera
  path is sampled every frame, so a walking actor visibly vibrated (and a camera framed on actors jumped with them).
- Not retail (it draws a whole number of ticks a frame): `event::pose` now draws each bound actor between its place
  before the last VM step and its current place, by the part of a tick waiting (`vm_ticks`), so bodies are drawn one
  tick behind the VM. Rotation and tilt go the short way round. A placement (s2c 0x05B / 0x065 snap, or a jump of over
  2 a tick) is drawn at once. At 60 fps or below nothing changes.
- The VM, its host (`drawn_at`), our own place (`keep_pose`) and the event's end still use the stepped pose.

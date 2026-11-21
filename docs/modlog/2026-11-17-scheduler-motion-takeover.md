## A scheduler motion that follows a counted one takes over its pose (2026-11-17)

- User report: at the end of a trust cast a Mithra holds both hands out in front. Ours dropped them to her sides for
  a moment, then raised them again.
- The cast's scheduler (Nine's log, spell 922):
  - `ms0` (count 8, 150 ticks);
  - then `ms1` (hands out, count 1), stretched over its op's 80 ticks, out blend 0;
  - then `ms2` (count 1, 100 ticks, blend in 20), started at the tick `ms1` ends.
- Ours, in the same frame:
  - saw `ms1`'s counted end first and brought the status motion back (blend in 0: a snap to idle);
  - then queued `ms2`, which blended in over 20 ticks from that idle pose.
- Retail: a counted end with nothing queued after it sets model +0x79c (vtable +0x1fc(1)), and queueing a region-0
  motion clears it (vtable +0x1fc(0)). FUN_100c6380 brings the status motion back only while it is set (section
  "Event motions, actor fades, PC battle motions"). So `ms2`, queued as `ms1` ends, keeps the status motion out, and blends from `ms1`'s last pose.
- Ours now collects the frame's op-0x05 starts (the swing's one-shot and the play's) before the status test. A
  counted end counts only when none starts that frame.
- Seen live (Lnxcsfix, Mithra, Lehko Habhoka in West Ronfaure, side view, a shot every 0.15 s): `ms1 -> ms2`, the
  hands out throughout. Nine's log: 14 of her 15 trust casts went `ms1 -> idl -> ms2` before.

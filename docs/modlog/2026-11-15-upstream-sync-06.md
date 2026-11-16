## Effect audit fixes 6: update ops 0x59, 0x5a, 0x44, 0x43 (2026-11-15)

- Update op 0x5a (0x1004af16; 1,775 generators across the game files, mostly zone glows and fog: Inner Horutoto's
  lamp glows `x0..`, fog0): k = last frame's sort key +0x12c - w4; k <= 0 (or unordered): scale = (w1, w2, w3); else
  scale_i = w_i + k x w5, so the sprite grows with depth. Seen: the zone 192 lamp glows keep their size on screen.
- Update op 0x59 (0x1004af85; 380 generators): rot.z = fmod(sort key x w1 + w2, 2 pi) - pi (constants 0x1032b168
  double 2 pi, 0x1032a3b8 pi).
- Update op 0x44 (0x1004bfdf; 14 generators: bst_ reki / pika, DAT 0's gn30..gn33, kon1 / kon2 / smk1 / smk2):
  when S[s] holds a keyframe and S[s+2] the update stream's op 0x2c record (emission op 0x69, 0x10050b27,
  FUN_10049330(0x2c)), the record's word 1 (op 0x2c's drag factor) = key(t), no first-value override. The record is
  shared by the generator's elements, but every generator using it runs 0x44 just before its 0x2c, so each element
  drags by its own key(t): ours keeps the value for the 0x2c that follows.
- Update op 0x43 (0x1004d67b): the sound volume samples the plain day fraction (FUN_1004e310 returns DAT_1065cb14) x
  127 (FUN_1004e5c0), with no 0x3f-style repeat count.
- The GPU / CPU update check (render/effects.rs gpu_update_matches_cpu, now on DAT 332 and 105) also mirrors ops
  0x2e / 0x48's view-space distance (audit 3) and 0x59 / 0x5a; the CPU takes the GPU's last-frame sort key.

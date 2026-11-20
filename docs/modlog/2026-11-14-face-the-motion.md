## Face-the-motion elements: any motion counts (2026-11-14)

- FUN_10044350 orientation modes 1 / 2 (flags >> 6 & 7) turn the element by the atan2s of its position less its last
  one, with no minimum length. Stoneskin's spikes (effect 2854, `g000` / `g001`: template flags 0x1242, mode 1) sit
  on a ring (op 0x1f, 6 by count) with a 0.611 rad tilt (op 0x09) and a 1e-6 outward drift (op 0x08) that only gives
  each its heading, so the tilt points out from the centre. Ours skipped the turn below 1e-12 squared, so every
  spike tilted the same way (user: "supposed to all be tilted out from the center"). Now any motion turns it; none
  leaves it as retail's atan2(0, 0) does.

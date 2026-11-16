## Effect audit fixes 4: emission 0x9b, end op 1's one emission, fast-forward children (2026-11-15)

- Emission 0x9b (0x1004fff0, 361 generators): the generator takes the parent's actors, and the element starts at the
  parent's world position through this generator's inverse frame, x over scale A's x; ours added the parent's local
  position *(ours does not rebuild the attachment for the new actors until the next frame)*. 0x86 keeps the local add
  (its frame copy is not done).
- End op 1 (0x1004e81d..0x1004e861): +0xd6 = ftol(dt + 1) for the burst's update, then restored, so it emits once
  whatever the frame time.
- Fast-forward (FUN_100575a0): 0x03 / 0x73 children start caught up (0x100579c4 -> 0x10057aa4 -> FUN_10056b20, as
  0x09 / 0x57), and 0x85's kill runs (FUN_10056ef0).

## Two-frame motion chunks: retail's 30 fps step (2026-11-14)

- FUN_1001a8e0 (a queue entry's frame step): a chunk of 2 frames whose speed x 0.5 is 0.5 steps a whole frame per
  call (frame += 1.0), every other chunk ticks x speed x 0.5 x rate. Retail calls it once a drawn frame at its 30 fps,
  so such a loop swaps its two keys every 2 ticks: the wings of bees and hornets (model 272 / 273, skeleton k_be,
  `idl1`: 2 frames, speed 1, 12 tracks; `idl0` the body, 12 frames at 0.3) beat at 15 Hz.
- Ours called it once a rendered frame: at 144 fps the keys swapped every frame and read as two still, ghosted
  wings (user: Temple Bees and Davoi Hornets "don't flap"), and the beat changed with the frame rate. Now the whole
  step comes every 2 ticks of elapsed time (motionq Entry::flip) *(inf: retail's frame is 2 ticks at 30 fps)*. Test
  two_frame_chunk_steps_every_two_ticks.

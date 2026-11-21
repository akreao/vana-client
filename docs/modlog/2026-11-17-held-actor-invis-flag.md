## InvisFlag cleared under a held actor (2026-11-17)

- FUN_100973f0 gates only two writes on the hold count (+0x84 == 0): the InvisFlag colour (0x27808080 / 0x00808080
  with +0x128 bit 7 set) and the last step (a colour with alpha left back to 0x80808080). With InvisFlag clear, the
  bit 7 check (cleared, colour 0x80808080) runs before the hold test, so the flag's colour is undone even while a
  task holds the actor.
- Ours returned for all of it while held. A weapon aura's loop (Excalibur, file 65916: `!in1` -> `auon` -> `lop0`,
  never ending) starts again the moment InvisFlag clears and holds the actor, so `!hide` off left Nine translucent
  for good (user, 2026-11-17). wepfx::colour now follows the order above.
- Seen live: Lnxcsfix as RDM 99 with Excalibur 119 III: `!hide` translucent (after a bot's Cure tasks on her ended),
  `!hide` again solid with the aura, `!hide status` 0.

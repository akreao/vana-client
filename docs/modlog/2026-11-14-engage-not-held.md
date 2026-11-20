## Our engage is never held (2026-11-14)

- Retail copies the server status (+0x16c) into the applied one (+0x170) only when the entity's busy counter (+0x11c)
  is 0 (FUN_1008e9a0 -> FUN_1008b460, asm 0x1008b4a6: ECX = entity +0xa0, the actor; FUN_10083320 reads its
  entity's +0x11c), unless actor +0x100 (Conflict teams only, FUN_1008e290: +0x1ff in 2..99). Every hold a monster
  puts on its target raises that counter: op 0x56 / 0x20 through FUN_100601f0 -> FUN_10083300 -> FUN_100a4140. An
  Orcish Farkiller's shot (category 11, effect 3916) holds its target 160 ticks (op 0x56), its swings 48 (op 0x20),
  so being shot at kept our engage parked for seconds, and our first rounds landed before the draw (user, Davoi: "the
  weapons haven't been drawn yet").
- Ours: the 0x037 handler applies our own idle -> engaged change at once whatever holds us; every other change (the
  disengage, death) still waits for the holds *(ours, on the user's word against the decompile's parking)*. The log
  prints "status N parked (was M): <holds>" when ours is held (combat::held_why, Effects::held_by). Live (Lnxcsfix,
  Davoi, an Orcish Farkiller's shot holding her until about 43.05 s): /attack at 42.08, engaged and drawing at 42.42,
  the first round at 45.19 with its swing.

## A freed skill file takes its effects: Nightmare Scythe's red sphere (2026-11-17)

- User report: Nightmare Scythe's red sphere (file 37131 `g000`, endless, flag 0x1000) stays after the weapon skill.
  `st44`'s op 0x1e stop misses it in retail too ("Dolls go with their actor"; checked again):
  - op 0x1e passes the child task's actors (0x10058d5f: FUN_10056c30 / FUN_10056c40), and op 0x03 starts the child
    on the undolled pair;
  - FUN_10052a50 compares the clone's own actor 2 (clone +0x30, FUN_1003a8c0, asm 0x10052b38..51), the doll since
    op 0x23 moved it (FUN_10061da0).
  - That note's conclusion, that retail ends `g000` only with the target, was wrong.
- What ends it is the skill file going:
  - The skill files sit on the caster's model list (+0x7e4) as 0x24-byte entries (FUN_100d4410; class FUN_1001b710,
    vtable 0x1032a2e0) holding the file's resource at +0x1c, referenced when used (FUN_1001b7b0 -> FUN_10071e90).
  - At `main`'s end, FUN_100d44b0(caster, 6) unlinks every kind-6 entry and deletes it (vtable +0x18(1)).
  - The destructor (FUN_1001b760 -> FUN_1001b7d0) releases the resource (FUN_10071ec0) and calls FUN_10073b20, which
    deletes the file's resource tree once its count is 0.
  - A clone references only its template (the +0x28 walk of FUN_10071e90 / FUN_10071ec0 stops at a node whose
    handle is itself), so live clones do not keep the file.
  - So the clones of its generators go with it *(inf: the tree's delete takes the clones of its templates)*.
- Ours: Effects::release_file kills every live instance and task of the file. The client calls it for each dropped
  skill file that no other actor holds (extra list or skill_want). Test fx::tasks::tests::released_skill_file_takes_its_clones.
- Seen live (Lnxcsfix, Mithra DRK 75, Mythril Scythe, a Snipper in Valkurm Dunes): the sphere goes at +5.4 s, when
  "skill files dropped" is logged; before, it sat by the corpse until the despawn about 15 s later.

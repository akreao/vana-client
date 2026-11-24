## Effect attach modes 9 / 0xa: op 0x11 on a doll, the skeleton before its first pose (2026-11-10)

Modlog-gaps item 128 ("Effect attach mode 9 with op 0x11: whether doll-attached generators run away in retail; the
not-yet-posed skeleton's identity", MODLOG "Effect attach modes 9..0xc: the bone frame"). Addresses are Steven's
FFXiMain.dll; all read in the disassembly.

- The doll's bone frame. Vtable +0x1d0 of the doll / control / collision / lift classes (0x1032f938, 0x1032dfc8,
  0x1032db40, 0x10330288, also 0x1032cfe0 and 0x1032e3c8) is FUN_10082180: out = out x T(vtable +0x1bc()), the
  doll's position +0x404 (FUN_100abd90), through FUN_10027c70 (a local identity with row 3 = the four floats,
  FUN_10027cd0, then FUN_10027d10: this = this x arg). Nothing resets out, and the callers FUN_1003bd90 /
  FUN_1003be10 (cases 9 at 0x1003c379, 0xa at 0x1003c3c3) hand it on untouched; the caster / target getters
  FUN_1003b6d0 / FUN_1003b700 do not unwrap a doll. So what a doll's frame is depends on what the caller passes.
- The two callers pass different matrices.
  - FUN_10053e60 (the generator's start at 0x1005330c, the spin ops 0x0e / 0x0f / 0x10 at 0x1004a0b8 / 0x1004a238 /
    0x1004a3b8, emission op 0x45 at 0x1005007a, 0x10059fd7) passes vtable +0x28 = FUN_10053e00: the generator's own
    matrix +0xa4, made once as the identity. A doll's first start gives T(position); every later call adds the
    position again. The spin ops call it every frame, so they would run away, and their turn would compound too
    (they turn the rows of the same matrix). No data does this: of 10,911 mode 9 / 0xa generators none has a spin
    op (census below).
  - Op 0x11 (update jump table 0x1004a814 entry 0x1004a533, and the finder FUN_10053dd0) is FUN_10053c80: it takes
    a math pool matrix (DAT_1047cf7c +0xa10 [+0xa90], one slot above the stream walker's own at 0x10049766), calls
    FUN_1003be10 on it, and lerps the generator's matrix toward it by k = byte / 255 (bit 0 row 3, bit 1 rows 0..2).
    The pool is 32 matrices handed out as a stack (FUN_1006ce40 sets the pointers once; nothing clears a slot), so
    for a doll the target is whatever the slot's last user left, times T(position), and the product stays in the
    slot for the next user. The generator's own matrix is not part of it: op 0x11 does not run away the way the spin
    ops would. What retail shows is not defined: with a stale matrix that carries a world translation the target
    sits a world position away from the doll, and two doll generators in a row add the position twice. The walker
    runs from ten call sites at different pool depths (generator tasks 0x1005399b / 0x1005425e, element ops 0x25 /
    0x33 at 0x1004b5fa / 0x1004b80f, ...), and element updates use the same slots in between.
- Ours before: op 0x11 (Effects::follow) rebuilt a doll's frame from the generator's own matrix, as the start does,
  so with k = 255 the generator moved by the doll's position every frame (seen in the sweep: 4, 8, 12 ...). Now the
  target of op 0x11 is built on its own matrix (attach_matrix_into), taken as the identity *(inf: the scratch's
  content)*: the doll's position, no rotation. The start and the spin ops still build into the generator's matrix,
  as retail.
- The skeleton before its first pose. FUN_100d49d0 (vtable +0x1d0 of the seven skeleton classes) loads the identity
  into out (FUN_100279b0), adds T(+0x6a8) on a throttled frame, and FUN_100d4a90 -> FUN_1002c500 -> FUN_1002c520
  writes the point frame only when model +0x30 has bit 4. FUN_1002c2d0 sets 0x12 there, called from the actor's
  update FUN_100cbde0 (0x100cc950 and the dead / ground branch) once FUN_1002a140 has computed every skeleton of
  the model; with no skeleton on the list FUN_100cbde0 returns before it. Until then a mode 9 / 0xa frame is the
  identity (the world origin), and the 0xb / 0xc heading is -atan2(1, 0) = -pi / 2. FUN_10085060 starts a part
  file's looping generators from the file's arrival callback (FUN_100d1400 -> FUN_100d1450), so they can start in
  that state; whether a file can arrive before the actor's first update is not traced (the loader's order).
- Ours: Actors::never_posed (the client: the pose step has not run for the actor yet, kept over a look change) makes
  the bone frame the identity. Before, such a start took the rest pose. Generators with op 0x11 leave it at the
  next frame; of the 1,190 looping mode 9 / 0xa generators 1,160 have op 0x11 (the 30 without are in zone files
  116, 118, 120, 122, 138, 155 and in 56811, 70502).
- Census (dat::generator::tests::mode9_doll_census, 35 s): 10,911 mode 9 / 0xa generators, 9,629 with op 0x11 in
  1,580 files, none with a spin op; 473 of those files also make dolls (scheduler ops 0x15 / 0x16 / 0x22 / 0x23).
  Op 0x11 words: bits 3 with k 255 on 8,364, bit 0 only with k 255 on 1,143, the rest slower pulls (k 35..195).
  fx-sweep without mob models hits a doll with mode 9 and op 0x11 in 13 files: 60757, 60802, 61033, 61124, 61126,
  61129, 61151, 61183, 87004, 87024, 87201, 87205, 87222 (all on a caster doll).
- Checked offscreen: `fx-shot 230 --category 11 --anim 1958 --caster -100,0,-10 --victim -96,0,-10 --eye
  -100,-3,-17` (file 61151: `grw1 frm1 frm2 frm6 kmr1` on the caster's doll) keeps the flames at the caster; before
  they left by the caster's position a frame. Tests doll_follow_stays, attach_never_posed_identity.
- Not done: the throttled frame's T(+0x6a8) on the unposed identity; the scratch's real content (it would need a
  debugger on retail).

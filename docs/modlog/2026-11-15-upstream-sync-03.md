## Effect audit fixes 3: generator op 0x04, sub-clones, 0x39, 0x4e / 0x4f, 0x8f, colour keys, 0x2e / 0x48, 0x64 (2026-11-15)

- Generator op 0x04 (0x10049a10, 1,021 generators): it is linked only by the clone ctor (FUN_100536c0), which also
  writes word 4 = float(resource d8 & 0x1ff) as the keyframe's first value (FUN_100544d0 / FUN_100546f0). A resource
  running as itself (zone, sub-area, weather, a burst's template) sets its countdown to 32767 once x > 0 and stops;
  a clone (task, actor model, footstep, element sub-clone) samples with that first value.
- Element sub-clones (emission 0x44 / 0x53 / 0x6a, 0x1004ee4a): attach `&= 0xfffefff0` (mode 0), and +0x64 (the
  damage number) only from a parent that is a clone itself.
- Emission 0x39 (0x10050bc6): the distortion classes' base S[s+2] = their offset +0x194.
- Emission 0x4e / 0x4f (0x1004eb5d): with op byte 0x0c bit 0 and a frame (FUN_100473f0), the point goes through
  M, then (b90 x M)^-1 (D3DX +0x48 / +0x24 / +0x18): the view rotation's inverse.
- Emission 0x8f (0x1004f41a): no source actor (FUN_1003b6d0), no tint; ours used the zone's light (13,369 generators,
  most zone ones without an actor).
- Update ops 0x18..0x1b (explicit test) and 0x3c..0x3e (FUN_1004e220) store 0 for a key below 0; ours wrapped.
- Update ops 0x2e / 0x48 measure |view position| (vtable +0x44; camera-relative elements in view space), 0 behind the
  eye for classes other than meshes and ground projections (FUN_10045d00). The mesh classes' bounding radius term:
  "Effect audit fixes 5".
- Update op 0x64 (0x1004b249) writes this frame's position and step back into S[s..s+2] / S[s+3..s+5], so it turns by
  the change of heading; ours compared against the spawn point and direction.
- The rest of the element / generator audits: kind 1 and the mesh radius in "Effect audit fixes 5"; update 0x44 /
  0x5a / 0x59 / 0x43 in 6; orientation bit 0, the MMB cull and offset, op 0x15, D3a + 0x200000 in 7; emission 0x86 /
  0x98..0x9a / 0x7e / 0x7f / 0x89 / 0x94 / 0x69 / 0x93, update 0x6a / 0x51 in 8; 0x46 in "Event VM audit fixes 2";
  emission 0x9b and end op 1 in 4; DistRing's copy was done 2026-11-10 (render/squeeze.wgsl).

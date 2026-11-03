# Addendum: entity +0x18 (yaw) during events, for text code 0x1d (asm 0x100c2754)

Same DLL (sha256 8097dbf8…550c). **(inf)** = inferred. Follows [batch 5](2026-11-03-modlog-gaps-batch5.md) item 1.

**Answer: while an actor is bound to an event, script turns go into its event body and not into entity +0x18. So code
0x1d prints the entity's direction from before the event, or the last server update. It does not print a turn the
script made before the `1d` line.**

- **0x1d** (0x100c2754..0x100c27a6): entity = table 0x10480af0[message actor 1, +0xbfe]; it reads **entity +0x18**, adds
  a constant (0x10330274), normalises it (FUN_100dd9d0), multiplies by 0x10330270, and takes the result & 7 as the group
  0xf word. There is no body read here.
- **Bound = entity +0x120 bit 7.** The body is entity +0xd4 → +0x260. In the body, +0x140 / +0x144 / +0x148 is the
  position and +0x150 / **+0x154 yaw** / +0x158 / +0x15c the rotation.
- **Turn ops write the body when bound:** e.g. op 0x4a (FUN_100b6700) at 0x100b67f0..0x100b6824 writes body +0x150 = 0,
  +0x154 = atan2, +0x158 = 0. Only the unbound branch, 0x100b682c, writes **entity +0x14 / +0x18 / +0x1c**. The other
  body-yaw writers are the same kind: ops at 0x100b2dac, 0x100b2f1a, 0x100b326a, 0x100b56a4, 0x100b5e60, 0x100b603c,
  0x100b6165, 0x100b68ff, 0x100bc192, 0x100bc20b; VM variable 0x7f83 (0x100af480..0x100af496 write, 0x100af62a read).
- **At bind** the body is filled from the entity: 0x100bce3c..0x100bce5c copies entity +0x14..+0x1c into body
  +0x150..+0x158 (also 0x100bc0e3.. from entity +0x34..+0x3c; 0x100bcf9a.. zeroes it).
- **FUN_1008cce0** only sets entity +0x166 and +0x120 |= 0x10; it writes no direction.
- **Every body-yaw reader** (all `+0x154]` reads in the DLL, checked one by one):
  - 0x1008cdfe / 0x1008cee6: to the actor's drawn rotation (actor vt+0x1c0), the 12.5% easing at 0x100c65d6..;
  - 0x100933b2: to entity **+0xe4** (the last scripted direction used for @tl0 / @tr0);
  - 0x10088002 / 0x1008f8a5: into locals (pose / placement);
  - 0x100af630: VM variable 0x7f83;
  - **0x1008eecc / 0x1008eee7: the only copy back into entity +0x18 (and +0x38).** This is in the unbind FUN_1008ee30.
    It runs only when the actor is bound and **entity +0x138 is negative**. It also copies the position to +4 / +0x24,
    then frees the body.
- **Packet writers of entity +0x18 / +0x38:** 0x00D at 0x10099f6f..0x10099f8e and 0x00E at 0x1009bb12..0x1009bb29
  (direction byte @0xb through FUN_100dd990). The bound test before them (0x10099ea8..0x10099eb6) gates only the ground
  snap (FUN_10181290), not the write. So a server update during an event still changes +0x18.
- Not all ~200 `[reg+0x18]` stores were walked; many are other structs. The body path above is complete, judging from
  the full list of +0x154 accesses.

**So, for a bound actor mid-event, 0x1d reads entity +0x18 = the direction at bind time, updated only by 0x00D / 0x00E
packets. A script turn shows in the drawn model and in VM variable 0x7f83, not in the word. After the event, a turn
reaches +0x18 only if the unbind runs with +0x138 < 0** *(inf: what sets +0x138's sign bit was not traced)*.

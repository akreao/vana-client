## Effect audit fixes 8: the parent record, attach copies, emission 0x86 / 0x98..0x9a / 0x7e / 0x7f / 0x89 / 0x94 (2026-11-15)

Census from every generator in the install (335,703). A generator a scheduler or task runs is updated with no parent
element (FUN_10054220 calls FUN_100496f0(gen, 0)); only element clones, 0x3c bursts and end-op-1 bursts have one.

- The parent record a child reads (fx Effects::take_back) kept the parent's UV and bias at 0 after its first read
  back, so emission ops 0x4a (972 generators: the parent's UV) and 0x9d (69: its bias) read 0. They are read back
  now (back words 20..22).
- FUN_1003c740 (ops 0x45 and 0x86) copies attach bits 0..16 and 24..27 (ours missed the size-scale nibble 24..27), and
  the size-scaled test follows the copy.
- The +0x108 snapshot takes the generator's scales A / B at op 1 (0x100511c6..); op 0x45 then writes the parent's
  scales only into the generator (+0x78 / +0x84), so they show on later elements, not this one. Ours put them on this
  element's snapshot.
- Emission 0x86 (0x1004fac3..0x1004fc80): the attach copy without the actors; under op-1 flag 0x800000 with a parent
  frame the snapshot is identity with the parent frame's translation and the parent's scales A / B (FUN_10047450 /
  90); else an element without a frame borrows the parent's (FUN_10044c20); then pos += the parent's local position.
  Its 6 generators (ROM/318/59 co01..co05, hk01) are only run by schedulers tgt1 / tgt2, so it never runs in the data.
- Emission 0x98 / 0x99 / 0x9a (0x1004f126 / f1ae / f238): with a parent whose life +0x110 is not 0 (ours tested max
  life): its life and max life; then with k the slot of op 0x3e / 0x3f / 0x40 in the parent generator's emission
  stream (FUN_100492f0), S[s + n] = P[k + n] and S[s + 3] = P[k + 3]. The parent exports its first such op in GEN_BITS
  23..31 and the two values in back words 25 / 26. Users: only 0x98 (6 generators); ppp3 (ROM4/0/21) and tb04
  (ROM9/8/8) copy slots their update streams never read.
- Emission 0x7e / 0x7f (0x1004ff54 / ff8c): a point light (IsType 0xb) takes its parent's +0x1a0 intensity / +0x19c
  range. Only ROM2/0/16 p_e4 has a parent (p_e2, by end op 1): it now continues p_e2's keyed light (0x5b 'e2lp')
  instead of restarting at 0x58's 80.
- Emission 0x89 (0x1004ea4d): S[s] = word 1; a non-zero word 1 is a handle whose +0x14 goes into the next 0x4e / 0x4f's
  word 1. Word 1 is 0 or a stale file value, and none of its 7 generators can start (their only references are op 0x65
  records, which nothing runs).
- Emission 0x94 (0x1004fed3): only an op 0x4e on both sides (the parent's 0x4e list for this 0x4e); a 0x4f keeps its
  own list. Its 11 generators' parents have neither, so nothing changes in the data.
- Update 0x6a / emission 0x93 write op 0x65's word 3 (+0xc); the only references to an op-0x65 record are the link
  step (0x10048fbf), the update table's no-op, 0x93 (the only `PUSH 0x65` of the 39 op-finder calls) and 0x6a: nothing
  reads word 3, so both stay no-ops. Update 0x51 (continue in another generator's stream): 0 of 335,703 generators.

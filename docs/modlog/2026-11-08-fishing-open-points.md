## Fishing: the open points (2026-11-08)

The "Left" / "Open" points of "Fishing", read in FFXiMain 30261006:
- FUN_10127ea0 runs after each successful c2s 0x110 mode 3 (the 2013 controller FUN_10089100 at 0x1008936e, the
  no-water end 200, and 0x100894a1, the result; the PS2-era FUN_100883e0 at 0x1008863e / 0x100886ed) and from the menu
  opener FUN_1015ef60 and 0x100ae3a0. It is the event message queues' cancel: FUN_10126a00 on both queues (DAT_104e2150
  / DAT_104e2154: their pages freed, the rem's rows reset, +0x34 = 1), DAT_10488208 = 2, and both rems' vt+0x4c. During
  fishing no event can run, so the only thing it can drop is a page pushed by s2c 0x03B (below). Ours has no 0x03B, so
  there is nothing to cancel yet.
- s2c 0x03B (handler LAB_100a1600, registered by FUN_100fc340(0x3b, ..)): u32 id @4, u16 ActIndex @8, u16 MesNum @0xa.
  It needs a valid index (FUN_10086fc0) whose actor's id (+0x78) is @4 and not 0; the text is FUN_100964a0(8, MesNum &
  0x7fff), source 8 = DAT_10487e84, the zone's dialogue file (FUN_1008a7d0: language slot 0x6b + zone, 0x6e + zone -
  0x100, or 0x6d + sub-zone - 1000 for 1000..0x513). It sets DAT_104804cc, both message actors (DAT_10482f40 /
  DAT_104855aa) to the index and DAT_10488208 = 1, then shows the line with mode {0x96, 2, 0}: bit 15 clear through
  FUN_100ad5e0 with the "%s : " prefix (0x1035bb48) and the speaker's shown name (+0x214 in Monstrosity, else +0x7c),
  bit 15 set through FUN_100ad660 with no prefix. Mode 0x96 is the event message mode (ops 0x1d / 0x2b / 0x49 / 0xb0):
  the line goes into both event message queues and opens the rem (pages reach the log as they open), with its
  key waits, instead of being printed straight to the log as 0x036's modes are. LSB never sends it.
  **Not built yet** (ours keeps the event message inside the running event; a 0x03B arrives outside one).
- Plain rods draw their line as part of the rod model. Hume willow (39317, ROM/90/68) and Hume rod 39307 hold mesh
  `hf_s` (0x2a) with its texture, skeleton `sao` (0x29, 9 bones) and the fh00..fhd0 / idl0 motions (0x2b); the +1 rod
  (Mithra 39481, ROM/334/82) has the same plus the `!f00..!f02` / fgl / mfs / flp0 / fkla schedulers and their
  generators. FUN_100cae60 only starts `!f01` / `!f02` (rods 14 / 15); no DLL code draws a line. Ours draws the rod
  model with its motions, so the line is the mesh's (to be looked at in game).
- The /fish latch DAT_10487e49 is set only by /fish (0x1007acaa) and cleared by s2c 0x052 mode 4 (0x100a20d5), the zone
  reset FUN_10093720, and the two controllers on a fishing or 1..4 status (0x10088449, 0x1008916a). Retail has no
  timeout either: a request LSB drops without an answer leaves the latch set until one of those, as ours.
- Other anglers: FUN_10089100 runs only for our player (+0x1d0 bit 9); others get the rod actor and status motions
  (done), nothing else.

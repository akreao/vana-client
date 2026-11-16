## Event VM audit fixes: 0xac, the door ops, 0xb5 (2026-11-15)

From a full pass over retail's dispatcher (FUN_100bc280) against event_vm.rs / client/event.rs:
- Op 0xac (FUN_100bb180) sub 1 writes the event actor's +0x174 = val@2 whatever 0xab 3's lock says (asm
  0x100bb1ee..0x100bb215; 2019 FUN_100b92f0 alike). The scripts lock with 0xab 3, set 1 with 0xac 1, then 0 and
  unlock (about 450 of its 655 sites), so those scene poses never showed. Subs 2 / 3 act on actor@2 (FUN_100afac0(2)),
  not the event actor: 1,377 / 53 sites name other NPCs (e.g. zone 280 at 0x0679).
- The door ops 0x4c / 0x4d / 0x4f / 0x8e / 0x8f (FUN_100b6950 ..) skip their +0x174 write while the event actor's
  +0x120 bit 2 (0xab 3) is set. +0x174 is one field: a door state and an event status replace each other (ours
  keeps them in w.doors.event and event_status, each write now removing the other).
- Op 0xb5 sub 0 (FUN_100b10f0 -> FUN_10093430) renames only when +0x210 (the Monstrosity word) is 0.
- Ops 0x2c / 0x2d / 0x51, 0x59 7 / 8, 0x71 0x50..0x55 (the credits roll, not a picture window), 0xac 0 / 4, 0x9c and
  the ops with no site in the event files: "Event VM audit fixes 2".
- Op 0xb3 is the Fishing Ranking, all subs done (2026-11-02); 0x9e writes a flag nothing reads.

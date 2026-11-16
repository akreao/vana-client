## Scheduler ops 0x7c / 0x7d: weather and clock (2026-11-15)

- FUN_10057fb0 case 0x7c (with a weather manager, DAT_1065cb04): FUN_101874f0(manager, 4cc @8, clock value of i16 @6,
  0) (asm 0x1005801e..0x10058067). With the last argument 0 a new tag kills the old weather's effects at once, then
  FUN_1018a7a0 starts the tag's set; i16 @6 is the env's blend in clock units (1500 a second, as op 0xae).
- Case 0x7d: FUN_1018c210(u32 @8): hour (FUN_1018bd00), minute, second and sub-second of the value are written into
  the clock DAT_1065cb1c (FUN_1018bf50 / bf00 / beb0 / be80), its day kept. Units: 60 a second, 3600 a minute,
  216000 an hour. Op 0x77 writes the same clock.
- Ours (fx::EnvOp, client/event.rs env_op): 0x7d sets the event's held clock (minute = value % 5184000 / 3600);
  outside an event it is not applied (ours runs on the server's time). 0x7c sets op 0x77's weather while the event
  holds one, else goes on the weather chain like op 0xae; the effects switch at once.
- Used by the Zilart ending's `sha0` (area 519): 0x7d 0 and 0x7c `fine` (blend 0) at the start, 0x7d 43200
  (00:12) before the camera `sha1`, so the woods go from 619's 0000 env to between its 0010 and 0020 keys (the
  explosion's lit horizon). Also `kon0` (516).

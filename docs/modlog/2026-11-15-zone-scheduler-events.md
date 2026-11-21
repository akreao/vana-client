## Zone schedulers run through the event runner (2026-11-15)

- Event op 0x2d (zone_sched) / 0x51 (its stop) start and stop a scheduler of the zone's (or event area's) effect file
  by 4cc, the same CMoSchedularTask as op 0x45 (sched_file). Ours started only its effects side, so its camera paths
  (op 0x04), fades (0x0f / 0x51) and crossfades (0x10), which the event runner plays for 0x45's, never ran: in the
  Rise of the Zilart ending (zone 163 event 7) the event-area scenes 516 (`kon0`, `lat0`), 517 (`mel0`) and 519
  (`sha0`) kept the last shot's camera, pointing at nothing (the "floating knights" were the cave scene's actors
  seen from that stale camera). Now 0x2d / 0x51 go through the same Scheduler / SchedulerStop path as 0x45 / 0x52.
- Live (Lnxcsfix, event 7 replayed; compared with the user's retail recording): the Ark and the Zilart crowd, the
  band of light over the purple plain, the Ark breaking, the 519 forest ruins all show as retail.
- Not done: scheduler op 0x7c (FUN_101874f0: the zone env's weather to the 4cc @8 over u16 @6) and op 0x7d
  (FUN_1018c210: the clock), which `kon0` uses: retail's first 516 shot has a lighter purple sky.

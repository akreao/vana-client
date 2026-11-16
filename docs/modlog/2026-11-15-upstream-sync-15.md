## TODO later: stale effect handles and the inferred notes (2026-11-15)

- User, after the Ru'Avitau cutscene showed bugs the opcode audit missed: mark these for later.
- Stale handles in fx: elements and instances are referred to by slot index with no generation check
  (`parent_elem`, `driven` and the like), so a reused slot hands a dead owner's role to another effect (the violet
  stars, "Effects: a burst's parent belongs to one update"). Add a generation check to every such handle.
- The `*(inf)*` notes in this file (683 of them): each is a guess that may steer code. Check each against the
  decompile, subsystem by subsystem, and fix or confirm it (the event-area sub-area rule was one, and wrong: "Ro'Maeve's
  spring in an event area: the sub-area trigger").

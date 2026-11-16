## Bursts: a burst-only generator stops again after each burst (2026-11-16)

- Found while chasing a user report (an effect on Nine kept flashing until a relog). Effect audit fixes 1 cleared a
  burst resource's stop before each emission-op-0x3c / end-op-1 burst (retail never soft-stops a resource that runs
  only for bursts). But ours also updates that instance every frame, and retail has no per-frame task for it: it emits
  only when burst (MODLOG "Emission op 0x3c ... run the named generator resource ... once with the new element as
  parent"; the per-frame generator task passes parent 0). Left un-stopped, ours emitted on its own interval between
  bursts, and its elements kept it alive, so it ran until the zone changed.
- Seen: Temper (file 2920) `stri/tog1` / `tog2` elements burst `gp03`; after 600 ticks the pre-audit build and the fix
  have no `gp03` left, the audit build still emits it (81 against 5 elements over the run; 62 with the fix, as every
  burst now emits).
- Now `burst` stops the instance again after the burst's emission when it is the burst-only resource (owned by an
  element), as well as after an end burst or the first one.

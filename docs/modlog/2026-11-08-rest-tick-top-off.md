## Rest ticks: a Regen / Refresh top-off keeps the heal held before it (2026-11-08)

client/resttick.rs `own_rise`. Gear and status Regen / Refresh are told from rest heals by their repeats: the same
amount rises again a look or two later (less when it fills the bar). Each rise is held 4.5 s for such a twin.
- Before: a smaller own rise that fills the bar matched any larger held rise through the full-bar rule, so the last
  rest heal before the bar filled was dropped as that top-off's twin and the rest's chain broke.
- Now: a twin drops only the amount it repeats exactly; a smaller rise that fills the bar drops nothing (the top-off
  itself is still not a tick). Test `a_heal_before_a_top_off_counts` (Refresh +1 every look, a +12 rest heal at 21.7 s
  leaving MP 1 short, the next +1 filling it: the heal counts, 13 in all).

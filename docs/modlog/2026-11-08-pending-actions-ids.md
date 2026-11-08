## Pending actions: answers matched to the ask (2026-11-08)

- Answers matched to the ask, not just its kind. A melee round (category 1) no longer answers a waiting
  spell, nor another action's start or finish: a 0x028 of ours confirms only with the asked id where it names one (a
  finish's param; a "ca.." start's first record value; abilities by their 0x01A param, the name-table id - 0x200;
  an interrupt's "sp.." names none). Items and monster skills (11, a mob skill id) keep the category alone. A
  battle message of ours is the refusal only if it is one of the request path's refusals for the kind (LSB's
  controllers and action states): a spell's 34, 35, 40, 47, 49, 71, 78, 155, 313 with its id, its recast's 18 and an
  interrupt's 16 with none; weapon skills 71, 89, 190, 191, 192, 216, 217 with none; job abilities 4, 5, 78, 87, 88,
  94 with none or the param; items 56 with none, 62 / 445 with none or the item id. Melee-side lines ("out of range"
  while a spell waits) and experience, wear-offs and the rest are no kind's refusal. Unit test
  pending_actions_match_the_ask.

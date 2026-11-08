## Pending actions: the game scripts' refusals (2026-11-08)

- The game scripts' own refusals clear the ask too. Using a spell scroll below its level printed "You
  cannot learn..." at once but the chip waited and then said the server had not answered: LSB's item, spell and
  ability checks (onItemCheck, onMagicCastingCheck, onAbilityCheck) answer with message ids of their own that were
  on no kind's list. Every id those checks return (followed through the helpers they call, xi.job_utils and the rest)
  is now listed, with the target check every kind goes through (12 already claimed, 155, 446, no id). Spells add 48,
  72, 235, 315, 316, 569, 581, 661, 700, 717 with the spell id, and 12, 155, 446 with none; weapon skills 12, 155,
  446; abilities 71, 72, 154, 155, 192, 199, 215, 216, 235, 307, 315, 316, 328, 336, 337, 347, 356, 428, 429, 446,
  523-525, 561, 574, 662, 666, 745 (a script's carries the ability as both values); items 12, 55, 78, 92, 111, 155,
  215, 246, 247, 308, 316, 356, 410, 443, 444, 446, 514, 517, 518, 523, 580, 759 with none or the item id, a scroll's
  95 / 96 (cannot learn, already known) whatever their value (the spell it teaches, which the item tables do not
  give), and the 0 a stopping status (stun, sleep) sends with the item id. The target check's "Target is currently
  blocking" comes as system message 225 (0x053), not a battle message: it refuses a spell, ability or item too,
  with no text (the mod API expands battle messages only). Unit test pending_actions_match_the_ask extended.

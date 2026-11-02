## Suggestion tests from recorded game states (ours, 2026-11-02)

- suggest.rs (plan G): `Facts` and `Suggestion` serialize; a `Case` is a game state with what the rules made of it
  (`rule`, `note`, `facts`, `expect`). Cases live in client/suggest_cases/*.json and `recorded_cases` replays each one
  against the rules (the whole list must match); `every_rule_has_a_case` checks each name in `RULES` (raise, shadows,
  ailment, curaga, cure, waltz, burst, close, provoke, job_ability, ninja_stance, samba, debuff, open_chain,
  strongest_ws, renew, between) has one.
- Recording: with `VANA_SUGGEST_RECORD=<folder>` the client writes each new non-empty set of battle-menu
  suggestions with its facts there (suggest-<ms>.json); fill in `rule` and `note` and move it under suggest_cases/.
  `VANA_SUGGEST_BLESS=1 cargo test suggest` rewrites the expectations from the current rules (check the diff).
- The first 17 cases are written by hand in the recorded format, one per rule (no game data in the build box); real
  recordings can replace or join them.

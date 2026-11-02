## The action registry (ours, 2026-11-02)

- client/registry.rs (`spell` / `ability` -> Info): valid targets from the DAT bits (spell +6, ability +0xa) named by
  the kinds target::SUB_FLAGS (table 0x103720c0) maps them to; spell range from +0x45 through table 0x103644a8 (255
  = none shown); the current target's flat distance when target::takes says the action may pick it; and the reason
  from magic_check / ability_check's group 7 line (FUN_1022cf30 / FUN_10224b70), else the effect magic::blocked
  dims by. Abilities carry no range in the DAT. The modern lists show it under the cost box. Not retail: retail only
  prints the refusal on Enter.

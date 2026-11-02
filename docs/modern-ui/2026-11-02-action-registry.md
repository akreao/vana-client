- The action registry (2026-11-02, ours; plan D): client/registry.rs gives, for a spell or an ability, its valid
  targets (the DAT bits by the prompt's kinds), its range (spell +0x45, table 0x103644a8), the current target's flat
  distance when the action may take it, and why it can't be used now (retail's group 7 refusal line from
  magic_check / ability_check, else the effect that stops it: asleep, silenced, amnesia ...). The magic and ability
  lists show it under the cost box ("Range 20 · Self, Party · Target 12.3"), the reason in red, and "Target out of
  range" past range + 0.5 (the prompt's red arrow). Range only warns: the row greys by retail's checks alone.

## Mannequins: the straight-down ground shadow, the range weapon (2026-11-08)

Settles MODLOG "Mog House mannequins, part 1" open points (DLL queue row 106).
- Ground shadow (root cause of the difference): FUN_100cbde0 asks the actor's vtable +0x404 before it picks the
  shadow light. CXiDancerActor's (FUN_100a4820) and the other classes' answer 0: the light is the point light's
  position minus the actor's (DAT_1048a308 type 1) or the directional light reversed. CXiMannequinActor's
  (FUN_100ab680) answers 1: the light is DAT_1048a32c = (0, -1, 0), straight down, whatever the zone's lights; then
  FUN_10033ea0 and the usual easing. Ours lit every caster from the floor's lights / the sun. Now
  shadows::casters takes the mannequin ids (World.furniture.mannequins) and render/shadow.rs `prep` skips the light
  pick for them (CasterGpu info.w bit 1). Test render::shadow::tests::shadow_shader_is_valid.
- The range weapon: hidden, as on a PC. The mannequin entity (status 0x31, kind 0) is in DAT_10480af0 and runs the
  entity update FUN_1008e9a0 (from FUN_10096640): once its model is in (FUN_100d12d0), FUN_1008c480's default case
  plays 'init' on the actor (vt+0x298, FUN_100cee40, the same for both classes), and a PC's 'init' runs hwpc (op 0x75
  hide wep2). FUN_100d0fc0 (the 'init' / 'con0' / 'con1' replay after a slot loads) is another path, gated on
  +0xa4 (0x25..0x34, 0x3d..0x53). Ours already starts the range mesh hidden (Character::init_hidden; mannequins load
  as Source::Pc). *(inf: str_sitchair_off was read for status 0x31 only far enough to see it reach the idle return)*
- Banks 3..31 for the other races: no code depends on it. FUN_100d1f70 adds the bank to the race's base file id and
  plays whatever mq0..mq7 that file has, as ours does (motionfile::mannequin_files).

## Title flyover: the movie zone's effects and their sounds (2026-11-16)

- User report: retail's title screen has sound effects (voices, it seemed); ours had none.
- File 23's schedulers hold no sound op (only 0x00 / 0x01 / 0x04 / 0x0e / 0x0f / 0x10 / 0x3b / 0x3d / 0x3e / 0x73 /
  0x7b..0x7e / 0xaf). Op 0x7b is FUN_1018b3a0(zone), the scene loader op 0x34 uses too (FUN_10178df0: file
  zone + 100), so a title movie loads its zone's whole scene: the auto-start generators run, their sound elements
  play (birds, wind, water, crowds). The world's ambient loop is not one of them: it plays only in the world
  (world+0x20 is 0 elsewhere).
- Ours: flyover.rs drew only the geometry. Now FlyoverData loads the zone's effect libraries
  (select::FxLibs::load_zone) and the Flyover runs them as the select and creator scenes do (select::LobbyFx): ticked
  on the movie clock (op 0x7d sets the minute), op 0x7c switching the weather's generators, drawn over the scene under
  the fades and blur, their sounds sent to the sound thread with voice bases TITLE_VOICES / NATION_VOICES (the
  creator's nation step flyovers, files 24..26, load the same way). Voices pause while their screen is not up and
  stop when a movie ends or the world starts.
- Checked: the title run's mov6 (Beaucedine Glacier, zone 111) starts 49 of its 163 generators and plays effect
  voice 0x3000001 (sound 2088); its level follows the camera's distance, as the world's do.

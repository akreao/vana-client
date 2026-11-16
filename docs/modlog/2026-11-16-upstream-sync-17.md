## A changed look: the old parts' effects go, `!in?` runs again (2026-11-16)

- User report: Excalibur's aura (main 545, Mithra file 65916: `!in1` -> `auon` -> the `lop0` loop of `gp00` / `fr00`)
  stayed when the sword was taken off and did not come back when it was put on again, until a relog.
- Retail rebuilds an actor whose look changes (FUN_10092900; MODLOG "A changed look without the blink"). The new
  drawobj starts with +0x840 bit 7 clear, so FUN_100973f0's next FUN_100cf060 runs `!in?` again; only FUN_100cf060 sets
  the bit and only FUN_100cf0b0 (`!kl?`) clears it. What the old model's files started on the actor goes with it.
- Ours keeps the actor through a look change (no blink, the user's choice) and never did that part: the old
  weapon's `!in1` loop ran on, and the equip state's `in_on` stayed set, so the new weapon got no `!in?`.
- Now when an actor's model files change (Effects::model_generators), the tasks and instances that the old model's
  files started on it go, and its equip state resets (reset_equip), so `!in?` and the weapon glows run for the new
  model on the next frame. Effects from other files (spells on it) stay. Test fx::equip::tests::look_change_restarts_in.
- Correction (same day): "the old model's files" was every file of the look, the race files too, and a cast's
  schedulers run from those (`shbk` / `ssbk`, Mithra file 23176). A gear swap mid-cast (precast / midcast sets) then
  killed the cast and the spell's `main` stopped before its effects: Fire I..IV showed nothing or only part. Now only
  what the old look's equipment part files started, and what files that left the look started, goes; the race and
  body files' tasks run on *(inferred: retail spells survive gear swaps)*. fx-shot's stand-in actors count as
  ready again, so it can run spells (category 4).

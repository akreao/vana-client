## Autorun follows the right drag's look (ours, 2026-11-07)

- Ours (modern MMOs, mouse only): while the right drag holds the facing (World::mouse_look, third person, no /follow
  or mod run), autorun's vector is the camera's forward each frame, so the character runs where it looks as with W
  held; A / D add their side as they do with W. Released, the run keeps its last way and A / D steer it 2 degrees a
  tick again (retail). Before, the run kept its old vector and the character faced the camera, running sideways.
  A run a mod starts with actions.run keeps its own heading. The controller and the keys are untouched.

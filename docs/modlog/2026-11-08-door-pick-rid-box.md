## Mouse picking: doors with a RID box, StreetLamp (2026-11-08)

- FFXiMain.dll (checked 2026-11-08): a door actor's setup FUN_100acba0(id) looks up the '_' RID record with its id
  (FUN_1017ad70 -> FUN_101747f0, records of 0xf4 with the 4cc @0x40) into +0x870; with none, and a name other than
  "StreetLamp", +0x874 = 1. Its pick rect FUN_100ad320: no record and +0x874 == 0 = not pickable; +0x874 == 1 the
  0.5-tall box at the door (already built); a record = the min / max of its 8 world corners (record +0x4c..+0xab,
  written by FUN_10173c10 from the unit box through S(|size|) RotY(yaw) T(pos)) through the view-projection. The pick
  FUN_10083750 then needs every corner in front (min w > 0) and takes the nearest; a door has no fine test.
- Ours: under_pointer finds the door's record in the zone's RID list by tag (zone::Region::box_corners), else the
  0.5 box unless the entity is named "StreetLamp". Test zone::tests::region_box_corners.
- The other "Not done" items of "Mouse picking in the world" and "Mouse cursor" checked 2026-11-08: the talk lock is
  in (event::talk_with refuses while World.control_lock is set, the "control lock" entry); cursor kind 1 over an
  enabled menu item is in (mouse_kind, GameUi::hover). Still open: "the hover's frame lag". FUN_10125360 runs the
  pick each frame (FUN_10126270 -> FUN_10158d90 -> FUN_10083750 with the current camera, FUN_10015250); the only lag
  found there is the pointer glide (+0x48 > 0: the drawn point moves half way to the cursor each frame), not traced
  to what starts it.

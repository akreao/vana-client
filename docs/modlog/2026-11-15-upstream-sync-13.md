## Ro'Maeve's spring in an event area: the sub-area trigger (2026-11-15)

- User report, with retail footage: Yve'noile's scene in the Shrine of Ru'Avitau (zone 178 event 53, params 917 1408
  1550) shows Ro'Maeve (op 0x34 #122) with a hole in the floor where the spring is, and violet stars and a violet band
  around Yve'noile when she appears.
- Ro'Maeve's scene (DAT 222) places 61 `id_*` names whose MMBs are 64-byte stubs: a bounding box and 0 pieces.
  `id_f_a01..d01` (mirrored) are the spring's ring and horns, `id_cath_min` / `id_cath_hnd` the cathedral's spire and
  hands, plus `id_m_*`, `id_room_p1` / `p2` and others. A scan of all 72,377 MMBs in the install found no other copy
  of any of them, and retail's MMB loader (FUN_1016fcd0: version > 1 takes the piece count) draws nothing for 0 pieces.
  They stand in for sub-area geometry: RID records `m3e0` (param 518, two boxes over the centre), `m3e1` (522) and
  `m3e2` (523). Sub-area 518 (file 618) holds the spring's ring and horns, the arch and the cathedral front.
- The per-frame trigger (FUN_10014390's caller, current decompile): while the zone renderer is up, the player actor's
  position (vtable +0x1bc) -> FUN_10181540 -> FUN_10178a20(1, id). There is no event-area check: it runs against the
  scene shown, an event's area too. In this event our actor's position falls inside 518's trigger, so retail loads
  518 with the area.
- Ours took only an event's op 0x75 requests while an area showed (an inference, now dropped). The trigger now runs
  there too (client/mod.rs update_sub_area), and only an unload an event asks for (FUN_10178900) sends no 0x0F2
  (subarea::Step::Unload carries it).
- Live (Lnxcsfix, the event above): 518 loads with the area; the spring's ring and horns, the arch, the cathedral spire
  and the honeycomb gate in the near rim match the retail footage at the same camera.

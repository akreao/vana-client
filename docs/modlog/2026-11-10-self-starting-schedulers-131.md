## Self-starting schedulers: the weather in the zone-in call, sub-areas, op 0x64's slot, op 0xa2 (2026-11-10)

Modlog-gaps item 131 ("Self-starting schedulers: whether the weather object runs in the zone parse's frame; sub-area
weather; which emission op fills update op 0x64's slot", MODLOG "Self-starting schedulers: end loops, the load-time
start, op 0x52 time windows, op 0xa2, update op 0x64"). Addresses are Steven's FFXiMain.dll, read in the
decompile and the disassembly.

- The weather object comes in the parse's call, with no task walk between. Zone-in is one function (FUN_10094190,
  run once when DAT_1048559c is set): FUN_1018b3a0(zone) -> FUN_1018b500 -> FUN_10178df0 reads and parses the zone
  file (FUN_10072fd0 -> FUN_10073430), whose link pass FUN_10056a50 starts every self-starting scheduler through
  FUN_10056b80: the task is built (FUN_10056ce0, countdown 0) and its update (vtable 0x1032b860 +0x20 = FUN_10057e10)
  is called at once by FUN_100748c0, which only swaps the running-task pointer (manager +0xd4c) around the call.
  Then FUN_1018b6a0 soft-stops every generator under the zone's `weat` folder (FUN_10053890(0): flag 0x800 off, the
  clone told to stop) and builds the environments (FUN_10186d00 -> FUN_10186c60 -> FUN_101873a0). Further down the
  same function sets the weather twice (FUN_101874f0: the previous tag with its times, then the current one with
  times 0, which, when the tags differ, deletes the first object and builds another). The progress marks in between
  (FUN_10253610) only log.
- The weather object's constructor FUN_1018a7a0 (unchanged from the earlier trace): the environment's folder, its
  `weat`, the tag's folder or `suny`; every task of every scheduler under `weat` is deleted (FUN_10056eb0 walks the
  whole task list +0xd50 for 'Ms' tasks of the resource, copies included), then the chosen folder's self-starting
  schedulers start and step at once, and FUN_10072540(5, 0, 0x1018aa50) starts the folder's 0x1000 generators with
  +0x64 = the object.
- What the one step inside the parse does. FUN_10057e10 adds an op's wait and then runs the op, so the first op
  always runs, and the ones after it while the waits are 0. Over all DATs (census
  `dat::sched::census::auto_first_step_census`): the 232 self-starting `weat` schedulers (thdr 100, suny 55, fine 49,
  clod 13, dryw 8, bolt 7) all open with op 01, op 0x52 and nothing else. Op 01 has no case; op 0x52 parks the
  task and may make one copy (FUN_10074920, placed after the running task), and the constructor deletes source and
  copy before the walk reaches them. So a `weat` scheduler of another weather shows nothing at zone-in, and ours
  (start_auto, then set_weather before the first tick at every load: no step at all) matches. The 813 others open
  with op 01 and then start generators or a child (op 02 / 03 / 0x0a) or park on op 0x52; ours runs that step at
  the next tick.
- Sub-area weather. FUN_10178900 parses the sub-area file the same way (its self-starting schedulers start in the
  parse) and FUN_10186d30 builds its environments. FUN_101873a0 looks the new environment up in the chain
  (FUN_10188900) and, when its predecessor has a weather object (+0x10), calls FUN_101874f0(predecessor's tag, 0, 0)
  on itself: the sub-area takes the zone's weather at once, and FUN_101874f0 passes later changes down the chain
  (+4). Its constructor works on the sub-area's own folder, so a sub-area `weat` folder would switch like the
  zone's. No file has one: of the 259 sub-area ids the zones' RID records name, none has a `weat` folder (census
  `dat::sched::census::sub_area_weat_census`); the constructor then falls back to the first `weat` it finds
  (FUN_10072290) and returns before the scheduler and generator passes. Nothing to build.
- Update op 0x64's slot is filled by emission op 0x87, and the update keeps it going. Emission case 0x87
  (FUN_1004e8d0): S[k..k+2] = the element's position (+0x54..+0x5c), S[k+3..k+5] = 0. All 175 generators with
  update op 0x64 have op 0x87 on the same slot and no other writer (census
  `dat::generator::tests::update_64_census`: pigeons, eagles, kites, ducks, d_ri/effe/kumo, the if_h / bif_ files).
  The update case 0x1004b249 does more than was noted: after the rotation it writes the slot (0x1004b40f..0x1004b44a):
  S[k..k+2] = the position, S[k+3..k+5] = d, the step it just measured, and sets +0x187. So d = this frame's move,
  a = -fpatan(last step z, last step x), and t = fpatan(d.x sin a + d.z cos a, d.x cos a - d.z sin a) is the turn
  from the last step to this one; each non-zero factor gives rot x / y / z = t x factor, wrapped once (above pi
  minus 2 pi, below -pi plus 2 pi; constants 0x1032a3b8 / 0x1032a3b4 / 0x1032a3b0). The birds bank into their turns
  (factors -10 pigeons, -20 eagles, -30 kites and ducks, on x). The first update after the emission has a zero last
  step, so t is the heading itself for that one frame (fpatan(0, 0) = 0); a frame with no move gives 0, or +-pi when
  the x term comes out as -0, and the frame after it the heading again. The turn is per frame, so the bank halves
  at twice the frame rate, in retail as here.
- Ours before: the update read the slot and never wrote it, so t was the angle of (position - start position) and the
  bank a meaningless roll that grew along the flight. Now render/effects.wgsl case 0x64 stores the position and the
  step (and `fpatan` gives x87's answers for two zeros), fx/elem.rs `update_ref` has the same case.
- Who starts the op-0xa2 schedulers: the model actor. FUN_100c4410 (the file hook of XiModelActor, vtable 0x10330aa0:
  from its constructors FUN_100c41b0 / FUN_100c41f0 and the model swaps FUN_100c4380 / FUN_100c43c0) picks the MMB
  ('fune', '_fun', else the first), and on the generic branch calls FUN_100c46b0: for every type-7 chunk of the file,
  FUN_10056e50 (an end loop or an op 0x52) and not FUN_10056e80 (so it holds op 0xa2) -> FUN_10056c70(actor, actor,
  0, 0), stepped at once. Op 0xa2 marks "the actor starts me on itself, not the parse". The 32 schedulers (census
  `dat::sched::census::furniture_auto_census`) are all op 01, 0xa2, 0x52 and then the show: the clock 62933
  m_ev/toke (st00 / st12: two minutes from 00:00 / 12:00, generators 8075 / 8202), the bells 62870..62872 (10:00 /
  13:00 / 16:00 for an hour), the chimes 51555 m_ev/suzu (18:00..02:00), the piano 51503 (22:00), the music box 33015
  m_ob/effe (00:00 / 12:00), the windows 33123 m_wa/san1 (06:00 / 18:00) and 33126 m_wa/win1 (03 / 09 / 15 / 21:00),
  the showcase 62927 m_bt/show (06:00 / 18:00 on and off). Ten of the files are furniture-table models (formulas
  +0x809f, +0xc7ef, +0xe75b), none in the skinned range 0x108..0x12b. 1627 hana/b_ha/han1 is an NPC model: no
  XiModelActor shows it, so nothing found starts its scheduler.
- Ours: dat::sched::furniture_start; Effects::on_furniture / stop_furniture (the start waits for the file, then
  spawns on (actor, actor) and steps); Mog House pieces (client/furniture.rs apply) and prop model actors
  (client/ships.rs create and the op 0xb6 0x11 swap, generic MMB branch only) call it. A model swap stops the old
  file's tasks first (FUN_100c42f0 -> FUN_10082b10 drops everything linked to the actor; ours only these).
- Tests: render::effects::tests::gpu_bird_bank (DAT 202 `hat2`: the slot holds the last position and step, roll =
  -10 x turn, 120 frames), fx::tasks::auto_tests::furniture_clock_strikes (62933: nothing from the parse; on the
  actor two parked tasks, at 12:00 a copy that starts 8075; none at 11:00; the actor's end takes all).
- Not done: laying the furniture out again restarts its schedulers here (the whole set gets new actors; retail's
  actors for pieces that stay were not traced), so a clock re-laid inside its window strikes again. The bank at
  frame rates other than 30 was not compared with retail footage. Not seen in a Mog House by a person yet.

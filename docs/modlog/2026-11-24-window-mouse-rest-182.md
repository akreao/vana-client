## The mouse on windows: the release buzzer, itemctrl's cursor and the generic windows left (2026-11-24)

<!-- dll-build: 30261102_0 -->

Modlog-gaps item 182 (split from 173). Addresses are 30261102_0, decompiled with Ghidra; FUN_1015f420, the hover tail
and FUN_101199b0 checked in Rizin.

- The left release (FUN_1015f420, from FUN_10160fd0 on FUN_10124f00(0x3e, 0x8b, 5)) runs only in mouse mode 2 (a
  window has the focus and the pointer shows, FUN_10126750) with no press on a scroll bar (+0x86, set by the press on
  FUN_10235e00). It hit-tests the focused window's children (FUN_1011a140, no frame test): the cursor's child gets
  message 5, another child nothing, and no child at all plays SE 0004 (FUN_10037c50: sound 0x3d "0004") unless mouse
  +0x87 is set. +0x87 is set while rem4line / rem4li2 is open (FUN_10137e70, cleared by FUN_10137eb0) and while the
  map's marker cursor is up (FUN_101fa130 from mapv2's Place, cleared by FUN_101fa1a0). A drag started outside the
  window is mode 5 (FUN_10126600: 0x41 square pixels), not mode 2, so its release plays nothing.
- itemctrl (vtable 0x10334e78, onkey FUN_1013bac0) and moneyctr (onkey FUN_10141fc0) move no cursor on the keys
  (messages 1..4 change the count), but the mouse moves it like any window's: the hover's new hit and the held button
  call FUN_1011a240, which sets +0x4c and sends message 9 (SE 0001: neither window clears +0x85). Message 5 then acts
  on the cursor's child, from a click or from Enter: child 1 confirms, 2 / 3 re-send 4 / 3, 4 / 5 re-send 1 / 2. A
  window opens on child 1 (FUN_101193d0 -> FUN_1011a320(1)).
- iuse (vtable 0x1033b6d8, onkey FUN_101f4970), sortyn (0x1033d148, FUN_10223ec0), bluesibo (0x1033a168,
  win_bluequip_onkey 0x101e1210), map0 (0x1033b9b8, win_mapframe_onkey 0x101f9270), mapscan (0x1033b928,
  win_mapframe_onkey_3 0x101fd140) and mapv2 (win_mapv2_onkey 0x101fda10) handle messages 5 / 6 only (sortyn also
  refreshes its owner's help on the others, FUN_1013b720): their mouse is the generic window's. None of them clears
  +0x85, so the hover's cursor moves play SE 0001. Their disabled children (state 5: iuse's Use / Drop, map0's Wide
  Scan, mapscan's Track, mapv2's View) are hit like any other.
- Only the focused window takes the mouse (FUN_101267f0 hit-tests the active window alone), so the item list under
  iuse, sortyn or itmsort2 takes none.
- The loop at the hover tail's end (over FUN_101603f0, every window but the focused one) calls FUN_101199b0, which is a
  bare `ret`: the pointer moves no focus there. Message 8 comes from each window's vtable +0x28, which FUN_10160fd0
  calls every frame (1 for the focused window DAT_10622a44, 0 for the others): e.g. win_equip_v28 (0x10134030) sends
  message 8 to itself when focused with the pointer in its frame (FUN_10127770: mode 2; FUN_1011a0a0).

Ours:
- `winhits::release_off_child` is the release's test; world_hud plays SE 0004 on a left release in the classic UI
  unless an event message is open (rem4line) or mapv2's Place cursor is up (`MapWin::placing`), the release ended a
  drag (65 px² from the press as retail's mode-5 entry, or a camera look; `World::release_drag`), or the pointer is on one of ours' own windows (a mod's
  window, Timers, Regime, Skillchain: the hand over it). It takes any child a window's mouse took this frame
  (`winhits::takes`) as a hit as well, for the windows ours draws without a record.
- itemctrl / moneyctr keep their cursor in `World::ctr_hold` (by window and open stamp, `GameUi::open_stamp`): the hand
  is drawn on it, the hover and the held button move it (SE 0001), a click on it is its message 5, and Enter while the
  window has the focus is turned into that child's key (`winhits::ctr_enter`). The help bar shows the cursor child's.
  Before, only the held button moved a stand-in child and the hand stayed on child 1.
- iuse, sortyn (over the item list, Auto's in itmsort2 and the storage view's), bluesibo, map0, mapscan and mapv2 take
  the mouse by `winhits::generic`; their disabled children are hit at their layout places (`winhits::record_all`).
  The item list takes no mouse while iuse, sortyn, itmsort2 or the yes / no prompt is over it. The map's windows take
  no release that ends a drag (the map's pan, retail's mode 5).
- Ours' own: a classic mod window owns the mouse over the game's windows under it (not over the game windows drawn
  above the mods, not in cinema mode, not while a scroll bar holds a drag).

Not built (new queue rows 192..195): jbpcat's page arrows 11 / 12; the mouse on mapv3 (rows and arrows 11 / 12),
scanlist, maplist, itmsort2 and the storage view's lists; message 8 by the windows' +0x28 hooks (ours still moves the
focus by pointer moves in equipwin, bluset, pupset, treasure, mannequin, macroedit and esedit); the lobby screens'
release buzzer and the held step's window frames there.

Tests: `client::winhits::tests::disabled_children_are_put_back`. Not checked on screen or live.

## The mouse on the map's and the item windows' other windows (2026-11-27)

<!-- dll-build: 30261102_0 -->

Modlog-gaps item 193 (split from 182). Addresses are 30261102_0, decompiled with Ghidra; FUN_1011a140, the hover tail
(0x10127220..0x1012735a) and win_itmsort_v28 FUN_101f4ca0 checked in Rizin.

- The list base (win_common_onkey_3 0x101f8280, its frame hook FUN_101f7fe0 at vtable +0x28) is the class of
  scanlist (vtable 0x1033ba60), maplist (0x1033bb18), bank (0x10339ed0) and the item list inventor (0x1033b720). The
  window's handler FUN_101196f0 runs first; the list base clears the window's +0x85, so it plays no SE of its own.
  - Message 9 (FUN_1011a240 sends it on a new cursor child while the pointer shows): a hit child (mouse +0x52) at or
    below the count (+0x20) plays SE 0001 when it is not the last cursor (+0x24); above the count it clears the hand
    (mouse +0x4c). Then the tail (0x101f855b): an entry index (FUN_101f8710: top + child - 1) at or past the count puts
    the cursor on the count's child (FUN_1011a240(count - top)), so the pointer on an empty row leaves the cursor on
    the last entry, with no SE.
  - The frame hook FUN_101f7fe0 clears the hand every frame while the focused list's hit child is past the count (or
    none) in mouse mode 2. The hover tail sets the hand from FUN_1011a140's out flag (a hit child not in state 5) at
    0x1012735a, after the hover's FUN_1011a240, so case 9's clear alone would not last.
  - The +0x58 hooks: win_scanlist_v58 0x101fd060 and win_maplist_v58 0x101fe3a0 do nothing with 9; win_bank_v58
    0x101de150 and FUN_101f4180 (inventor) pass it to the view's handlers (FUN_101dcc60 / FUN_101dd4a0 in the storage
    view), which ignore it. Message 5 (the release on the cursor's child) is the list's Enter, 6 its Escape.
- mapv3 (vtable 0x1033b970, win_mapframe_onkey_4 0x101fd680): its open (win_mapframe_v04_4 0x101fd280) clears +0x86
  (no SE 0002 from the handler) but keeps +0x85, and puts children 11 / 12 (the set arrows, links 11 11 1 1 11 11) in
  state 0xf, which FUN_1011a140 hits like any other. So:
  - the mouse moves the cursor onto rows 1..10 and onto the arrows with SE 0001, and so do Up / Down (they move by the
    links, FUN_1011c6b0: messages 1 / 2 read +0x18 / +0x17, so the rows wrap and both arrows lead to row 1);
  - message 9 does nothing, but while a name is typed (+0x38) it ends the entry (FUN_101fd450(1): the text into the
    slot, FUN_101fa440), and so does message 6 then;
  - message 5 on 0xb / 0xc re-sends 4 / 3 (the set before / after, as Left / Right); on a row it acts on it; while
    typing, a row is the entry's Enter (FUN_101c7ef0(1); its done callback FUN_101fd390 sends 0x11, the entry's end)
    and an arrow its caret keys (FUN_101c7ef0(4) / (5)).
- itmsortw and itmsort2 (both vtable 0x1033b690, handler FUN_101f5020: message 5 on 1 Auto's prompt, 2 Manual, 3 the
  switch / the bin; 6 / 8 close) and itmstora (0x1033b648, menu_itmstora_fn_2 0x101f5430: 5 switches, 6 / 8 close)
  are generic windows: the base's SE 0001 on a cursor move and SE 0002 / 0004 on message 5 (+0x86), a child in state 5
  passes nothing on. Their frame hook FUN_101f4ca0 puts Manual (child 2) in state 5 while the item list's cursor entry
  (FUN_101f26f0) is a temporary item (FUN_100eb630 = 3), back in state 0xb on another container, unchanged with no
  entry. In the storage view the inventory side never holds container 3.
- The map's disabled children (map0's Wide Scan, mapscan's Track, mapv2's View) are state 5: no hand over them.

Ours:
- `winhits::list` is the list base's mouse: a row with an entry takes the cursor (SE 0001), a child past the count
  puts it on the last entry without SE and shows no hand, the click on the cursor's row is Enter, a right click on the
  window Escape. scanlist, maplist, the item list, and the storage view's focused list (bank, or the inventory side's
  inventor moved to x 200; read-only, the container in inventor) use it. The item list took its rows from the window's
  width at an 18 px pitch with no count test and no right click; it now hits the layout's children.
- mapv3 takes the mouse (map.rs `mapv3_pointer`): children 1..12, the arrows' Enter re-sent as Left / Right, Up / Down
  from an arrow to row 1, SE 0001 on Up / Down (ours played none), the name entry ended by a hover move, a click on a
  row or a right click. The modern Markers list shows row 1 for an arrow cursor.
- itmsort2, itmsortw and itmstora take the mouse by `winhits::generic`; itmsort2 shows Manual in state 5 (its kind-4
  sprite, no hand, Enter SE 0004) on a temporary item; itmstora's Enter on an enabled row plays the base's SE 0002
  (ours played none).
- The map keeps the children it draws disabled by hand with its windows (`MapWin::drawn`); `GameUi::mark_disabled`
  takes them after the frame's clear, so the pointer over them shows no hand.

Not traced: which of the hover (FUN_101267f0, run from data) and the frame hooks (FUN_10160fd0) comes first in a
frame; the held button's move onto a child past the count (FUN_1015f360) would then clear the hand that frame only.
The arrows' caret keys while typing (ours' name entry has no caret). mapv3's +0x34 bit 2 (cleared by its open outside
event maps) and whether message 6 closes it through the base (FUN_1015fd60) rather than its own case 6.

Tests: `client::winhits::tests::list_moves_and_clamps`, `client::map::marker_tests::mapv3_set_arrows`,
`client::map::marker_tests::rename_ends_with_the_text`. Not checked on screen.

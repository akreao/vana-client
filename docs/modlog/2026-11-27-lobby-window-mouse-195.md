## The lobby's window mouse: the release buzzer and the windows' frames (2026-11-27)

<!-- dll-build: 30261102_0 -->

Modlog-gaps item 195 (split from 182). Addresses are 30261102_0, decompiled with Ghidra; FUN_10241450, the
dbnamese open and the registry entries checked in Rizin.

- The lobby's screens are windows of the same window manager as the world's: FUN_1015f920(name, 1, 0) opens each with
  the focus (`raise_focus_next_frame`), and FUN_10160fd0 runs the left release FUN_1015f420 on them (key 0x8b's mask
  0x3e passes there). The title opens loby1win then loby2win (FUN_10241ce0), so loby2win has the focus; character
  select opens chfwin then dbnamese (FUN_10244c00), then gives chfwin / chswin the focus for the options and the
  slots; the prompts (ptc6yesn) and menu ok open with the focus; the rename opens chmkname. The creator
  (FUN_1023ad30) opens chdummy and then each step's window (chmkrace .. chmkjobs, chmkname, chmktown); the world step
  opens chmkname and chmkserv, then worldsel (menu_worldsel_fn 0x1023ecd0, moved by FUN_1011a5a0), and chmkpass for
  the pass.
- The release (FUN_1015f420) is the world's: mouse mode 2 (the focused window not opening, +0x10 != 0xe, and the
  pointer shown), no press on a scroll bar (+0x86), FUN_1011a140 on the focused window's children; no child hit plays
  SE 0004 unless mouse +0x87 is set, which only rem4line / rem4li2 and the map set. FUN_1011a140 takes every child not
  in state 0x10 (window +0x77 is 1 from the constructor FUN_10118750): the prompt's warning icon (child 3), the creator
  steps' page arrows (chmkrace 9 / 10, chmkhair 3 / 4, ...), chmkname's two children, chmkpass's, and all 14 worldsel
  rows (the list base FUN_101f8630 / FUN_101f7cd0 puts no row past the worlds in state 0x10) are hits.
- A left drag stays in mode 2 in the lobby: mode 5 (FUN_10126600) is the 0x8c button's drag (the 0x8b one only with
  FUN_1025f680, DAT_10663924 +0x44 == 2) and needs the creator (FUN_1023aac0, DAT_1066a19c) or game states 0x60 /
  0xd0; mode 4 (FUN_101264d0) needs state 0x60. So a left drag's release over no child buzzes there.
- The held step (FUN_1015f360) wants the pointer in the focused window's frame (FUN_1011a0a0: window +0x3a .. +0x40,
  both ends in):
  - loby2win's frame is its layout frame, 143 x 260 at the window's place; the open FUN_10241450 rescales only the
    children, so the held step never reaches the title buttons (the hover still moves the cursor).
  - dbnamese's open (FUN_10242f20, vtable 0x10336b18 +4) gives the window a 640 x 480 frame (FUN_1011ba30).
  - chfwin, chswin, ptc6yesn, ok, worldsel and chmkpass: their frames as drawn.
- loby2win's child boxes (FUN_10241450, 0x1024151b..0x1024157d): each child's box (FUN_10119d90) scaled by
  (W / 1024, H / 768) with `__ftol` of the scaled left / top and width / height. FUN_10119d90 is asked with the
  cursor's nudge: child 1, the cursor at the open (cursor_start under registry flag 0x10000) with window +0x63 = 1 from
  the constructor, is read a pixel up-left first. The buttons are drawn at these places (FUN_10241640), and the hit
  test adds each child's header offset (0, -3).

Ours:
- `winhits::LobbyFocus` is a lobby screen's focused window as drawn: its frame and its children's hit boxes (header
  offsets in). `winhits::lobby_release_off` is the release's test; the title, character select and the creator play
  SE 0004 on a left release over no child of it (classic UI, the pointer shown, no request under way).
- The title takes the mouse on loby2win's child boxes (`screens::title_child_boxes`) instead of the buttons' sprite
  extents; the held step tests loby2win's frame.
- Character select: the list's held step tests dbnamese's 640 x 480 frame (was the grid's sprite bounds); chfwin,
  chswin, the prompt and ok test their frames (were the child boxes) and are hit with the header offsets (Yes / No sit
  3 px higher than ours had them).
- The creator's steps, worldsel, the confirm and ok test their frames; the ok window takes the release, not egui's
  click.

Not built (new queue rows 229..232): the hover over the creator steps' page arrows and the worldsel rows past the worlds
(FUN_1011a240 moves the cursor there, message 9; the step onkey FUN_10238d40 returns on 9 / 10 and takes message 5 on
9 as back, on 10 as next); the prompt's icon under the hover; the lobby popups (the login error and the select
screen's error ok windows) and lobycwin's release; where retail takes the title buttons' centring without expansions
from (ours draws and hits them 100 layout px right; FUN_10241640 draws at the child boxes, so the shift would have to
move them: not traced).

Tests: `client::winhits::tests::lobby_release_buzzes_off_the_children`, `title_boxes_scale_the_layout`. Not checked
on screen or live.

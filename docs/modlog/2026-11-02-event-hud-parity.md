## Modern UI: event HUD parity with retail (2026-11-02)

- Retail check (FFXiMain.dll, 30261006_0, Ghidra): an event does not hide the whole interface. The party window class
  (vtable 0x1033b078) has no event check in its draw (FUN_102041d0) or update (FUN_10203c80); FUN_10203c80 takes
  only raid1 / raid2 down while FUN_10087070(1) holds. The target window goes unless EventPara2 bit 1 keeps it, and
  the compass closes (both already in our vanilla HUD). The log, party window, status icons and top bar stay. This
  holds for the new character opening (0x1f7) as for any event.
- The modern layer now matches: the target panel and the focus / last-target panel follow `event::keeps_target`, and
  the location panel (the compass's stand-in) is hidden while an event runs. Chat, party, status icons and the top
  bar stay, as in retail; the alliance panels, minimap, timers and favourites strip were already hidden.

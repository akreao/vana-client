## Ctrl / Alt shortcuts: the common test (2026-11-17)

Addresses are 30261006_0's.

- The shortcut records (built in FUN_101ae140) run each frame in the field (FUN_101ae3e0), each through FUN_101adf20:
  the press edge, then its condition set (FUN_101ae010, the set number from the record's +0x100):
  - Set 1, A "/attack" (0x3b) and H "/heal" (0x3d): no event hold (FUN_10087070(0)), plus the common part.
  - Set 4, E Equipment (0x36), I Items (0x39), C check / talk (0x3f) and M / J / W / B (0x41 / 0x7f / 0x80 / 0x9f):
    no event hold, the UI shown, no window with flag 0x20000 (FUN_1015ecc0), plus the common part.
  - The common part: no sub-target prompt (T +0x59, +0xc4; T = DAT_1057876c), not dead (own status not 2 / 3), no
    Num+ focus (T +0x68 == 0: neither the log nor the status icons), not in Mog House layout mode (DAT_106631d4 +0x1c
    == 0).
- Ours checked M / J / W / B only for an event, death and the prompt, and A / H / E / I / C for nothing.
  - Alt+A under a forgotten prompt sent /attack. Then the battle menu opened at the engage and cancelled the prompt.
  - The cancel put back the prompt's saved target and lock: the monster that died before. It was dropped at once
    with the lock (user, 2026-11-17).
  - The prompt in that run was most likely the engaged battle menu's Switch Target (Enter on its first row) *(inf:
    from the key log)*.
- All of them now share one test (mod.rs shortcut_ok): no event, alive, no prompt (or party cursor), no Num+ focus
  (log_active, log_focus != 2), no layout; E / I / C / M / J / W / B also no 0x20000 window (World.main_open). The
  UI-shown part needs nothing: with the UI hidden no key arm runs.
- egui-winit turns Ctrl+C into a Copy event before any key event, so the Copy arm takes the same test.
- Opening a prompt is now logged with its callback and the saved target and lock.
- Seen live: Lnxcsfix with a `/ma "Cure" <st>` prompt up: Alt+A sent nothing; after Escape it sent /attack. Ctrl+C
  under the prompt checked nothing (after Escape: "examines you"); Ctrl+M over the map opened nothing (after Escape:
  the magic list).

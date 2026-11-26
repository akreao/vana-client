## Focused status icons keep the d-pad from the target (2026-11-26)

<!-- dll-build: 30261102_0 -->

- With the status icon list focused (key function 0x78: Num+, Ctrl / Alt+F, the pad's top button), the d-pad's
  Left / Right moved the icon cursor and also stepped the target (key functions 0x2a / 0x29), and Up / Down stepped
  the party target (0x27 / 0x28). The pad's d-pad is both the arrows (the window state makes it a menu) and the pad
  word, and the field command table only waited for windows other than the icon list.
- Retail runs that table from FUN_10159bc0 (30261006_0: FUN_101585f0) only with the sub-target prompt up or no window
  holding the keys (+0x74 clear and the input lock count DAT_1057876c +0x6a below 1); in retail the focused list keeps
  Left / Right to itself (seen). Ours: `field_table_open` counts the focused list as such a window.
- Full keyboard: key functions 0x27..0x2a have no key bindings (pad and compact keyboard only), so the arrows never
  reached the target there.

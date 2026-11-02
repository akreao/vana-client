## Right-click menus (ours, 2026-11-02)

- Retail has no context menus; its right click on the world cancels a sub-target prompt, clears an unlocked target or
  stops autorun (target::click, FUN_101585f0 result 0x17). With the modern UI on, a right click that did not drag and
  lands on an actor (Self::under_pointer, the left click's own pick) opens the modern layer's menu for it instead;
  anywhere else, or with a prompt up, retail's right click is unchanged.
- The menus only reach existing paths: the item list's keys (iuse Use / Drop, itmsort2 Auto-sort, with their
  questions), the storage view's 0x029 move and its rules (items::usable / tint, away from the moogle), the text
  commands (/check, /attack and /pcmd add take a server id in Internal mode, FUN_1007a1c0), /focustarget <pN>, the
  chat line's "/tell name " (charlnk's Tell) and trade::start_on.

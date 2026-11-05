## Escape with no target opens the main menu (Modern UI, ours) (2026-11-05)

**Deviation from retail.** With the Modern UI on, the keyboard's Escape opens the main menu (as Minus does, on the
page it was closed on) when there is no target and nothing else for Escape to close: no window, prompt, action
menu, chat line or event. Retail only clears the target and takes a line off the log (keys 0x92 / 0x95); that is
unchanged when a target is up, in the classic UI, and for the controller's cancel slot (`World::pad_esc` marks a
pad Escape so it never opens the menu). `client/mod.rs`, the Escape arm of the world key match.

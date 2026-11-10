## Check while engaged: Ctrl+C and the battle menu's Check at once (2026-11-10)

Issue lenonk/vana-client#176.

- Ctrl+C (FUN_10130280) checks the target at once, engaged too: that function has no prompt. It takes the target
  (FUN_10152f20), does nothing while the inspect window is up (+0x74) or for the same id again within 2 s, talks to
  an NPC with +0x13c bit 0x400, and checks a player or a monster (+0x120 bit 13) with c2s 0x0DD. Ours went through
  the battle menu's Check and so opened the sub-target prompt while engaged.
- The battle menu's Check (30261006_0 FUN_1012b880 cases 9 / 10) opens the sub-target prompt over monsters
  (FUN_10159440 with LAB_1012bec0, flags 0x20) when the player actor's +0x170 is 1, else checks at once. Ours checks
  at once there too (user, 2026-11-10), and Monstrosity's Check likewise; `SubThen::Check` is gone.
- Seen live (Lnxcsfix, classic, West Ronfaure): engaged on a River Crab, Ctrl+C and the menu's Check each gave "The
  River Crab seems too weak to be worthwhile." with no prompt; the help bar kept the Check row's help.

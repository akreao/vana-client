
## Set Bazaar: the glyph and the Monstrosity row already match

Batch 7 item 30 traced both halves of the Set Bazaar row, and ours already match:

- The bazaar glyph **0x9c** is the last entry of `FUN_10097500`'s chain before the default, so every earlier match
  (away, GM, Campaign, recruiter, party seek, Level Sync, mentor, New Adventurer, POL, link dead, Monstrosity, team)
  wins over it — `plate::main_glyph`'s order.
- Its bit (actor +0x128 bit 9) has exactly two writers: s2c **0x00D u32 @0x20 bit 31** and s2c **0x037 u32 @0x2c
  bit 29**, which is what `net.rs` reads. This corrects MODLOG L9987, which had 0x037 at @0x20 bit 31 as well.
- The row is disabled in Monstrosity by `FUN_1014e720`'s `FUN_10118b60(5, 5)` at open, gated on `FUN_10097d50`
  (char data +0x40db6 != 0 or main job 23). `menu_disabled_on` already returns child 5 for the social page in
  Monstrosity, and the row's own handler has no second check.

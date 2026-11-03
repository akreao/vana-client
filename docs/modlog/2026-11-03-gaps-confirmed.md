
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

## Status window: the stats request retries are our send gate's job

Batch 7 item 28 traced persona's frame `FUN_10147030`: while the request has not gone out it counts frames down and
calls `FUN_100fc3c0(0x61, 0, 0)`, which allocates the c2s 0x061 through the **1000 ms per-id gate**. Refused, it
counts a try and sets the counter to 20 frames; after 12 tries it gives up silently, with no busy strip. On a
successful send it sets the busy bit 0x40 on statcom2, which the 0x061 answer (`FUN_100994a0` -> the stored
`LAB_10147010`) clears.

Our side reaches the same place from the other end: the send gate holds a refused 0x061 in its queue and releases it
as soon as the per-id window opens (the RETRIED ids), so the request always goes out within a second, `status::open`
shows the busy strip until `on_stats`, and retail's give-up path has nothing to trigger it. The only difference left
is that our strip appears at open rather than on the frame the packet leaves, which is at most that one second.

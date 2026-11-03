
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

## Op 0x53 has no motion length to read

Batch 8 item 38 settles what op 0x53 waits on. There is **no length field**: `FUN_100b4e10(0, 0)` calls
`drawobj(actor1)->vt+0x2a0(4cc, drawobj(actor2))` = `FUN_10084d40`, which is busy when

1. the model's files are not all loaded (vt+0x1e4 -> `FUN_100d12d0`), or
2. the actor's motion bank +0xb4 exists but is not ready, or
3. a live scheduler task exists for the 4cc and that actor pair: `FUN_10057010` walks the task list for +0x1c ==
   `'Ms'`, +0x80 == the resource's id and the two actors equal.

If no scheduler resource is found for the 4cc anywhere (the actor's own files, a skeleton actor's bank, or the global
bank), it is **not busy** and 0x53 advances at once. So "how long the motion plays" is only how long that task runs:
until its op stream ends or something kills it. Nothing reads a frame count.

Ours waits while the named motion is still playing, which is the same thing for a motion started by 0x5b / 0x66 (the
scheduler is what drives it), and is the nearest equivalent in a client that plays motions directly. The headless
host's 30 frames are a test stub, not a claim about retail.

Item 38 also corrects MODLOG on 0x5f sub 7: `vt+0x2ac` (`FUN_10084fb0`) walks the actor's attached list for an element
of type 0x1032cfa0 whose +0x78 is the 4cc, so it waits on that object, not on a scheduler. Our timer started by
0x5f sub 5 / 6 stands in for that object's lifetime.

## Op 0x7a on our own actor: the server does see the pinned place

Batch 8 item 40 closes the inferred link ours was already built on:

- The pin writes the **drawobj**, and the per-frame entity update `FUN_1008cd90` copies drawobj +0x34..+0x3c into
  entity +4..+0xc once the model is loaded, plus the rotation for the player.
- The c2s 0x015 builder `FUN_100983f0` sends that entity copy (position @4/@8/@0xc, the yaw byte @0x14), and 0x015 is
  never gated for our own id, so **the server receives the pinned position and facing every frame**.
- When the pin ends (`FUN_10061f50`) the drawobj is left where it was, so the player stays there and the next 0x015
  carries that spot.

Ours sets `world.pos` / `world.yaw` from `Effects::pinned` each frame, before the 0x015 send, and leaves the position
alone when the pin expires, so no change was needed.

## Scheduler ops 0x8c / 0xa4 / 0xa5 write the entity's base names, as ours do

Batch 9 item 41 names the fields behind the behaviour ours already models. The entity keeps a **base** set (+0x190
idle, +0x194 sit idle, +0x198 walk, +0x19c run, +0x1a0 chair idle) and a **current** set (+0x1a4 / +0x1a8 / +0x1ac
idles, +0x1b0 walk, +0x1b4 run):

- Op 0x8c writes the current idles **and** base +0x190; 0xa4 writes +0x1b4 and base +0x19c; 0xa5 writes +0x1b0 and
  base +0x198. Op 0x79's battle name goes to the drawobj only.
- `FUN_1008a1a0` recomputes the current set from the base set on every status, mount or appearance update, and
  `FUN_1008f740` copies it to the drawobj. So the op's name comes back instead of idl / wlk / run until
  `FUN_1008a160` resets the bases (entity init, or the +0x130 bit 4 path, which is op 0xab).
- A mount (`FUN_10087dc0() == 2`) ignores the bases and uses the fixed dft / dft / df1 / ark / hsr set; sitting and
  chair idles come from +0x194 / +0x1a0, which the ops never write.

That is exactly `World::my_motion_bases` / `Entity::motion_bases` with the `rebase` recompute, the MonStat 2
override, op 0xab's reset flag and 0x79 going only to the model, so nothing changed. Our sit and chair idles come
from the status motion table and are likewise untouched by the ops.

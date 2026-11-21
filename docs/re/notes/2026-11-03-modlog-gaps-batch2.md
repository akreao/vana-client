# MODLOG gaps, batch 2 (RE notes)

Copied from Steven's `/run/media/SSD/FFXI/re-notes/modlog-gaps-batch2.md` (2026-11-03). Target and tools as in
[batch 1](2026-11-03-modlog-gaps-batch1.md) (sha256 8097dbf8…550c). All addresses are VAs in this DLL. **(inf)** =
inferred. Event handlers resolve through the table at 0x100bc960 (`jmp [op*4+0x100bc960]` at 0x100bc290). The VM loop
FUN_100bccd0 clears +0x25a, dispatches, and loops while +0x25a == 0 (1 = yield this frame).

## 1. Event ops 0x46 sub 2, 0x69, 0x6a, and op 0x1f's units

Entries: 0x1f -> FUN_100b2ea0; 0x46 -> FUN_100b6440; 0x69 -> FUN_100b7a60; 0x6a -> FUN_100b7af0 (old FUN_100b5c30 /
5cc0).

### Op 0x46 (FUN_100b6440, sub table 0x100b6674)
- Gate: the player actor +0x120 bit 9 (0x100b6454..0x100b6460). Clear: every sub is 2 bytes and does nothing.
- Lengths with the gate set: 0 = 2, 1 = 2, 2 = 4, 3 = 8, >= 4 = 2.
- Sub 2 (0x100b6541..0x100b6567): `FUN_100af310(2, DAT_10351220 == 1)`: var@2 = 1 while the normal follow camera is
  on. A query. DAT_10351220 starts at 1; only FUN_10084210 sets 1 (also puts the camera back on the player: focal
  280.0 if DAT_10487f80, else config 0x49) and only FUN_100842d0 sets 0. FUN_1001ee60 / FUN_10020ba0 / FUN_10021110
  return at once while it is 0 (inf: follow-camera updates).
- Sub 0 = follow camera on: FUN_100ae9b0(the first-person flag saved at event start), FUN_1003d5c0 (kills camera
  tasks, inf), FUN_10084210, FUN_1001e780 / FUN_10021680.
- Sub 1 = follow camera off: FUN_100ae9b0(0), FUN_100842d0, FUN_10221c30 on DAT_106671b8 (hides a HUD part, inf).
- Sub 3: as sub 0 with FUN_100ae9b0(0), plus a point (val@2/@4/@6 x 0.001) in DAT_10489920..2c, latched only the
  first time (DAT_104897ea bit 0, never cleared).
- vana-client: behaviour matches (`Effect::Camera { flag: true }` frees the camera behind the player); only
  event_ops.rs's wording "event camera on" reads the other way round.

### Op 0x69 (FUN_100b7a60): sound volume, 4 bytes
Level = 1.0 if u8@1 == 0, else 0.0. Mask = val@2.

| Bit | Calls | Sets | Channel |
|---|---|---|---|
| 1 | FUN_10036020 (kills fade DAT_10463928) | DAT_10351704 = cfg x level | 3D world sound effects (inf) |
| 2 | FUN_10036070 (kills fade DAT_1046392c) | DAT_10351700 | interface sounds ('0001'..'0039') |
| 4 | FUN_10035f80 (kills fade DAT_10463930) | DAT_10351708 | zone ambient loop (DAT_10463910) |
| 8 | FUN_10035f00 | all the above and the music stream | stream volume = cfg x level x 127, or stopped at 0 |
| 0x10 | — | DAT_10351710 = level | mute for the chat `<call>` sounds (FUN_100380f0) |

### Op 0x6a (FUN_100b7af0): timed volume fade, 7 bytes
Target = val@1 x 0.001; duration = val@3 in 1/60 s ticks; mask = val@5 (1 -> kind 2, 2 -> kind 1, 4 -> kind 4, 8 ->
kinds 1 / 2 / 4 / 8; 0x10 ignored). Each fade is a 0x48-byte task (FUN_10035a00); tick FUN_10035940 `rem -= ticks`,
`level = start + (target - start) x (1 - rem / total)`, deleted when rem < 0; the destructor applies the target.

### Op 0x1f walk_to (FUN_100b2ea0)
- Sub 0, 8 bytes: target = val@2 / @4 / @6 x 0.001 (x, z, y). No yield. Also FUN_1008af20, and with +0x120 bit 9
  model vt+0x29c with 0x306c7440 / 0x30727440.
- Sub 1, 2 bytes, advances only on arrival: heading atan2(-dz, dx); step = FUN_10014cf0() x speed (+0x168) / 60; snap
  and advance when step >= distance (or with no zone resource). Yields every frame; actor +0x124 |= 0x20000.
- Sub >= 2: hangs (inf: unused).
- Ground snap after every sub unless +0x128 bit 14 or +0x120 bits 20 / 21 (FUN_10181290 player / FUN_10181300 others,
  50.0).
- Units: coordinates in thousandths of a world unit; no duration; FUN_10014cf0 = the frame's elapsed 1/60 s ticks
  (at least 1.0); speed in world units per second, set by op 0x32 (val@1 x 0.1).

## 2. Scheduler task end: when a task restarts

- 0x10056c70 here is a task-create wrapper (FUN_10056ce0 allocates 0x14a bytes; speed 0.7 and +0x142 = 1 when the
  parent's resource +0x1c > 1 and its kind isn't 0xe / 0xf).
- The end check is FUN_10057c20 (old + 0xfb0), called by the tick FUN_10057e10 only when the current op is the end op
  and the countdown is 0; true -> +0x145 = 1 (finished). FUN_100575a0 also calls it.
- Restart when any entry of the resource's list C ([R+0x78], fixed up by FUN_10056a50) starts with byte 1 (walk:
  stop at byte 0, step `((dword >> 8) & 0x1f) x 4`), or when the loop flag +0x141 is set. Op 1 inside the op stream
  does not count.
- FUN_100573e0 restarts: speed 1.0, parent (+0x84) and action dropped, ip = stream start; +0x141 kept.
- **FUN_10057c20 never looks at child tasks.** (Child waiting elsewhere not searched.)
- vana-client's `fx/tasks.rs` already does this (`dat::sched::loops`, `Task::looping`, the restart in `step`).

## 3. Name plates during events

- MODLOG's FUN_100ce4d0 is old-build. The plate draw is FUN_100d0510 (vt+0x284 in all 7 drawobj vtables): one path
  for PCs, NPCs and monsters. The pass FUN_100817f0 has no event check.
- FUN_100d0510 skips: first person for the ridden / followed actor (inf); +0x8ac bit 0; +0x128 bit 3; drawobj +0xa4 in
  1..4 or 0x54; model id 0xb22..0xb31; /names (DAT_1047d694 hidden; DAT_1047d695 0 none, 4 "otherpc"); FUN_100830c0
  fails (off screen, too deep, own plate in first person, drawobj +0xb2 != 0, DAT_104568fc+8 == 0).
- The event-aware part is the text, FUN_100977f0: no name unless +0x120 bit 9 and not +0x128 bit 3. Bound to the
  running event (+0x120 bit 7 and DAT_104807f5): shown only if +0x12c bit 16 (event op 0x92, FUN_100b9ba0) and +0x120
  bit 17 (script hide, ops 0x22 / 0x4e) are clear. Otherwise +0x120 bits 14 / 15 and +0x130 bit 7 must be clear. Both
  need FUN_100973f0 (InvisFlag, unbound actors only) == 0.
- Non-participants lose the model (and the plate): FUN_1008e4d0 per frame, for actors without +0x120 bit 8, not
  bound-and-running, +0x138 >= 0: unload bit 23 from bits 14 / 18, from (DAT_104808fc | DAT_1047fb40) & 0x02 for PCs
  (player included) and & 0x10 for everything else. DAT_1047fb40 = event Mode. Exempt: +0x130 bit 3, doors /
  elevators (+0xee 3..5), FUN_100961d0. Bound and running: hidden only by bit 17.

## 4. Top bar netstat during events; item overrides +0x4c / +0x50

- netstat (DAT_10663acc) has no event check; event start does not close it (its registry entry A 0x00008093, B 0x205
  takes neither 0x800 nor 0x1000 from FUN_1015ef60). It goes only with the whole UI: cinema mode (rem4line +0x48,
  event op 0x67 or a 0x00A zone-in event with Mode 0x80; cleared by op 0x68, event end, zoning), UI hidden (key
  function 0x56), or the 2D pass off.
- Window item record: +0x40 help (group 0xe) / +0x44 title (group 0xd) from data, +0x4a enabled, **+0x4c help
  override, +0x50 title override**. Reader FUN_101186a0 (focused cursor item only): help = +0x4c ?: +0x40 ?: header;
  title = data title non-empty ? (+0x50 ?: +0x44) : header title. Writers FUN_1011adb0 (+0x4c) / FUN_1011ade0 (+0x50).
- Title overrides: menuwind init FUN_10140090 item 12 "Mog House" (0xd 9, help 0xe 0x2a) in a Mog House outside
  Monstrosity, else "Map" (0xd 8 / 0xe 0x28); Monstrosity item 2 "Customization" (0xd 0xa5 / 0xe 0x2bc), item 5 help
  0xe 0x2b5. myroom FUN_10145240 item 6 "Close Mog House" (0xd 0x94 / 0xe 0x29b) when G+0x40c40 != 0, else "Open Mog
  House" (0xd 0x93 / 0xe 0x29a).
- Help-only overrides: menuwind item 12 (0x2a / 0x29); myroom item 3 (0x2b); K.O. window (0xe 0xf0); Monstrosity
  retry / end; the macro window; the treasure pool (9 0xc2); FUN_102003f1 (7 0x73); auc1 (0x43 / 0x44); link4 /
  link13 (0xe 1 / 2); prty2 / 4 / 8 and msglist (0xe 0xf); scoption; acoption (0x41c).

## 5. s2c 0x00E SubKind 5 / 6: the 0x9000 model word

- MODLOG's FUN_100d0ef0 is old-build. The model-load switch is FUN_100d2f60 (kind +0xee, table 0x100d40dc): 0 / 1,
  2, 6, 7, 8 have cases, 3 / 4 / 5 default.
- SubKind 5 (0x1009cd38..0x1009cda1): +0xee = 6; word = u16 @0x32 | 0x9000 (OR, not replace). On a change slot 9
  (+0x10e) = word, +0xf5 |= 2; then if the dirty mask +0xf4 != 0, slot 0 (+0xfc) = 0.
- SubKind 6 (0x1009cda2..0x1009ce4f): +0xee = 7, the same; raw @0x32 in 0x7b8..0x7cb or 0x7d3..0x7da -> +0xf6 = 2
  (not traced).
- Event op 0xb6 sub 0x0c (0x100b1493): +0xee = 7, slot 9 = the value without | 0x9000, +0x130 |= 0x20000.
- Decode: bits 0..11 = model id, always the NPC table (FUN_100c51c0(0, id): < 1500 -> 1300 + id; < 3000 -> 50295 +
  id; < 3500 -> 96907 + id; else 98239 + id). Bits 12..15 = the look slot (FUN_100d0b50: `slot = (w >> 12) & 0xf`):
  9 = the whole-model slot.
- Slot 9 attach (FUN_100d0b50): kind 6 with race 0 -> primary model, race != 0 -> +0x864 (attached, inf); kind 7 with
  slot 0 == 0 -> primary, else +0x864. Kind 6 with a race then takes the PC race path. Kind 7 with slot 0 loads the
  base first (FUN_100c51c0(2 if +0x130 bit 21 else 0, +0xfc, or +0x166 in costume)), then slot 9 attached at +0x864.

## 6. Own actor +0x124 bits 22 / 23

Bit 22 = Away, bit 23 = Anonymous. MODLOG's "0xc6" is message 7 0xc6; there is no s2c 0x0C6 handler.
- Writers: actor init (0x880, both clear); zone-in FUN_10094190 from G+0x40db0 bits 3 / 4 (s2c 0x00A u32 @0x20 bits
  12 / 14); s2c 0x037 u32 @0x28 bits 5 / 7 (FUN_1009d0f0, the normal path); s2c 0x00D mask 4 @0x20 bits 12 / 14; s2c
  0x00E mask 4 clears bit 23.
- Readers: /prcmd add (0x1007da8b..0x1007dab7): either bit -> 7 0xc6 "You were unable to make a request to join.", no
  c2s 0x11C (the name-lookup path LAB_1007dc00 does not test them); name colour FUN_1008db40 (bit 23 -> anon colour
  DAT_104e1ba4); name glyph FUN_10097500 (bit 22 -> PC glyph 0x90, the away icon).

## 7. rem4line, the event-page mark

- The mark is log row byte +0x12: 1 = a row of the open event page (FUN_101273c0, modes 0x98 / 0xbe); 2 = a held chat
  line. FUN_10129290(threshold) zeroes +0x12 on rows whose serial (+0xc) >= threshold, marks 1 and 2 alike.
- rem +0x50 is always 0 (written once at 0x101367d3 from a zero-filled header dword), so the close (FUN_10136ab0),
  Escape while DAT_10485a70 == 1 (FUN_10136c30) and a 0x4000 page advance (FUN_10136800) clear every row of the owner
  log (window 1 for rem4line, 2 for rem4li2), held chat lines included.
- The ordinary page advance (FUN_10126a90) passes queue entry +0x10, from a print header field nothing writes
  (0x10127d1f..0x10127d57: stack garbage, inf).
- Held lines caught into the spool (FUN_10162560) -> FUN_101371e0(serial), mark cleared.
- The page arrow (FUN_10136b40): never on page 1; on later pages only with queue +0x20 bit 0 (`7f 31`, flags 0x8001);
  none on a page with more than 4 rows left.

## 8. Select screen: Escape, and "Connecting to the server..."

- The lobby prompt is "menu    ptc6yesn" (DAT_10669004, handler FUN_1023d050), not comyn. Enter -> result = cursor -
  1. Escape (0x1023d14c..0x1023d16e) answers No only while +0xa5e1 is set; openers clear it, only FUN_10164580 sets it
  (when +0xa5e0 == 1, only from FUN_10243f10: types 0xb / 0xc, the PlayOnline settings-file prompts). The log-in
  (0xd), rename (5), delete (4) and creator prompts ignore Escape. hnyesno (FUN_1023f700): Escape -> -1 with SE 0003.
  ptc9dele (FUN_1023d2e0): no Escape case.
- "Connecting to the server..." has no retail source (DLL ASCII / UTF-16 / Shift-JIS, FFXi.dll, FFXiResource.dll,
  polboot.exe, 39541, all ROM DATs). The nearest is group 8 id 3 "Connecting to lobby server." (lobby step 1,
  FUN_100ed7e0 at 0x100edada). Group 8 neighbours: 0 "Searching for lobby server.", 1 "Initializing connection.", 2
  "Resetting connection.", 5 "Setting up connection.", 0xb "Connection to lobby server was lost.", 0xc5 "Disconnecting
  from lobby server.".

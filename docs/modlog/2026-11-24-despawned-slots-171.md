## A despawned actor keeps its slot (2026-11-24)

<!-- dll-build: 30261102_0 -->

Queue row 171, left open by row 147 (`2026-11-14-map-session-actor-rules-147.md`, "Not done"). Ours dropped an entity on
a despawn; retail keeps the actor's slot. client/mod.rs (`despawn`, `new_entity`, the `NetEvent::Entity` arm),
client/event.rs (`parked`, `unloaded`, `first_waiting`, `holds`, `unbind`).

**The slot's life.** The actor table DAT_10481ca0 (0x900 slots by index) gets a slot from the 0x00D / 0x00E handler
when the index has none (FUN_1008a120 at 0x1009b57a / 0x10099b05), even for a despawn. Nothing frees a server
actor's slot but the zone's teardown (FUN_10093b70 at 0x10093bad, FUN_10093720); the other writers of 0 are the
client-made event actors (FUN_100ab170 / 100ab320 / 100ab550, FUN_100c3e70). A slot is wiped in place
(FUN_1008ee30 + FUN_10092900 + FUN_1008a350(0)) when another id takes the index, by FUN_10099a40's sweep, or by a
partial update after a despawn (+0x145 = 0).

**The despawn** (0x00E 0x1009b7e4..0x1009b945, 0x00D 0x10099c6a..0x10099cd2), not taken for a reserved actor
(+0x138 bit 31, row 147):
- 0x00E with the packet's SubKind 5 / 6 / 7 (u8 @0x30 & 7): +0x120 bit 0 off, +0xef = 0 and +0x12c bit 6 off (the
  race op 0xb7 set, ours `race`), +0xf4 = 0 and the 16 look words +0xfc..+0x11a = 0 (0x1009b841..0x1009b87d). Ours:
  `race` and `appearance` go, and `look_gone` keeps the freed model's look from being made again: only a look an
  update brings makes it.
- 0x00E: +0x124's sub-animation fields back to 0x880 (all despawns; ours did this only for a held actor).
- 0x00D: +0x1fa = 0 (the character's pet index; no field in ours on this base).
- Both: the mark +0x124 bit 12, +0x145 = 0, and the hide bit +0x120 bit 14 unless the actor takes part in an event
  (+0x120 bit 8). Ours: `Entity::despawned`, `awaiting_full`, `flags |= HIDE`.

Any later update for the slot clears the mark first (0x1009b7de / 0x10099c07); the rules of row 147 follow (a partial
one resets the slot and asks, a full one is applied onto the kept slot, a reserved actor takes any).

**The frame pass** FUN_10096640, every frame:
- 0x100968b0..0x100968f9: a slot with the mark, not bound (+0x120 bit 7), not taking part (bit 8), not ours and not
  client-made (+0x130 bit 3) gets FUN_1008ee30 (a no-op without bit 7) and FUN_10092900: its model (+0xa0) and the objects hung on
  it (+0xa4 / +0xa8 / +0xcc / +0xd0, FUN_10092760's ten) go. The slot, with its name, flags, place and (but for
  5 / 6 / 7) look, stays.
- 0x10096a4a: FUN_1008e9a0 (the actor's move, its status machine FUN_1008b460, and the lists the loader FUN_10095900
  walks) runs for no such slot, so no model is made while the mark stays.

Ours: `event::parked` (despawned, no event holds it, not ours) joins `event::unloaded`, so `unload_models` frees the
model, keeps its look in `World::unloaded`, and makes it again once the mark is gone (the update that cleared it).
The per-frame passes skip a parked slot: the move loop, chairs, rests, doors (their state goes, so they start again as
a new door), lifts, ships (the model goes). The update that brings a slot back places it as a new model is placed (no
slide from the old place; the floor snap of FUN_10091220, which is in the model maker FUN_1008f740).

**Readers of the mark** (all test it with "not bound and not taking part"):
- FUN_10098210 (target / radar / follow test, 0x100982e9): ours `target::last_target`, `radar::dots`.
- FUN_1007a4c0 (`/target` by name, 0x1007a561): `textcmd`.
- FUN_1007bd80 (/returntrust): ours reads the party rows, not the slots; nothing to do.
- s2c 0x05A's copy to the trusts (0x100a09fe): not built in ours.
- FUN_100aeea0 (the event start's walk, 0x100af0c6..0x100af0db): a slot that exists takes part (bit 8) first, then a
  marked one or one with +0x145 = 0 is asked for again (FUN_100aec70) and waited on. Ours (`first_waiting`): `slot`
  and `present` apart. Correction in passing: ours counted an actor waiting for its full update as present.
- Lookups by id (FUN_100afb00 -> FUN_1008b380) test only that the slot holds the id, so the event VM finds a parked
  actor (`LiveHost::ready`), op 0xac sub 2 reserves it, and an emote line names it.

Ours-only readers (mods' entity list and name checks, the chat's near-call names) leave parked slots out, as they
left despawned entities out before.

**Events.** A despawn while an event holds the actor no longer drops it at the event's end (`EventState::despawned`
is gone): the mark stays, and the frame after the event lets go the pass frees its model. `event::holds` is now bit 8
as retail has it (bound in a running event, or passed by a waiting event's walk); `unbind` (FUN_1008ee30) also takes
the actor out of a waiting event's walk.

**Op 0xac sub 2 on a despawned actor** (Mog Garden): the slot exists, so it is reserved. The server's next update
then only moves and names it (row 147's reserved view), clears the mark and keeps the hide bit; with the reserve the
event's filter does not take its model, so it is made again from the kept look, and the script shows it.

**Checked.** Unit tests `despawn_keeps_the_slot`, `parked_slots`, `despawned_participant_is_missing`. Not checked
live.

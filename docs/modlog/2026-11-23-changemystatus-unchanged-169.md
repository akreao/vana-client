## PlayOnline: ChangeMyStatus sends nothing that changes nothing; status 0x72's body (2026-11-23)

<!-- dll-build: 30261102_0 -->

Modlog-gaps queue row 169 ("PlayOnline ChangeMyStatus in polcore 1.18.12: sends nothing when no value changes after
one went through; status 0x72's 0x20-byte body; the handle list's store 0/8"), left open by row 146. polcore 1.18.12
(the one installed beside FFXiMain 30261102_0) decompiled with Ghidra, the branches below checked in Rizin; FFXiMain
30261102_0 and the Viewer's app.dll searched for the slots' callers.

**ChangeMyStatus (slot 0x30c, FUN_10019b90 -> FUN_10019bc0, poll FUN_10019f70).**
- FUN_10019bc0 keeps a "something changes" flag at the transaction's +0xde. It starts as `0x10104dc0 == 0`
  (`sete` at 0x10019c33), where 0x10104dc0 is set at the end of every ChangeMyStatus that went through (state 6) and
  cleared only by the session reset FUN_10018d60 and FUN_10018d00. Each argument polcore takes (content, character
  >= -1; flag 0 / 1; presence 0..4; activity 0..0x3ff; p5 >= 0) sets it when it differs from polcore's value
  (0x100c99d0, 0x100c99d4, 0x10104dc4, 0x100c99d8, 0x100c99dc, 0x10103340).
- FUN_10019f70 state 1 (0x10019fce): flag clear -> state 6 and FUN_10018480(txn, 0), result 1. Nothing is connected
  or sent, polcore's status is not touched, and the session value (0x10103348, slot 0xfac) and the authCode
  (0x1045cc70, slot 0xea0) stay as the last real reply left them. So byte 0x18 of the body ("all kept" after a
  change, LSB's `bNoChange`) is never sent by 1.18.12.
- Arguments are checked before anything is built: content above 0x3f -0x1c16, character above 0x3f -0x1c03,
  character or content below -2 -0x1c16, flag other than 0 / 1 / -2 or presence outside 0..4 / -2 -0x1c11. Then
  presence 3 is refused (-0x107) with no content, or for a handle whose byte at 0x10103358 is 1; only app.dll sets
  those (slot 0x13d0, FUN_10018c60; FFXiMain's thunk has no caller) and LoadHandleList clears them.
- After a change that went through, FUN_10020f70 updates polcore's own copy of our entry in its friend list cache
  (0x1044716c, dirty bits); local only, nothing sent.
- What this means for FFXiMain's status job (FUN_100f1a10 sends the object's block in force, then the pending one;
  docs/modlog/2026-11-11-friend-status-zone-in-140.md): the block in force is read from polcore's own getters
  (FUN_100f78d0: slots 0x2f8, 0x2f4, 0x330, 0x338, -1, 0x33c, which return exactly the values above), so the first
  request changes nothing and polcore sends nothing for it. On the wire a status change is one ChangeMyStatus.

**Status 0x72 (-0x14c2, "bound to an Xbox profile").** The only status any polcore transaction treats apart; only
ChangeMyStatus (state 4 at 0x1001a09b) and LoadMyStatus (FUN_10019860 state 4) do. Both still read the reply's body
with its checksum, 0x20 and 0x18 bytes (a bad checksum is -0x140f), and keep its byte 0 and dwords @8 / @0xc
(0x10104db8 / da8 / dac) for slot 0x718 (FUN_10018b50). Neither FFXiMain (its thunk FUN_10311ec6 has no caller) nor
app.dll calls slot 0x718, so nothing reads them. ChangeMyStatus then returns -0x14c2 without ending the transaction
(0x1001a130: no FUN_10018480), so the cached server (0x1045bf78) stays; LoadMyStatus ends it with the code, which
clears it.

**Every failed transaction forgets the cached server.** FUN_10018480 with a negative code calls FUN_10018b10, which
clears the 0x14 bytes at 0x1045bf78. Ours cleared it only on a refused reply header; now on every failure (the
connect, the handshake, I/O, a bad checksum), but ChangeMyStatus's status 0x72.

**The handle list's store (0/8).** FFXiMain does run it: lobby step 3 subs 0x12 / 0x13 (FUN_100ee2a0, thunk
FUN_1031160d = slot 0x2d4 -> FUN_100191b0, poll slot 0x2d8 -> FUN_10019300), after a registration's character
store. FUN_100191b0 runs the order pass FUN_10019240(1) and counts the handles marked dirty (0x1045bf20); with none
and 0x10103344 clear, state 1 goes to 7 unsent. Only the handle edits mark them: create FUN_10019670 (slot 0x314),
delete FUN_100195d0 (0x318), set FUN_10019030 (0x2b8), swap FUN_1001a300 (0x320). FFXiMain calls none of them but
0x2b8, from the "oplev" menu nothing opens (row 140); the Viewer does. The order pass renumbers only where two
handles share an order, which LoadHandleList's own pass (FUN_10019240(0), FUN_1001a480) has already undone. So in
the game the store never sends; the old note "nothing in the game changes a handle" holds, though the call is made.
Not built, as it sends nothing (a comment in `Profile::register`).

**Ours.**
- `polpro::change_my_status` returns `Ok(None)` when polcore sends nothing, `Err(code)` for refused arguments;
  `Status::start` / `Status::loaded` hold polcore's starting status and its LoadMyStatus reading (one place for both
  sessions). `polpro::transact` reads 0x72's body (`Bound`) and clears the server on every failure.
- PlayOnline sign-in (`net/pol`): `Profile::change_status` keeps the session value and returns it when nothing is
  sent; the logout's `(-2, -1, -2, -2, -1, -2)` is not sent when no character was active.
- LSB's xi_profile (`client/friends.rs`, `net/profile.rs`): ChangeMyStatus went out as a fixed body (content 0, flag
  0, activity 1, open status 0 = keep, no byte 0x19) and the status job sent two requests for each change. Now the
  session keeps polcore's side (`PolSide`: LoadMyStatus's status, the handle and character lists' tables) and every
  ChangeMyStatus goes through polcore's rules with FFXiMain's arguments: the active-character call `(its handle, its
  entry, 0, -2, -1, 1)`, after which the status object's block in force is read from polcore (`in_force`); the job's
  two requests (the first not sent); the return from the viewer's `(in force, flag and status from polcore, -1)`.
  LoadMyStatus is asked with polcore's 0x14-byte zero body, and only after the IRC sign-on, the Viewer's order (chat
  sign-in, then LoadMyStatus): xi_profile reads the saved open status at the sign-on and answers its default, Online,
  before it, which the active-character call (presence kept = polcore's) would then have written back over an
  Invisible. A failed active-character call builds no status object (retail stops at lobby step 3); ours tries it
  again 30 s later and holds status changes until it goes through. The friend, handle and character lists still load
  at the start, apart from the sign-on, so a failed sign-on leaves them in. A dropped IRC session is a new PlayOnline
  session: polcore's status starts over (FUN_10018d60), LoadMyStatus is read again at the next sign-on and the
  active-character call made again (xi_profile clears the active character at each sign-on). Our open status after the loads is polcore's presence
  (4 when LoadMyStatus says none).
- Tests: change_my_status_sends_only_a_change, change_my_status_refusals, load_my_status_reply,
  an_unchanged_status_is_not_sent, bound_refusals_read_their_body, a_change_is_sent_once.

**Not done, not checked.**
- Not run against PlayOnline (no server) nor live on LSB; LSB's xi_profile reads the same bytes (found / slot / flag
  / open status / bNoChange) and answers ChangeMyStatus the same whatever the body says beyond those.
- An LSB character past the handle's 8 slots is unlinked (LSB attaches only the first 8 to its one handle): polcore
  finds no slot and sends "not found", so LSB shows that character as not in game. Retail would register it to a
  handle with a free slot first and, with none, stop at lobby step 3 (-1); the fix is on LSB's side (more handles).
- ChangeMyStatus's 0x72 leaves retail's transaction slot busy with its socket open until it times out (100 s) or is
  cancelled; ours closes the connection.
- LoadMyStatus's other kept fields (the byte at 0x76 -> 0x10104da0, dwords @8 / @0xc -> 0x1010339c / 98) are
  outside this row and were not traced to their readers.

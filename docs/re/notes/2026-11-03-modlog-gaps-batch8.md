# MODLOG gaps, batch 8 (queue items 31..40)

Target and tools as in modlog-gaps-batch1.md (`~/ffxi-pad-re/FFXiMain.unpacked.dll`, sha256 8097dbf8…550c). All
addresses are VAs in this DLL. **(inf)** = inferred. Queue: modlog-gaps-queue.md.

## Item 31: Linkshell Concierge — a focus change between link5 and lscswin (gaps L234, MODLOG L15593)

**The hand-off is a deferred raise-and-focus request, handled next frame by the window manager; its only visible
effects are the hand cursor moving, the pointer warping, and the list's help line. Nothing else changes in either
window.**
- **Who asks:** link5 msg 8 (Num+, or the pointer over lscswin) → FUN_101e4440: `FUN_1015db80(lscswin, 0)` + SE 0013
  (FUN_10037e90). lscswin's select FUN_101e49e0: msg 6 (Escape) and msg 8 → `FUN_1015db80(link5, 0)`, **no SE**; msg 5
  (Enter) → FUN_101e4980: link5 +0x78 = {child 1: 0, 2: 1, 3: 2, 4: 5} (jump table 0x101e49cc), then
  `FUN_1015db80(link5, 0)`, no SE. (The filter change shows on the next row rebuild: link5's rows FUN_101e3730 read
  +0x78 every frame.)
- **FUN_1015db80(win, flag)** (manager 0x10621838): win +0x71 = manager +0x60 (a request order number, then +0x60 += 1),
  win +0x98 = flag. Nothing happens yet.
- **Next frame, FUN_1015fa00** (the manager's per-frame, 0x1015fbd8..0x1015fc80): for order n = 1 .. +0x60 − 1, the
  window with +0x71 == n (not closing, +0x6e == 0):
  1. FUN_1015dbd0 unlinks it, FUN_1015db10 re-inserts it at the front of its layer (+0x60 layer < 5: just behind the
     first window of layer ≥ 5; else at the very front) → it draws on top of its layer;
  2. +0x71 = 0;
  3. if its +0x34 bit 0 is clear, or the flag (+0x98) is set: **FUN_1015e850(win)** makes it the focus now, and
     FUN_1011abf0 / FUN_10124700 warp the pointer to the centre of its cursor child (when the pointer is on screen).
  Then the counter is reset to 1 and **the focus = FUN_1015e7e0()**: the first window in the list that is not
  (+0x34 bit 0 and +0x98 == 0), not +0x6e, not +0x70 and not type 0xe — i.e. normally the one just raised.
- **FUN_1015e850(new)** (focus change): the old focus: **FUN_10119200(0)** (manager +0x74 bit 0 cleared and FUN_1011a560:
  its hand hidden), its +0x98 = 0, then its handler's vt+0x24(0). The new one: **FUN_10119200(1)** (FUN_10119390 if
  it has a type-0 part, FUN_1011a490, +0x74 bit 0 set, FUN_101186a0: the hand and help line on its cursor child), its
  handler's vt+0x24(1); FUN_10124b70 sets the pointer's +0x48 = 18.0 and warps it to the cursor child if a window is
  focused. Manager +0x58 = the previous focus.
- **The handlers' vt+0x24:**
  - link5 (list base vtable 0x103394c0, +0x24 = FUN_101f6940): vt+0x48(state) (link5's is FUN_10148f40, a no-op),
    then gained → FUN_101f6720(0): the cursor row's help (row record +0x4c through FUN_1013a320(…, 2)), or the empty
    list's +0x40 text; lost → FUN_1013a320(0, 2): help slot 2 cleared *(inf: FUN_1013a320 → FUN_10221030 sets the
    help strip's slot)*.
  - lscswin (vtable 0x10339530, +0x24 = 0x1012ac70 `ret 4`): nothing. Its open FUN_101e4950 put the cursor on child 1
    once; a focus change keeps its cursor where it was (its draw FUN_101e4970 is `ret`, so the active filter is never
    marked).
- Both windows stay open throughout; only link5's Escape / Enter (FUN_101e4380 / the pick) closes lscswin.

Build: Num+ / hover over lscswin → queue focus to lscswin with SE 0013; lscswin's Escape / Num+ → focus back to link5
silently; its Enter → set the filter (0 / 1 / 2 / 5) and focus back silently. On the frame after a request: raise the
window, move the hand and help to it (the list shows its row help when focused and clears help slot 2 when not), and
warp the mouse pointer to the new cursor child. lscswin keeps its cursor between visits.

## Item 32: Search — the generic handler before arealist's list (gaps L232, MODLOG L9580)

**Confirmed: every window's keys go through the generic layer FUN_10118260 first, which then calls the handler's OnKey
(vt+0x18). arealist keeps the generic sounds because its +4 doesn't clear +0x85 / +0x86.**

FUN_10118260(window, msg) (gdecomp/all_decomp.c `//@@FUNC 10118260`):
- Returns 0 at once for a type-0xe window, while +0x56 > 0, or when the cursor part is in state 0x10 (batch 5 item 6).
- Only when **+0x5c** (keys on) is set, and with **+0x5d** (Escape only) letting only msg 6 through:
  - **1..4 (Down / Up / Right / Left):** if +0x61 and there are parts, the link target t = FUN_1011b220(cursor, msg); a
    state-0x10 target is refused. t == cursor → +0x5a = msg (the "stayed" marker the list base tests); else +0x5a = 0.
    FUN_10118e90(t) moves the hand; **SE 0001 if +0x85 and the cursor changed**; +0x4c = t.
  - **5 (Enter):** if **+0x86**: SE 0004 on a state-5 (disabled) part, else SE 0002. Disabled → return 0, **no OnKey**.
    Else the part's state = 1 (pressed), return value = FUN_1011b2a0(cursor), +0x5e = 9 (the press flash).
  - **6 (Escape):** SE 0003 if **+0x84**. If window +0x34 bit 1 is clear → FUN_1015e790(window, 1) closes it and
    **returns 0 without OnKey**. With bit 1 set it falls through to OnKey.
  - **9 (pointer onto a child):** SE 0001 if +0x85; a disabled part → return 0; else the part's +0x28 = −1 and
    FUN_10119970(1) (hover state).
  - 0x14: the part's state 2; 0x1d: as Enter without the SE.
- **Then, always** (unless returned above): if the window has a handler (+0xc), is not type 0xe, and has a layout
  (+4) whose +0x46 group passes FUN_1015e0f0 → **handler vt+0x18(msg, cursor)**.
- The three SE flags +0x84 / +0x85 / +0x86 are **1 by default** (window ctor 0x1011739d..0x101173a9). +0x34 holds
  the registry flags (FUN_10155xxx at window creation, e.g. 0x10156039 *(inf: by the store pattern)*).

**arealist** (registry 0x31002: bit 1 set):
- The list base's open FUN_101f68b0 clears +0x86 and +0x85 (the list plays its own sounds), but arealist's vt+4 is
  **FUN_1020f4e0** (+0x6d = 1, +0x6e = −1, nothing else), so **the generic SEs stay on**.
- Its OnKey is the list base FUN_101f6c70 (scrolling; batch 5 item 10), which then reaches arealist's messages
  FUN_1020ef00 (msg 5 Enter at 0x1020efd0, 6 Escape at 0x1020ef99, 8 Num+ the view toggle at 0x1020ef48: modes 2 / 3 / 4
  ignore it; +0x5c 1 ↔ 0, +0x58 = 0, rows rebuilt FUN_1020ebf0).
- So in arealist: a cursor move plays 0001 from the generic layer **and** the list base's 0001 on the same key (the
  list base's "the generic layer moved the cursor → SE 0001" path), i.e. two 0001 triggers in one frame *(inf: they
  sound as one)*; at a clamped end the generic layer is silent (+0x5a set) and only the list base's scroll SE plays,
  except Down past the last of fewer than 5 rows (MODLOG L9482's 0001). Enter: generic 0002 (0004 on a disabled row),
  then arealist's Enter. Escape: generic 0003, then arealist's fold / close (bit 1 keeps the generic layer from closing
  it). Hover: 0001.

Build: run a generic step before every window's own key handler — link moves with SE 0001 when the hand moved, Enter
with 0002 / 0004 (disabled parts stop there), Escape with 0003 and a close unless registry bit 1 — each SE gated by
the window's three flags (default on, cleared by the list base's open but not by arealist's).

## Coordinator question: can an event choice be seen while cinema mode (rem4line +0x48) is on?

**No. The choice window is "menu    query", it has no exemption, and nothing turns cinema off for a choice. In cinema
the choice is opened but neither drawn nor answerable; the event waits on it until the script (or the event end)
clears cinema.**
- **Registry** (table 0x103722d8, 44-byte entries: name[16], 16 zero bytes, +0x20 handler global, +0x24 flags A (window
  +0x34), +0x28 u32 B (byte 0 → window +0x30, byte 1 the layer → +0x60, u16 at +0x2a the group → +0x38; FUN_1015e350
  region at all_decomp.c ~221788)): entry 39 "menu    query" → handler DAT_1057845c, **flags A 0x00011012 (no 0x40)**, B
  bytes 05 02 0c 00. For comparison: entry 35 rem4line DAT_105781cc flags 0x12; entries 201 / 207 mapframe / cnqframe flags
  0x20042 (the only 0x40 windows). So the per-window draw FUN_101177d0 returns early for query in cinema
  (0x101177ee..0x10117803; batch 5-build §1).
- **The choice op** is event op **0xd4** (table 0x100bc960[0xd4] → stub 0x100bc91f → FUN_100b2280; old-build
  FUN_100b0960 region). Sub 2 (0x100b22e2) / sub 4 (0x100b23c4) call **FUN_100b25f0**, which builds the options and
  opens the window through **FUN_1015eaa0** → FUN_1015ea30 (object DAT_1057845c, vtable 0x10335280, 0x54c bytes) +
  `FUN_1015e350("menu    query", 1, 0)` (0x100b266d). The other subs create / fill the same object (FUN_1015ea30 at
  0x100b2323 / 0x100b238b / 0x100b23e8 / 0x100b2458 / 0x100b2507). None of them reads FUN_1015dce0 or rem4line +0x48,
  and none calls the cinema setter FUN_10136870 (its only callers: op 0x67 / 0x68, zone-in Mode 0x80, the event end,
  FUN_10014ad0, text code 0x03).
- **rem4line doesn't draw choices.** The options live only in query's own object (built by FUN_100b25f0 /
  FUN_1014e070); rem4line's draw has no reference to DAT_1057845c *(inf: the 10 readers of DAT_1057845c are the
  registry open / close at 0x1015dacf / 0x1015ea30..0x1015eab8, 0x101656b9 and 0x101f9e25, none in rem4line)*.
- **Input:** every menu key reaches query through FUN_1015dd00, which returns 0 while FUN_1015dce0 (cinema or UI
  hidden) is true (0x1015dd03..0x1015dd0a); the mouse helpers too. query's frame hook (vt+0x28 FUN_1014e3e0 →
  FUN_1014e300) and open (FUN_1014de10) read no keys directly *(inf: no FUN_10123a70 calls in them)*. So the choice
  can't be answered in cinema either.
- **Result:** op 0xd4 yields every frame until query returns a pick; with cinema on, that never happens until op 0x68
  runs (it can't, the script is waiting) or the event ends. Retail scripts therefore turn cinema off (op 0x68) before a
  choice *(inf: the only way out)*. The same holds while the UI is hidden (FUN_10160700), where the player can unhide.

Build: don't exempt the choice window in cinema mode; keep it hidden and its keys blocked (matching retail), or treat
a choice reached with cinema on as a script error worth logging.

## Item 33: Search — rec +0x14 vs +0x18; flags 0x400 and 0x10000 (gaps L231, MODLOG L7868)

Offsets below are the 0x48-byte record of MODLOG L7700 (rec). The list's row data puts it 4 bytes in (data = rec + 4:
name data+8, flags data+0x34, id data+0x1c, job data+0x24), which is how FUN_1020a740 / scoption address it.

**+0x14 (type 0xe, 32 bits) has no reader in the search UI; every consumer uses +0x18 (type 8, the 20-bit char id).**
Every `FUN_1020a420` (selected row) site was scanned for a load of data+0x18 / +0x1c: only +0x1c (= rec +0x18) is read.
- scoption open FUN_1020c540: self test `S+0x3c390 == rec+0x18` → Befriend (child 2) disabled; FUN_102031f0(rec+0x18,
  1, 1) the party-member test for Invite.
- scoption select (0x1020cae5 / 0x1020cafa): rec+0x18 into FUN_100e8fd0 (the invite / alliance path).
- The comment fetch FUN_1020b3e0: `FUN_100e3040(rec+0x18, 0, 3, LAB_1020b440, rec+0x18)` (0x1020b3f8), then the whole
  record is copied to 0x10664448.
- Nothing reads 0x10664448 + 0x14 / + 0x18 by address *(inf: the callback works from its own argument)*. Whether
  /blacklist reads +0x14 (MODLOG's note) was not re-traced here; within search, LSB's 0 there is harmless.

**Flag 0x400 = a PlayOnline-only presence** *(inf for the name; the behaviour is from the code)*:
- Row icon (FUN_1020a740, 0x1020a756..): icon priority anonymous 0x4000 → 7; **0x400 → 5** (keytops3 70); linkdead
  0x800 → 2; Ballista; away 0x100 → 6; auto-group 0x20 → 4; seeking 0x8000 → 3; party requests 0x10000000 → 9; else
  mentor bit 0 ? 8 : 1.
- scoption: +0x14 = (0x400 != 0); child 1's help becomes 0xe 0x11 ("Send a Knock Message.") instead of 0x12, and its
  action calls FUN_101ffc20 (PlayOnline's knock) instead of the /tell line (0x1020ca93); Invite is disabled (0x1020c65a
  chain). So the person isn't in FFXI: they can only be knocked.

**Flag 0x10000 = the first icon drawn small**: FUN_1020a740 at 0x1020a8c8 passes colour **−2** to the status icon cell
(0x0e) when set (−1 for a Ballista team, else 0x80808080). Cell 0x0e with −2 draws the icon at **scale 0.9 at (x − 4,
y + 2) in 0x80808080** (MODLOG's cell table), so the icon shrinks and shifts left. Nothing else tests 0x10000 in the
search code (the other 0x10000 tests at 0x1010429a / 0x101079a6 are the request builder's condition bits).

Build: key every search action on rec +0x18; keep +0x14 parsed but unused. Flag 0x400: icon 5, Tell → knock (we can
show it disabled), no Invite. Flag 0x10000: draw the row's status icon at 0.9 scale, 4 px left. LSB sends neither.

## Item 34: Auction house — is the remembered state cleared on zoning? When does the menus' +0x24 run? (gaps L220, MODLOG L14491 / L14526)

**1. Yes: every zone change wipes the whole character data block, including the parcel store S+0x2cc64.**
- **FUN_100f9eb0** (the map-session setup, called only from FUN_100f9a30): `DAT_104dfd98 = slot[n]` and then
  **zero-fills 0x1038e dwords (0x40e38 bytes)** from its start (0x100f9ed6..), i.e. S+0 .. S+0x40e37. That covers the
  parcel store S+0x2cc64..+0x2cdcb (and everything else in S: the 0x00D / 0x01B fields, +0x40daa, the per-id send times
  +0x41b4, …). It then opens the socket to DAT_104df770's ip / port.
- FUN_100f9a30 is called by the connection state machine **FUN_100f02b0** state 7 (0x100f038a) and by FUN_100ef1a0
  (0x100ef494, the login sequence).
- **s2c 0x00B** (FUN_100fb730): for a zone change (state ≠ 1 / 4 / 8) it raises S+0x3c35c flags (FUN_100fc180 4 /
  0x40), queues c2s 0x00D and **stores the new ip @8 / port @0xC into DAT_104df770**. FUN_100f02b0 state 8 waits for
  flag 8, then state 9 FUN_100f9db0 (teardown) → state 0 → … → 7: FUN_100f9a30 → FUN_100f9eb0 with the new server
  *(inf: the flag-8 step; the state chain is from the decompile, all_decomp.c ~146690..146780)*.
- So the 8 parcel slots, the last AskCommit and the received bits start empty in each zone; the client relies on the
  server re-sending (LotCheck on the first Sales Status build, MODLOG L14491). Nothing outside S remembers them.
- Not in S (kept across zones): the auction window objects (DAT_10663428 etc.) and the saved sorts (AUCSORT.DAT).
  DAT_10663428 +0x89e (below) is in the window object, so the remembered row survives a zone change unless the window
  is rebuilt *(inf)*.

**2. FUN_101da710 is the category menus' vt+0x24 = the focus-change hook**, so it runs on every focus gain:
- Code (0x101da710..0x101da72a): `if (state == 1 && DAT_10663428) DAT_10663428 +0x89e = 0` (the remembered cursor row
  reset); state 0 (focus lost) does nothing.
- It sits in 13 vtables 0x48 bytes apart (pointers at 0x1033881c, 0x10338864, … 0x10338c0c: each vtable +0x24), the AH
  category menus (aucweapo, aucarmor, aucammo, …).
- vt+0x24 is called only by the window manager's focus change **FUN_1015e850** (item 31): `vt+0x24(0)` on the window
  losing the focus, `vt+0x24(1)` on the one gaining it. So +0x89e is reset each time a category menu becomes the focus:
  when it opens (FUN_1015e350 with focus), **and when it gets the focus back** after a window over it closes (the
  manager's end-of-frame pick FUN_1015e7e0 → FUN_1015e850), or after a FUN_1015db80 hand-off.

Build: clear the parcel slots (and every other per-session S field) at each zone change; reset the AH list's
remembered row whenever a category menu takes the focus, including on return from auclist / auchisto.

## Item 35: Macro palette — the default cursors; FUN_101262e0 (gaps L174, MODLOG L10129)

**Correction: FUN_101262e0 is not a click. It is "the mouse is driving the menus" (pointer state 2, no event lock);
the palettes use it to switch on hover.**
- **FUN_101262e0(mouse = DAT_104e1d4c)**: true when mouse +0x4d == 2 (pointer over a focused window, set by
  FUN_101252c0 when the manager has a focus) and FUN_10087070(0) == 0 (no event lock). 23 callers, all per-frame hover
  tests paired with FUN_10118c10(window, &pointer) (a plain rect test on window +0x3a..+0x40).
- **mcr1edit's frame FUN_101a47e0** (0x101a4c3d..0x101a4cc0), and mcr2edit's twin (0x101a5326): when it is true,
  - pointer inside the **other palette** (mcr2edit's window) → **FUN_101a4540(col)**: E+0xd = E+0xe = col, the focused
    palette E+0xc flips 1 ↔ 2, FUN_1015e350(other, 1, 0) (focus) and **SE 0001**;
  - pointer inside **mcrbedit** → the palette sends itself msg 8 (vt+0x18(8, …)) = Num+ (opens / focuses Button Edit).
  - "macro"'s frame (0x101a5a88) and mcrledit's (0x101a62f4) do the same: hover over mcrledit = Num+ *(inf: same
    pattern, not read line by line)*. mcrbedit's own (0x101a6055).
  So the palettes switch by **pointer hover**, not by a left click (MODLOG L10129's "left click" guess is wrong).

**Default cursors** (window open FUN_1015e350 → FUN_10118870):
- No macro window has registry flag 0x10000 (mcrmenu / mcrselec 0x220002, mcrselop 0x210000, mcr1edit / mcr2edit /
  macro 0x20002, mcrbedit / mcrledit 0x10002), so each **restores its last cursor** from the layout object (+0x1a,
  saved by FUN_10118830 at every hide / close). The layout object is zeroed at creation (FUN_1011aeb0) and FUN_10118870
  clamps < 1 to 1, so **the first open of the session is child 1** everywhere:
  - mcrselec: child 1 = book 1 (its open FUN_101a6580 doesn't move the cursor; it shows book (cursor − 1)'s name);
  - macro: child 1 = Prev (8, 8); the title is 3, lines 4..9. The "4" in macro's layout head is head bytes 2..3 (not the
    cursor; *(inf: a frame / style field)*);
  - mcrselop, mcrbedit, mcrledit, mcrmenu: child 1.
- **mcr1edit / mcr2edit override it**: their open (0x101a48a4..0x101a48ba) and their focus hook vt+0x24 (state 1;
  FUN_101a4920 / FUN_101a4fe0, at vtable +0x24 0x103373ec / 0x1033743c) put the cursor on **column + 1**
  (FUN_101a44b0: E+0xd for palette 1, E+0xe for palette 2). So the edit palettes always open / regain focus on the
  remembered Ctrl / Alt column (0 at first → keycap 1), and a hover switch carries the column across.

Build: remember each macro window's cursor across opens (first open: child 1); the edit palettes take column + 1 on
open and on every focus gain; switch palettes when the pointer hovers over the other one (SE 0001), and treat hovering
over Button Edit / Line Edit as Num+.

## Item 36: Equipment sets — who calls mcres20 vt+0x2c; the status gate's bit 0x40; chat mode 0xca's colour (gaps L175, MODLOG L10203)

**1. vt+0x2c is the window manager's session broadcast FUN_1015eea0(on).** It walks the whole registry (0x103722d8,
44-byte entries) and calls every existing handler's vt+0x2c(on, manager +0xaa). mcres20's **FUN_101a7010(on, first)**
registers the three packet callbacks only when `first` (+0xaa) is non-zero: FUN_100e5e70(LAB_101a6fb0) (s2c 0x117),
FUN_100e5ef0(LAB_101a8100) (0x11b), FUN_100e5f20(LAB_101a8150) (0x11c).
- Broadcast(1): **FUN_10093b70** (the field entry, 0x10093c1b, after the zone-ready calls) and **FUN_1015d880**, which
  is registered as an **s2c 0x00B** handler (0x10094821: `push 0x1015d880; push 0xb; call 0x100fc3a0`).
- Broadcast(0): FUN_1015d860, called by the zone-load reset FUN_10093720 (0x10093985); after a broadcast(0) the manager
  clears +0xaa.
- +0xaa = 1: the manager init FUN_1015d900, and FUN_1015f3e0 when its mask has 0x200 — sent by FUN_1015ef60 when the
  game state *DAT_10456a28 becomes 0x40 (0x1015ef98 / 0x1015f032).
- So the callbacks are (re)registered on the first broadcast(1) after start-up or after entering game state 0x40, i.e.
  once per session in practice *(inf: the exact order of the 0x40 transition against the first FUN_10093720 at login
  was not traced)*; the registrations are plain function pointers and stay valid while the handler object exists, so
  later broadcasts (1, 0) change nothing. The object exists from load, so this does not depend on mcres20 being open.

**2. The status gate's bit 0x40 = wearing a costume.** FUN_101cc0d0(mask, quiet) (MODLOG L15596): bit 0x40 tests
FUN_101cc040 (player entity +0x120 bit 5) and prints group 7 0xb1 "That command cannot be performed while wearing a
costume." /equipset's mask 0x46 = 0x40 | 0x04 | 0x02.

**3. Chat mode 0xca has no entry in the mode table, so the line is drawn in the default colour 0x80808080 (white at
MODULATE2X 1.0) with filter group 0.**
- The line (FUN_101a7af0 → 0x101a7c90): record byte 0 = **0xca**, byte 1 = 3, text group 0xc 0x15d, printed through
  FUN_100ad660.
- This build's tables (MODLOG L1429's 0x10362b50 / 0x103629c0 are old-build): the mode table **0x1036e888** ({u16 mode,
  u8 colour id, u8 filter group}, 188 entries, searched by FUN_10129cd0) and the colour table **0x1036e6e8** ({u32 id,
  u32 colour}, 50 entries, FUN_10129d00). The modes around it: 0xc8, 0xc9 (colour id 0x51 → 0xff5020ff), 0xcc..0xde; **no
  0xca or 0xcb**.
- The log record setup FUN_101273c0 (0x10127418..0x10127422): no entry → 0x101276f1: filter group (+0x14) = 0, colour =
  FUN_10129d00(0) = 0x80808080 (the lookup stops at the terminator before matching id 0), +0x12 = 0, and the per-mode
  switch is skipped.

Build: register the 0x117 / 0x11b / 0x11c handlers at start-up (no window needed); refuse /equipset in a costume
(7 0xb1); print "Set #N equipped." in plain white, unfiltered.

## Item 37: Log window 2 — game state 0x40's screen; registry A 0x10; do FUN_10129090's rows count toward the open ticks? (gaps L183, MODLOG L16991)

**1. Game state 0x40 = the front end before the world loads** (the lobby / character select loop) *(inf for the
screen's name; the state machine is from the code)*. The top-level state machine FUN_10011a00 (switch on +0x44):
case 4 loads FFXi.dll's pieces (FUN_10013ac0, FUN_100185a0) → **case 5: `*DAT_10456a28 = 0x40`, runs FUN_10013ad0
each frame** until it returns; result 0 → resets the input flags (DAT_10487f80 / autorun DAT_10487f81 = 0), FUN_10253690
and goes to case 6 (state 0x70, loading into the world, then 0x60 in the field); result 1 → case 0xd (exit). So 0x40 is
what runs between start-up / logout and entering a zone. Entering it broadcasts 0x200 (FUN_1015ef60, 0x1015ef98),
which **closes every registry window whose B bit 3 is clear** (FUN_1015f3e0 walk: `(mask & 0x200) && !(B & 8)`) — both
logs have B 0x125 — and sets the manager's +0xaa (item 36). Leaving 0x40 broadcasts 0x400.

**2. Registry A 0x10 has no reader.** Registry layout corrected: A (flags) is entry +0x24 → window +0x34; B is entry
+0x28 (u32). Every window +0x34 read in the window / manager code (0x10115000..0x10165000; decompile scan) and the
registry walks were checked: the bits tested are 0x1, 0x2, 0x4, 0x8, 0x20, 0x40, 0x80, 0x100, 0x200, 0x800, 0x1000, 0x2000,
0x4000, 0x8000, 0x10000, 0x20000, 0x40000, 0x80000, 0x100000, 0x200000, 0x400000, 0x800000, 0x1000000, 0x2000000,
0x10000000, 0x20000000 — **never 0x10**. The FUN_1015ecc0 mask finder is only called with 0x20000 / 0x20000000 / 0x1000.
So A 0x10 on the logs is inert in this build *(inf: no other path reads window +0x34 bit 4)*. (B's own bit 0x10 is used:
the open-by-state check FUN_1015e0f0 in game state 0x60 tests B & 0x10 while DAT_1057876c +0x68 == 2 and B & 0x20 while it is 1.)

**3. Yes — each rebuilt row adds one period to window 1's open ticks.**
- FUN_10129090 (the filter walk that refills a ring from the history store) calls **FUN_10162560(logwindo, 1)** for
  every row it writes (0x1012916b..0x10129171; ecx = [0x1062193c] = logwindo's handler, registry entry 31; logwin2 is
  0x10621940, entry 32).
- FUN_10162560(win, count) with count ≠ 0: +0x34 (open ticks) += +0x54 (ticks per line) (or = it if negative), +0x3c
  += 0x14, capped at +0x4e (max lines) × +0x54 and rounded up to a whole period. The print path FUN_10128ce0
  (0x10129021) calls it with its own flag; FUN_101b7ee0 (the chat line opening) twice.
- So a window 2 open (FUN_101295d0(8, …) → FUN_10129090, up to 50 rows) tops **window 1**'s open ticks up to its full
  height (the cap), which window 2's rule reads through window 1's fill (+0x5a) *(inf: the visible effect is window 1
  growing to full when window 2 opens or a mask edit refills it)*.

Build: game state 0x40 (front end) closes both logs; ignore registry A 0x10; when a log is refilled from history, add
one period per row to window 1's open ticks (capped at max lines × period).

## Item 38: Motion length (+0x2a0) — what keeps op 0x53 waiting (gaps L67, MODLOG L1271 / L1676)

**There is no length field: op 0x53 waits exactly while the matching scheduler task is alive (or while the files
aren't ready). MODLOG L1676's description holds; the addresses are old-build. This build:**
- Op 0x53 → stub 0x100bc527 → FUN_100b4e00 → **FUN_100b4e10(0, 0)** (0x5f sub 7 is the same body with param_3 = 1, which
  calls vt+0x2ac instead). Actors from operands 1 and 5 (both need +0x120 bit 9, model loaded), the 4cc from operand 9;
  `drawobj(actor1)->vt+0x2a0(4cc, drawobj(actor2))`; true → yield (+0x25a = 1), false → advance 0xd bytes.
- **vt+0x2a0 = FUN_10084d40** (all 7 drawobj vtables 0x1032d710, 0x1032e890, 0x1032ecb0, 0x1032f0d0, 0x1032f4f0,
  0x10330f40, 0x103313e8), returns busy:
  1. vt+0x1e4 (→ FUN_100d12d0, the model's files all loaded) false → **busy** (still loading);
  2. the actor's motion bank +0xb4 present but not ready (FUN_10071060) → **busy**;
  3. find the scheduler resource for the 4cc: the actor's own files (vt+0x1ec(…, 7, 4cc)), else for a skeleton actor
     (FUN_1002c8f0(0x10330ebc)) FUN_100ce480(…, 7, 4cc, −1, 0, 0), else the global bank ([DAT_1047d128+0x40008],
     FUN_100724c0(…, 7, 4cc));
  4. found → **FUN_10057010(res, actor1, actor2)**: walk the live task list ([DAT_1047d140 + 0xd50], next +0xc) for a task
     with +0x18 == actor1's server id, +0x1c == 0x4d730000 ('Ms', a scheduler task), +0x80 == the resource's id, and
     actors 1 / 2 (FUN_10062770 / FUN_100627d0) equal to the pair → busy while one exists;
  5. no resource anywhere → **not busy** (0x10084e3a).
- So "how long the motion plays" = how long that scheduler task runs: until its op stream ends (its own ops, incl. 0x05
  motions and waits, × the task speed) or something kills it (a newer scheduler of the same slot, unbind / kind-2
  drop). Nothing reads the motion's frame count here.
- vt+0x2ac = FUN_10084fb0 (0x5f sub 7): walks the actor's attached list (+0x68, next +0x2c) for an element whose type is
  0x1032cfa0 and +0x78 == the 4cc → busy while such an element exists *(inf: a generator / effect object named by the
  4cc)*. MODLOG's "handled as the same wait" is not exact: 0x5f sub 7 waits on that attached object, not on a scheduler.

Build: op 0x53 = wait while (model files not loaded) or (a running 'Ms' task for that 4cc and pair exists); end at
once if no such scheduler resource exists. Op 0x5f sub 7 = wait while an attached object of type 0x1032cfa0 with that
4cc exists. No fixed 30 frames.

## Item 39: ops 0x5b / 0x66 finish in one call when the file is cached (gaps L70, MODLOG L2408)

**Confirmed: the load request and the "ready?" test happen in the same call, so with the file already resident the op
starts its scheduler and advances on its first frame.**
- Op 0x5b → stub 0x100bc567 → FUN_100b51f0 → **FUN_100b5210(0, 1, 0)**; op 0x66 → 0x100bc5bf → FUN_100b5200 →
  **FUN_100b5210(1, 1, 0)** (kind 0 / 1; the third argument adds the 0x2a8 speed operand for the other users).
- FUN_100b5210(kind, yield_after, with_speed), asm 0x100b5210..0x100b54ff:
  1. Actors from operands 3 (actor 1) and 7 (actor 2), each checked against its server id (+0x78) and needing +0x120
     bits 9 (model) and 7; the 4cc = operand 0xb. Otherwise skip (advance 0xf).
  2. **If the per-kind "requested" flag (body [+0x260] + 0x160 + kind) is 0**: request the file on actor 1's drawobj
     (kind 0: FUN_100d2110 with a file id from operand 1's range — +0x7d68 below 0x200, +0xbfef below 0x400, +0xdc19
     below 0x800, +0xe95b below 0xc00, else +0x10323; kind 1: FUN_100d2220(operand 1)); set the flag = 1. *Then the
     same call continues.*
  3. **FUN_100d12d0(drawobj)** (all model files loaded, item 38): false → yield (+0x25a = 1) and return without
     advancing; the next frame re-runs only this test (the flag stays 1, no second request).
  4. True → if the 4cc is not 0 / 'xxxx': FUN_10087d40, actor1 entity +0x1ec = 4cc, +0x1f4 = actor 2's drawobj,
     **drawobj vt+0x298(4cc, actor2, 0)** (start the scheduler); flag = 0; advance 0xf; **+0x25a = 1** (yield_after).
- So a cached file costs one call (request, ready, start, advance; the VM then yields once because yield_after is 1),
  not two. An uncached file costs 1 + the load frames. The wait is on the model's whole file set (FUN_100d12d0), not on
  the single file *(inf: a file still loading for another reason delays the start too)*.

Build: in 0x5b / 0x66, after issuing the load, test readiness in the same step and, if ready, start the scheduler and
advance immediately (then yield). Keep the per-kind requested flag so a waiting op doesn't re-request.

## Item 40: Op 0x7a pinned to our own actor (gaps L102, MODLOG L11885 / L13112 / L13151)

MODLOG L13112 / L13151 already give the pin itself (our position = the pin point, no collision or ease; facing
actor 2's yaw + π for points < 0x30). This closes the inferred link to the server.
- **The pin moves the drawobj, and the entity copies it every frame.** The entity update **FUN_1008cd90** (all_decomp.c
  ~86006..86014): when the entity's model is loaded (+0x120 bit 9) and DAT_104808fa == 0, **entity +4..+0xc =
  drawobj +0x34..+0x3c** (the position the pin wrote); for the player (+0x1d0 bit 9 with bit 9 set, and +0x12c bit 13
  clear) the rotation entity +0x34.. = drawobj +0x44.. as well (85990..85997).
- **c2s 0x015 sends that copy.** The builder **FUN_100983f0** (called from the per-frame FUN_10096640 at 0x10096b08):
  position = entity +4, rotation = entity +0x14 (except a pinned-ride case: +0x120 bit 7 with DAT_104807f5 reads the
  mount's +0x260 +0x140 / +0x150 instead); entity +0x24..+0x30 = +4..+0x10 unless +0x12c bit 13; it goes out only when
  x + y + z != 0, DAT_10482f18 == 0, and the 1 s gate isn't hit (FUN_100fc3c0(0x15, id, id): force = the char id, so
  never gated). Fields: @4 / @8 / @0xc position, @0x12 the move-time counter, @0x14 the yaw byte (FUN_100dd970), @0x15
  bits (0: +0x124 bit 18, 1: bit 19, 2: HackMove | +0x120 bit 21 | GroundFlag), @0x16 the target index (FUN_101597f0),
  @0x18 FUN_100dc120().
- So while we are pinned, **the server receives the pinned position and facing every frame** *(inf: whether
  FUN_1008cd90 runs before FUN_10096640 in the frame was not traced; at worst a one-frame lag)*. After the pin (dtor
  FUN_10061f50: +0x7a4 = −1, +0x5fc = +0x5c4, +0x608 = 1.0) the drawobj stays where it was left, so the player stays
  there and the next 0x015 carries it.
- Keys while pinned: the control update FUN_100a57f0 still runs (player input FUN_100a5dc0 / FUN_100a5ec0), but the pin
  in the model update FUN_100c63d0 replaces the position each frame *(inf: input has no visible effect while pinned)*.

Build: apply op 0x7a pins to our own player too — set our position / yaw to the pin result each frame, send them in
the normal c2s 0x015, and leave the player at the last pinned spot when it ends.

# MODLOG gaps: what is not built, not known or not checked

Snapshot of 2026-11-03, read from `MODLOG.md` (20,360 lines, 539 entries) and the 60 unfolded fragments in
`docs/modlog/`. It lists every item the journal itself marks as not done, open, left, not yet, unknown, not
decoded, not traced, inferred or not checked live, grouped by area.

- **Source** is a line in `MODLOG.md` (`L1234`), or a fragment file. `MODLOG.md` only grows at the end, so its
  line numbers stay valid; fragment names change when `docs/fold.py` folds them (search for the entry title then).
- **Status** comes from the journal text, not from a code audit. Where a later entry says an item was built, it
  moved to "Closed by later entries" at the end. The rows once marked *(may be closed)* were checked against the
  later entries and the code on 2026-11-03: closed ones moved to the end, partly closed ones keep only what is left.
- **Settle by**: **DLL** = read retail FFXiMain.dll (30261102_0 since 2026-11-10; addresses in rows written before
  that are 30261006_0, move them with `python3 docs/re/dll-index/dll.py map ADDR`) in Ghidra / Rizin; **Live** = try it in game;
  **Footage** = compare with the retail client or a recording; **LSB** = depends on what the server sends;
  **Upstream** = a design question for upstream (lenonk/vana-client), listed only, not answered here.
- **Addresses**: many MODLOG / `event_ops.rs` addresses are from an older FFXiMain.dll than Steven's unpacked one
  (sha256 8097dbf8...; only 40 of 219 event handler addresses match). The RE notes in [notes/](notes/) map them.
- Inline inference markers (*(inf)*, *(inferred)*, *(not traced)*, *(as read)*: 826 lines in 270 entries) are
  too many for this page. They are indexed one per line in [modlog-inferred.md](modlog-inferred.md), generated
  from the same files.

## Summary

Rows below often mix kinds, so these are rough counts of rows that carry each kind:

| Kind | Rows | Typical way to settle |
|---|---|---|
| Not built (a retail feature missing in ours) | ~120 | Implement from the DLL |
| Unknown, not traced, inferred or not decoded (an RE question) | ~77 | DLL |
| Built but not checked live or against retail | ~20, plus the backlog in section 14 | Live / Footage |
| Blocked: the server (LSB) never sends the data | 8 | LSB, or leave |
| Upstream design questions | 1 (Lua mod grants, issue #70) | Upstream |
| Closed by later entries (kept for reference) | ~68 | none |

The event VM has no ops left unimplemented: the 2026-11-02 audit closed every missing op, and 0x97 (cloth
wind) closed with the cloth simulation on 2026-11-03 (`2026-11-03-cloth.md`). What remains under events is
detail inside implemented ops.

---

## 1. Network, session, lobby and sign-in

| Item | Status | Source | Settle by |
|---|---|---|---|
| Map packets: Huffman-compressed payload, MD5 and Blowfish over whole blocks ("Later") | Not built | L40 | DLL (polcore / zone coder) |
| Send cadence: the adaptive +50 ms step when the server lags; the 1000 ms per-id refusal | Not built | L2242 | DLL |
| Bazaar / trade sends: the per-id 1000 ms send refusal (FUN_100fc3c0) *([batch 6](notes/2026-11-03-modlog-gaps-batch6.md) item 12 and the [batch 11](notes/2026-11-03-modlog-gaps-batch11.md) addendum: every c2s id, gated or forced per call site; refused sends dropped, or retried by the event / zone-in / stats callers)* | Built | L16460, L16478 | DLL |
| Djarum reached 0 HP twice with no mobs in the zone (once after /logout) | Unknown, not reproduced | L120 | Live (chat log now shows the source) |
| Lobby netwait: retail's backdrops hnbackwi / ptcbgwin behind the run; op 5 as one Task with the log-in; the 3600-frame timeout (FFXI-3113); FFXI-3115 after a failed refresh | Not built | L15896, L15916, L15937 | DLL |
| Lobby: what the PlayOnline calls return under xiloader (step 3 sub 14 flag 0x8000); error 0x68's field +0x144; FUN_1023be30's 0x6c path | Unknown | L15953, L15955 | DLL |
| Select screen: delete's second confirmation ptc9dele (FUN_10233c00) | Not built | L2832, L15957 | DLL |
| Select screen: chswin's file details line; other languages' count lines; DAT_10623ac4 (cursor kept on reopen); chfwin; the 149-frame head-point refresh; the arc-length camera speed | Not built | L529, L2832, L2780 | DLL |
| Select screen: FFXI-3117 (five failed user-file operations) has no counterpart in ours | Not built (different login flow) | L2832 | DLL |
| Rename flow | Built, untested live (no character with renamef) | L2832 | Live |
| Creator: child 3's element state 0x10 (FUN_10118b60(0x10, 3)) *([batch 5](notes/2026-11-03-modlog-gaps-batch5.md) item 6: state 0x10 is "inert": drawn at 0x80808080, never the cursor, no mouse hit, keys dropped, no help line. ptc6yesn's child 3 is the warning icon, which ours draws and never selects)* | Matches | L15623, L15640 | DLL |
| Creator world step: the local name filter FUN_101ba240 (3322); the mouse and right click on chmkserv / chmkpass *(items 48 + 49, [fragment](../modlog/2026-11-11-creator-name-filter-48-49.md), 2026-11-11: the filter is PlayOnline's dictionary "entryz.dic" matched as polcore does (`poldic.rs`), copied from the player's installer; [batch 9](notes/2026-11-03-modlog-gaps-batch9.md) item 48: a click is Enter and a right click Escape, as ours)* | Built | L15980, L16015 | DLL |
| Config tool 0023 Simple Character Creation: the forced-on cases (app state 4, the fallback device) and the screen tint / music on leaving | Not built | L18552, L18561, L18570 | DLL |
| PlayOnline sign-in: whether a server checks the token or the stale addr20 bytes 8..19 (ours zero); the handshake 0x0c..0x17; characters not yet linked to a handle; Friend List / POL messages through net/pol (ours use LSB's xi_profile) | Unknown, not verified | L19522 | Live on a POL server, DLL |
| Map session: the 0x00E name rules (kinds 2..4 at >= 0x700, 16 characters, "NPC" fallback); reserved actors; index ranges; an event-held actor's despawn clearing +0x145; SubKind 7's words @0x3E..@0x42 | Not built | L19747, L19802 | DLL |
| Game file updates: the PlayOnline patch protocol beyond what src/patch.rs does ("Not yet") | Not built | L4145 | DLL (polcore) |
| Portability: macOS (no build or test machine yet; Windows is done: CI builds windows-latest and the Linux-only spots are gated) | Not built | L3116 | Live (needs a Mac) |

## 2. Event VM and cutscenes

| Item | Status | Source | Settle by |
|---|---|---|---|
| Installed DATs are older than LSB's text ids, so message ids can be shifted (msgshift table); ids between two segments stay unknown | Known limitation | L835, L1086 | Data |
| Motion length (+0x2a0) comes from the motion; the headless host keeps it busy for 30 frames | Answered *([batch 8](notes/2026-11-03-modlog-gaps-batch8.md) item 38: there is no length field at all. vt+0x2a0 is busy while the model's files load or a live 'Ms' scheduler task for that 4cc and actor pair exists, and ends at once when no scheduler resource exists; the live wait on the playing motion is our equivalent, the headless 30 frames stay a test stub)* | L1266 | DLL |
| Name plates in cutscenes *(answered, [batch 2](notes/2026-11-03-modlog-gaps-batch2.md) §3: no event check in the plate draw; bound actors keep it unless op 0x92 / bit 17; non-participants lose the model by event Mode & 2 / & 0x10)*; audited 2026-11-03 with [batch 5](notes/2026-11-03-modlog-gaps-batch5-build.md) §2: ours unloaded doors, elevators, ships and op 0xac sub 2 actors too, now exempt. Left: the hold count +0xf2, the +0x1ff status rule | Built | L2408 | Code check |
| The mouth's 8-tick blend out (0x23 and unbind) *([motion queues](notes/2026-11-11-motion-queues.md): FUN_100ce160 gives every entry of the mouth region (4) FUN_1001b520(8.0): each runs 8 more ticks at its weight, then goes. Ours: `Actor::mouth_off` on op 0x23's close, unbind and op 0x7c, on the model's region queues, MODLOG "Motions: the model's region queues")* | Built (not checked in game) | L2343, L2408, L14046 | DLL |
| 0x5b / 0x66 finishing in one call when the file is cached (ours take 2 frames) | Built *([batch 8](notes/2026-11-03-modlog-gaps-batch8.md) item 39: FUN_100b5210 requests the file and runs FUN_100d12d0's ready test in the same call, keeping its per-kind requested flag so a waiting op does not re-request)* | L2408 | DLL |
| Motion-file model kinds | Not built | L2408 | DLL |
| 0x46 sub 1's HUD window FUN_10217e90 *(inf: hides a HUD window)* *([batch 5](notes/2026-11-03-modlog-gaps-batch5.md) item 5: "menu    buff" hidden until the event ends)* | Built | L2408 | DLL |
| 0x1f7 (San d'Oria opening) not played live since the VM-speed fix; timing is the headless VM's | Not checked live | L2408 | Live |
| Op 0x89 (actor +0x840 bit 1 and position held for dur ticks; *inferred: holds the actor in place*); also its head-look suppression *([batch 6](notes/2026-11-03-modlog-gaps-batch6.md) item 13: no hold; head-look off for dur x speed ticks or until a move > 1 in x / z)* | Built | L7951, L4535 | DLL |
| Event text codes 0x1d (speaker's compass direction), 0x13 *(answered, [batch 5](notes/2026-11-03-modlog-gaps-batch5.md) item 1: not Japanese, the raise / tractor offer)* | Built | L15345 | DLL |
| Grammar 2 / 3 whole-text conversions (key items with their own article) | Not built | L14678 | DLL |
| Event positions: the +0x128 bit 14 and +0x120 bit 0x100000 ground-snap skips (only 0x200000 is ours) *([batch 6](notes/2026-11-03-modlog-gaps-batch6.md) item 14: the bits are GroundFlag and HackMove)* | Built | L19371 | DLL |
| Op 0x71 subs 0x50..0x55 (a full-screen picture, zone 291) | Not built | L4983 | DLL |
| 0x7f0b / 0x7f8b special values (+0x124 bit 25, 475 sites); FUN_100977f0's gate for PCs *(answered, [batch 3](notes/2026-11-03-modlog-gaps-batch3.md) §2: the speaker's / player's gender bit (1 male); ours reads 0 for both, and dialog `7F 90` / `7F 91` use the player's gender; the PC see-through is FUN_100973f0, already built; [batch 4](notes/2026-11-03-modlog-gaps-batch4.md) §2..3: 0x00D / 0x00E mask 4 write it for every actor, default female; `7F 90` / `7F 91` are the message's actor 1 / 2)* | Not built | L4893, L4900 | DLL |
| Op 0x7e / 0xac subs 5 / 0x55 (FUN_10097a20 spawning the chocobo / mount in an event) | Not built | L5246 | DLL |
| Event op 0x34 area swaps: retail's file slots (ours restarts the auto schedulers; inferred) | Unknown | L13514 | DLL |
| Event sound: Audio::event_volume reaches the mixer, but the live runs had audio off | Not checked live | L1634 | Live |
| Mhaura ferry: the call that puts the 0x00E pose on an undriven ship is not found *(inf)* | Unknown (behaviour now matches retail) | L20050 | DLL |
| The message formatter's backward-jump stop: what FUN_10153310 does with such a string *([batch 5](notes/2026-11-03-modlog-gaps-batch5.md) item 3: retail never stops, the client hangs in the loop; ours stopping at a repeated state is deliberate, nothing to build)* | Answered | `2026-11-02-msgfmt-backward-jump.md` | DLL |
| Event ops 0x83, 0xa8 / 0xb9, 0x50, 0x84 / 0x86 / 0x95 / 0x96 / 0xa4 | Built, not checked live | the fragments `event-game-clock`, `event-marker-clear`, `event-scheduler-stop`, `event-actor-flags` | Live |
| Login events start on a black screen | Built, not checked live (needs a new character) | `2026-11-02-login-event-black.md` | Live |

## 3. Actors, motions and schedulers

| Item | Status | Source | Settle by |
|---|---|---|---|
| Motion schedulers bntg / klbt / hwat / res0; the move direction for atr / atb / atl (moving always picks f) *(item 134, [fragment](../modlog/2026-11-10-disengage-schedulers-134.md),  PR #354, 2026-11-11)* | Built | L1771 | DLL |
| Faded actors: retail draws the model opaque into an offscreen target, then blends it as one SRCALPHA / INVSRCALPHA quad (no self-overlap); ours blended each triangle *(answered, [batch 3](notes/2026-11-03-modlog-gaps-batch3.md) §4)*; built 2026-11-03 for GPU-skinned actors (`ZoneRenderer::render_faded`). Left: CPU-posed actors, the second-RT path, the +0x9c effects | Built | L1702, L1771 | Live |
| Hit reactions: two overlays at once (a new one replaces the old); the CPU pose for the locked camera ignores it | Not built | L4829 | DLL |
| Model-file generators: the un-hide restart (FUN_10084320 re-runs FUN_10083970) *([batch 9](notes/2026-11-03-modlog-gaps-batch9.md) item 44: stale row, MODLOG "Show (FUN_10085a10 ...)" built it)* | Built | L7567 | DLL |
| Riding: ground tilt of info[0] 1 / 2 models (FUN_100d54c0); op 0x6e for other actors; the riding `cdam`; the local player's wait for the stand motion before `coff` *(item 135, [fragment](../modlog/2026-11-10-riding-self-move-135.md),  PR #355, 2026-11-10)* | Built | L12422, L13095 | DLL |
| Riding yaws (Tiger 0x40 for Hume F, Elvaan F, Mithra) | Not checked against footage | L12469 | Footage |
| Ops 0x9c / 0x9d (no file uses them); the 0x038 handler's filters | Not built | L13095 | DLL |
| Ground tilt: writers of +0x128 bit 14 and +0x120 bit 20; statuses 0x1a / 0x1b / 0x1d / 0x1e *(inferred: transport)*; whether vt+0x1c4's bones include +0x998; long bodies' no-roll and scale overshoot *(item 136, [fragment](../modlog/2026-11-10-carrier-statuses-136.md),  PR #357, 2026-11-11)* | Built | L12689, L12740 | DLL / Footage |
| Motion regions: FUN_10019ed0 carries the old entry's frame into a status motion's start frame *([motion queues](notes/2026-11-11-motion-queues.md): the enqueue is FUN_1001ad00 in this build; each region carries the phase of its own last entry, plus half the blend at the new rate. Ours: crate::motionq, one queue per region, MODLOG "Motions: the model's region queues")* | Built per region (not checked in game) | L13899 | DLL |
| Eye blink: the entity copies (+0x190..+0x1b4) the 0x8c / 0xa4 / 0xa5 setters also write | Matches *([batch 9](notes/2026-11-03-modlog-gaps-batch9.md) item 41: the ops write the entity base names so the new name survives every later status recompute; a mount overrides them, sitting and chair idles are never written, 0x79 is model-only and FUN_1008a160 resets the bases - which is our motion_bases / rebase path)* | L11933 | DLL |
| Op 0x7a pinned to our own actor | Built *([batch 8](notes/2026-11-03-modlog-gaps-batch8.md) item 40: the pin moves the drawobj, FUN_1008cd90 copies it into the entity every frame and FUN_100983f0 sends that copy in c2s 0x015, which is what ours already does; the player stays where the pin left it)* | L11885 | DLL |
| Model hide / show: the skeleton's `fvib` clone at the show; the +0x764 / +0x814 clones | Not built | L13203 | DLL |
| Near-camera fade after the cylinder (vt+0x260, first person, fishing) *([batch 9](notes/2026-11-03-modlog-gaps-batch9.md) item 42; [2026-11-11](../modlog/2026-11-11-easy-rows-zone-in-forced-follow.md): the forced follow (s2c 0x037 @0x46) and its first-person hide are built)* | Built | L12096 | DLL |
| Character bump during events; whether any actor reaches +0x59c >= 1 | Built *([batch 9](notes/2026-11-03-modlog-gaps-batch9.md) item 43: +0x59c is the load fade and every actor reaches 1.0 about 32 ticks after its files load; the bump needs it at 1.0, and the bump has no event test of its own - it simply runs inside the move step, which returns early whenever we cannot move)* | L8858, L8889 | DLL |
| Lifts: the camera's look point; LSB's Metalworks door data not checked against the doors *([batch 12](notes/2026-11-03-modlog-gaps-batch12.md) item 71: the look point has no lift handling; the eye rides and holds the auto-adjust 10 ticks, which `camera::ride` already does)* | Matches; LSB door data not checked | L8776 | DLL / LSB |
| Ship actors: whether a new task first updates in its frame; vt +0x178 and the default speed 1.0 assumed; the far plane; the lighting area; the 10-second no-skip window; +0x130 bit 21's writer *([fragment](../modlog/2026-11-10-ship-actors-137.md): a new task and a new scheduler both take the frame's ticks in the frame they start; vt +0x178 is the path drive's hold on the actor and always passes; speed 1.0; the far plane is 65535 and a ship has no distance test; area 0, lit as the scenery; the window is a signed 32-bit difference below 600 ticks; bit 21 is the skinned furniture actor's)* | Built; the scheduler's two-frame end and the drive's end snap traced, not built | L18113 | DLL |
| Ships: per-run data for each voyage (no captures; pirates' run 227 / 228) | Blocked: no data | L13802, L13831 | Captures |
| 0x00E SubKind 5 / 6: the 0x9000 word *(answered, [batch 2](notes/2026-11-03-modlog-gaps-batch2.md) §5: bits 0..11 NPC-table model id, bits 12..15 the look slot; kind 7 has a case)*; built: ours keeps the low 12 bits (MODLOG "0x00E SubKind 5 / 6: the model word's low 12 bits"). Not done: SubKind 6's +0xf6 = 2 for raw words 0x7b8..0x7cb / 0x7d3..0x7da, and slot 9 attached over a race or slot 0 base (op 0xb6 sub 0x0c, kind 7) | Built apart from the SubKind 6 flag and slot 9 attach | L15593 | DLL |
| /sitchair: trusts ignoring chair / sit / heal statuses in 0x00E (asm 0x1009c148) *(built: MODLOG "Trusts ignore rest statuses in 0x00E")* | Built | L18831 | DLL |
| Dual wield: a 'b' scheduler with +0x8b5 calls vtable +0x250(1), no effect seen *([batch 5](notes/2026-11-03-modlog-gaps-batch5.md) item 8: it sets drawobj +0x840 bit 6, read only as scheduler system value 0x3b)* | Built | L16279 | DLL |

## 4. Effects, lighting and rendering

| Item | Status | Source | Settle by |
|---|---|---|---|
| Effect schedulers: element ops 0x25 / 0x33 (per-frame child emission) *(FUN_10057c20 never waits for children: [batch 2](notes/2026-11-03-modlog-gaps-batch2.md) §2; 0x29 / 0x2a, op 0x80 and stores 0x14..0x18 closed: L10569, L11275, L6913, `fx/tasks.rs`)* | Not confirmed in code | L3454 | DLL |
| Scheduler ops 0x6f / 0x70: point-light intensity / range multiplier ramps (element +0x1a4 / +0x1a8 to f32 @0x10 over dur x speed) *(answered, [batch 3](notes/2026-11-03-modlog-gaps-batch3.md) §5; 0x56 and the end callback closed)* | Not built | L3531, L3463 | DLL |
| Effect sound: FUN_10054fb0 (capsule nearest point) needs a clean asm read; FUN_100528e0's effect on the generator's +0xa8; whether a soft stop (op 0x2d) ends persistent sound elements *(item 114, [fragment](../modlog/2026-11-08-effect-sound-114.md),  PR #309, 2026-11-08)* | Built | L3595 | DLL |
| Area-light tint: the crossfade's +0x14 countdown; FUN_10180380's weather argument; where gen+0xc4 is first set; which effects use 0x8f / 0x90 / 0x69 *(item 115, [fragment](../modlog/2026-11-08-area-light-115.md),  PR #310, 2026-11-08)* | Answered | L3877 | DLL |
| Event op 0xae sub 7 (area-light) | Not built | L3822 | DLL |
| Point lights: the slot leak for refs to 'c' / unknown keys; the windowed-mode gamma boost (FUN_1000b290); the two-area-light scenery mode; unnormalised normals of scaled placements *(bump map closed: "Config tool 0017", L18650)* *(item 117, [fragment](../modlog/2026-11-08-point-lights-117.md),  PR #314, 2026-11-09)* | Built | L4019 | DLL |
| Point lights: c0..c3 and the order of the obj+0x80 inverse; the light buffer not checked on screen *(item 118, [fragment](../modlog/2026-11-09-point-lights-118.md),  PR #321, 2026-11-09)* | Built | L4079, L4007 | DLL / Footage |
| Sub-areas: door collision during a sub-area load (FUN_10176dd0) *(item 119, [fragment](../modlog/2026-11-09-subarea-doors-119.md),  PR #322, 2026-11-09)* | Built | L4297, L4445 | DLL |
| Scenery bit 31: alpha blending in retail, ours alpha-tests *(item 120, [fragment](../modlog/2026-11-09-scenery-blend-120.md),  PR #323, 2026-11-09)* | Matches; nothing to build (the # placements now clamp their UV) | L4325 | DLL / Footage |
| Scenery ambient may be twice retail's (D3DRS_AMBIENT = ambient x 128 / 255); point-light falloff differs (ours (1 - (d/R)^4)^2, retail hard cut); two-light mode not modelled for scenery / furniture *(item 121, [fragment](../modlog/2026-11-09-scenery-ambient-121.md),  PR #327, 2026-11-09)* | Built | L15364, L15397 | Footage, then DLL |
| Water in a hollow (scenery blending) | Not checked against retail | L4460 | Footage |
| Distortion: DistMorph (key mesh 0 only); the D3m loader's format-7 fallback (FUN_1003fee0; 4 meshes parse 6 bytes off) *(item 122, [fragment](../modlog/2026-11-09-d3m-layout-probe-122.md),  PR #329, 2026-11-09)* | Built | L11627, L11663 | DLL |
| Distortion: DistRing (retail squeezes the viewport into the 256x256 copy) *(item 123, [fragment](../modlog/2026-11-09-distring-copy-123.md),  PR #330, 2026-11-09)* | Built | L13430 | DLL |
| Distortion: fog under vertex fog is driver-defined; whether device +0x194 is the 3D target in every mode; the +0x198 target and after-image path; retail's soft 256x256 look as a setting *(item 124, [fragment](../modlog/2026-11-09-distortion-copies-124.md),  PR #331, 2026-11-09)* | Built | L11643 | DLL |
| Specular meshes: the cube's tint copy black on every backend; MIPFILTER inherited; the copy's half-texel shift; stage 2 in the no-groups path; camera +0xd4 vs +0xb90; FUN_10075700's texture warp *(item 125, [fragment](../modlog/2026-11-09-specular-stages-125.md),  PR #332, 2026-11-09)* | Built | L11734 | Footage (3446 or DAT 109 `cry1`), DLL |
| Ground projection: FUN_1003eb30's use of D3a +0x194; whether 'hi17' is D3m or MMB; MZB lists with bit 0x2000 at load; lit decals' colour; +0x198 default; objects with +0xbc 0xffffffff while no sub-area is loaded *(item 126, [fragment](../modlog/2026-11-09-ground-projection-126.md),  PR #337, 2026-11-09)* | Built | L11851 | DLL |
| Weapon after-image: whether retail finds Afbs that sit only in mob-move effect files; which removal callback fires; ZBIAS and WORLD during the draw; NaN vertices *(item 127,  PR #338, 2026-11-10)* | Built | L11992, L12056 | Footage (a trust's Swift Blade, 50288), DLL |
| Effect attach mode 9 with op 0x11: whether doll-attached generators run away in retail; the not-yet-posed skeleton's identity *(item 128, [fragment](../modlog/2026-11-10-doll-bone-frame-128.md),  PR #339, 2026-11-10)* | Built | L11516, L11479 | DLL |
| Effect motions from specs: op 0x7b's two-point offsets, 0x33 and modes 7 / 8 at start, the spin ops' attachment rebuild *(item 129, [fragment](../modlog/2026-11-10-attach-specs-129.md),  PR #340, 2026-11-10)* | Built | L11265 | DLL *(0x40..0x47 closed: "The actor shine overlay")* |
| Camera shake: retail's element order within a frame; FUN_10087070(0)'s bits; which consumers read the shaken matrix; the rumble sources and pad flags; above 30 fps retail shakes faster (n = 1); lobby / creator / title cameras do not shake *(inf)* *(item 130, [fragment](../modlog/2026-11-10-camera-shake-130.md),  PR #341, 2026-11-10)* | Built apart from the harbour generators' start order and op 0x5f's frame | L11594 | DLL |
| Self-starting schedulers: whether the weather object runs in the zone parse's frame; sub-area weather; which emission op fills update op 0x64's slot *(item 131, [fragment](../modlog/2026-11-10-self-starting-schedulers-131.md),  PR #342, 2026-11-10)* | Built | L13464, L13514 | DLL |
| Time-of-day lighting: light yaw world+0x1cc; DAT_1047bf18's override; the writers near 0x10059273 / 0x100604d7; Ambient DAT_1065ca34's source *(item 132, [fragment](../modlog/2026-11-10-zone-sky-132.md),  PR #343, 2026-11-10)* | Built | L13256..L13279 | DLL |
| Zone lighting: floats @0x50 / @0x54 / @0x68 of the lighting record *([batch 7](notes/2026-11-03-modlog-gaps-batch7.md) item 22: @0x50 / @0x54 are copied to the camera and never read, @0x68 has no reader; nothing to build)* | Answered | L478, L479 | DLL |
| Effect generators: data 0x80; DAT_10443fe4 +0x28; the view matrices' purposes *(item 133, [fragment](../modlog/2026-11-10-doll-clones-133.md),  PR #344, 2026-11-10)* | Built | L3374 | DLL |
| Lamp Lights (ours): lamps with no data light in their zone get no light | Not built | L18362 | Ours |
| Actor ground shadows: +0x66c's source; ZBIAS overlay order on ATI *([batch 9](notes/2026-11-03-modlog-gaps-batch9.md) item 45: latched when the model is made (no-shadow flags, zero model alpha); ATI only uses smaller ZBIAS steps in the same order)* | Built | L18408, L18428 | DLL |
| Effects carried over a zone change: the glowing square at Nine's left hip | Not reproduced | L14424 | Live |
| Gamma-space blending for the title flyover, select and creator scenes | Closed by "Correction: the lobby scenes already blend in gamma space" (L18742) | L6291 | none |
| GL backend: no BC textures, and wgpu-hal cannot copy compressed textures out ("Not implemented yet") | Backend limit | `2026-11-02-gl-bc-mips.md` | none |
| Cloth: the pool-allocated buffer is uncleared, so retail's first sweep starts from stale data (ours from zero) | Known difference | `2026-11-03-cloth.md` L48 | none; live cloth test still Steven's |

## 5. Sound

| Item | Status | Source | Settle by |
|---|---|---|---|
| Footsteps: zone flag +0x1cf alt name (surface 3 -> 2), the model-scope fses retry, the culled-pass skip (+0x840 bits 20..21), 'fvib', the footstep-effects option DAT_10444158 (taken as on) *([batch 12](notes/2026-11-03-modlog-gaps-batch12.md) item 72: the alt name is areas 704..706 only, now built (session flag bit 0x100 taken as clear); a failed lookup already retried each pass. Open: the culled-pass skip, and the note's "no model-scope search in this build", which ours still has)* | Built apart from the culled-pass skip | L3717, L3734..L3755 | DLL |
| Sound levels: the resets at event end / SE slider change (FUN_100ae3f0 / FUN_10035e80) *([batch 6](notes/2026-11-03-modlog-gaps-batch6.md) item 18: one reset a zone, so a normal event's end restores nothing; a slider move resets every channel)* | Built | L6303 | DLL |
| Zone ambient loop: a time / weather / indoor change crossfades over 150 sound ticks; a zone change cuts the old loop *(answered, [batch 3](notes/2026-11-03-modlog-gaps-batch3.md) §7; ours cuts every change)* | Not built | L5932 | DLL |
| Title buttons: press sounds on the character list rows, Config pages, the login screen *([batch 12](notes/2026-11-03-modlog-gaps-batch12.md) item 73: the press is the generic Enter effect (N = 20 from 9), already ours on the character list (`select.rs`) and the Config pages; the login screen is ours, not retail's)* | Matches | L7195 | DLL |
| Number windows' sounds: retail plays nothing on Up / Down / Left / Right or at a limit, only OK 0002 / cancel 0003; ours plays the cursor sound *(answered, [batch 3](notes/2026-11-03-modlog-gaps-batch3.md) §8)* | Not built | L6841 | DLL |

## 6. HUD, chat log and text

| Item | Status | Source | Settle by |
|---|---|---|---|
| Chat: the spam filter (config 0xce; kinds 1, 0x1a, 0x22, 0x23) *([batch 12](notes/2026-11-03-modlog-gaps-batch12.md) item 74: `client/spamfilter.rs` already matches the loader, the i / s / n flags, the classes and allowed-before-prohibited)* | Built | L5953, L5960 | DLL |
| Chat: the profanity mask (vulgar2.dic, FUN_101ba200) *(the word engine itself is built for the name filter, `poldic.rs` `Engine`, [fragment](../modlog/2026-11-11-creator-name-filter-48-49.md); the mask modes 0 / 4, FUN_10049960, are still to add)* *([batch 12](notes/2026-11-03-modlog-gaps-batch12.md) item 75: config 0x18 switches it; the masking engine is in the PlayOnline host's interface table (DAT_10456a2c + 0x5a8 / 0x5ac), not FFXiMain)* | Blocked: host module | L5970 | DLL |
| Chat: the line cut at a control byte (FUN_1018fc40) *([batch 6](notes/2026-11-03-modlog-gaps-batch6.md) item 20: the assist kinds cut name and text apart; which packet carries MODLOG's icon-bearing filtered shout / yell is not traced)* *([2026-11-11](../modlog/2026-11-11-easy-rows-zone-in-forced-follow.md): the icon lines are the chat filter's swapped shout / yell / assist lines, icon 0xEF 0x4D before the name)* | Built | L5974 | DLL |
| Chat: the echoes' rank icon (mentor / mastery rank) *([batch 7](notes/2026-11-03-modlog-gaps-batch7.md) item 21; MODLOG "Chat: the assist echoes' rank icon")* | Built | L5985 | DLL |
| Chat: Shift-JIS text (ours decode UTF-8); SJIS quotes 0x8167 / 0x8168 print '?' *([batch 12](notes/2026-11-03-modlog-gaps-batch12.md) item 76: retail indexes the bytes as they are. The quotes already survived; 398 pairs and the bytes Shift-JIS lacks did not, and now keep their bytes)* | Built | L5953, L1634 | DLL |
| Chat: c2s 0x0B5 / 0x0B6 at 150 bytes (ours 127) *([batch 5](notes/2026-11-03-modlog-gaps-batch5.md) item 2: 150 already; 0x0B6 now sized 4 × ceil((len + 0x15) / 4))* | Built | L5953 | DLL |
| s2c 0x027 / 0x043: the event-lock drop ((DAT_104808fc \| DAT_1047fb40) & 0x40) | Not built | L6010 | DLL |
| Invisible (own +0x128 bit 6) and /linkshell (7031 0x25) in chat log fixes *([batch 6](notes/2026-11-03-modlog-gaps-batch6.md) item 19: talk and /l refusals were in; /returntrust added. The zone-in source, char +0x40db2 bit 1, is not mapped to a packet field)* *([2026-11-11](../modlog/2026-11-11-easy-rows-zone-in-forced-follow.md): the zone-in source is s2c 0x00A Flags1 (u32 @0x20) bit 29, read with the other zone-in flags)* | Built | L5884, L5886 | DLL |
| Full log: ~100-line history (ours now keeps 1000 wrapped rows, refilled to that depth on a rewrap); its wheel handler *(inf)* *([batch 12](notes/2026-11-03-modlog-gaps-batch12.md) item 77: the 100 rows are the view; Window 1 / 2 page older through the 20 x 50-row store, channel tabs stop at 100; one row per notch, as ours already did. Ours now keeps 1000 rows)* | Built | L2581, L8726 | DLL |
| Auto-translate: what window FUN_101ddcb0 is and the s2c 0x0CC chain to FUN_101dd780; FUN_101bceb0 (config 0xce); what FUN_1022aff0 tests and why ids 0x181..0x183 end the key item scan; the French / Japanese second key *(item 139, [fragment](../modlog/2026-11-11-auto-translate-unknowns-139.md): all answered. FUN_101ddcb0 draws the comment editor, whose mode 3, the linkshell message editor behind the holder / pearlsack submenu's caption, is now built; lines show the brackets; the ids are the three nation maps that open the Magical Maps section; the second keys are read for any language file, the French second words and per-language compares wait on a client language)* | Built apart from the non-English candidate lists | L6175 | DLL |
| Auto-translate: the item / key item groups built at run time (the 200-record item cache); kind 6; the nested FD 06 block *(item 138, [fragment](../modlog/2026-11-11-auto-translate-groups-138.md): the record cache ring, both groups rebuilt when the root list opens, Key Items before Items, item phrases use the record name, variable blocks expanded)* | Built | L6116, L6149, L6192 | DLL |
| Macro editor: Shift-JIS input; the palettes' flip zoom (FUN on set change, 20 + 20 ticks) *([batch 12](notes/2026-11-03-modlog-gaps-batch12.md) item 78: the fields' byte limits 15 / 8 / 60 were already ours; the flip is a fade, now built for the play palette. The editor palettes' copy is not)* | Built apart from the editor palettes | L10057, L10085 | DLL |
| Macro palette: the default cursors; FUN_101262e0 as the left click in the palettes' frame | Built *([batch 8](notes/2026-11-03-modlog-gaps-batch8.md) item 35: FUN_101262e0 is "the mouse is driving the menus", not a click, so the palettes switch on hover; no macro window has registry flag 0x10000, so each opens on its last cursor, and the edit palettes take column + 1)* | L10096 | DLL |
| Equipment sets: who calls mcres20 vtable +0x2c FUN_101a7010; the status gate's bit 0x40; chat mode 0xca's colour *([batch 8](notes/2026-11-03-modlog-gaps-batch8.md) item 36: the manager's session broadcast; bit 0x40 = wearing a costume (7 0xb1); mode 0xca has no table entry, plain white)* | Built | L10170 | DLL |
| Status icons: the 240 x 4 line and per-cell corner sprite; whether retail hides the buff window in cutscenes | Unknown | L7340, L7359 | Footage |
| Top bar: netstat in cutscenes; item overrides +0x4c / +0x50 *(answered, [batch 2](notes/2026-11-03-modlog-gaps-batch2.md) §4: no event check, only cinema mode / UI hidden; +0x4c help, +0x50 title overrides)*; audited 2026-11-03: ours had no cinema mode; built from [batch 5](notes/2026-11-03-modlog-gaps-batch5-build.md) §1 (`event::cinema`). Left: the +0x4c / +0x50 overrides audit | Partly built | L2494, L2502 | Code check |
| Target window: the state codes and model range; the pet line; the event mode *([batch 12](notes/2026-11-03-modlog-gaps-batch12.md) item 79: the event mode (op 0xb4 2 / 3) is built; the model range 0xb22..0xb31 is the luopans' no-plate gate (MODLOG "Name plates: scheduler op 0xb4's lift"). Open: the pet line, whose master search reads actor +0x74, +0x1fa and +0x1d0 bit 0x100, not mapped to our fields)* | Built apart from the pet line | L2541, L4485 | DLL |
| Name plates: scheduler op 0xb4 (+0x8c0, 8 `cst0` schedulers); the mount sub-case per-race tables; k = FUN_10022340(race) *([batch 12](notes/2026-11-03-modlog-gaps-batch12.md) items 79 / 80: op 0xb4 now lifts the plate; the mount sub-case and k were already decoded and built (L13554, `ride::bone0_drop`); the luopan models 0xb22..0xb31 now have no plate)* | Built | L8708, L2172 | DLL |
| Name plates under the screen fade: retail's one-frame delay | Not built | L5646 | DLL |
| Name plate letters: extent - 1 for the other widths | Not checked against retail | L10638 | Footage |
| netstat: the 3 x network page (modern list feature 17) | Not built | `2026-11-02-helpdesk-reply-mark.md` | DLL |
| Log window 2: game state 0x40's screen; registry A 0x10; whether FUN_10129090's rows count toward the open ticks *([batch 8](notes/2026-11-03-modlog-gaps-batch8.md) item 37; [2026-11-11](../modlog/2026-11-11-easy-rows-ui.md): 0x40 is the front end, registry A 0x10 has no reader, a refill tops up window 1's open ticks)* | Built | L16955 | DLL |
| Friend List: Away back to Online at login *(inf)*; deleting / unblocking / reordering (retail has none in game: the user's call); polcore's message format and status mapping *(item 140, [fragment](../modlog/2026-11-11-friend-status-zone-in-140.md): the reset runs at every zone-in, not only at login, and is built with the status re-read after a PlayOnline return and the status job's rules. polcore's side, [fragment](../modlog/2026-11-11-polcore-notice-message-140.md): the notice's status mapping confirmed and capped, the message data and header titles now polcore's)* | Built apart from the user's call | L17174, L17143, L17196, L17222 | DLL (polcore) |
| Message formatter: the other callers (party join, merit prompts, auction house, level sync, /sea counts); FUN_10080350's source; FUN_10128ce0's flags; FUN_101f4bb0's date lists | Not built / not traced | L6651, L6706 | DLL |
| Trust party lines: grammar kinds not seen used | Not built | L14144 | Data |
| `17 n` moogle motions: S+0x40daa and +0xee *([batch 6](notes/2026-11-03-modlog-gaps-batch6.md) item 17; MODLOG "Moogle motions: the NPC we talked to goes first")* | Built | L19915 | DLL |
| Mask-0 0x00E: kind 9 (FUN_100bd760 / FUN_101c8450) *([batch 6](notes/2026-11-03-modlog-gaps-batch6.md) item 16: message source 9 = the trust lines, language slot 0x22)* | Built | L19835 | DLL |

## 7. Menus and windows

### Items, equipment and storage

| Item | Status | Source | Settle by |
|---|---|---|---|
| Inventory: the job sprite after "Lv%2d "; a slot label under an equipped icon; persona offsets while equipping *(blue names: FUN_100fbc70 = charged item ready, [batch 1](notes/2026-11-03-modlog-gaps-batch1.md) §1, PR #129; scroll bars L8480; count L5453)* *([batch 10](notes/2026-11-03-modlog-gaps-batch10.md) item 57: the job sprite and blue names were settled earlier; persona's layout position is (16, 48); slot labels show through the icons; the wardrobe number is drawn)* | Matches | L1383, L1325 | DLL |
| Item info: item types 0xf / 0x11..0x15 / 0x1a; the de / fr layout and ja rows | Not built | L5077 | DLL |
| Item info: its mode and place while the equipment set editor is open | Unknown | L10248 | DLL |
| Item use menu: the blue / grey row tints for gear in the direct-use list | Not built, not seen live | L5199 | DLL / Live |
| Equipment screen: the unequip toggle | Not seen live | L10449 | Live |
| Equipment screen: the list's cursor row after a grid Enter *(as read)*; the opening set (persona, money, list at (314, 48)) *([batch 9](notes/2026-11-03-modlog-gaps-batch9.md) item 50: both readings confirmed; equipwin.rs)* | Built | L10419, L10503 | DLL |
| Equipment sets editor: the mouse; the wardrobe number glyph (unknown) | Not built | L10228, L10217 | DLL |
| Delivery Box: the augment Ex rule and the equipped linkshell in the picker colours; the monstrosity / menu-lock gates; caret movement in the name; the one-frame Recv-pair miss | Not built | L5700 | DLL |
| Automaton customization: flow B's double SE 0002; SE 0004 repeat under the pointer; unfocused cursor while a slot is picked; pupsibor's starting cursor *([batch 10](notes/2026-11-03-modlog-gaps-batch10.md) item 56; [2026-11-11](../modlog/2026-11-11-easy-rows-ui.md): SE 0004 repeats every frame under the pointer with a pick)* | Built apart from the unfocused cursor box | L16873 | DLL / Live |
| Blue magic sets: the open order *(inferred: bluequip last takes the focus)* *([batch 6](notes/2026-11-03-modlog-gaps-batch6.md) item 11: confirmed, persona, bluehelp, bluinven, bluepoin, then bluequip with the focus; bluset.rs opens on Focus::Equip)* | Matches | L11083 | DLL |

### Shops, trade, bazaar, auction

| Item | Status | Source | Settle by |
|---|---|---|---|
| NPC shops: the Up wrap at the list's top (not checked against FUN_101f67c0) *([batch 5](notes/2026-11-03-modlog-gaps-batch5.md) item 10: retail doesn't wrap; the list base FUN_101f6c70 scrolls at the edge rows and stops silently at the ends, as shop.rs move_list already does)* | Matches | L6339 | DLL |
| NPC shops: DAT_1057876c + 0xc4; the focus back to shopmain; sell order = inventory view order; English width 274; tint helpers FUN_1022b680 / FUN_1022b570 / FUN_10223360 / FUN_102092b0; the Escape sound *([batch 10](notes/2026-11-03-modlog-gaps-batch10.md) item 53: +0xc4 = a pending target prompt, width 256 + 18, the sell tints; the buy tint helpers are the scroll / attachment / key item tests in shop.rs)* | Built | L6373, L6349 | DLL / Footage |
| Guild shops: the cursor reset on source switch; kind 0x19 squeeze or clip; Trade refusal codes other than 0xfb / 0xfd; Stat 2 / bit 31 with Stat 0; DAT head byte 13 | Inferred / unknown | L6485, L6470 | DLL / LSB |
| Field manuals: the query window's scroll bar | Not built | L6545 | DLL |
| Player trade | Built, not tested live (needs a second account) | L6597 | Live |
| Trade: the costume refusal bit 0x40 (FUN_101cc040) *(answered, [batch 1](notes/2026-11-03-modlog-gaps-batch1.md) §7: player +0x120 bit 5 from the costume state machine FUN_10095e90; its four helpers in [batch 3](notes/2026-11-03-modlog-gaps-batch3.md) §1: the status lists only take a costume off, +0x120 bit 7 = bound to the event; [batch 4](notes/2026-11-03-modlog-gaps-batch4.md) §1: 0x40 is checked after 0x04, before 0x08; §4: the costume id is an NPC model id, drawn as the whole body; [batch 5](notes/2026-11-03-modlog-gaps-batch5-build.md) §6: other players too, from 0x00D @0x30)*; built 2026-11-03 (the refusal, the model for us and others) | Built | L6568 | Live |
| Set Bazaar: the name's bazaar glyph (0x9c); the row disabled in Monstrosity | Matches *([batch 7](notes/2026-11-03-modlog-gaps-batch7.md) item 30: both bit writers are 0x00D @0x20 bit 31 and 0x037 @0x2c bit 29 as we read them, the glyph is last in FUN_10097500's chain, and socialme child 5 is already disabled in Monstrosity)* | L9921 | DLL |
| Bazaar View Wares: FUN_10210880's flag (taken as "list is live"); hidden windows from the group walk; the Escape sound; shopbuy's registry flags vs ours stacking the confirm on the log *([batch 10](notes/2026-11-03-modlog-gaps-batch10.md) item 54: the flag = resized for this source; Escape plays the generic 0003; shopbuy has no log-stacking flag)* | Built apart from the group walk's hidden windows | L16503 | DLL / Footage |
| Bazaar View Wares | Not seen live (needs a second character) | L16460 | Live |
| Auction house: whether anything clears the remembered state on zoning; the menus' +0x24 rerun *([batch 8](notes/2026-11-03-modlog-gaps-batch8.md) item 34; [2026-11-11](../modlog/2026-11-11-easy-rows-ui.md): the parcel slots are cleared at each zone-in; the menus' focus hook resets the list's row, as ours)* | Built | L14474, L14493 | DLL |

### Party, alliance, search, linkshell, social

| Item | Status | Source | Settle by |
|---|---|---|---|
| Party-list cursor: the fellow row *(answered, [batch 1](notes/2026-11-03-modlog-gaps-batch1.md) §4: drawn above the pet row, reached past the last row; built with the pet row from [batch 5 build](notes/2026-11-03-modlog-gaps-batch5-build.md) §5)* | Built | L5444 | Build |
| Alliance windows: live test (one session cannot show them); quartermaster 433 / level-sync 493 marks | Not checked live / not built | L5488, L5485 | Live |
| Party menu: the request-history prompt (comyn 9 0x140); the auto-pool area rule (actor +0x18c); the stage-1 mark; Level Sync's empty list text; the mouse in levelsyn | Not built | L8932, L9764 | DLL |
| Party: the receiving side of invites | Untested live (no bot invites) | L8939 | Live |
| Search: /seacom and /seacomup on the same store; scoption's Request / Cancel Request (/partyrequestcmd) | Not built | L9482 | DLL |
| Search: rec +0x14 vs +0x18; flags 0x400 and 0x10000 *([batch 8](notes/2026-11-03-modlog-gaps-batch8.md) item 33: every action keys on +0x18; 0x400 = PlayOnline only (icon 5, knock), 0x10000 = the small status icon; search.rs)* | Built | L7835 | DLL / LSB |
| Search: the generic handler before arealist's list *([batch 8](notes/2026-11-03-modlog-gaps-batch8.md) item 32: the generic key layer runs first and arealist keeps its sounds; arealist.rs)* | Built | L9547 | DLL |
| Linkshell: the generic Escape path from link2.. back to link5 *(inf)*; 0x0E4 (logs only); /lsmes's +0x124 bit 27 *([batch 10](notes/2026-11-03-modlog-gaps-batch10.md) item 55; [2026-11-11](../modlog/2026-11-11-easy-rows-zone-in-forced-follow.md): /lsmes needs the LinkShellFlag too; 0x0E4 asks the level (LSB only logs it))* | Built | L9444, L9586, L9592 | DLL |
| Linkshell Concierge: a focus change between link5 and lscswin *([batch 8](notes/2026-11-03-modlog-gaps-batch8.md) item 31: the window manager's deferred raise and focus; only the hand, help and the pointer change. The pointer put on the new cursor child at every focus change is not built for any window (queue row 163))* | Matches | L15560 | DLL |
| Linkshell: the comment fetch on Enter in the Link List (FUN_1020b3e0 as scresult kind 4) | Not built | L10832 | DLL |
| Block list window: the wheel / scroll bar drag; the window groups (blklist 0x1c suspends gaugewin and others); fewer words open blklist | Not built | L12511, L12520, L12121 | DLL |
| Region Info: Campaign's windows; the mouse on cnqframe; Japanese placements; beseige2 / campsan sprites | Not built / not traced | L9654, L9718 | DLL |
| Status window: the stats request retries | Matches in effect *([batch 7](notes/2026-11-03-modlog-gaps-batch7.md) item 28: retail retries c2s 0x061 against its own 1 s per-id gate; our gate holds 0x061 and sends it when the window opens, so the request always goes out and the give-up path cannot be reached)* | L9017 | DLL |

### Help Desk, Primer, config and misc windows

| Item | Status | Source | Settle by |
|---|---|---|---|
| Help Desk faqsub: the page header against row 0; the hand's frame rate (counter / 24); the keyword box's field frame *([batch 10](notes/2026-11-03-modlog-gaps-batch10.md) item 52; [2026-11-11](../modlog/2026-11-11-easy-rows-ui.md): the hand holds each frame 4 drawn frames)* | Built apart from the keyword box's field frame (footage) | L10025, L10010 | DLL / Footage |
| Help Desk faqmain: where "Input:REQUIRE" is enforced; where 9 0x7c is drawn; the parameter evaluation (GAME.INFIELD / GMTIME); the '#' pages; the err_%03d.bin viewer | Unknown | L10332, L10269, L10322 | DLL |
| Adventuring Primer: the 8 empty row bars for the first 10 frames; a drag's release flipping the page; /primer while up; guide01's arrow sound *([batch 10](notes/2026-11-03-modlog-gaps-batch10.md) item 51; [2026-11-11](../modlog/2026-11-11-easy-rows-ui.md): the bars always draw, /primer while up restarts the wait, arrow clicks sound 0002 + 0001)* | Built | L15290 | DLL / Footage |
| Config: the Show / Hide packet; the macro palette's position rule; auto-disconnect's PlayOnline timer | Not built | L2937 | DLL |
| Config: chat-type filters (server); the mouse movement reference; 0x3e clip scale; 0x40 animation step | Not built | L3069 | DLL |
| Config lists: the key item / quest / mission list pages | Not built | L8495 | DLL |
| List scroll bars: the post / delivery / trade / synthesis lists and the synthesis history | Not built | L8731 | DLL |
| The page modifier (0xa8): the arrows' hover state (a Ctrl-click on an arrow does nothing in retail; ours switches) | Built *([batch 7](notes/2026-11-03-modlog-gaps-batch7.md) item 25: FUN_10221cd0 / FUN_10221ce0 disable and enable the paged list; over the arrows there is no hand and the list ignores keys, and a Ctrl / Alt click pages the disabled list, i.e. does nothing. RoE's arrows and jbpcat's page areas skip the test and are left alone)* | L13962 | DLL |
| Cursor memory: the other KEEP windows' cursors (myroom, configwi, cmbmenu) | Not built | L9171 | DLL |
| Missions: the held-key header stop | Built *([batch 7](notes/2026-11-03-modlog-gaps-batch7.md) item 26: a header landed on by a held key is backed off one row downwards and the base's +0x60 is set to Up, which stops the next held move wrapping at the edge; a fresh press skips the header as before)* | L9102, L9153 | DLL |
| Status window: Monstrosity (mp_stat) from the main menu | Not built at the time | L9017 | *(closed: "Monstrosity, part 5: mp_stat")* |
| Mouse picking: the hover's frame lag, the talk lock; kind 1 over objects | Not built | L5747, L4556 | DLL |
| Mouse picking: "StreetLamp" without a box is not pickable (the box path) | Not built | L5730 | DLL |
| Mouse: right-button release clearing the target (key 0x8c); ours drags the camera with it *([batch 7](notes/2026-11-03-modlog-gaps-batch7.md) item 27: our right release already clears the target, drops a sub-target prompt and stops autorun as FUN_101587f0 does. Retail has no right-drag camera at all: a right drag only turns the cursor into an 8-way arrow and stops the edge pan, and the camera turns when the cursor touches the window edge (FUN_10124bc0). Our mouse-drag camera is a deliberate deviation (modern MMOs); the edge pan is not built and is a design question for Steven / upstream)* | Deviation (release built) | L3221 | DLL |
| GM prompt / Moblin Maze Mongers: d5 bits 4..10; message 7's middle-button source; the late 0x0D8 at the event end; helpwind's line with grid focus; the pulse timing assumes 30 fps | Unknown | L17934 | DLL |
| Maze Mongers: LSB logs GP_CLI_COMMAND_DUNGEON_PARAM as not implemented | Blocked: LSB | L17848 | LSB |
| Creator random names: FUN_101ba240 (validity check, ours skips it) *(items 48 + 49, [fragment](../modlog/2026-11-11-creator-name-filter-48-49.md), 2026-11-11: a random name the dictionary refuses is dropped)* | Built | L5346 | DLL |

## 8. Map window

| Item | Status | Source | Settle by |
|---|---|---|---|
| Markers (mapv3 / mapv2, the .MRK file, sets 0..3) | Not built | L5029, L2564 | DLL |
| Party dots (c2s 0x0D2 / s2c 0x0A0 positions) | Not built | L5029, L5041 | DLL / LSB |
| Change Map; the Mog House branch (myroom); group 7 messages 97 / 98; the no-key-item chat line (group 10 id 7) | Not built | L2564, L5029 | DLL |
| The left-drag joystick | Not built | L5011 | DLL |
| The view and scroll-to formula, fade levels and the arrow taken at open only: as decompiled, flagged open | Inferred | L5012 | Footage |
| The collision map id under the player (key 0 used); the arrow's heading sign *([batch 7](notes/2026-11-03-modlog-gaps-batch7.md) item 24: the key is Ground::map_key (MODLOG L4598); the arrow turns by +yaw taken at open (FUN_101f8ea0, no negation), as map.rs draws it)* | Matches | L2545, L2558 | DLL |

## 9. Mog House

| Item | Status | Source | Settle by |
|---|---|---|---|
| Moogle menus: the furniture list's Change Jobs row; Friend List row (flistmai); op 0x85 not wired | Not built at the time | L5527..L5564 | DLL *(several closed later: "Change Jobs", "The Friend List")* |
| Layout mode: what reads target system +0x74 bit 4 and myroom +0x7e4 | Not traced | L14283 | DLL |
| Layout mode: each furniture reload adds its textures to the GPU again (31 -> 75 over five moves) | Known bug | L14308 | Ours |
| Mannequins: whether a mannequin shows its range weapon; the other races' banks 3..31; the ground shadow uses the zone light | Not checked / not built | L14748, L14743, L14732 | DLL / Footage |
| Furniture: whether the 3 blended pieces keep z writes *(inf)* *([batch 7](notes/2026-11-03-modlog-gaps-batch7.md) item 23: yes. FUN_101844c0 sets neither ZWRITEENABLE nor ZBIAS, so furniture keeps the actor pass's writes; zone scenery turns them off)* | Built | L15364 | DLL |
| Gardening: what 0xbf8's 'main' shows *(inf: a vanish effect)*; a plnt window resumed on a stale pot; the item list's Escape sound; container 9's view order; /garden in mode 3 | Unknown | L17042 | DLL |
| Camera collision: Mog House furniture in the camera ray | Not built | L11145 | DLL |

## 10. Combat and battle

| Item | Status | Source | Settle by |
|---|---|---|---|
| The target cursor for Switch target | Not built | L1506 | DLL |
| Skillchains: the 0x04 MagicBurst message (Swipe / Lunge); 67507's purpose *([batch 9](notes/2026-11-03-modlog-gaps-batch9.md) item 47: the client formats nothing from the info bits (scheduler value 0x2b only, built); 67507 is not reachable from the burst table)* | Answered | L6959, L6943 | DLL |
| Knockback: vt+0x304 / +0x103 / +0x104 in FUN_100a57f0 *([batch 6](notes/2026-11-03-modlog-gaps-batch6.md) item 15: vt+0x304 = our own control actor; +0x103 / +0x104 are the approach / return moves of scheduler ops 0x11..0x14; [2026-11-11](../modlog/2026-11-11-easy-rows-zone-in-forced-follow.md): no scheduler in any DAT has those ops, not built)* | Closed | L7179 | DLL |
| Cast bar: Chainspell (status 48) skipping it; items (mode 1) | Built *([batch 7](notes/2026-11-03-modlog-gaps-batch7.md) item 29: FUN_1022b3d0 opens the bar only when FUN_1021e670(0x30) == 0, a scan of the 32-u16 icon list for 48; item use opens mode 1 for record +0x26 x 20 ticks before c2s 0x037; mode 4 draws the seconds only)* | L4837 | DLL |
| K.O. window: raise / tractor offers (s2c 0x0F9) | Not built at the time | L4860 | *(closed: "The raise prompt", "The raise offer during a zone-in event")* |
| Timers Extra: blue magic's and pet moves' added effects | Not built | L13649 | Ours |
| Engaged state: FUN_1013eff0 *(inferred)*, DAT_106218ee's source, FUN_1015e6e0's exact effect *([batch 9](notes/2026-11-03-modlog-gaps-batch9.md) item 46: the pool test, config 0x9e forced off on narrow screens, FUN_1015e6e0 = the generic hide)* | Built | L6784, L6790 | DLL |
| Monstrosity: party slot reuse and +0x4c; job 23's icon / name in party rows and /check; record words +0x2c..+0x30; the list's +0x5c tag | Unclear | L15685, L15702, L15714, L16078 | DLL |

## 11. Fishing and mini-games

| Item | Status | Source | Settle by |
|---|---|---|---|
| Fishing: FUN_10127ea0 (drops queued text after a mode-3 send *(inf)*); other anglers | Not built / not seen live | L15050 | DLL / Live |
| Fishing: how plain rods draw their line *(inf: a mesh in `hf_s` animated by fh motions)*; LSB drops a blocked 0x01A without an answer, so the latch could stick | Unknown | L15117 | DLL / LSB |
| The old (PS2-era) fishing: statuses 6, 0x26..0x2b, 0x32..0x35; s2c 0x081 | Not built (LSB never sends it) | L15055, L15004 | LSB |
| Chocobo racing: grade -1 (unknown) | Unknown | L17512 | Data |

## 12. Blocked by the server (LSB never sends it)

| Item | Source |
|---|---|
| Campaign data (s2c 0x071 mode 2: the sender is commented out in LSB) | L9697 |
| Corsair dice colours need player +0x2cf4a, set only by s2c 0x063 sub 4 Monstrosity data | L6510 *(closed later: "Monstrosity, part 7: the dice colours")* |
| The old fishing system (s2c 0x081) | L15055 |
| Alter Ego Points upgrade (LSB re-sends 0x08E unchanged) | L12152 |
| Maze Mongers dungeon param (c2s logged "Not implemented") | L17848 |
| The airship ride (needs an airship pass); the skillchain helper's properties (LSB keeps them server side) | L18170 |
| Remapster's 109 extra maps: no file says which zone or floor (later found to be duplicates) | L18170, L18285 |
| PvP word s2c 0x0B4 @0x10 (LSB sends 0) | L15191 |

## 13. Ours (non-retail) and upstream questions

These are listed only. Per the project rules, design questions that belong upstream stay open here.

| Item | Status | Source |
|---|---|---|
| Lua mods: `mods.call` runs the provider's function with the provider's grants; whether it should check the caller's grants is upstream's permission model (issue #70) | **Upstream question, open** | `2026-11-02-lua-mod-safety.md`, `2026-11-02-lua-mods-call-suspended.md` |
| Point lights in place of baked lamplight: the user chose retail's look then ("Parked"); later built as the Lamp Lights Extra | Ours, parked then built | L4109, L18329 |
| Modern UI flyouts: top / bottom joints | Not built (flyouts later taken out) | L19029, L19127 |
| Cutscene NPC pose easing between 60 Hz VM steps | Ours, not retail | `2026-11-02-event-body-smoothing.md` |
| Remote actors over holes in the floor; casting right after stopping; auction house additions; synthesis amount | Ours, not retail (deviations by design) | L17758, L19393, L17778, L5815 |

## 14. Live-check backlog

Built from the DLL but never seen in game (beyond the rows above):

- Lock-on release after a kill; the loading indicator during a real zone change (L2623, L2602).
- The lock-on live check and its input refusals (L3221).
- Target flash at night (L3028).
- Relogging after the effects' GPU cache fix (L11184).
- Linkshell with a real shell: create, equip, switch, Create Pearl (L9415; partly closed by "Linkshell: a real shell, live", L14878).
- Link List (L10819), alliance windows (L5488), party invites received (L8939), player trade (L6597), bazaar View Wares (L16460).
- Equipment unequip toggle (L10449); item use direct list (L5199).
- Chat call sound lines (L7410); charlnk clipboard (L17120).
- The 0x1f7 opening after the VM-speed fix (L2408); the event ops built on 2026-11-02 (section 2).
- Main menu's Map zoom resume (L10390).
- Ground tilt and riding yaws against footage (L12739, L12469).
- Cloth sway on the 27 cloth models (`2026-11-03-cloth.md`).

## 15. Retail 30261102_0 (the November 2026 update)

<!-- dll-build: 30261102_0 -->

What FFXiMain.dll 30261102_0 added or changed against 30261006_0, from the full function and data comparison
(`docs/modlog/2026-11-10-rebaseline-30261102.md`). Addresses in this section are 30261102_0. Queue numbers 151..159.

| Item | Status | Source | Settle by |
|---|---|---|---|
| In-line colour code 9 (`1e 09`) is 0x60404040 now (table 0x10336510 + 0x24): the client reads it from the DLL (signature fixed 2026-11-10); which retail text uses the code, and that ours draws it grey | Built, not checked | 2026-11-10-rebaseline-30261102.md | DLL, Live |
| List cell kind 0x4d (FUN_101f60f0 accepts it, FUN_101f6180's jump table 0x101f7b98 case 0x101f7b57: FUN_1015d340(u16 value, x, y + 2, 0.5, 0, colour, 0)); which window sets it, and ours' list cells | Not built | 2026-11-10-rebaseline-30261102.md | DLL |
| statcom2 child 0xd: enabled at main level above 0x62 with key item 0xd21 or (new) 0xd2f (FUN_1014a0b0); what the child is and whether ours enables it at all | Not built, not traced | 2026-11-10-rebaseline-30261102.md | DLL |
| Mog House layout Move keys (FUN_101cf930): the 0x8d / 0x8e steps are skipped while FUN_101d01b0 (bit 0 of *(this + 0xf0)); the re-centring now runs before the FUN_101d01d0 test. What the bit is, and ours' Move | Not built, not traced | 2026-11-10-rebaseline-30261102.md | DLL |
| s2c 0x08F (FUN_100e5290): 0x2b dwords to session +0x2d67c, callback session +0x2d728, "received" bit 5 of session +0x40e65; cleared by FUN_100e4340 / FUN_100e4420 / FUN_100e44e0. Field layout as fe_equip reads it (six sets of ten slot bytes from +0x2a, step 0x14; the rest unread so far) | Not built (LSB sends a stub) | 2026-11-10-rebaseline-30261102.md | DLL, LSB |
| c2s 0x0C5 (FUN_100e5200: 0x10 bytes, +4 set, +5 ten slot bytes, from FUN_10117b70) and c2s 0x0C6 (FUN_100e5260: 4 bytes, asks for 0x08F; from fe_equip's open FUN_101173d0 while the bit is clear) | Not built (LSB has stubs) | 2026-11-10-rebaseline-30261102.md | DLL, LSB |
| Menu fe_main (object DAT_105795f4, vtable 0x10335680: FUN_1014caa0 enables children by level and key items 0xd21 / 0xd2f, FUN_1014cb40, events FUN_1014cb50); its creation in FUN_10147740 and deletion in FUN_10148100; where it opens from; the class at vtable 0x10335638 that gained the destructor FUN_101480d0 | Not built | 2026-11-10-rebaseline-30261102.md | DLL |
| Window fe_equip (DAT_104e2d20, 100 bytes, vtable 0x10333fe8, FUN_101170f0): open FUN_101173d0, FUN_10117480, draw / fill FUN_101174b0 + FUN_10117580, FUN_101176e0, keys FUN_10117750, FUN_10117290, FUN_101172c0, FUN_10117980; helpers FUN_10117310, FUN_10117360, FUN_101173b0, FUN_101179d0..FUN_10117c00 | Not built | 2026-11-10-rebaseline-30261102.md | DLL |
| Window fe_inven (DAT_104e2d24, 0x60 bytes, vtable 0x10334058, FUN_10117c30): FUN_10117d60, FUN_10117d90 + FUN_10117e60 (the list), keys FUN_10118260, FUN_10117cb0, FUN_10117ce0; helpers FUN_10117d30, FUN_10118110..FUN_10118220 | Not built | 2026-11-10-rebaseline-30261102.md | DLL |

## Closed by later entries

Items an early entry lists as not done or open that a later entry built or answered. Kept so nobody re-opens them.

| Early item | Source | Closed by |
|---|---|---|
| Combat: effect files, PC stance / swing, multi-hit, party / alliance relations, chat filters, the kesu fade | L1506 | "Effect schedulers", "Event motions, actor fades, PC battle motions", "Multi-hit melee rounds", "Battle message relations", "The full log's tabs and the log filters", L4755 |
| Event motions not loaded; actor alpha fades; other scheduler ops; item use, iteminfo, sorting, containers | L1634 | "Event motions ..." (L1644), "Event schedulers run in the effects engine", "Item use menu", "Item info window", "Item sort window", "Storage views" |
| Dual-wield files; motion-scheduler sounds | L1771 | "Dual wield", "Actor sounds 1..4" |
| No map window; camera roll; the alliance windows; 0x5c volume subs | L2408 | "Map window", "Cutscene camera roll", "Alliance windows", "Event sound volume" |
| Map: fades, recentring scroll, pan / zoom keys, sub-map key, Wide Scan | L2564 | "Map window: pan, zoom, fades", "Clock grid position and map sub-keys", "Wide Scan leftovers" |
| Log: filters 1..8, charlnk, mouse wheel / scroll bar | L2581 | "The full log's tabs and the log filters" (L16117), "charlnk", "List scroll bars" |
| Config: keylayou, list scroll bars, style lock, second log window, window open / close effect, shadows, damage pop-ups | L2937, L3069 | "Config: keylayou", "Config lists", L10114, "Log window 2", "Window open / close animation", "Actor ground shadows", "On-screen damage numbers" |
| Buff window | L2623 | "Status icons" |
| Effect sound ops, weapon ops 0x75 / 0xa3, per-race weapon-skill files, op 0x46, config 0x92 filter | L3450 | "Effect sound", "Weapon meshes hidden and shown", "Weapon skills from NPCs and trusts", "Emission op 0x46", "Effects filter" |
| Footsteps; race death voice; one-at-a-time SEs | L3554 | "Footsteps", "Scheduler op 0xac", "Motion name bases ... one-at-a-time loops" |
| Scenery lighting per area | L4297 | "Scenery and actors lit by their area's environment" (L4301) |
| Mog House furniture, s2c 0x02E and moogle menus, 2F | L4414 | "Mog House furniture", "Moogle menus" |
| Elevators, ships | L4445 | "Actor collision and lifts", "Ship voyages", "Ship actors" |
| Magic menus: sort windows, mount list, blue magic sets, cast bar, mouse | L4796 | "Magic / ability sort windows", "The mount list", "Blue magic sets", "Cast bar", "MacroPalette: the mouse" etc. |
| Alliance lot column / lnowin | L5488 | "Treasure pool window: loot, lootope, lnowin" |
| Mount model under the rider; riding layouts; emote list | L5515 | "Riding", L5553, "The emote list" |
| Op 0x51 over the scene | L5646 | "Op 0x51: the level over the scene and the windows" |
| Treasure pool window, lot / pass | L5913 | "Treasure pool window" |
| Block list; PvP word | L5953 | "The block list", "SysMesFilterLevel and the PvP word" |
| Auto-translate input (Tab, menu fep) | L6116 | "Auto-translate input", "Auto-translate in the macro and comment fields" |
| Guild shops; can-equip tint | L6339 | "Guild shops", "Shops: the can-wear name colour" |
| Scroll colours kinds 2 / 3 | L6510 | "Monstrosity, part 7", "Shops: automaton attachment colours" |
| Player trade; trade mouse; trade with no target | L6561 | "Player trade", "Trade windows: the mouse", "Trade with no target" |
| 7f ab / 7f ac; /playtime | L6651 | "Event text date codes 7f a0..ac", "Text commands: toggles, /ver, /playtime" |
| Knockback, guard / parry, op 0x71, skillchain burst | L6860 | "Knockback", "Per-record reactions", "Skillchains" |
| Tell sound "0p40" | L6900 | L7416: retail finds nothing either; not played |
| Status icons Num+ focus; party member icons, and their live test (L7394) | L7324, L7370 | "Status icons: Num+ focus", "Status icons: party members", "Party member status icons, live" (L14902) |
| Engaged menu: loot window, mouse hold, inspect | L7433 | "Treasure pool window", "Action menu: the mouse hold", "/check on a player" |
| Search: grammar, criteria windows, comment fetch, acoption, LS member list | L7763 | "Search: the /search grammar", "Search menu, part 2", "charlnk ... and acoption", "Linkshell: the Link List" |
| Party menu: Member List, View Request, Level Sync, Focus, Rearrange | L8932 | "Party menu: Member List, Level Sync ...", "Party requests" |
| Status window: Currencies, Unity, Merit / Job Points, Alter Ego Points | L9017 | "Currencies", "Unity", "Merit Points and Job Points", "Alter Ego Points" |
| Key Items: event op 0xcc 0x20 | L9231 | `2026-11-02-event-key-item-window.md` (PR #53) |
| Search menu part 1 rows | L9269 | "Search menu, part 2", "Search: arealist, scsibori" |
| Bazaar buyer side | L9921 | "Buying from a player's bazaar: View Wares" |
| Macro auto-translate, caret, mouse | L10057 | "Auto-translate in the macro and comment fields", "Macro fields: the caret", "MacroPalette: the mouse" |
| GM reply page | L10293 | "Help Desk: the GM reply page" |
| Link List (link12) | L9416 | "Linkshell: the Link List and member actions" |
| Effect shine overlay ops 0x40..0x47 | L11265 | "The actor shine overlay" |
| Gray select screen on relog | L11185 | "The gray select screen after a relog" |
| Near-camera character fade | L11145 | "The near-camera character fade" |
| Mog House layout mode parts 1..4; mannequin dressing | L14242..L14699 | "Mog House layout mode, part 2..4", "Mog House mannequins, part 2: dressing" |
| Event text 0x1e inline colours; 0x17 emote | L15345 | "Log lines: in-line colours and indents", "Event text `17 n`" |
| Creator: netwait / netbar, world step | L15623 | "Lobby: netwait and netbar", "Creator: the world step" |
| Friend List: netstat's Help Desk count, second count, busy strip 0x10 | L17174, L17176 | `2026-11-02-helpdesk-reply-mark.md`, `2026-11-02-netstat-second-count.md`, `2026-11-02-busy-strip-0x10.md` |
| rem4line magenta tint | L17275 | `2026-11-02-overflow-tint.md` |
| Ship actors (0x00E SubKind 4); path drive ops 0x26 / 0x27 | L13802, L3895 | "Ship actors" (L17979, L17995) |
| Idle variants dft / df1 | L13299 | "MonStat idles" (L17606) |
| Lamp Lights in sub-area instances | L18362 | "Lamp Lights: sub-area instances" (L18764) |
| Event ops 0x20 / 0x21 (Maze Mongers) | L17810 | same entry |
| Every missing event VM op from the 2026-11-02 audit, and 0x97 cloth wind | memory / fragments | PRs #39..#56, #118 |
| *Checked 2026-11-03: the rows marked "may be closed"* | | |
| Event-untargetable actors (+0x128 bit 4) in the targeting keys | L4509 | "Event actor toggles, first batch" (L14057); `target::candidate` skips `target_off` |
| Where PC battle motions load from | L1471 | "Event motions, actor fades, PC battle motions", section "PC battle motions" (L1705) |
| Spawn-in 'spop': ops 0x07 / 0x59 / 0x2e motion locks | L4759 | "Scheduler task ops: holds, locks, loops, flow" (L11275) |
| Combat motions: multi-hit rounds; the weapon attach switch on sheathing | L4703 | "Multi-hit melee rounds" (L4869), "Drawn weapons ride the grips" (L20151) |
| Actor sounds: the hide skip, `?ead` / `sdep`, fishing | L7409 | "Motion name bases, `sdep`" (L13309, L13313), "Scheduler op 0xac" (L12836), "Fishing" (L15109) |
| D3b morph: normals / lighting; op 0x6e | L3652 | "Lit D3b / D3m effect elements, element-update op 0x6e" (L13370) |
| UI sounds: the `<call>` sounds | L6900 | "Chat call sounds" (L7411) |
| Chat line: the cursor inside the line | L6192 | "The chat line's caret" (L10179) |
| HUD: chat line colours per record | L309 | Font Colors (L2928), `cfgapply::chat_colour` |
| Main menu: the arrow pages and where Log Out lives | L253 | L294 (menuwind / socialme pages; socialme child 12 Log Out) |
| /pcmd add NAME outside the zone; /befriend | L7596, L12353 | "Search: the search server client and name lookups" (L7618), Friend List /befriend (L17163) |
| Engaged menu: party-list cursor cancel on engaging; Sprint's area rule | L7433, L6724 | "Action menu: ... party cursor on engaging" (L8363); Sprint is Conflict only (L7431, `combat.rs` FUN_1012af30 cases 0x16..0x18) |
| Text commands with no consumer (/ignorepet ... /autotarget) | L7496 | "The toggles' consumers" (L12160); /bell note, /ignorepet, /hidetrust not seen live |
| Records of Eminence: Enter on rows 2..4 | L9195 | "Records of Eminence lists" (L10971) |
| Synthesis: the history and favourites, cmb%d.dat | L5763 | "Synthesis menu, history, /lastsynth" (L5789, L5818) |
| *DLL batches 1 and 2, 2026-11-03* | | |
| Op b1 sub 4 length | L835 | [batch 1](notes/2026-11-03-modlog-gaps-batch1.md) §8: subs >= 3 never advance in this DLL (the event stalls); `event_ops.rs` was right |
| Ops 46 sub 2, 69 / 6a, units of 1f | L835 | [batch 2](notes/2026-11-03-modlog-gaps-batch2.md) §1: 0x46 sub 2 queries the follow camera; 0x69 / 0x6a are sound volume / fade (ours matches); 1f in thousandths, speed in units / s |
| Scheduler task restart on a type-1 op | L1266 | [batch 2](notes/2026-11-03-modlog-gaps-batch2.md) §2 (FUN_10057c20: list C byte 1 or +0x141; ours matches) |
| Party requests: own +0x124 bits 22 / 23 | L10906, L10933 | [batch 2](notes/2026-11-03-modlog-gaps-batch2.md) §6: Away / Anonymous; /prcmd add refusal in PR #131 |
| rem4line: which rows lose their mark | L17291 | [batch 2](notes/2026-11-03-modlog-gaps-batch2.md) §7; PR #133 |
| Select screen: Escape in a prompt; "Connecting to the server..." | L2799, L2832 | [batch 2](notes/2026-11-03-modlog-gaps-batch2.md) §8; PR #132 (Escape = No only in the settings-file prompts; the line is ours, retail's nearest is group 8 id 3) |
| Actor collision: the object id skip | L2694 | [batch 1](notes/2026-11-03-modlog-gaps-batch1.md) §3: the loaded sub-area id; already built in `dat/collision.rs` |
| Storage / item lists: the equipped linkshell row | L5403 | [batch 1](notes/2026-11-03-modlog-gaps-batch1.md) §2; PR #130 (grey in storage, green in item lists) |
| Help Desk faqmain: the language row | L10322 | [batch 1](notes/2026-11-03-modlog-gaps-batch1.md) §6: dormant in this build (never drawn or selectable); nothing to build |
| conf6win's key-reset prompt | L4631 | [batch 1](notes/2026-11-03-modlog-gaps-batch1.md) §5; PR #127 |
| *DLL batch 3, 2026-11-03* | | |
| Op 0xab 9 / 0xa | L5254 | [batch 3](notes/2026-11-03-modlog-gaps-batch3.md) §3: 2 bytes, no operands (the Z register note was op 0xb6 0xd / 0xe's); 9 forces the effects multiplier (config 0x3b, the Misc. weather option *(inf)*) to 1.0, 0xa restores it. Ours never applies 0x3b, so effects are always full: nothing to build |
| Positional effect volume | L918 | [batch 3](notes/2026-11-03-modlog-gaps-batch3.md) §6: listener the camera eye, near 3 / far 30 linear, height x 3, x 0.65 behind; `fx/sound.rs` attenuate matches |
| Scheduler op 0x56; the 0x100a7440 end callback | L3531, L3463 | [batch 3](notes/2026-11-03-modlog-gaps-batch3.md) §5: 0x56 holds each target's status motion (built in `fx/tasks.rs`); the callback (0x100a8db0 here) drops the caster's skill files when its last `main` ends (built) |

# MODLOG gaps, batch 5: build questions (RE notes)

Copied from Steven's `/run/media/SSD/FFXI/re-notes/modlog-gaps-batch5-build.md` (2026-11-03). Target and tools as in
[batch 1](2026-11-03-modlog-gaps-batch1.md). All addresses are VAs in that DLL. **(inf)** = inferred.

## 1. Cinema mode: what stops drawing

Cinema mode behaves like "UI hidden", except the event text box (rem4line) stays. The flag is `[DAT_105781cc] +0x48`.
Only three sites read it through the global: the per-window draw at 0x101177ee, the getter FUN_1015dce0, the window
docking at 0x1015f7cf.

- FUN_1015dce0 (0x1015dce0..0x1015dcf9) returns `rem4line+0x48 != 0 || FUN_10160700()` (manager 0x10621838 +0xb4 ==
  1, UI hidden). 20 call sites: cinema and "UI hidden" share almost all their gating.
- Per-window draw FUN_101177d0 (0x101177ee..0x10117803): counts +0x56 down, then, if cinema is on, the window is not
  rem4line's own (`[rem4line+8] != window`) and its flags +0x34 lack 0x40, returns at 0x10117ed7 before drawing. Only
  mapframe and cnqframe (0x20042) have 0x40 in the registry 0x103722d8 (entries 201 / 207).

| UI part | Drawn in cinema? | Evidence |
|---|---|---|
| rem4line (the event text box) | yes (its own window is exempt) | 0x101177fa |
| mapframe, cnqframe (flag 0x40) | yes | 0x101177ff |
| map0 / mapv2 / mapv3 / mapscan / maplist | no (no 0x40) | registry 200..206 |
| logwindo / log window 2 (flags 0x93) | no; the log's shrink / fade FUN_101624b0 is frozen too | 0x10117803; 0x101628a3 |
| rem4li2, spoolmsg, splmsg2, fulllog | no | 0x101177fa |
| titlewin, titlehan, helpwind, netstat | no | 0x10117803 |
| compass (DAT_10666e84, outside the window loop) | no: FUN_1021ed90 skips the draw FUN_102203b0 but still runs the fade update FUN_1021f240 (inf) | 0x1021eda4..0x1021edab |
| partywin, prty1..8, raid windows | no | 0x10117803 |
| targetwi, subwindo | no | 0x10117803 |
| buff (status icons), gaugewin, casttime, eventtim | no | 0x10117803 |
| chat input line | not drawn, and it can't be opened | 0x10117803; 0x101b0065 |
| menu hand cursor | no; the menu mouse shapes are suppressed | 0x10125e8e..0x10125efe |
| OS mouse cursor FUN_10124eb0 | yes (end of FUN_1015f6a0), shape limited to 0 / 4 / 5 | FUN_1015f6a0 tail |
| name plates, target arrow (3D passes) | not gated by this flag (batch 2 §3 rules apply) | no reader there |

Input blocked while it is on (all through FUN_1015dce0, so also while the UI is hidden):
- All menu keys: FUN_1015dd00(key) returns 0 at 0x1015dd03..0x1015dd0a without dispatching to the focused window
  (FUN_10118260); also blocks the "key 7 opens menuwind" fallback (FUN_1015ee40). Every pad / keyboard menu command
  from FUN_101ab820 and the mouse helpers FUN_1015dd90 / de50 / def0 / df20 go through it.
- The scroll-bar mouse FUN_10234930 (skipped at 0x1015f6d8); the mouse cursor state machine FUN_10125360 (menu drag /
  hover / wheel / shapes).
- Player control FUN_101579c0: 0x101579e2 blocks pad key 0x24; 0x10157a83 (state 0x60) skips FUN_10157e20 (cancel /
  Esc), FUN_10159780, FUN_101585f0 (Enter on the target: the action menu) and FUN_101581e0 (targeting keys) (inf on
  the labels). Movement (FUN_101582f0 / FUN_101583f0) still runs.
- Macro palettes: FUN_101a33e0 returns 0 at 0x101a340c. Hotkey categories 3 / 4 (FUN_101ae010 at 0x101ae05f /
  0x101ae0ca); categories 1 / 2 still work.
- The chat line can't be opened: chat vtable 0x10337ad8 slot 16 = FUN_101b0060 returns false (0x101b0065), checked by
  the keydown FUN_101b29f0 and others. A line already open keeps editing (inf).
- List-row hover in the delivery box (0x1014f2c9) and hnhead (0x1023f7bf). Docking FUN_1015f790: with cinema on and
  FUN_10087070(0) (event lock) it returns 0 and doesn't re-stack windows.
- Not blocked: advancing the event text (FUN_10088b50 / FUN_10088bb0, no cinema check; inf).

Writers, all through FUN_10136870(this, on, x, y) -> `[ecx+0x48]` at 0x10136879 on both `[0x104e2150]+0x30` and
`[0x104e2154]+0x30`:
- Set: event op 0x67 (FUN_100b79c0, with x / y); zone-in with event Mode bit 0x80 (0x100952e7..0x1009531f, x = y = 0);
  text code 0x03 (FUN_100c1650 case at 0x100c22ef): y -> DAT_104899b2, on, x from DAT_104899b0. Text code 0x02
  (0x100c22c5) only stores x in DAT_104899b0 and doesn't turn it on.
- Clear: op 0x68 (FUN_100b7a20); the event end (0x100ae8ee / 0x100ae8fe in FUN_100ae3f0); FUN_10014ad0 (the state-0x90
  select / logout path); the ctor FUN_10136380.
- Correction: "FUN_10132030" is not a function in this build (0x10132030 is mid-function).
- rem4line's own reads of +0x48 only change its layout: in cinema the box sits at the absolute x / y (+0x4a / +0x4c),
  with no frame or cursor (window +0x84 / +0x86 cleared), and no longer snaps to the log window.

## 2. FUN_1008e4d0's exemptions

Correction to batches 2 / 3: FUN_1008e4d0 only sets / clears +0x120 bit 23 ("unload this model"). The exemptions are
applied by the consumer FUN_10095900, which decides whether to call FUN_10092900 (delete the model).

FUN_1008e4d0, exact order (a = actor, P = player FUN_101cc4f0):
```
a+0x120 &= ~bit23                                   // every frame
if bit9 (model):
  if bit7 && DAT_104807f5:  bit17 ? HIDE : SHOW      // bound during an event
  elif !(bit14|bit15): SHOW                           // FUN_10085a10: drawobj +0xb2 bit1 clear; a+0x120 &= ~bit16
  else: if bit18 && !(a+0x138 bit31): bit23 = 1; HIDE // FUN_10085a50: +0xb2 bit1 set; bit16 set
if bit8: return
if (P+0x130 bit11) && P+0x1ff in {2..6, 40..43}
   && (a+0x1d0 bit0) && (a+0x128 & 0x3800) < 0x1800 && a+0x1ff not in {2..6, 40..43}: bit23 = 1
if FUN_10193890(5) && (a+0x12c bit22): bit23 = 1, unless a is in the list FUN_100e8b80(0).. (inf: party / alliance)
if bit7 && DAT_104807f5: return
if a+0x138 bit31: return                            // event-reserved (op 0xac sub 2)
if a+0x120 & (bit14|bit18): bit23 = 1
if (a+0x78 & 0xff000000) == 0 (PC): if (DAT_104808fc|DAT_1047fb40) & 0x02: bit23 = 1
else:                               if (DAT_104808fc|DAT_1047fb40) & 0x10: bit23 = 1
```
(+0x1ff = a status byte from 0x00D @0x29 at 0x1009a652; its 2..6 / 40..43 meaning not resolved.)

FUN_10095900 (bit 23 consumer; +0x12c bit 29 = done this frame):
- Pass 1 (priority actors: +0x12c bit 27 / 28, +0x1d0 bit 2, or the player's pet): delete only if bit 23 &&
  FUN_100961d0 && FUN_10096200 && !(+0x130 bit 3); else load under budget.
- Pass 2: every actor with `!FUN_100961d0 || !FUN_10096200` is loaded unconditionally (no bit 23, no budget). This is
  the real exemption.
- Passes 3..5: delete when bit 23 && !(+0x130 bit 3); the over-budget delete is also blocked by +0x130 bit 3.

(a) +0x130 bit 3 = client-made local actor, never culled. Set only by client paths that create local actors (local
actors from slot 0x3ff down, the Mog House furniture-item actor, mannequins, "Furniture", the Mog House "Door (Back to
Town)", FUN_101d4a20); cleared by the actor reset FUN_1008a350.

(b) FUN_100961d0 returns 1 ("may be culled") only when `(short)+0xf2 <= 0` and +0x120 bit 7 and bit 8 are clear. +0xf2
is a hold count (inc FUN_100833b0, dec FUN_100833c0). So bound actors, participants and held actors keep their model.

(c) +0x138 bit 31: only event op 0xac (FUN_100bb180) sub 2 sets it, sub 3 clears it and queues c2s 0x016 for that
actor. Bits 0..30 are player-side state (not traced).

(d) Doors / elevators / ships are exempt: FUN_10096200 returns 0 for +0xee in {3, 4, 5} (0x00E @0x30 & 7 = 2 / 3 / 4),
so pass 2 always loads them.

## 3. 0x00E SubKind 6: actor +0xf6 = 2

+0xf6 picks the status-motion file set for a non-PC model. Only reader FUN_10084460; its one caller FUN_100d1c80 loads
the actor's tag-3 extra motion file (after all 9 look slots load, kinds +0xee 0 / 1 / 6).
- Writers: actor init (0); 0x00E SubKind 1 at 0x1009cb2a: 1 when u16 @0x32 == 0x18; SubKind 6 at 0x1009ce1b / 0x1009ce45:
  2 when raw @0x32 is in 0x7b8..0x7cb or 0x7d3..0x7da.
- f = 1 or 2: file id = word table 0x1035d834[f] + +0x8a0 ({0xdfd9, 0xdfd9, 0xdf99}: f 1 -> 57305 + idx, f 2 -> 57241 +
  idx); f 0 or >= 3: no file. The index +0x8a0 is set by FUN_100d1c30(idx) from the status-motion change FUN_1008b460
  (idx = +0x1c0, or +0x1c0 >> 3).
- (inf) The two sets are two skeleton families of humanoid NPC models.

## 4. Event op 0x69 mask 0x10: DAT_10351710

- FUN_100380f0 (the chat `<call>` sound start): if a call sound task is up (DAT_10463938 != 0) it does nothing; else it
  makes the task and plays the type-0x3d sound with volume = ftol(DAT_10351710 x volArg x 127.0), task +0x34 = 120 ticks,
  +0x3c = the rumble flag. At 0 it still starts the sound, at volume 0.
- FUN_100381d0 (that task's tick): at 0x10038208, if DAT_10351710 == 0 the task deletes itself at once (no rumble);
  otherwise it drives the pad rumble pulses.
- Writers: the sound init FUN_100355f0 sets 1; event op 0x69 sets the level (1 if u8@1 == 0, else 0). No reset at event
  end or zone change.

## 6. Costume on other players

- FUN_10095e90 runs for every actor the frame loop buckets, other PCs included.
- On: s2c 0x00D @0x30 (u16) differs from +0x166 -> stored, +0x120 bit 4 (pending). Next frame: pending, id != 0, not
  bound -> model freed, bit 5 set, +0x128 |= 0x40000000, +0xee = 2; the model reloads: case 2 of FUN_100d2f60 =
  +0x130 bit 21 ? FUN_100c51c0(2, slot 0) : (bit 5 ? FUN_100c51c0(0, +0x166) : FUN_100c51c0(0, slot 0)).
- Off: 0x00D @0x30 = 0; bound to an event (comes back once unbound); a status in {2, 3, 7, 0x21, 0x2c, 0x2f, 0x30,
  0x3f..0x53} or {5, 6, 0x26..0x2b, 0x32..0x35, 0x38..0x3e, 0x55} while on (inf: it then flips on / off every frame
  while the status lasts). A motion does not turn it off.
- +0x130 bit 21 = a furniture model (the Mog House furniture-item actor): type 2 table FUN_101cce00.

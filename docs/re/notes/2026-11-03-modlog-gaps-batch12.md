# MODLOG gaps, batch 12 (queue items 71..80)

Target and tools as in modlog-gaps-batch1.md (`~/ffxi-pad-re/FFXiMain.unpacked.dll`, sha256 8097dbf8…550c). All
addresses are VAs in this DLL. **(inf)** = inferred. Queue: modlog-gaps-queue.md.

## Item 71: lifts — the camera's look point (gaps L106, MODLOG L8850); LSB's Metalworks doors (not DLL)

**The look point gets no lift handling of its own; only the eye rides, and riding pauses the camera's automatic
adjustments for 10 frames after each height change.** The camera update FUN_1001ee60 (this build, same address;
all_decomp.c ~20880..21500):
- Every call: the hold DAT_10456d74 −= frame ticks (floor 0).
- **Riding** (player vt+0x19c, the lift flag +0x5e2): eye.y (+0x48) += the player's y − last frame's y (DAT_10456da0)
  and, if that change ≠ 0, **hold = 10**; then DAT_10456d9c = the player's position (the "last" for next frame). The
  same hold = 10 is set by any manual camera turn / zoom this frame (the rotate / pitch paths above it, ~20946..20999).
- **What the hold suppresses** (~21444..21463): the eye-below-look-point clamp (the eye is pushed up when it is within
  DAT_10456d78 under the target, only with hold == 0 and the auto-adjust option DAT_10456d7c on), and the "settle" that
  clears the collision state +0xf0 when the camera has been blocked 4+ frames (only with hold == 0). The look point
  itself (+0x50) is computed and eased from the player as usual.
- So during a ride the eye moves exactly with the player and the auto-adjust stays off until 10 frames after the lift
  stops *(inf: frames = calls; at 30 fps about 1/3 s)*.

LSB's Metalworks door data (MODLOG L8853) is a data check against LSB, not a DLL question — not done here.

Build: on a ride, move the eye by the player's height change and hold the camera's auto-adjust (eye-under-target clamp,
blocked-camera settle) for 10 frames after the last change; ease the look point as usual.

## Item 72: footsteps — the zone flag +0x1cf alt name; the fses lookup retry; the culled-pass skip (gaps L152, MODLOG L3744 / L3768..)

(`fvib` and the footstep-effects option DAT_10456a70 are settled in batch 11 item 70.) The step code is
**FUN_100d4bc0** (this build).
- **The alt name flag is scene (DAT_1065cb04) +0x1cf, set only for areas 704..706.** FUN_1018b500(area) (the area / zone
  loader, batch 11 item 66) clears it, then sets it when the session flag word S+0x3c35c (FUN_100fc160) has **bit 0x100
  clear** and **area − 0x1bc ∈ 0x104..0x106** (areas 0x2c0..0x2c2 = 704, 705, 706; the "≥ 0x2bc" test before it is
  implied). With it set and the surface (vt+0x218) == 3, the step name's first two bytes become "02" (0x3230), i.e.
  **surface 3 borrows surface 2's sound** in those areas. *(inf: what bit 0x100 of S+0x3c35c means — not traced.)*
- **The name** (MODLOG's layout confirmed): byte 0 '0', byte 1 '0' + surface, byte 2 the foot kind (+0x879: '0'+n under
  10, 'a'+n−10 above; 0 or no surface → no step), byte 3 '1' + the shoe (+0x87a). Folder 'fses', or **'fser'** when the
  weather 4cc (FUN_101880b0) is 'rain' / 'squl' / 'bolt' and the actor isn't under cover (+0x17c == 0).
- **The lookup retry**: when the wanted folder (+0x804) differs from the cached one and the zone resource (scene +0x1ac)
  is set: FUN_10072290(…, 1, folder) — the global resource search (zone DAT, then DAT 0). **Null → no step, and the
  cache is left as it was, so the lookup runs again on the next pass** (every frame in range) until it succeeds; found
  → +0x800 = it, +0x804 = the 4cc, +0x7fc / +0x80c / +0x810 cleared. There is no separate model-scope search in this
  build *(MODLOG's "model-scope retry" reading doesn't apply here)*.
- **The culled-pass skip**: within range (squared distance < DAT_1032a3d8 from the camera) a pass with the drawobj not
  drawn (+0xb2 != 0) sets **+0x840 bits 20..21 = 2** (0x100000 cleared, 0x200000 set); MODLOG L3768's decrement rule
  then skips the next 2 passes' steps. The contact points (9, 8, 0xb, 0xa) are still read that pass.
- Out of range or with footstep effects off: FUN_100d6550 (the wade loop / dust cleanup) and no step; footstep effects
  on and surface 9 (water): the wade path FUN_100d64d0 with +0x650 = vt+0x220 (the water height).

Build: alt surface 3 → 2 only in areas 704..706 (flag word bit 0x100 clear); retry a failed fses / fser lookup every
pass; after a culled pass skip 2 passes' steps.

## Item 73: the press effect on the character list rows, Config pages and the login screen (gaps L155, MODLOG L7224)

**The press effect is generic: every window's cursor part gets it on Enter, drawn by the generic window draw with N =
20 — so the character list rows, the Config pages and the login screen all show it; only the title (loby2win) draws
its own with N = 9. The sounds are each lobby window's own.**
- The generic key layer FUN_10118260 sets the window's **+0x5e = 9** on every accepted Enter (msg 5; a disabled part
  returns before it, and +0x5d drops all but Escape) — independent of the SE flags +0x84..+0x86.
- The generic window draw FUN_101177d0 (0x10117999..0x101179d8): for the cursor part, while +0x5e > 0 it draws the
  part at (+2, +2) with **FUN_10119f80(x, y, w, h, 0x14 = N, +0x5e)** → FUN_10119fb0: the gold 0x80805020 part and the
  two expanding rings (MODLOG L7224's formula with N = 20, c counting down from 9, so t = N − c runs 11..19). +0x5e is
  counted down by frame ticks in the window frame (0x10117774..0x10117790).
- The title's loby2win draws its buttons itself (FUN_1023ffd0 → FUN_10119fb0 at 0x10240231 with N = 9). Other direct
  callers of FUN_10119f80: playermo's rows (0x1012b44b), main-menu pages (0x1013fff4, 0x10141384), FUN_1014f1e0, the
  Config windows (0x1019202f, 0x101928d4..0x10192a92, 0x101937b1), link2..4's Equip row (0x101e60be), 0x101fc518,
  0x102057f7, scoption (0x1020c7f8..0x1020c912) — windows that draw their rows themselves call it the same way.
- **Sounds:** every lobby holder clears +0x84 / +0x85 / +0x86 in its open hook (MODLOG L7228: the title, dbnamese
  FUN_10241910, the prompts, ok) and plays its own; for the character list: Enter on a used slot 0002, on an empty one
  0004, Escape 0003 (MODLOG L2815). The Config and login windows' own sounds were not re-traced here.

Build: apply the generic press effect (N = 20, starting at 9) to every window's cursor part on Enter — list rows,
Config pages and the login fields included; keep N = 9 for the title buttons.

## Item 74: the chat spam filter, config 0xce (gaps L162)

**The filter hides or replaces lines whose sender or text contains a prohibited.txt pattern, unless an allowed.txt
pattern matches first. Each pattern is a substring with optional [ ] character classes and three flags (i, s, n).**
- **Gate (caller 0x100e3908..0x100e398d):** only kinds 1, 0x1a, 0x22, 0x23 reach it (byte table 0x100e3d7c by kind − 1),
  and not GM lines (Attr bit 0). The block list FUN_100e0a50 runs first (all kinds except 0x19).
- **FUN_100da5e0(name, text):** config 0xce = 0 → 0xff (no action). Otherwise it copies the text through FUN_101b7260
  (→ FUN_101ac990, at most 0x97 bytes), then FUN_1018fd30 strips every two-byte code that FUN_1018fa20 rates 2 or 3
  *(inf: the colour and auto-translate codes)*, and calls FUN_101bceb0. Only a result of 1 acts: config 1 → the line is
  dropped (jump to 0x100e3d64); config 2 → the text is replaced by group 9 string 0x159, and the line is marked filtered
  (local flag +0x18 = 1, which picks the formats with the 0xEF 0x4D icon, MODLOG L5994).
- **FUN_101bceb0:** −1 if the filter object DAT_10662dc0 is not loaded; else FUN_101bcf80(name, text).
- **Loading (FUN_101bd820):** prohibited.txt into +8 (count +0x2718), and only if that worked, allowed.txt into +0x2720
  (count +0x4e30); +5 / +0x271c = "list loaded". The parser FUN_101bd880:
  - The file is UTF-8 (MultiByteToWideChar 65001), at most 0x23f0 bytes (larger → error), at most 0xc80 characters.
  - One pattern per line (CR, LF or CRLF), at most **100 lines**, at most **30 characters** each (longer is cut).
  - Entries are 100 bytes: +0 flags, +4 the pattern (UTF-16), +0x42 "has a class" byte, +0x43.. one literal byte per
    character.
  - A line may start with **`(` + any of `i`, `s`, `n` + `)`**: i = 4, s = 2, n = 8. Without the closing `)` the flags
    are 0. `\(` at the start escapes the paren.
  - `\` before `( ) [ ] ^ - .` makes that character literal. The fullwidth ［ ］ ＾ － ． (U+FF3B, FF3D, FF3E, FF0D, FF0E)
    are always literal.
  - A bad `[`, `]`, `^`, `-` or `.` placement clears the class byte, so the line is matched as plain text.
- **FUN_101bcf80 (the match):** returns 0 if prohibited.txt is not loaded. Name (16) and text (0x400) are converted to
  UTF-16 (FUN_1011dee0) and built into variants:
  - Normalising (FUN_101be030 / FUN_101be0b0): lowercase a–z, fullwidth ａ–ｚ, à–ÿ (except ÷) and Cyrillic а–я →
    uppercase; then ASCII 0x21..0x7e → fullwidth (+0xfee0); halfwidth katakana U+FF66..FF9D → fullwidth via the table
    0x1035cbdc.
  - The name: raw and normalised. The text: raw; normalised (spaces kept); raw without spaces; normalised without spaces
    (iswspace, FUN_10312bee).
  - Call 0x101bd17e pushes them as: p2 raw name, p3 normalised name, p4 raw text, p5 normalised text, p6 raw text no
    spaces, p7 normalised text no spaces.
- **Per list (FUN_101bd360 allowed, FUN_101bd1e0 prohibited), each entry in order:**
  - Unless flag n: test the name, the normalised one if flag i.
  - Then the text: no flags → raw; i → normalised; s → no spaces; i + s → normalised without spaces.
  - A plain entry is a substring test (FUN_10312c8d, wcsstr). An entry with a class uses FUN_101bd510: `[...]` matches
    one character from the set, `a-b` is a range, `^` after `[` negates it.
  - The first entry that hits returns 1.
- The result: allowed.txt hits → **2** (kept; the caller ignores 2); else prohibited.txt hits → **1**; else 0.
- So **i means ignore case and width, s means ignore spaces, n means text only (skip the sender's name)**. The parser
  treats the pattern the same way: with s it drops spaces from the pattern (FUN_10312bee); with i it normalises each
  character (FUN_101be030 / FUN_101be0b0). For an i entry with a class it then turns the now-fullwidth ［ ］ ＾ － ． back
  into ASCII `[ ] ^ - .`, except for characters marked literal.
- Misc. 4 (FUN_1019bdd0) keeps 0xce = 1 / 2 only while FUN_101bced0 (the load) returns > 0. Where the files are read
  from (FUN_101c4c80 / FUN_101c4b50 / FUN_101c47c0) was not traced *(inf: the USER folder)*.

Build: load prohibited.txt / allowed.txt with the format above. For chat kinds 1, 0x1a, 0x22, 0x23 that are not from a
GM: strip codes, then test allowed first and prohibited second, against the name and the text variant chosen by each
entry's i / s / n flags. A prohibited hit hides the line (0xce = 1) or swaps its text for group 9 0x159 with the
filtered icon (0xce = 2).

## Item 75: the profanity mask, vulgar2.dic (gaps L163)

**Config 0x18 turns the mask on or off. When on, the client loads vulgar2.dic and hands each text to a masking engine
in the host interface table, not in FFXiMain. So the word matching cannot be read here; only the setup and the call
sites can.**
- **Switch:** FUN_10193800(0x18, 0x101ba460, DAT_10662db8) (all_decomp L270363) watches config 0x18. The watcher
  0x101ba460 acts only on message 0x18 and calls FUN_101ba390(value).
- **FUN_101ba390(value):**
  - A negative value is clamped to 0. It stores the value at +4.
  - Value 1 → off: FUN_101ba2d0 frees the dictionary buffer +0xc and clears +8 / +0xc / +0x10.
  - Any other value → on. If +0xc is already loaded it stops there. Otherwise:
    - +0x10 = size of "vulgar2.dic" in file group 0x13 (FUN_101c4b80);
    - +0xc = a buffer tagged "Vulgar" (FUN_101907a0), and the file is read into it (FUN_101c47d0);
    - +8 = FUN_1031056c(buffer, &opts, size), the engine's create call through the host table DAT_10456a2c + 0x5a8.
  - opts is 7 bytes: u32 0, byte 0x3f '?', then the two bytes at 0x1036fd84 = 0x81 0x9B (Shift-JIS ○) *(inf: '?'
    replaces one-byte characters and ○ two-byte ones)*.
  - Any failure sets +4 = 1 (off) and frees.
- **FUN_101ba200(text):** −1 if DAT_10662db8 is not set. Else FUN_101ba350: only while +4 == 0 and +8 != 0, it calls
  FUN_10310577(+8, text, strlen) = host table + 0x5ac, which masks the text **in place** (same length). It returns 1.
- **Call sites:**
  - The chat printers FUN_100d9b20 (0x100d9da3) and FUN_100d9e50 (0x100da0b7), covering the kinds in MODLOG L6001:
    - speaker kinds: masked from the line's first ':' or ' ';
    - NS kinds 0xd..0x10 and 0x1c: the whole line.
  - FUN_101ca250 (0x101ca2a0): a text-measure routine. When its last argument is set it masks a 256-byte copy before
    measuring, so wrapped widths match the masked text.
  - FUN_101dd780 (0x101dd9d8), the s2c 0x0CC chain (MODLOG L6206).
  - The party-seek list "expparty" (0x10200ebf, 0x10200f12): each comment is masked after the "%s (%s %s):" /
    "%s (%s %s / %s %s):" header is built; strings 0x1037f144..0x1037f158.

Build: watch config 0x18 (1 = off; anything else = on). When on, load vulgar2.dic (file group 0x13) and mask the chat
bodies, the 0x0CC text and the party-seek comments in place, keeping the length: '?' for one-byte characters, ○ (0x81
0x9B) for two-byte ones *(inf)*. The engine's dictionary format and match rules live in the host interface
(DAT_10456a2c + 0x5a8 / + 0x5ac) and need that module to settle.

## Item 76: Shift-JIS chat text; the quotes 0x8167 / 0x8168 (gaps L166)

**Retail never converts chat text. Bytes go from the packet into the log and onto the screen as Shift-JIS. Each
two-byte pair maps straight to a glyph index in the font sheet. The quotes " " (0x8167 / 0x8168) are glyphs 135 / 136,
so they print. Our '?' comes from decoding to UTF-8 and encoding back, which retail does not do.**
- The text walker treats 0x81–0x9F, 0xE0–0xEF and 0xFA–0xFC as lead bytes (MODLOG L265). The pair's glyph index comes
  from **0x1011c940(ptr)**: Shift-JIS → JIS row / column, then a table of row ranges. Its inverse, index → Shift-JIS, is
  0x1011cb50. Checked against each other: 0x8540 ↔ 704, 0x8740 ↔ 936, 0x889F ↔ 1024, 0xE040 ↔ 5399. A JIS code ≥ 0x9320
  (all of 0xFA–0xFC) gives **0xcb**, which the inverse maps to 0x8181 (＝), so the fallback glyph is ＝ rather than '?'.
  The usgaiji codes 0xEF1F..0xEF2F are handled before this (MODLOG L266).
- 0x1011c920(ptr) = "is 0x85 0x9F..0xDF", a range test used by the walker *(inf: the European accented letters in row
  0x85; the caller was not traced)*.
- Port of 0x1011c940, checked against the asm at 0x1011c940..0x1011cb47 (the 0x2e46 test is unreachable in the
  original too):

```python
def glyph(lead, trail):                      # 0x1011c940
    r = (lead - 0x71 if lead <= 0x9f else lead - 0xb1) * 2 + 1
    c = trail - 1 if trail > 0x7f else trail
    if c >= 0x9e: r += 1; c -= 0x7d
    else:         c -= 0x1f
    j = (r << 8) | c
    if j >= 0x9320: return 0xcb
    for lo, k in ((0x7921, -0xcd1), (0x5021, -0xdec), (0x3021, -0xdc1)):
        if j >= lo: return 94 * r + c + k
    for lo, k in ((0x2f21, 0x21e), (0x2e21, 0x271), (0x2e46, 0x271), (0x2d5f, 0x351), (0x2d40, 0x359),
                  (0x2d21, 0x35a), (0x2a21, 0x223), (0x2921, 0x276), (0x2821, 0x203), (0x2751, 0x1b3),
                  (0x2721, 0x1c2), (0x2641, 0x18b), (0x2621, 0x193), (0x2521, 0x13e), (0x2421, 0xec),
                  (0x2361, 0x93), (0x2341, 0x99), (0x2330, 0xa0)):
        if j >= lo: return c + r + k
    if j == 0x227f: return 0xf2
    for lo, k in ((0x2272, 0x56), (0x225c, 0x5d), (0x224a, 0x68), (0x223a, 0x70)):
        if j >= lo: return c + r + k
    return 94 * r + c - 0xbdf                # row 0x21 and the start of row 0x22: 0x8140 -> 96
```

- The rest of the chain is raw bytes too: the input line's +0x6c4 buffer (MODLOG L8165), the history slots (cut at
  0x96, Shift-JIS safe), the auto-translate FD blocks, and the spam filter. Only the filter converts, and only into its
  own UTF-16 copy (FUN_1011dee0, item 74). The profanity mask works on the bytes in place (item 75).

Build: keep chat as bytes end to end, with no UTF-8 step. Walk Shift-JIS (the lead-byte set above) and index the font
with glyph() → 0x8167 / 0x8168 draw as glyphs 135 / 136. For unknown pairs use glyph 0xcb (＝), not '?'.

## Item 77: the full log's history and its wheel handler (gaps L170)

**The "~100-line history" is the size of the view's window, not the history. The full log keeps a 100-row ring (+0x20
wraps mod 100). At its edge it pulls rows from the buffer's history store, which holds pages of 50 rows in 20 rotating
"%d_%d.log" files (MODLOG L16177). So the wheel and the keys can scroll back past 100 rows. The wheel moves one row per
notch.**
- **Handler FUN_10138230** (full log vtable 0x10333b68, "menu    fulllog"):
  - While +0x60 is set (a tab rebuild in progress, set by message 8), messages 1–5, 9, 0x16–0x19 and 0x1b are dropped
    (byte table 0x10138694 / jumps 0x1013868c).
  - Otherwise the byte table 0x101386dc / jumps 0x101386b0 apply.
  - `hand` = FUN_10138000 (the window has focus, FUN_1015edd0) **and not** FUN_10123a70(0x3e, 0xaf, 4, −1), the key
    function 0xaf held *(inf: the Ctrl / Alt scroll-only modifier, MODLOG L2603)*.

| msg | what |
|---|---|
| 1 (Down) | hand and +0x1c < +0x58: the hand row + 1. Else same as 0x17 |
| 2 (Up) | hand and +0x1c > 1: the hand row − 1. Else same as 0x16 |
| **0x16 (wheel, older)** | unless +0x25 (at the oldest end): FUN_101380e0 once; on success +0x24 = 0 |
| **0x17 (wheel, newer)** | unless +0x24 (at the newest end): FUN_10138180 once; on success +0x25 = 0 |
| 4, 0x18 (page older) | at the oldest end: hand → row 1. Else up to +0x58 (rows shown) × FUN_101380e0; when one fails: +0x25 = 1 and with hand the row → 1 |
| 3, 0x19 (page newer) | the same with FUN_10138180; at the end the hand row → +0x58 |
| 0x1b (thumb drag) | delta = the bar's +0x14 − (store total DAT_10578158+4 − +0x58 − +0x20 + 1) (FUN_10138200); steps that many rows newer (delta > 0) or older |
| 6 (Escape) | hides "menu    fulllog" and "menu    charlnk", focuses logwindo (DAT_1062193c), FUN_10123ea0(0xf) |
| 8 | the next tab: FUN_10138910(+0x54), 0xb wraps to 0; +0x60 = 1, FUN_101295d0 rebuilds (+0x64), +0x24 = 1, +0x25 = 0, +0x20 = 0; FUN_10037b90 |

  - Exit 0x10138666: SE "0001" (FUN_10037b30, id 0x31303030) when the hand row changed or anything scrolled.
- **One row older, FUN_101380e0:**
  - While +0x20 < (the rows held, DAT_10578158 +0x1a) − +0x58, it just steps +0x20 + 1.
  - At the edge, only on tab 0 or 8 (Window 1 / 2): FUN_1012a650(tab, 0) loads an older row from the store; +0x20 is
    shifted by the change in FUN_10129c60 (the ring's count).
  - On the channel tabs it fails, so they stop at what is held, ≤ 100 rows (MODLOG L16181).
- **One row newer, FUN_10138180:** while +0x20 ≥ 1 it steps − 1. At 0, FUN_1012a730(tab, 0) loads a newer row on any
  tab; a failure sets +0x24 = 1 (the newest end).
- Both wrap +0x20 into 0..99 and clear the opposite end flag on success.
- So each wheel notch scrolls **one row**, the hand stays on its screen row, there is no mouse hit test on rows or tabs
  (MODLOG L16182), and the scroll bar's total comes from the rebuilt row count.

Build: the view is a 100-row window over the paged store. Window 1 / 2 scroll older past 100 rows by loading 50-row
pages; channel tabs stop at the rows they hold. Wheel: 0x16 one row older, 0x17 one row newer, SE 0001 when it moves.
Up / Down move the hand and scroll at the ends (only scroll while key function 0xaf is held). Pages = rows shown.
Thumb drag steps rows by the delta.

## Item 78: the macro editor's Shift-JIS input; the palettes' flip (gaps L173)

**Shift-JIS input:** the macro fields are field objects FUN_101c5be0 on the shared text input FSTEXTINPUT
(DAT_10662d40), the same engine as the chat line. So they store raw Shift-JIS bytes, and the limits are byte counts.
- Field modes (MODLOG L10111): the book name mode 4, 15 bytes; the title mode 8, 8 bytes; the lines mode 9, 60 bytes
  (auto-translate on).
- A two-byte character costs 2. The edit buffer keeps the raw bytes at E+0x6c4, the caret as a byte offset at E+0xa58
  (moved by whole Shift-JIS units, MODLOG L17754), and FUN_101abc50 turns 0x8140 into a space.
- FUN_101c5be0 copies the starting text: through FUN_101b7280 into +0x1d5 (limit param_3 + 1), raw into +0x12 and
  +0x3a0, length in [0x1a4].
- The Japanese client takes the IME composition path (T+0xf0ec = 0); English, German and French take the direct path
  (MODLOG L6234).
- No conversion is needed for storage: mcr*.dat holds the same bytes (61-byte lines, MODLOG L10121).

**The flip is a fade, not a zoom.** On a set change the palette's frame fades out over 20 ticks, then fades in over 20.
- **Trigger:** the play palette's handler 0x101a3960 (table 0x101a3a10) with +0x28 set. +0x28 is written each frame as
  "key function 0x76 or 0x77 held" (FUN_10123a70(0x3e, 0x76 / 0x77, 4, −1), 0x101a3a61..0x101a3a8e) *(inf: the Ctrl /
  Alt palette keys)*.
  - msg 1 → FUN_101a2a50(2); msg 2 → FUN_101a2a70(2) (the set step in S, one each way).
  - Then 0x101a3d00(1) refreshes the cells at once, and 0x101a3cf0 sets +0x29 = 1.
  - 0x101a36b7 does the same for the other palette (DAT_106627a0) after another set change.
  - The second palette's copy is at 0x101a40e1.. / 0x101a43a0.
- **Per-frame state, +0x29 (0x101a3a92..0x101a3b09):**
  - 1: the frame part (FUN_10118220, the part with id 0) gets +0x44 = 2, +0x48 = 20; then +0x29 = 2.
  - 2: wait until the part's +0x44 == 1, then +0x29 = 3.
  - 3: the part gets +0x44 = 3, +0x48 = 20; then +0x29 = 0.
  - 0x101a3d00(0) runs every frame either way.
  - The editor palettes mcr1edit / mcr2edit use the same machine on +0x48 (0x101a4cc3.., 0x101a53bf..).
- **Part fade (the generic part tick, 0x10119707..0x10119799):**
  - +0x48 counts down by frame ticks (FUN_10014cf0); at 0, state 2 → 1 and state 3 → 0.
  - The part's +0xc (its alpha) by state:

| +0x44 | +0xc |
|---|---|
| 0 | 1.0, or 0.58 (0x3f147ae1) when +0x4c is set (dimmed) |
| 1 | 0 (hidden) |
| 2 | 0.05 × t, so 1 → 0 (fade out) |
| 3 | 1 − 0.05 × t, so 0 → 1 (fade in); a result above 1.0 is stored as 0.9 |

  - Scale +0x10 / +0x14 stays 1 on this path (the open zoom is the separate +0x26 timer above it).
- So the new set's cells are shown from the first frame, and the whole palette fades out and back in over 40 ticks
  *(inf: the cells are children of the faded part and take its alpha)*.

Build: the macro fields edit raw Shift-JIS through the shared text input, with byte limits 15 / 8 / 60. On a set
change, refresh the cells, then fade the palette frame 1 → 0 over 20 ticks and 0 → 1 over 20 ticks.

## Item 79: target window — the plate gates (actor kinds, model range), the pet line, the event mode (gaps L178)

**Actor kinds and model range (the plate gate FUN_100d0510, the drawobj vtable slot +0x284; vtables 0x1032d710,
0x1032e890, 0x10330f40, …).** No plate is drawn when any of these holds, in this order:
1. First person (DAT_10487f80) and this drawobj's actor (+0x70) is the one FUN_10097a40 returns for the player
   (0x100d0510..0x100d0542) *(inf: the own character in first person)*.
2. vt+0x260 = 0x100a47b0: drawobj +0x8ac bit 0 (the invisible bit).
3. FUN_100845b0: actor +0x128 bit 3; with no actor, DAT_104568fc +9.
4. vt+0x34c = 0x100827a0: **actor +0xa4 in 1..4**.
5. vt+0x358 = 0x100827f0: **actor +0xa4 == 0x54**.
6. FUN_10098320(actor): **actor +0xfc (model id) in 0xb22..0xb31**. The same test skips those pets' TP on /partyinfo
   (MODLOG L12258). Its neighbour 0x10098340 tests +0xfc == 0x92c.
7. Then the names-off mode: DAT_1047d694 on, DAT_1047d695 = 0..4 through table 0x100d06d0. For example, case
   0x100d05a7 draws the plate only when the drawobj's +0x78 colour has alpha 0 (vt+0xcc).
- Also on the slot table: +0x354 = 0x100827c0 (+0xa4 == 3 or 4) and +0x364 = 0x10082810 (the +0xa4 setter).
- **+0xa4 is the actor kind**, set once by the constructor FUN_100c59b0 (MODLOG L13586):
  - 0 a main actor; 1..4 a chocobo; 5..0x24 a fishing rod; 0x3d..0x51 a chair; **0x54 a mount**.
  - So gates 4 and 5 mean "no plate on the chocobo or mount actor itself". The rider keeps its own plate.
  - Models 0xb22..0xb31 are a pet range with no plate and no /partyinfo TP *(inf: which pets was not looked up)*.

**The pet line (FUN_101501a0, 0x10150302..):** with a name actor A (+0x48):
- Two lines when FUN_1008e180(A) returns a master, or when A +0x12c bit 22 is set and FUN_1008e220(A) returns one.
  - FUN_1008e180: only if A +0x1d0 bit 0x100. It scans the actor table 0x10480af0..0x10482ef0 for a live actor (+0xa0
    set) whose +0x1fa or +0x2a0 equals A's +0x74 and whose +0x12c bit 22 is clear.
  - FUN_1008e220: the table entry at A +0x2a4.
- **Line 1:** A's name (+0x214 when u16 +0x210 != 0, else +0x7c), ellipsis-fitted (FUN_101c9d50) at (L + 5, T + 6),
  width W − 13.
- **Line 2:** "(" (DAT_10353e14) at (L + 9, T + 0x12), then the master's name ellipsis-fitted to W − 21, then ")"
  (DAT_10353e10) after it.
  - Both lines use the colour of the target cursor's actor (DAT_1057876c → FUN_101598a0, its +0x78), else 0x80808080.
    The one-line name +0x14 is cleared so it does not draw.
- Otherwise the name is strncpy'd to +0x14 (0x31 bytes) and drawn through FUN_101c9fa0 (MODLOG L4504), with split =
  local_4: 0 when a gauge was drawn (actor +0x128 bit 3 and +0x130 bit 20 clear), else 1.

**The event mode (+0x65):**
- When +0x65 != 0, the draw shows only the gauge FUN_101cb090(L + 0x1c, T + 0x1d, +0x66, &+0x64) and the name +0x14 in
  colour +0x68 through FUN_101c9fa0(…, centre 1, split 0), then returns.
- Writers:
  - **Event op 0xb4 sub 2** (FUN_100b0a00, jump table 0x100b1090, case 0x100b0b94):
    - value = arg 4 (FUN_100af4b0(4)), used only when 0..100;
    - name = the 16-byte string var 2 (FUN_100af960(2), byte 15 = 0);
    - sets +0x65 = 1, +0x68 = 0x80808080, FUN_10150a30(name), +0x66 = value;
    - opens "menu    targetwi" (FUN_1015e350(…, 1, 0)) unless it is open (FUN_1015ec50);
    - pc += 6.
  - **Op 0xb4 sub 3** (0x100b0c40 → 0x100b0e45): closes targetwi (FUN_1015e910); pc += 2.
  - **Fishing, mode 2** (FUN_10088bd0, state −1 at 0x10088c01): +0x65 = 2, white, the name "Fish" (0x1035baec), opens
    targetwi. The fish gauge is MODLOG L15059.
  - The window's close hook 0x10150910 sets +0x65 = 0 (and its part +0x30 |= 4).
- So an event can show a named gauge in the target window (for example a boss's or an objective's percentage). It
  stays until op 0xb4 sub 3 or the window closes.

Build:
- Hide plates on chocobo (+0xa4 1..4) and mount (0x54) actors, on models 0xb22..0xb31, and for the other gates in order.
- Pets: two lines, "name" / "(master)", in the cursor actor's colour.
- Event op 0xb4 sub 2: open targetwi with (var-2 name, value 0..100) in white until sub 3. Fishing uses the same mode
  with "Fish".

## Item 80: name plates — scheduler op 0xb4 (+0x8c0); the mount sub-case; k = FUN_10022340(race) (gaps L179)

**Scheduler op 0xb4 lifts the plate of the scheduler's actor by a float from the op for the op's duration, then puts it
back to 0. The mount sub-case and k were already decoded in MODLOG's 2026-10-25 entry (L13554, L13584).**
- **Op 0xb4** (table 0x1005dc1c[op − 2] → 0x1005c81d):
  - new 0x78-byte task (FUN_1005e040);
  - 0x10062180(sched, actor = FUN_10062770(sched), FUN_100627d0(sched), lift = u32 at op data +8, ticks =
    ftol(FUN_1005e590)).
  - FUN_10062770 / FUN_100627d0 take the scheduler's caster / target link (FUN_1003b6d0 / FUN_1003b700), stepping past
    CXiDollActor links to the owner.
- **Task** (vtable 0x1032bf70, link part 0x1032bf54 at +0x34):
  - constructor: +0x74 = float(ticks); **actor +0x8c0 = lift** (the f32 bits as given, 0x100621cb).
  - FUN_1005e590 = s16 op +6 × the scheduler's speed +0x9c.
  - per frame 0x100621e0: +0x74 −= frame time (DAT_1047bfa8 +0xeb0); at ≤ 0 it ends itself (vt+0x18(1)).
  - destructor 0x10062140: the linked actor's **+0x8c0 = 0** (0x1006215c).
  - Ending the link (0x10062100 / 0x10062120) ends the task too, so the lift does not outlive the scheduler's actor.
- **Use:** the plate projection (FUN_100d04c0, 0x100d04e3) adds +0x8c0 to the anchor's y before projecting. The same read
  is at 0x100d0710 (the arrow / target point's twin) *(inf: same projection path)*. Y is down in world space, so a
  negative lift raises the plate. The 8 `cst0` schedulers' −1.5 for 600 ticks (MODLOG L8741) is this op.
- No other writer of +0x8c0 in the actor range: 0x100c6208 is the constructor's 0. The 0x101d7f.., 0x10276fb4 and
  0x10281c7f hits are other objects' +0x8c0.
- **Mount sub-case (CXiSkeletonActor vt+0x1c4 = FUN_100d4560, 0x100d456d..0x100d47d9), already decoded:**
  - Bone-0 points get y += size × d for the applied status 0x55 (riding):
    - Buffalo (mount id 0x1d): d by race {0, .5, .5, .5, 1.1, −.7, −.7, .5, .05};
    - Craklaw (0x25): {0, .4, .4, .4, .4, −.8, −.8, .4, .4};
    - −1.3 unlinked or with k 0 / 6; else −0.5.
  - Status 5: −1.3. Chairs 0x3f..0x53: −0.5. Mount / chocobo / chair actors themselves: none.
  - **k = FUN_10022340** on the mount's info block (+0x878): moun[8 + t(race)], t = DAT_1032a41c [0, 0, 1, 2, 3, 4, 5,
    6, 7], 0xff → 0. Race FUN_10084420 (vt+0x3bc for a custom look, else entity +0xef). Mount id FUN_10084720 = the
    mount entity's +0x1db.

Build: scheduler op 0xb4 = set actor.plate_lift (+0x8c0) = op f32 @+8 for (s16 @+6 × speed) ticks, then 0. Add it to
the plate anchor's y before projecting. The ride drop is as in MODLOG L13554.

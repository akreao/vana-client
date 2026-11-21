## Gamepad: d-pad up / down with no target (2026-11-03)

- Field d-pad up / down did nothing until something was targeted. Retail's FUN_10158b60 only skips the party step
  without field +0x60 & 4 or a pick (0x10158b7d..0x10158b83); the result is then 0, and with 0x200 (always set with
  no prompt) a 0 result is our own actor (0x10158c20, ECX 0x1047d600 -> FUN_10081550). So with no target, up / down
  target us, and the next press steps the party from there. Ours bailed out before the 0x200 rule; fixed in
  target::party_pad_step (pure core target::pad_party_step, tested).
- docs/re/gamepad/ground_cursor.md said "a pick, else nothing"; corrected.

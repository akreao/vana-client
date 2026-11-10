## Character creator: packed motion channels with no deltas (2026-11-10)

Black creation screen after the master-into-custom merge (#350): every dancer was refused with `channels 31511:
channel size past the file`. Upstream twin lenonk/vana-client#197.

- PlaybackChannel v3 (kind 0x50003, FUN_1027ace0), packed: each channel is a 16-byte stream header (length, bits,
  step, initial value) and `length` bytes of deltas (FUN_102886a0: value(n) = initial + the sum of deltas 0..n-1).
  A channel whose stream is empty holds its initial value for every frame, so it costs 16 bytes however many frames
  the set has.
- The creator dancers' sets are mostly such constant channels. 31511 (Hume male): 488 channels x 241 frames =
  117,608 values in a 24,474-byte file, 438 of the 488 channels constant. The quality pass's bound (packed: at most
  four values a file byte, as if every value were a 2-bit delta) allowed 97,896 and refused it; all 8 races x 8 faces
  load under the new bound (`real_dancers_load`).
- The bound now: packed, at most one channel per 16 file bytes (its header), and at most 2^24 values in all (64 MB of
  floats); unpacked, a 4-byte float a value as before. *(inf: the 2^24 cap is ours, a safety limit against a corrupt
  frame count, not a retail limit; the creator sets are far below it (31511: 117,608).)*
- A dancer that fails to load is not asked for again every frame; the mark clears when the creator closes or a new
  one opens (ours).

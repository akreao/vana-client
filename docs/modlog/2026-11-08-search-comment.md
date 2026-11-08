## Search comments on Enter (2026-11-08)

- Enter on a person in the results window (FUN_1020c250) first calls FUN_1020b3e0, then opens scoption / acoption as
  before. FUN_1020b3e0: while no search query is out (FUN_1020a410: DAT_10385a04 != 4), a person whose flags have
  0x10 (a comment set) gets `FUN_100e3040(id, 0, 3, LAB_1020b440, id)` and the record is copied to DAT_10664448; the
  slot is busy until the answer. While a query is out it returns false and FUN_1020c250 plays 0004 (FUN_10037c50),
  then scoption still opens. The Link List's Enter (FUN_101e4d10) calls the same after opening the member window,
  without the sound.
- The request (slot kind 4, FUN_100e1c30's param 6; builder LAB_100e1f60 -> FUN_100d8ee0): wire type 8, data size
  0x14, the character id @0x10 (0 when the request's first word is 0). LSB: TCP_SEARCH_COMMENT,
  HandleSearchComment (accounts_sessions.seacom_message); it sends nothing for an empty comment.
- The answer (LAB_100e2060, kind 4 at 0x100e23d7 -> FUN_10106eb0 on reply + 0x18): u32 id, u16 length (at most
  0x80), the bytes; the callback gets {3, 4, the record}. LAB_1020b440: the stored record when the id matches, else
  none, into FUN_101dd780; busy ends (4).
- FUN_101dd780 (mode word {0xcc, 3}): nothing for an empty comment; else a header: "??????:" without the record or
  its flags field (present bit 6); "%s (？　？　？):" for anonymous (0x4000); "%s :" for job 0x17; else
  "%s (%s %s):" / "%s (%s %s / %s %s):" with 10 0x59 (level) or 10 0x7a (master level when rec +0x25 bit 0, i.e.
  flags bit 30) and the job names of group 0x12 (the sub job's level always 10 0x59). Then three rows, 40 bytes each:
  trailing spaces cut, auto-translate expanded (FUN_101b7280), control bytes as spaces (FUN_101c8f40), the profanity
  filter (FUN_101ba200), cut at 335 px (FUN_101ca570(0x14f)) with "..." after it, the Japanese quote rewrite
  (FUN_10190140, language 2 only), each printed in mode 0xcc (empty rows too).
- Ours: search::fetch_comment (Then::Comment, counted by search::busy), net::search::comment. Not done: the profanity
  filter (as elsewhere) and FUN_10190140. Not seen live.

## Macro editor: lines drawn with the auto-translate brackets (2026-11-24)

<!-- dll-build: 30261102_0 -->

- win_macro_v10 (FUN_101a70c0) draws each of the six lines that is not being edited from its text field's raw bytes
  (FUN_101c7420(0) = field +0x12; the expanded copy +0x1d5 is only made while the field is active, +0x694): the
  dictionary (DAT_10663ef8) expands it with FUN_101b8850(text, len, buf, 0x98, 1, 0) = FUN_101adf60 with the green
  and red brackets (EF 27 / EF 28) and no 7F 69 / 7F 64 codes, German's quotation marks go on that 0x98-byte buffer
  (FUN_10191710), and FUN_10191c70 draws it at (child x - 0xa0, child y + 5) cut to 0x13b with "..." (its last
  argument 0). The comment window does the same with a 0x200 buffer.
- FUN_101adf60's buffer: a character (1 or 2 bytes by FUN_1011ef10) or a block (FUN_101adbb0, brackets counted) that
  would bring the line to the buffer's size or past it ends the line there; a block with under 2 bytes left fails
  (-1) and ends it too. So at most 0x97 bytes of a macro line show.
- FUN_10191c70 (unlike the 16 px rule of FUN_101ca250): it drops the last character while the text (FUN_10191e00) is
  wider than the width - 14, and a cut text has "..." (DAT_1036d9dc) drawn right after it.
- The title (child 3) is the field's raw text drawn whole (FUN_10191db0, no cut); only the live field (FUN_101c76b0,
  0x78) is fitted. The title field is mode 8, eight bytes; the lines are mode 9, 0x3c bytes (FUN_101a6ac0 ->
  FUN_101c71b0).

Ours: `Dictionary::expand_cap` / `fep::braced_cap` (the buffer rule; `expand` is the uncapped call),
`GameUi::text_cut` (FUN_10191c70), macroedit's classic editor draws a line not being edited bracketed and the title
uncut. A phrase in the title (only ours' modern panel can put one there, by Tab) is drawn as its plain text; retail's
FUN_10191db0 would draw the block's raw bytes. With TrueType Text on, the brackets are drawn as a green "{" and a red
"}" (GameUi::modern_runs), as the modern chat log draws them; that face has no EF 27 / EF 28 glyphs. The modern macro panel keeps the plain text (its fields are edited in place). Test
`dat::autotrans::tests::capped_expand`. Not checked on screen.

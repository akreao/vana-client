## Auto-translate in the macro and comment fields (ours, 2026-11-02)

- Retail's English path completes phrases on the chat line only (FUN_101b5d10); the macro editor's fields
  (FUN_101c5be0 modes 8 / 9) and the comment editor's lines (mode 9) do hold phrase blocks (FD id FD), which
  macroedit.rs already drew and kept. Ours: fep.rs FieldFep (field_open / field_key / field_put) reuses the chat
  line's query and list keys for any field; macroedit::complete and seacom::complete route a field's keys to it while
  it is open (World::field_fep), put the pick in place of the word before the caret (macro fields: at the caret;
  comment lines: at the end, they have no caret) and refuse a pick past the field's Shift-JIS bytes (8 title, 60 line,
  40 comment) with SE 0004. fep::draw_list draws the list for the retail windows, modern/chat.rs candidates for the
  modern ones.
- The comment lines now count Shift-JIS bytes (a phrase 6) against the 40, Backspace takes a phrase whole, and both
  UIs draw the lines with their phrases written out (fep::plain).

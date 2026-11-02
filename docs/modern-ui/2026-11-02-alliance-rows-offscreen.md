- Alliance rows offscreen (2026-11-02): `modern-shot --alliance` draws both alliance windows with sample rows built as
  party_rows builds them (no MP bar, the alliance leader's blue mark, a member out of the zone, a trust without TP).
  Seen right there; the shot also learns the GPU's texture limit first now (the 2184-wide status atlas failed under
  egui's assumed 2048).
- "Not yet: alliance rows live" still stands: it needs a second party of bots.

- Pending actions (2026-11-02, ours; the plan's "state changes are server-confirmed" and its open question on latency
  and invalid targets; client/pending.rs, modern/pending.rs): a spell, weapon skill, job ability or item sent to the
  server (0x01A from /ma, /ws, /ja, /pet; 0x037) is pending until it answers. A chip over the cast bar (panel
  "pending", movable) shows "Cure III -> Raven  waiting..." once the wait passes 150 ms (a quick answer never flashes
  it). Our 0x028 confirms it (the chip goes; the cast bar and the effect take over). A 0x029 of ours before that is the
  refusal: the chip turns red with the message's own text ("Raven is out of range.") for 4 s. No answer in 5 s says
  "The server has not answered." The client's own refusals (group 7 lines before anything is sent) start nothing.

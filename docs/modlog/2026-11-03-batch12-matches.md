## Batch 12 rows that already match retail, and the profanity mask (2026-11-03)

- docs/re/notes/2026-11-03-modlog-gaps-batch12.md, checked against the client:
  - Item 71, lifts: the camera's look point gets no lift handling. The eye rides with the player's height change and
    holds the auto-adjust for 10 ticks after each change. `camera::ride` already does this.
  - Item 73, the press effect: the generic Enter effect (window +0x5e = 9, N = 20). Ours already has it on the
    character list rows (`select.rs`) and the Config pages (`GameUi::press`). The login screen is ours, not retail's.
  - Item 74, the spam filter: `client/spamfilter.rs` already matches the loader (UTF-8, 0x23f0 bytes, 100 lines, 30
    characters), the (isn) flags, the [ ] classes, allowed before prohibited, and the hide / replace modes.
- Item 75, the profanity mask: config 0x18 turns it on (anything but 1). On, it loads vulgar2.dic (file group 0x13)
  and masks chat bodies, 0x0CC text and party-seek comments in place. The masking engine is a call through the
  PlayOnline host's interface table (DAT_10456a2c + 0x5a8 / + 0x5ac), not code in FFXiMain.dll. Building it needs that
  module, so the gaps row is now "Blocked: host module".

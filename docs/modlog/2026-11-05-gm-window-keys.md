## Text fields of the GM and Settings windows keep their keys (2026-11-05, ours)

- The GM window (modern/gmwin.rs) and the Settings window's search (modern/config.rs) took the typed text from the
  egui events, but the key functions (input.rs, run first in `key_functions`) only knew the fields listed in
  `typing_elsewhere`, so each letter typed there also fired its game hotkey. Both now count as typing: the GM window
  while it has the keys (`gmwin::takes_keys`: shown, nothing over it, no event), the Settings search while it has text
  (`config::finding`). Key functions with mask 0x41 then fail, as for the chat line.

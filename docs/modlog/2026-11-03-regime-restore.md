## Training regime: kept state found after a restart (2026-11-03)

Ours (a fix to the Extra, not retail). The regime state is kept per character: in the data dir for a plain run,
beside the `FFXI_CONFIG` copy for the wrapper's runs. A regime begun in one kind of run was missing from the other, so
after a restart the Training window stayed away until the next kill credit, and then came back with only the
monster's name. A wrapper run now writes the data dir's copy too (`config::regime_paths`), and a zone-in reads the
newest copy that parses (a kept `null`, a cancel, counts as read).

## Training regime: kept state found after a restart (2026-11-03)

Ours (a fix to the Extra, not retail). The regime state is kept per character beside the config: in the data dir
for a plain run, beside the `FFXI_CONFIG` copy for the wrapper's runs. A regime begun in one kind of run was missing
from the other, so after a restart the Training window stayed away until the next kill credit, and then came back
with only the monster's name. At zone-in the state is now read from whichever of the two files is newer
(`config::regime_read_path`); it is still written only to the run's own place.

# MODLOG fragments

New `MODLOG.md` entries go here, one file per change, instead of being added to the end of `MODLOG.md`.
Every branch used to append to the same last lines of `MODLOG.md`, so any two open pull requests conflicted.
A new file per entry touches no shared lines.

## Adding an entry

1. Create `docs/modlog/YYYY-MM-DD-short-name.md`, for example `2026-11-02-temp-items.md`. The date is the day
   of the change; the name only has to be unique.
2. Write the entry exactly as it would appear in `MODLOG.md`: a `## Title (YYYY-MM-DD)` heading, then the facts.

   ```markdown
   ## Temporary items in the Items lists (2026-11-02)

   - Main Menu > Items now lists container 3 first (FUN_101f1100, 0x101f12a8..0x101f16c8). ...
   ```

3. To add to an entry that is already in `MODLOG.md`, write a new fragment with its own heading, such as
   `## Gamepad: window tabs (2026-11-02)`. Editing old text in `MODLOG.md` in place is still fine; only new
   entries go here.

## Folding into MODLOG.md

`python3 docs/fold.py` appends every fragment here to the end of `MODLOG.md`, oldest file name first, and
deletes the fragments. Run it on `master` (not on a feature branch) whenever convenient, and always before sending
work upstream, so `MODLOG.md` stays one complete file there. It folds `docs/modern-ui/` the same way.

- `python3 docs/fold.py --check` only checks the fragments.
- `python3 docs/fold.py --print` writes the would-be additions to stdout and changes nothing.

Until a fragment is folded, read it here alongside `MODLOG.md`.

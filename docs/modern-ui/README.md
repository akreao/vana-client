# Modern UI state fragments

Feature branches don't edit `docs/modern-ui.md`. Each branch writes what it adds to the modern UI as its own file
here, and these are folded into the doc's State section later. Every open pull request used to add its bullets to
the end of State (or edit the same open question), so any two of them conflicted.

## Adding an update

1. Create `docs/modern-ui/YYYY-MM-DD-short-name.md`, for example `2026-11-02-click-to-move.md`.
2. Write the State bullets exactly as they would appear in `docs/modern-ui.md`, starting with `- `:

   ```markdown
   - Click-to-move (2026-11-02, ours; user's pick: Ctrl + left click only): with the modern UI, Ctrl + left click
     on the ground walks there. ...
   ```

3. If the change answers an open question, finishes a "Not yet" item or makes an older bullet wrong, say so in a
   bullet of its own (for example `- Open question answered: click-to-move is Ctrl + left click (user, 2026-11-02).`).
   Whoever folds the fragments tidies the older text by hand. Don't edit `docs/modern-ui.md` on the branch.

The reverse-engineered facts behind the change still go in a `docs/modlog/` fragment (see `docs/modlog/README.md`).

## Folding

`python3 docs/fold.py` (on custom) puts every fragment here at the end of the State section, above its closing
`- Not yet:` lines, oldest file name first, and deletes them. It folds `docs/modlog/` into `MODLOG.md` at the same
time. Run it whenever convenient and always before sending work upstream. Until then, read these files alongside
`docs/modern-ui.md`.

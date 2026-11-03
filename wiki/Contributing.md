# Contributing

## Ground rules

- The goal is a faithful clone of the retail client first. Behaviour comes from the game's files and the original executable, not guesses.
- No game data and no code from the original client go into the repository.
- Pull requests run CI (build, clippy, tests on Linux and Windows). Clippy fails only on deny-level lints.

## MODLOG: the reverse-engineering journal

[MODLOG.md](https://github.com/akreao/vana-client/blob/master/MODLOG.md) records every reverse-engineered fact the code relies on: addresses in the original executable, DAT formats, packets. Anything inferred rather than read from the binary is marked as inferred. Read the relevant section before changing behaviour, and add what you learn.

New entries don't go into MODLOG.md directly, so branches don't conflict on its last lines:

1. Create `docs/modlog/YYYY-MM-DD-short-name.md`.
2. Write the entry as it would appear in MODLOG.md: a `## Title (YYYY-MM-DD)` heading, then the facts.

See [docs/modlog/README.md](https://github.com/akreao/vana-client/blob/master/docs/modlog/README.md).

## Modern UI state

[docs/modern-ui.md](https://github.com/akreao/vana-client/blob/master/docs/modern-ui.md) is the modern UI's plan and state. Branches don't edit it: state updates go in `docs/modern-ui/YYYY-MM-DD-short-name.md` as the bullets they would add. See [docs/modern-ui/README.md](https://github.com/akreao/vana-client/blob/master/docs/modern-ui/README.md).

## Folding fragments

On master, `python3 docs/fold.py` appends every fragment to MODLOG.md and docs/modern-ui.md, oldest first, and deletes them. `--check` only checks the fragments; `--print` shows what would be added.

## Tests and tools

- `cargo test --release` runs the tests; tests that need the game data skip themselves when it isn't installed.
- Offscreen tools render parts of the client to PNG for checking by eye; decoders print game files. See [[Command Line]].
- `play --input '<script>'` drives the client for test runs.

## Releases and pinned code files

- `release/publish.sh VERSION` signs the Linux and Windows builds with the release key and creates the GitHub release the in-app Update offers. See [[Building from Source#release-builds]].
- The [Code pins workflow](https://github.com/akreao/vana-client/blob/master/.github/workflows/code-pins.yml) checks the PlayOnline patch server every few hours, hashes every game .exe / .dll and, when the list changes, signs and publishes it as the `code-pins` pre-release. The patcher only writes a code file that is on this list.

## This wiki

The wiki is generated from the `wiki/` folder in the repository: edit the pages there in a pull request, and the [Wiki workflow](https://github.com/akreao/vana-client/blob/master/.github/workflows/wiki.yml) publishes them when it is merged. Edits made in the wiki's web editor are overwritten.

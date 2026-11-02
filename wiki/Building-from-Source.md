# Building from Source

Needs Rust 1.85 or newer (edition 2024).

## Linux

Install the headers (Debian / Ubuntu names):

```sh
sudo apt-get install pkg-config libpipewire-0.3-dev libclang-dev libasound2-dev libudev-dev
```

On Fedora and similar, libudev's headers are in `systemd-devel`.

Then:

```sh
cargo build --release
```

The binary is `target/release/vana-client`.

**Old PipeWire headers.** Where the system PipeWire is too old for cpal's bindings (Ubuntu 22.04 fails, 24.04 builds), leave PipeWire output out:

```sh
cargo build --release --no-default-features
```

Sound then goes through PulseAudio (or PipeWire's PulseAudio server) or ALSA.

## Windows

A native MSVC build edited and run from VS Code (Build Tools, rustup, Ctrl+Shift+B to build, F5 to debug). See [docs/building-windows.md](https://github.com/akreao/vana-client/blob/master/docs/building-windows.md) for the full steps, where files go and common errors.

## Build profiles

| Command | Output | Use |
|---|---|---|
| `cargo build --release` | `target/release/vana-client` | Everyday builds: incremental, no LTO |
| `cargo build --profile dist` | `target/dist/vana-client` | A binary to hand out: fat LTO, one codegen unit (slow to link) |

## Tests and lints

```sh
cargo test --release
cargo clippy --release --all-targets
```

Tests that need the game data skip themselves when it is not installed. GPU tests skip when no adapter is found.

CI ([ci.yml](https://github.com/akreao/vana-client/blob/master/.github/workflows/ci.yml)) builds, runs clippy and runs the tests on Ubuntu 24.04 and Windows for every push to master and every pull request. Clippy fails the run only on deny-level lints; warnings are reported. On Windows the GPU tests run on WARP, the software D3D12 renderer.

## Release builds

Published releases are built with:

- `release/build-linux.sh`: inside the Steam Runtime 3 "sniper" SDK container, so one binary runs on every current distro
- `release/build-windows.sh`: a MinGW cross build from Linux

`release/publish.sh VERSION` then signs both with the Ed25519 release key (`VANA_SIGNING_KEY`, kept by the publisher; its public half is `release/signing-key.pub`, built into the client), tags them and creates the GitHub release the in-app Update offers.

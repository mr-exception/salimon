# Issue #113 — Runtime composition modules

Refactored runtime composition helpers into six private `src/app/` modules.
`app.rs` retains `ClientApplication`, `ApplicationHandler`, update sequencing,
renderer recovery, redraw scheduling and native-thread automation dispatch.
No domain/renderer DTOs, dependency edges, gameplay gates, arithmetic policies,
update order or platform abstraction were changed.

## Changes

- `input.rs`: native physical-key translation and held steering/movement state;
  initial-press, repeat and synthetic-focus gates remain intact.
- `interaction.rs`: ship-local target gates and contextual prompt selection.
- `frames.rs`: ship/surface frame adapters; existing crate-visible `app` helper
  paths remain available to automation and carrying consumers via re-exports.
- `scene.rs`: world spheres, markers and Sun lighting plus live ship instruments.
- `diagnostics.rs`: observational body/domain/frame mapping and metric logging.
- `native.rs`: window attributes, size and cursor operations. Actual native
  lifecycle ownership remains in `app.rs`.
- All 19 original app regression tests moved unchanged beside their owners.
  Updated runtime architecture/maintenance guidance and root source routing.

## Validation

Environment: Ubuntu 24.04 x86_64, Rust 1.99.0 stable, Python 3.12.14.
Commands run from the repository root (Cargo installed through rustup):

| Command | Result |
| --- | --- |
| `cargo fmt --all -- --check` | Passed |
| `cargo clippy --workspace --all-targets --locked -- -D warnings` | Passed |
| `cargo test --workspace --locked` | Passed; existing ignored renderer test retained |
| `cargo build --workspace --locked` | Passed |
| `python3 -m unittest discover -s scripts -p 'test_*.py'` | Passed, 34 tests |
| `python3 -m pip install -r models/tools/requirements.txt` | Passed |
| `python3 -m unittest discover -s models/tests -v` | Passed, 48 tests; two Blender integrations skipped because Blender is unavailable |
| `python3 scripts/build_game.py --profile debug --output artifacts/build/debug` | Passed |
| `python3 scripts/build_game.py --profile release --output artifacts/build/release` | Passed |
| `git diff --check` | Passed |

A source comparison also verified exact unchanged bodies for `advance_game`,
`interact`, `sync_carried`, `carry_action`, `device_event`, `resumed`, `suspended`,
`resize_renderer`, `initialize_renderer`, `about_to_wait` and `exiting`, and
preservation of all 19 original helper regression test names.

## Graphical limitations

The local Xvfb preflight failed before a game launch: local/unix listening
sockets could not be established. A TCP-display preflight then failed virtual
keyboard setup (`/usr/bin/xkbcomp` unavailable). Dependencies were downloaded
and extracted to scratch because the host package installation is restricted.
Native E2E suites, screenshot evidence and packaged smoke were consequently not
run locally. The existing PR workflow runs the Linux graphical gates and all
three host build matrices; their result must be checked before merge. No
screenshots or reference-hardware performance claims are made by this report.

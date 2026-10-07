# Issue #124 — focused world modules

## Outcome

Extracted the non-resource implementation from `client/world/src/lib.rs` into
six private modules. The facade retains every existing public root contract and
resource module path; consumers and dependencies are unchanged.

- `coordinates.rs`: absolute `f64` positions and subtract-before-cast conversion.
- `catalog.rs`: celestial definitions, ordered catalog, anchor and reference speed.
- `geometry.rs`: surface/landing-volume distances and radial velocity.
- `proximity.rs`: deterministic nearest-body and nearby-solid selection.
- `precision.rs`: immutable markers, numeric spacing reports and regressions.
- `camera.rs`: engineering commands/state/snapshots and deterministic timeline.

All 28 original non-resource test bodies were moved unchanged (verified ignoring
whitespace) alongside their owning implementation. No catalog value, arithmetic
order, threshold, timeline sample, resource implementation or renderer policy
changed. Updated the world architecture/AI guide, root maintenance map and shared
math source inventory.

## Validation

Linux x86_64; Rust/rustfmt/Clippy 1.99.0; Python 3.12.14.

| Command/check | Result |
| --- | --- |
| `cargo fmt --all -- --check` | Passed |
| `cargo test -p salimon-world --locked` | Passed: 30 unit tests and 25 integration tests |
| `cargo clippy --workspace --all-targets --locked -- -D warnings` | Passed |
| `cargo test --workspace --locked` | Passed; existing renderer performance test remains ignored |
| `cargo build --workspace --locked` | Passed |
| `python -m unittest discover -s models/tests -v` | Passed: 53 tests, 2 Blender integration tests skipped because Blender is unavailable |
| `python -m unittest discover -s scripts -p 'test_*.py'` | Passed: 34 tests |
| Original test-body comparison | Passed: all 28 non-resource tests preserved |
| Changed guide/report relative file links | Passed |
| `git diff --check` | Passed |

## Limitations

Native graphical scene/precision/depth checks were not run: Xvfb is unavailable,
and an attempt to install the Linux graphical prerequisites with `apt-get update`
failed on environment setgroups/seteuid restrictions. No screenshot or native GPU
coverage is claimed. Reference macOS/Windows hardware and performance checks were
not run. No Blender source/export changes were made.

The implementation is ready for PR review; issue closure is deferred to merge.

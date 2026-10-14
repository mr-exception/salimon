# Issue 189 — Mechanical Core fueling

Added authored-center E interaction, atomic one-carried-fragment consumption,
immediate mass removal, configurable material yields/rate and automatic conversion
with simultaneous bus discharge, blackout recovery and full-capacity waste.

Validation: `git diff --check` passed. Added domain varying-mass/material, empty
bus, full bus, zero-yield and repeat-intake tests, targeting tests and cargo
ledger integration regression. Rust toolchain unavailable locally; Actions owns
format/lint/workspace tests/build validation. Native M1 signoff excluded by scope.
No new asset/export is needed; targeting derives from the authored Core collider.

## Completed CI validation

[Native builds run 38579099249](https://github.com/mr-exception/salimon/actions/runs/38579099249)
passed on Ubuntu 24.04, macOS 14 and Windows 2022 at commit
`6118caaf3a664015d0aaa7b3e43171f57f8918be`.

- `cargo fmt --all -- --check`: passed.
- `cargo clippy --workspace --all-targets --locked -- -D warnings`: passed.
- `cargo test --workspace --locked`: passed.
- `python -m unittest discover -s scripts -p 'test_*.py'`: passed.
- `python -m unittest discover -s models/tests -v`: passed; Blender-only
  tests skipped on the hosted runners as declared by the test suite.
- `python scripts/build_game.py --profile debug --output artifacts/build/debug`
  and the equivalent release staging command: passed on all three OSes.
- Linux software-Vulkan baseline, cockpit-window capture, resource-collection
  and ship-EVA evidence suites, headless rendering contracts, fragment CPU
  checks and packaged X11 input smoke: passed through the existing workflow.

These hosted checks do not provide physical Apple M1 gameplay/performance signoff.

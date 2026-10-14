# Issue 203 — Assisted flight energy reservation

Implemented isolated atomic budgets for current landing/takeoff paths, rejection
feedback, completion refund, blackout continuation and deterministic spending.
Portable reservation API is reusable by recovery when its path is implemented.

Validation: `git diff --check` passed. Added reservation and controller brownout
regressions; existing all-body timing/partition tests cover reserved paths.
Rust toolchain is absent locally and installation endpoints are unavailable;
workspace format/lint/tests/build are delegated to GitHub Actions. Native Apple
M1 signoff is intentionally excluded, as specified by the issue.

Estimate is conservative full forward power × duration for kinematic assists.
Recovery landing itself is separate planned work.

## Completed CI validation

[Native builds run 38578724957](https://github.com/mr-exception/salimon/actions/runs/38578724957)
passed on Ubuntu 24.04, macOS 14 and Windows 2022 at commit
`64f5d304e79832886e8df6ea54efc6633c00f1db`.

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

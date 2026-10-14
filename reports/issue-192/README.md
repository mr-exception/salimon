# Issue 192 — Cockpit blackout authority

Steering now consumes only the powered interval; outage and cockpit exit clear
held input/ramp residue without changing signed throttle, heading or momentum.
Native and automation steering use the same powered ownership/assist gate.
Power restoration resumes selected thrust independently of cockpit occupancy.

Validation: `git diff --check` passed. Added signed blackout/coast/restoration,
exact steering depletion-boundary, exit-ramp and denied-input replay regressions.
Workspace format/lint/tests/build run in Actions; local Rust toolchain unavailable.
Apple M1 signoff excluded by issue scope.

## Completed implementation validation

[Native builds run 38598387950](https://github.com/mr-exception/salimon/actions/runs/38598387950)
validated implementation commit `45cb176678f3dc90a7727a9a98cadd85f2943123`
on Ubuntu 24.04, macOS 14 and Windows 2022 (Rust 1.99.0, Python 3.12).

- `cargo fmt --all -- --check`: passed.
- `cargo clippy --workspace --all-targets --locked -- -D warnings`: passed.
- `cargo test --workspace --locked`: passed, including all 52 ship tests.
- `python -m unittest discover -s scripts -p 'test_*.py'`: passed.
- `python -m unittest discover -s models/tests -v`: passed; declared Blender-only
  tests skipped on hosted runners.
- `python scripts/build_game.py --profile debug --output artifacts/build/debug`
  and the equivalent release staging command: passed on all three OSes.
- Linux software-Vulkan baseline, cockpit-window capture, resource-collection
  and ship-EVA evidence suites, headless rendering contracts, fragment CPU
  checks and packaged X11 input smoke: passed through the existing workflow.

The zero-duration conversion step regression found during CI was fixed:
a no-op step preserves pending steering input, while actual blackout still clears
it. Existing steering/turning regressions and the explicit zero-step check pass.
Final follow-up changes only record test evidence in this report. These hosted
checks do not provide physical Apple M1 gameplay/performance signoff.

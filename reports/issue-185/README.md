# Issue #185 — shared fixed-step gameplay scheduling

Native redraws and explicit automation now admit elapsed time to the same 16 ms
accumulator. Native catch-up retains the existing two-tick/4 ms budget and all
unprocessed time; automation drains completed ticks and retains fractional time.
Ship direct flight no longer truncates long intervals to 100 ms. Character
fallback calls integrate bounded slices instead of discarding elapsed time.
Lifecycle resets explicitly pause across discontinuities and account for backlog.

Regression coverage includes elapsed-time partitions, long direct travel,
inherited EVA travel, bounded stall catch-up and lifecycle reset. The existing
ship/character/fragment update order remains the canonical per-tick composition.

## Validation

Linux x86_64; Rust/Cargo 1.99.0. Final check results are recorded below.

Graphical launch was not run locally: this container lacks Xvfb/Vulkan desktop
packages and package installation is blocked by its package-manager lock.
Apple M1 smoke/regression remains outside this cloud issue. Linux graphical
checks remain in the existing GitHub Actions workflow.

- Passed: `cargo fmt --all -- --check`, `git diff --check`.
- Passed: `cargo clippy --workspace --all-targets --locked -- -D warnings`.
- Passed: `cargo test --workspace --locked` (all tests and doctests; existing
  opt-in benchmark/GPU tests remain ignored by default).
- Passed: `cargo test -p salimon-character --locked timing_tests`.
- Passed: `/usr/bin/python3 -m unittest discover -s scripts -p 'test_*.py'`
  (55 tests). Earlier runs during compilation hit existing 100/200 ms subprocess
  deadlines; the idle rerun passed without code changes.

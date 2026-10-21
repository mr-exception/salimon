# Issue #382 — integrated pause checkpoint menu

2026-10-21: integrated current #371/#381 lifecycle and full pause with manual
Save Now, verified status/UTC, Save & Exit, retry, cancel exit and explicit risk
exit. Living gameplay window close shares the failure-aware route. No second
write follows verified/risk exit. The single worker/store, terminal archives,
restart, ordinary autosaves and offline-only fatigue are preserved.

The previous session passed 712 workspace tests (23 opt-in ignored), build,
Clippy, formatting and 95 Python tests, but its checkout was lost before publish.
This restored tree is being verified again; only final results below establish
current validation. The prior Xvfb attempt failed before display startup because
listening sockets are unavailable. Native OS-input/DPI/appearance/M1 checks
remain unrun under [#376](https://github.com/mr-exception/salimon/issues/376),
which already covers the pause/Save Now requirements. No visual baseline is approved.

## Fresh restored-tree verification

Linux x86_64, Rust/Cargo 1.99.1, 2026-10-21:

- `cargo test --workspace --locked`: 712 passed, 0 failed, 23 opt-in ignored.
- `cargo clippy --workspace --all-targets --locked -- -D warnings`: passed.
- `cargo build --workspace --locked`: passed.
- `cargo fmt --all -- --check`: passed.
- `python -m unittest discover -s scripts -p 'test_*.py'`: 95 passed.

The worker/store regressions cover paused manual writes, newer-request
coalescing, full/corrupt injected stores, retries without ticking, explicit risk
exit, verified reload/fallback, terminal archives/offline publication and 500
fragments. Dispatch tests cover focus/repeat/synthetic/settings admission,
unchanged paused save bytes and cancel-on-resume. UTC and CPU raster output are
covered; those checks do not certify native readability or OS delivery.

Review/merge this PR before the updated Q-wheel PR #387. It closes #382 on merge;
remaining native acceptance stays explicitly under #376.

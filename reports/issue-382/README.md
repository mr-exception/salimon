# Issue #382 — checkpoint control preparation (BLOCKED, draft)

Implemented the backend portion: typed SaveNow/SaveAndExit/Retry/CancelExit/
ExitWithRisk policy; verified Idle/Saving/Saved/Failed status and captured UTC
receipt; bounded coalescing into the existing active-run worker/store; no false
exit success from an older write; explicit failure choices; retry while gameplay
time is frozen. Auto 60-second/event scheduling remains. Requests do not tick
simulation or apply offline fatigue.

This does **not** complete or close #382. Its explicit blocker #381 is open and
itself requires #371 (which depends on #369/#370). No pause UI exists in main;
adding save buttons before full freeze would violate the issue's state-consistency
contract. The portable adapter methods are intentionally unused by native UI until
that prerequisite is implemented. Remaining: accessible pause menu/Settings entry,
input/cursor/Q-wheel integration, complete freeze/resume coverage, status and UTC
presentation, native screen/DPI/layout checks. Keep the PR draft.

Linux validation, 2026-10-19:
- Focused `cargo test -p salimon-client --locked checkpoints`: PASS, 10 tests.
  Includes real native checkpoint roundtrip/fallback/500 fragments, policy tests,
  async manual stationary-state save/reopen, rapid coalescing, disk-full retry
  without advancing game time, and corrupt-write rollback/explicit-risk exit.
- Faults use injected CheckpointStore implementations through the production
  SaveManager and worker; no actual filesystem exhaustion is claimed.
- Workspace build: PASS (`cargo build --workspace --locked`).
- Workspace all-target Clippy with `-D warnings`: PASS.
- `cargo fmt --all -- --check`: PASS; 93 Python script tests: PASS.
- Final workspace test run was interrupted by environment replacement before its
  completion was observed; NOT claimed as a completed pass. The preceding #368
  tree completed 597 workspace tests (22 existing opt-in ignored). The final
  checkpoint changes passed the focused 10-test run above before replacement.
- The resumed runtime has no Rust toolchain; earlier completed results are
  retained here, while fresh final-tree verification remains for CI.
- Native pause-menu visual/DPI tests NOT RUN: blocked by missing #381/#371 UI.

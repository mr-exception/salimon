# Repeated Linux CI stage failure

## Diagnosis

Actions run [38086122402](https://github.com/mr-exception/salimon/actions/runs/38086122402)
at `0ab2204` failed in the combined ten-minute graphical step. Windows/macOS,
Linux Rust/Python/build checks and CPU stress measurements passed. Main run
38085923764 failed at the same combined stage.

Downloaded artifact 11681688993 contains **29 completed scenario results, all
passed**, including 500-fragment load and required resource/ship evidence.
Their recorded durations total 589.946 seconds. `baseline.log` is 218,809,919
bytes: the CLI printed every per-step world snapshot to both `tee` and Actions.
The step spans 21:05:45–21:16:46 UTC despite its ten-minute timeout. The evidence
points to aggregate stage/logging overhead rather than a failed gameplay check.
The decoded full job-log connector failed with `Transport closed`; diagnosis
uses downloaded artifacts and job step timestamps, not an invented log excerpt.

The [compact failure manifest](failed-run-summary.json) records source metadata,
scenario status/duration and original artifact paths. Paths refer to the CI
workspace; the complete uploaded artifact remains the full state evidence.

Earlier fixes addressed distinct protocol readiness and long physics-command
budgets. This change addresses the remaining combined CI-stage budget and
unbounded console-state expansion, without changing those contracts.

## Fix

- Add opt-in `--summary` CLI output retaining status, errors, elapsed time and
  evidence paths. Full snapshots, protocol logs and screenshots stay in artifacts;
  default CLI JSON remains compatible. Failed scenarios still exit nonzero.
- Use compact output in every workflow E2E command, including optional evidence.
- Run baseline, cockpit-window, resource and ship/EVA evidence as separate
  bounded steps (10/3/10/5 minutes). Preserve every existing route and capture
  command, lavapipe configuration, failure propagation and always-run upload.
- Keep production simulation, scenario deadlines and assertions unchanged.

Applying the summary projection to the failed run yields 9,425 bytes rather
than 219 MB of console state (exact byte counts in the manifest). Each suite
gets its own deadline instead of competing for the nearly exhausted shared one.

## Validation

- Passed: `python -m unittest discover -s scripts -p 'test_*.py'` — 43 tests,
  including persisted full success/failure evidence, compact error reporting and
  CLI failure exit/path preservation.
- Passed: `git diff --check`; workflow command/coverage comparison.
- Not run locally: native graphical/Rust checks; this container has no Rust,
  Xvfb or Vulkan installation. Runtime code is unchanged. The updated PR run
  must validate the complete Linux pipeline, including packaged smoke.

CI confirmation is recorded after the new run completes; do not interpret this
report as a guarantee against every future Linux failure.

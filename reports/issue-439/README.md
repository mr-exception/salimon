# Canonical new-game and offline restoration (#439)

Implements the missing cross-owner regression for a new canonical journey:
real published iron/silicate mining → physical intake → partly funded Recovery
Module fabrication → NativeStore checkpoint → closed-game reload → one completed
tray identity → installation → second disk reload. Explicit test positions isolate
mining and machine intake; this is not a measured acquisition/travel route.

The focused test passes. It checks exact reconstructed source against independent
launch reconstruction, retained depletion and consumed fragment identities,
unchanged starter consumables and active-time clock, completed output mass/job,
no replay at the same timestamp, and durable installed identity. Existing launch,
motion/contact and scanner tests supply rollback, interruption, coast/collision,
funded cadence, expiration and history coverage in the workspace gate.

Production Linux Xvfb/Mesa llvmpipe Vulkan routes pass **62/62 steps** across
`universe-mining.json`, `offline-resume-progress.json` and
`offline-clock-recovery.json`. Original loading, clock failure/recovery and resumed
captures, exact state/raw frame samples and source/binary hashes are retained in
[native evidence](native-evidence.json) and the adjacent compressed traces. The
original capture cohort uses parent `5113d628865e8fe863ec40c7dbe3092b8fbe347b` plus
the new regression; hashes identify its actual source and executable. Later Hall
changes are explicitly gated to their separate developer fixture.

The first actual-clock interruption probe failed with a protocol timeout before
its interruption assertion; [the failed result](restart-failed.json) is retained.
The maintained driver now isolates desktop input and retains process/protocol
logs for every process rather than losing diagnostic evidence.

Commands from repository root (use the configured X11/Vulkan environment):

```sh
cargo test -p salimon-client --locked canonical_mine_fabricate_disk_reload
python3 scripts/verify_universe_expansion.py --binary target/debug/salimon-client --output artifacts/issue-439/native --scenario universe-mining.json --scenario offline-resume-progress.json --scenario offline-clock-recovery.json
python3 scripts/verify_offline_resume.py --binary target/debug/salimon-client --output artifacts/issue-439/restart-final
```

The [final actual-clock restart probe](restart-result.json) **passes** on the
final executable: SIGKILL preserves both original slot hashes; two new processes
publish generations 2 and 3 with exactly **0 s / 37 s** of observed credit.
The aborted controlled +60 s is never published. Lossless per-process protocol
and stderr/stdout traces are indexed in that result.

[Shared verification](verification.json) records final build/Clippy/fmt, 147 Python
tests and documentation passes, plus 1,289 passing workspace tests (51 existing
opt-in ignored). The full local suite preceded the final Hall nonfinite-camera
guard/default-admission assertions; all four final Hall tests pass separately.
Published-head CI supplies the final complete workspace check. Controlled +60-second launch fixtures are distinct from
actual local elapsed-time restarts. No physical OS suspend/wake, device input,
Metal/Windows or M1 FPS is inferred. Those remain [#557](https://github.com/mr-exception/salimon/issues/557)/[#563](https://github.com/mr-exception/salimon/issues/563); presentation [#561](https://github.com/mr-exception/salimon/issues/561),
balance #660 and human opening experience #661 remain separately unverified.
No merge, release or baseline promotion is performed.

Documentation impact: session integration and the existing launch/scanner/motion
owners retain authority. The maintenance map, validation and runner guide now map
the regression and diagnostic logs to these checks. No schema, numerical physics
contract, material recipe or survival tuning changes.

Original frames include [loading](offline-resume-progress-step-012-offline-progress.png),
[verified resume](offline-resume-progress-step-022-offline-resumed.png) and
[mined material after reload](universe-mining-step-024-canonical-reloaded.png).

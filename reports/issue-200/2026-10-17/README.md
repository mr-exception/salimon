# Issue 200 — refreshed integrated M1 propulsion verification

Executed 2026-10-17 (Asia/Nicosia) on the physical Apple M1 iMac21,1 / 16 GiB /
macOS 26.2 (25C56) / Metal / 1920×1080 / AutoVsync. Rust 1.89.0,
Python 3.14.2. Implementation prerequisites #195, #198 and #199 are merged.
This evidence PR follows #269 in the review stack and uses its exact release
binary; [digest/build provenance](build-info.json). No gameplay code changed.

## Covered regression results

All **38 baseline routes** and **seven realistic-flight evidence routes** pass.
[Structured route matrix](results.json), [baseline state/results](baseline/),
[flight state and screenshots](flight/). The current seven resource and four
ship/EVA graphical routes also pass in the immediately preceding
[#236 report](../../issue-236/README.md); the binary is identical.

The local OS-input probe uses actual CoreGraphics events directed through the
native event loop, with foreground PID verification. It does not send protocol
steering/throttle/interaction commands. E2E supplies deterministic initial
conditions, explicit 16 ms simulation ticks and inspection/capture; this is
native input correctness evidence, **not wall-clock FPS**. Focus/input helpers
run sequentially on the idle desktop; no other game/test window runs concurrently.

| Area | Evidence and asserted behavior |
| --- | --- |
| Forward/reverse, inertial coast, steering | [OS flight checks](os-flight/checks.json): native held throttle repeat and release preservation; five-second A/D holds, pitch/roll, unchanged coasting velocity, native reverse motion |
| Sustained/repeated mouse-look | Same OS flight probe: six successive positive relative drag events, then six inverse events; view changes and returns within 1e-6 while ship orientation remains under ship controls |
| Loaded ship, zero-G blackout, charge conversion/recovery | [OS Core checks](os-core/checks.json): native Space drifts/coasts without power; native E consumes the held bootstrap once, restores power through gradual conversion, preserves the other 499 IDs |
| Collision/crash, assisted recovery, takeoff | [OS crash checks](os-crash/checks.json): actual grounded state, native L recovery, landed completion, native L takeoff and moving inertial handoff with zero throttle |
| Normal landing and cutoff, assist locks, flight telemetry, EVA | Seven [realistic-flight routes](flight/) plus all baseline routes, using production gates |

OS observations, helper stdout (including target/foreground PIDs), input duration,
state logs and direct GPU screenshots are retained in those directories.
The probe source is [input_probe.py](input_probe.py), adapted from the reviewed
#198 native input probe to extend A/D holds from 0.8 s to 5 s and exercise repeated
mouse events. The unchanged reviewed Swift helper source is
[here](../../issue-198/2026-10-16/input.swift).

## Reproduction

Copy the staged #236 release binary into the local macOS app bundle and compile
the helper as in [#254's report](../../issue-254/README.md), then run sequentially:

```sh
SALIMON_REFERENCE_SIZE=1 RUST_LOG=info python3 scripts/salimon-test suite --summary --binary artifacts/build/native-236/salimon-client --artifacts artifacts/native-200-20261017/baseline
SALIMON_REFERENCE_SIZE=1 RUST_LOG=info python3 scripts/salimon-test suite --group realistic-flight --evidence --summary --binary artifacts/build/native-236/salimon-client --screenshot-command '["native"]' --artifacts artifacts/native-200-20261017/flight
SALIMON_REFERENCE_SIZE=1 RUST_LOG=info python3 reports/issue-200/2026-10-17/input_probe.py --binary artifacts/SalimonAcceptance.app/Contents/MacOS/salimon-client --helper artifacts/native-input --output artifacts/native-200-20261017/os-flight --scenario flight-regression
SALIMON_REFERENCE_SIZE=1 RUST_LOG=info python3 reports/issue-200/2026-10-17/input_probe.py --binary artifacts/SalimonAcceptance.app/Contents/MacOS/salimon-client --helper artifacts/native-input --output artifacts/native-200-20261017/os-core --scenario cargo-blackout
SALIMON_REFERENCE_SIZE=1 RUST_LOG=info python3 reports/issue-200/2026-10-17/input_probe.py --binary artifacts/SalimonAcceptance.app/Contents/MacOS/salimon-client --helper artifacts/native-input --output artifacts/native-200-20261017/os-crash --scenario crash-recovery
```

All 483 Rust tests, 80 Python contracts, Clippy, formatting and locked release
staging passed for this exact runtime in [#236](../../issue-236/README.md).
Those CPU/build checks were not repeated for this evidence-only PR. Probe source
compilation and relative report links were checked; `git diff --check` passed.

## Scope and limitations

This closes the bounded integrated native propulsion/input regression coverage,
including the sustained steering/mouse gap in the historical 2026-10-15 report.
It does not claim unrestricted human exploratory play or a strict visible-display
60 FPS floor. Current foreground full-size 500-sphere settled evidence is
[separate](../../issue-236/performance.md); active-only tails, reference frame-tail
calibration and larger mixed-piece stress remain #255. Native gameplay correctness
must not be treated as performance signoff. No disk/backend persistence, orbital
simulation or other deferred feature was added. Historical reports retain their
original coverage limits and are not rewritten as if those runs tested this build.

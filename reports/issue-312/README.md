# #312 — Native input, lifecycle and takeoff on Apple M1

Verified 2026-10-20 on the reference M1 iMac21,1 / 16 GiB / macOS 26.2 (25C56), Metal, 1280×800 physical drawable, 2× scale. Tested runtime/assets `4d8faf8f108ebf046690b55f2e71d31bbc2e571b`; later stack commits contain evidence only. [Machine](../issue-299/environment.json) and [binary SHA-256](../issue-299/build-info.json). Prerequisite #302/#309/#295 implementations are merged.

## Passed evidence

- **14 native lifecycle checks:** actual macOS Escape/Enter pause and resume, frozen paused state/time despite E/L/F/toolbar input, slots 1/2, F not activating mining, F2 tour/gameplay guards, held W crossing F2 and release without stuck steering, native window minimize/restore and no steering replay. [Checks and raw input/state](evidence/lifecycle/).
- Native held throttle repeats, release retaining selection, yaw/pitch/roll, coasting velocity, reverse selection and eventual reverse motion. [Long probe](evidence/os-flight-current/). Its separate exact inverse-mouse check failed and is not included in this keyboard pass.
- **Two dedicated mouse checks:** HID mouseMoved changes look and the inverse restores it. [Dedicated probe](evidence/mouse-current/). A longer sequence missed its inverse event; see limitations below.
- **Five native L checks:** crash grounding, recovery request, landed recovery, takeoff request, then Flying at zero speed and throttle. [Native recovery](evidence/os-recovery/).
- Maintained cockpit-nose, space-airlock, moving-EVA and nearby-EVA evidence routes all pass; maintained inertial-flight passes all 426 steps. [Ship/EVA](evidence/ship-eva/), [flight](evidence/flight/). Maintained cargo flight/blackout results are retained alongside them. Core actual native E outage/restoration evidence is [#299](../issue-299/README.md).
- **Six 21-step takeoff routes:** Earth, Moon and Mars, each dry and with 500 production 10 kg iron spheres (5,000 kg). Repeated landing cannot cancel takeoff; handoff is Flying with exactly zero speed and zero selected throttle. [Current routes and snapshots](evidence/takeoffs-current/). Initial on-foot takeoff attempts were correctly rejected and remain in the failed-attempt evidence.

All state transitions use production commands after fixture initialization. Native keyboard/mouse events come through the explicitly authorized macOS helper. `window minimize/restore` invokes the actual native window API and checks resulting events/state, but it is not physical system sleep. E2E clocks use explicit fixed steps; these results are not FPS or live wall-clock acceptance.

## Reproduction

The [native lifecycle probe](native_lifecycle.py), [flight/recovery probe](native_flight.py) and [six-body/load route runner](takeoffs.py) run from the repository root. The helper adapts [the historical reviewed source](../issue-198/2026-10-16/input.swift): [current source](input.swift) sends mouseMoved HID events without click edges, because the old session-tap drag failed to deliver look here.

```sh
swiftc reports/issue-312/input.swift -o artifacts/native-20261020/input-current
python3 reports/issue-312/native_lifecycle.py --binary artifacts/native-20261020/SalimonReview.app/Contents/MacOS/salimon-client --helper artifacts/native-input --output artifacts/native-20261020/312/lifecycle --scenario flight-regression
python3 reports/issue-312/native_flight.py --binary artifacts/native-20261020/SalimonReview.app/Contents/MacOS/salimon-client --helper artifacts/native-20261020/input-current --output artifacts/native-20261020/312/mouse-current --scenario flight-regression --mouse-only
python3 reports/issue-312/native_flight.py --binary artifacts/native-20261020/SalimonReview.app/Contents/MacOS/salimon-client --helper artifacts/native-20261020/input-current --output artifacts/native-20261020/312/os-recovery --scenario crash-recovery
python3 reports/issue-312/takeoffs.py
python3 scripts/salimon-test suite --group ship-eva --evidence --summary --binary artifacts/native-20261020/build/salimon-client --screenshot-command '["native"]' --artifacts artifacts/native-20261020/312/ship-eva
```

Listed probes/routes pass, except the separately retained long-flight inverse event. Logs include original and current attempts. Syntax compilation and `git diff --check` passed. No runtime changes; unrelated Rust/model suites were not rerun. Python 3.14.2 / Rust 1.89.0; no Python 3.12 tooling claim. CPU stress verification ran concurrently with these functional checks, so no timing or performance inference is made.

## Failures and remaining acceptance

The legacy full probe expected negative velocity after only six simulated seconds at whatever throttle a wall-time hold produced. Its initial selection was -15%, insufficient for the claimed reversal from ~104 m/s. The current harness observes up to twenty explicit 600-frame windows and checks actual reversal, without changing thrust/time rules. The old drag injection also produced no look; the HID event variant passes the dedicated forward/inverse check. In the longer current sequence, forward look arrived but the inverse left the direction unchanged. That failed check remains visible; it is not waived as a product pass or attributed to the game without delivery evidence.

Initial takeoff extensions retained the terrain scenario's deliberate cockpit exit. The requests were rejected because authority was on foot, not because takeoff failed. The corrected route stays in the cockpit through landing and then tests takeoff; all six pass. Both attempts and complete raw results are retained.

**Actual physical macOS sleep/wake, normal-play live-clock discontinuity and longer-sequence mouse delivery remain in [#453](https://github.com/mr-exception/salimon/issues/453).** The shared desktop was not put to sleep. Minimize/restore and fixed stepping do not certify those gates; closing #312 records evidence plus that explicit transfer. Baseline approval and hardware frame budgets remain #314/#255, not approved here.

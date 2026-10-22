# #453 — Native lifecycle and relative-mouse verification

## Outcome

2026-10-22 on reference Apple M1 iMac21,1 /16 GiB /macOS26.2, Metal,
physical1920×1080 /2× scale. [Binary identity](evidence/binary.json); production
runtime is the #348 modal fix, with subsequent branches adding tests/docs only.
No runtime lifecycle/input changes are made in this PR.

**14/14 bounded native lifecycle checks pass.** A longer flight sequence passes
all nine keyboard/throttle/coasting checks, but **relative-mouse motion fails**.
A dedicated mouse-only repeat also fails to move the look direction. Physical
system sleep remains **not run** pending a dedicated shared-desktop window.
This is a draft, not completed #453 signoff, and does not close the issue.

## Actual executed checks

The owner explicitly authorized the reviewed native macOS input helper in this
session. Its #348 adaptation selects the largest target window and refuses key/
mouse input on a foreground-PID mismatch. These are real OS events with explicit
fixed-step E2E simulation: they do not establish normal wall-clock or physical
sleep/wake behavior. Exact commands:

```sh
SALIMON_REFERENCE_SIZE=1 python3 reports/issue-312/native_lifecycle.py --binary artifacts/native-20261022/build-fixed/salimon-client --helper artifacts/native-20261022/input-review --output artifacts/native-20261022/453/lifecycle --scenario flight-regression
SALIMON_REFERENCE_SIZE=1 python3 reports/issue-312/native_flight.py --binary artifacts/native-20261022/build-fixed/salimon-client --helper artifacts/native-20261022/input-review --output artifacts/native-20261022/453/long-flight --scenario flight-regression
SALIMON_REFERENCE_SIZE=1 python3 reports/issue-312/native_flight.py --binary artifacts/native-20261022/Salimon348Fixed.app/Contents/MacOS/salimon-client --helper artifacts/native-20261022/input-review --output artifacts/native-20261022/453/dedicated-mouse --scenario flight-regression --mouse-only
```

Lifecycle pass includes Escape/Enter, frozen paused authority/time, slots1/2,
F-not-mining, F2 gameplay/tour guards, held-W release across tour, actual native
minimize/restore and no replayed steering. [Checks](evidence/lifecycle/checks.json),
[observations](evidence/lifecycle/observations.json),
[log](evidence/453-lifecycle.log).

Long sequence passes held throttle repeats/release, yaw left/right, pitch, roll,
released steering stability, coasting velocity preservation and reverse motion.
**The mouse-positive observation does not change the look direction** after
subtracting the moving camera origin. The later inverse equals the original
because neither event moved the view; the legacy probe's `inverse=true` is
**vacuous, not a successful inverse-delivery pass**. Both commands exit1 with
explicit failed checks; their raw records are retained without reclassification.
[Long checks](evidence/long-flight/checks.json),
[long observations](evidence/long-flight/observations.json),
[dedicated checks](evidence/dedicated-mouse/checks.json),
[dedicated observations](evidence/dedicated-mouse/observations.json).

Helper event posting succeeded and focus metadata matches target PID, but this
alone does not prove winit received a raw mouse event. No app/capture defect or
root cause is asserted without that boundary evidence. This differs from the
[2026-10-20 dedicated pass](../issue-312/README.md); historical failures and passes
remain valid only at their recorded contexts. The current normal-play #348 tray
attempt also retained its original pose/item and is not a collection pass.

## Physical sleep gate and prepared probe

The owner authorized an actual sleep/wake cycle. Before executing it, read-only
macOS power assertions reported active call/audio use. #453 explicitly says
**“Do not sleep a shared desktop while unrelated work is active; schedule a
dedicated native session.”** A timing question remains pending for that changed
state. No application was stopped and no power settings/schedules were changed.
Automatic one-shot wake scheduling requires administrator access here;
`sudo -n true` reports that a password is required. No password or elevation was
requested or used. [Sanitized readiness record](evidence/sleep-readiness.json).

The [prepared probe](sleep_probe.py) launches an isolated normal wall-clock run,
records verified before/after saves and actual own-window images, holds W across
`pmset sleepnow`, requires real macOS Sleep/full-Wake log evidence, then checks
stable position without fresh input and bounds live simulation advancement by
actual awake wall time. Integer-second power-log endpoints have an explicit
rounding allowance, not a gameplay/performance waiver. Its source has been
syntax-checked but **the physical probe has not been executed**. The operator
must be ready to wake the Mac manually after about20 seconds; no automatic wake
or sudo workaround is installed. Example once the dedicated window is confirmed:

```sh
python3 reports/issue-453/sleep_probe.py --binary artifacts/native-20261022/Salimon348Fixed.app/Contents/MacOS/salimon-client --helper artifacts/native-20261022/input-review --source-checkpoint reports/issue-376/evidence/initial/landed/before.sav --output artifacts/native-20261022/453/sleep --run-sleep
```

Until real execution/evidence, renderer recovery, physical sleep clearing,
normal live-clock discontinuity and the longer mouse inverse acceptance remain
unverified. Minimize/restore, a prepared script or #376 launch/offline proof is
not a substitute.

## Checks and ownership

Python syntax, documentation/base-impact and whitespace checks pass. No
production code changed; runtime build/tests/Clippy from #348 remain inherited.
[Runtime lifecycle ownership](../../client/runtime/architecture.md),
[runtime invariants](../../client/runtime/invariants.md) and the existing
[runner input-isolation contract](../../scripts/README.md#deterministic-input-boundary-313)
are unchanged. Fourth PR is based on the #451 branch, preserving requested order.

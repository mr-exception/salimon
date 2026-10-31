# #557 — local M1 platform evidence

Collected 2026-10-31 on the owner's Apple M1 iMac (iMac21,1), 16 GB,
macOS 26.2, AC power, built-in Retina display. Actual renderer: Apple M1,
Metal, integrated GPU, timestamp queries available. The external display is
available at 75 Hz; these measurements use the built-in 60 Hz/scale-2 context.

The owner explicitly selected **automated evidence now, physical checks later**.
This delivery does not establish physical human input, actual OS suspend/wake,
optical latency, other-platform compatibility or subjective approval. Exact
remaining criteria transfer to [#720](https://github.com/mr-exception/salimon/issues/720).
Source closure records delivery/transfer, not passage of those criteria.

## Delivered and checked

| Check | Actual outcome |
| --- | --- |
| Locked optimized runtime build | Passed, `cargo build --release --locked -p salimon-client` |
| Real Metal declarative routes | **53 routes / 2,783 steps passed**: 21 universe/offline/scanner/flight routes and 32 construction routes |
| Normal launch from unrelated cwd, isolated saves/settings | Passed: macOS OS-injected Enter admitted Continue; F2 switched to PrecisionTour and back |
| Native audio device | Passed: built-in speaker CoreAudio callbacks; 211 active / 220 after mute, 48 kHz stereo F32 |
| Runtime release tests | **564 passed, 25 ignored**, zero failures; ignored device/export probes run separately as recorded |
| Python tooling suite | **152 passed**; fixture-generated failure messages in the log are expected test inputs |
| Formatting / documentation paths and base impact | Passed; final logs retained below |
| Physical mouse/keyboard, held controls across real sleep/wake, physical scale changes | **Unrun**, deferred by owner; #720 |

The smoke terminates its isolated process after its checks (`-15`); this is not
a clean-exit/save durability claim. Portable/native scenario save checks retain
their own scope. Device callbacks prove actual device admission/continuity during
this probe, not acoustic recording, listening approval, hardware switching or
sleep recovery. See [telemetry](audio-device.json).

## Evidence and provenance

[Native index](native-index.json) maps every scenario, step count, original
result hash, lossless trace and original GPU PNG. Paths in that index are relative
to `reports/`; original absolute artifact paths are provenance, not portable links.
Raw result traces are `.json.xz`; decode with Python `lzma` or `xz -dc`.
They preserve every original state and measurement, without duplicating protocol
and per-step copies. Original GPU images are in [#561's package](../issue-561/README.md).

[OS smoke result](os-smoke/result.json), [startup](os-smoke/startup.png),
[gameplay](os-smoke/gameplay.png) and [received logs](os-smoke/stderr.log)
use PID-scoped window captures, excluding the surrounding desktop. Compositor
capture is separate from GPU readback and optical scanout.

Runtime source is `ad5a55e0590505708466cce59e4a8ba5844552b4`; no Rust, assets,
shaders or gameplay rules change in this PR. Tooling commit `125944f1` precedes
the clean tracked corpus capture. Binary SHA256:
`3f0e6e0f4638ebd9523e79ac1ef4a5657f8b6bb723ca54e5e75e4d07cd042560`.
Rust/Cargo 1.89.0, locked release. Exact commands/environment are retained in the
[collection manifest](../issue-563/commands.json), with original
[source hashes/context](../issue-563/source-context.json).
Recovery collection was added afterward; its exact collector hash is retained
[with that route](../issue-660/recovery/collector-context.json).
The [package check](package-check.json) verifies all 53 lossless result streams
against their original hashes, all 128 corpus PNG hashes and report-local links.
The shared [evidence hash inventory](../issue-563/evidence-sha256.json) covers
all retained report artifacts; paths are relative to `reports/`.

Useful logs: [build](logs/build-release.log.gz),
[runtime tests](logs/runtime-release-tests.log.gz),
[final Python tests](logs/final-scripts.log.gz),
[final documentation checks](logs/final-docs.log.gz), [fmt](logs/fmt.log.gz).
No additional Clippy campaign was run: this PR changes tooling, guides and evidence,
with no Rust/Cargo changes. Remote current-head checks are reported on the PR.

## Deferred physical session

[#720](https://github.com/mr-exception/salimon/issues/720) preserves the complete
physical control/display/startup/audio/held-input/sleep procedure, including
historical stale generation8 and expired-hold/0.21 s overlap failures. Follow
[the maintained Mac procedure](../../docs/macos-qualification.md), with normal
clock, isolated saves, fresh generation/current UTC for each Save Now, OS power
timestamps and original video. No dark-screen or synthetic event substitutes.

Ownership: runtime [architecture](../../client/runtime/architecture.md) /
[invariants](../../client/runtime/invariants.md). Related outcomes:
[#561](../issue-561/README.md), [#563](../issue-563/README.md),
[#660](../issue-660/README.md).

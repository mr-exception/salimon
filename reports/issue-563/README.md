# #563 — optimized M1/Metal frame and stress evidence

Actual Apple M1 iMac, 16 GB, macOS 26.2, AC, built-in 60 Hz display,
1920×1080 drawable, physical scale 2, normal AutoVsync. Locked release binary,
Rust/Cargo 1.89.0; exact runtime/binary identity is in [#557](../issue-557/README.md).
Measurements ran serially after compilation, with no other Codex test/build/GPU
campaign during sampled windows. Ordinary desktop background activity is not
controlled laboratory isolation. Thermal history/system GPU memory are unmeasured.

**Collection passed; performance qualification and owner budget decisions did not.**
The strict normal-canonical p95 comparison fails in all three accepted runs.
The 1,000-object mixed stress result is also far outside 60 FPS guidance. All
pending budgets, historical attribution failures and remaining hardware workload
qualification transfer to [#722](https://github.com/mr-exception/salimon/issues/722).

The [readable canonical summary](canonical-summary.json) is derived from the
lossless per-trial results; raw frame/memory samples remain in `result.json.gz`.

## Ordinary canonical new journey

Normal launch, isolated saves/settings, actual macOS OS-injected Continue, no
E2E/optional pack, five-second warmup, three fresh 30-second foreground samples.
The scene remains the initial outside-Scout view, not moving flight. Per-frame
logging and 250 ms FrameClock interval cap are explicit instrumentation limits;
uncapped present-wall timings and latest GPU pass timestamps are retained.

| Trial | Consecutive frames | Submitted frames/s | Interval median / p95 | GPU pass p95 | Sampled RSS maximum |
| --- | ---: | ---: | ---: | ---: | ---: |
| [3](canonical-3/result.json.gz) | 1,804 | 60.00 | 16.661 / 18.609 ms | 6.999 ms | 134,736 KiB |
| [4](canonical-4/result.json.gz) | 1,803 | 60.01 | 16.673 / 18.587 ms | 7.183 ms | 134,992 KiB |
| [5](canonical-5/result.json.gz) | 1,801 | 60.00 | 16.658 / 18.586 ms | 7.390 ms | 134,512 KiB |

Verified Continue took 1.727 / 1.722 / 1.745 s. Every sample is focused and
non-occluded with consecutive frame IDs. The unchanged 16.667 ms p95 comparison
**fails**. Roughly 60 submitted frames/s is not optical scanout, a 1% low floor,
general flight/streaming acceptance or approval to revise this comparison.
macOS `ps` provides sampled process RSS, not process high water or GPU/unified
system allocation; those fields remain unavailable. GPU timestamps measure the
recorded pass, not all display/compositor work or hardware counter attribution.

Trials [1](canonical-1/result.json.gz) and [2](canonical-2/result.json.gz) are retained
rejections: AX window lookup failed for the direct executable; then a later
focus loss invalidated the sampling window. Trial 2's raw frames remain retained.
The collector now uses PID-scoped CoreGraphics window inventory/capture and
restores activation before warmup; sampled focus loss still fails. No rejected
window enters the accepted table. Normal timing collection preceded the first
tooling commit; runtime bytes are identical, and no later-head timing pass is
inferred. Raw per-trial stderr and compositor originals remain beside results.

## Fragment workloads

[Raw summary](stress/summary.json) and compressed original `.stdout.gz` files
retain twelve foreground current-cap windows: empty/500 objects × facing/away ×
three fresh processes, seed 155, zero warmup, minimum 3 s and 100 completed
intervals. All twelve satisfy collection/admission/accounting checks.

Empty-window frame p95 spans 17.203–17.283 ms. The 500-object full-window p95
spans 17.196–17.402 ms; physics p95 spans 6.203–6.498 ms. **Each 500-object
window retains only 24 moving samples and then settles**. These mixed-window
tails do not establish active-only acceptance; each view has only 72 moving
samples across three runs. No 100-sample active-only tail or ten-drop legacy
budget approval is claimed. Raw state flags, IDs/mass and lifecycle are retained.

The separate [1,000 current-mixed probe](mixed-stress/summary.json), facing,
seed 155, zero warmup, retained **101 moving samples over 38.201 s** to meet the
100-interval minimum. Frame median **480.719 ms**, p95 **604.737 ms**, max
**632.373 ms**; physics median **475.419 ms**, p95 **603.477 ms**. It completes
collection within the 90 s process deadline but **fails performance guidance**.
This is one stress run, not repeat/regression or arbitrary-density acceptance.

## Reproduction and limits

[Commands/environment](commands.json), [cap command/focus record](stress-command.json),
[mixed command/focus record](mixed-stress-command.json) and retained
`collection-drivers/` specify exact argv and opt-in PID activation. Use
[the maintained Mac guide](../../docs/macos-qualification.md) and
[fragment contracts](../../scripts/FRAGMENT_BENCHMARKS.md). All raw stdout files, summaries, stderr and failed attempts remain, with a shared evidence hash manifest.

No physics, cadence, population cap, numerical limit, asset ceiling or proposed
budget was changed/approved. Legacy ten-drop acceptance, the six CPU-render and
three settled attribution failures, matched lighting/audio controls, moving
flight/terrain/streaming, construction workload budgets, hardware memory and other
platforms remain in #722. Functional stepped route timings are separate from
these normal-clock measurements. Runtime/physics/renderer architecture and
invariants are unchanged; this is tooling/evidence delivery.

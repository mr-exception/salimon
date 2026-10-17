# Native performance comparison

Medians of three per-window p95 values, same physical M1 and foreground admission. All 500-piece cases remain naturally asleep with original IDs/mass. Raw p99/max/spikes are preserved in the linked summaries and compressed intervals. Native windows ran sequentially, after compilation/capture tests finished.

| Workload / build | Intervals total | Median intervals/s | Frame p95 ms | Physics p95 ms | CPU render p95 ms | GPU latest p95 ms | Max interval ms | Draw calls |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| onboard before / onboard | 5403 | 59.96 | 17.120 | 1.406 | 0.956 | 1.608 | 28.467 | 13 |
| onboard after (#255) / onboard | 5402 | 59.95 | 17.125 | 1.420 | 0.945 | 1.624 | 42.964 | 13 |
| spheres before / sphere-deck | 1443 | 59.97 | 17.114 | 1.391 | 0.949 | 1.636 | 28.068 | 13 |
| spheres before / sphere-pile | 1443 | 59.96 | 17.129 | 1.547 | 0.959 | 1.651 | 42.530 | 13 |
| spheres after / sphere-deck | 1442 | 60.00 | 17.150 | 1.376 | 0.954 | 1.661 | 28.840 | 13 |
| spheres after / sphere-pile | 1443 | 60.00 | 17.165 | 1.546 | 0.946 | 1.667 | 38.955 | 13 |

Existing 20% same-device relative gates on each corresponding repeat: **0 regressions**. Pass.

Onboard windows: 2 s warmup + 30 s collection, preparation limit 2,400 steps. Sphere-deck/pile: 2 s + 8 s, same preparation limit. Seed 155, facing camera, minimum 100 completed intervals per window. Prepared sleep is verified throughout, not inferred from the requested state. 500 onboard pieces total 16 kg; 500 production mining spheres total 5,000 kg. The workloads are not interchangeable.

Before binary: #236 digest `7dd87ac17e3545e5365a27adb562176bf35974666684d9ff4b5c06e462ad5953`; after: `a7e2bdbec8d32dff611e870daebf2b42465c507c7c1277c737203164f83dae4b`. Current onboard windows/raw files are reused from [#255](../issue-255/settled/), avoiding duplicate collection. Before/current sphere intervals and focus records live in [benchmarks](benchmarks/). Machine/build metadata and their limits are in the report.

Raw spikes and p99 remain reviewable; passing p95 gates does not establish zero stalls or a physical display FPS floor. GPU latest-pass values can repeat asynchronously. Metal allocator residency is unavailable; upload/scratch/texture budgets are not total GPU resident memory. Active performance fails separately under #288.

## Offscreen Metal terrain pass costs

512×384 synthetic fixtures, 30 GPU samples per scene, debug test harness, identical geometry. These are pass timings, not native gameplay FPS.

| Scene | Before median / p95 ms | After median / p95 ms |
| --- | ---: | ---: |
| earth-near | 0.063 / 0.066 | 0.070 / 0.074 |
| earth-far | 0.041 / 0.050 | 0.040 / 0.046 |
| earth-orbit | 0.070 / 0.079 | 0.069 / 0.077 |
| moon-near | 0.059 / 0.066 | 0.070 / 0.075 |
| moon-far | 0.039 / 0.049 | 0.042 / 0.048 |
| moon-orbit | 0.069 / 0.078 | 0.071 / 0.078 |
| mars-near | 0.059 / 0.069 | 0.064 / 0.072 |
| mars-far | 0.041 / 0.047 | 0.041 / 0.046 |
| mars-orbit | 0.071 / 0.078 | 0.070 / 0.074 |

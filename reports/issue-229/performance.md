# Native performance observations

| Pieces | Repeat | Intervals | Median intervals/s | Frame p95 ms | Physics p95 ms | CPU render p95 ms | GPU latest p95 ms | Draw calls |
| ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| 0 | 1 | 481 | 59.90 | 17.200 | 0.003 | 1.170 | 1.539 | 12 |
| 0 | 2 | 481 | 59.89 | 17.239 | 0.003 | 1.173 | 1.541 | 12 |
| 0 | 3 | 479 | 59.94 | 17.246 | 0.003 | 1.153 | 1.541 | 12 |
| 500 | 1 | 481 | 60.00 | 17.079 | 1.286 | 0.912 | 1.618 | 13 |
| 500 | 2 | 482 | 59.98 | 17.131 | 1.384 | 0.954 | 1.609 | 13 |
| 500 | 3 | 480 | 59.94 | 17.049 | 1.378 | 0.940 | 1.611 | 13 |

Historical same-device #199 foreground results used source `875f65c`, 10 s warmup / 30 s windows. Current windows are shorter and source/assets have changed. These are descriptive medians of three p95 observations, **not matched statistical regression gates**. The old schema lacks native admission records and is intentionally rejected by the new automatic comparison. Its separate OS foreground evidence is linked in [the historical report](../issue-199/2026-10-16/README.md).

| Metric, 500 pieces | Historical median p95 | Current median p95 | Descriptive delta |
| --- | ---: | ---: | ---: |
| frame_ms | 17.120 ms | 17.079 ms | -0.2% |
| physics_total_ms | 1.375 ms | 1.378 ms | +0.2% |
| cpu_render_ms | 0.971 ms | 0.940 ms | -3.2% |

Current empty frame-p95 values (17.200/17.239/17.246 ms) are also above the provisional 16.67 ms guidance. Native cadence remains about 60 successful-present intervals/s in both populations. No strict per-frame floor, active-only tail signoff or total GPU-memory claim follows. Raw maxima/spikes remain in the retained samples.

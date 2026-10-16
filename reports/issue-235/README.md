# Issue 235 — portable sphere workload evidence

Adds reproducible production-adapter and renderer-preparation probes, complete
raw evidence, strict report validation and scheduled weekly execution. PR CI
remains source-build only. Portable regressions cover hold/aim loss/cap repeat/
early depletion in existing mining routes, all-material pickup/carry/drop/Core
integration from #234, and the new 500-sphere contact/sleep tests from #233.
The new renderer gate checks every 10/100/500 workload with and without a growth
preview; no feature-specific GPU or engine dependency is added.

## Measured CPU results

Linux x86-64 shared cloud container; Rust 1.89.0 **test profile**, runtime and
renderer unoptimized, physics/math optimized at level 2, one codegen unit, no
debug symbols. These are CPU adapter timings, not presented-frame measurements.
Each row contains 100 retained 16 ms simulation updates. Setup/mining/preparation
are outside the timer. The 500-piece pile has 100 columns and five layers; all
resting populations reached sleep naturally after 33 steps (0.528 simulation s).
Active windows expose motion/sleep counts and may settle during sampling.

| Spheres | Workload | Mean adapter ms | p95 ms |
| ---: | --- | ---: | ---: |
| 10 | resting | 0.049 | 0.069 |
| 10 | active | 0.213 | 0.474 |
| 10 | pile-resting | 0.047 | 0.062 |
| 100 | resting | 0.499 | 0.611 |
| 100 | active | 1.857 | 3.651 |
| 100 | pile-resting | 0.523 | 0.745 |
| 500 | resting | 4.749 | 8.057 |
| 500 | active | 10.485 | 18.662 |
| 500 | pile-resting | 3.956 | 4.311 |

All resting rows retained every body asleep with zero integration, pair visits,
substeps and SAT projection builds. Identity/mass checks pass throughout active
and resting runs. The runtime benchmark uses 10 kg iron pieces; functional
integration independently covers ice/silicate/iron and short deposits.

## Matched renderer preparation comparison

Fixed 1:1 camera at a universe-scale origin, identical centers, 10 kg per material,
100 CPU preparation samples per case. Old six-shard assets and shared spheres
are measured in the current production preparation path; no historical full-frame
or old-physics performance is invented. All instances survive frustum submission.
The preview row includes one extra maximum-size sphere.

| Variant | Base count | Preview | Batches | Submitted triangles | Encoded bytes | Mean prepare ms |
| --- | ---: | --- | ---: | ---: | ---: | ---: |
| legacy-shards | 10 | False | 6 | 476 | 480 | 0.023 |
| legacy-shards | 10 | True | 6 | 556 | 528 | 0.023 |
| legacy-shards | 100 | False | 6 | 5246 | 4800 | 0.252 |
| legacy-shards | 100 | True | 6 | 5326 | 4848 | 0.269 |
| legacy-shards | 500 | False | 6 | 26522 | 24000 | 0.968 |
| legacy-shards | 500 | True | 6 | 26536 | 24048 | 0.971 |
| sphere | 10 | False | 1 | 800 | 480 | 0.021 |
| sphere | 10 | True | 1 | 880 | 528 | 0.020 |
| sphere | 100 | False | 1 | 8000 | 4800 | 0.282 |
| sphere | 100 | True | 1 | 8080 | 4848 | 0.159 |
| sphere | 500 | False | 1 | 40000 | 24000 | 0.900 |
| sphere | 500 | True | 1 | 40080 | 24048 | 0.875 |

Spheres reduce six batches to one, but the 80-triangle sphere increases submitted
triangle count versus these old shards. This is a batching/geometry tradeoff,
not proof of a GPU speedup. Encoded bytes are staging records, not actual GPU
transfer volume (the live cache can skip unchanged records). Scratch capacity
is retained CPU capacity, excluding allocator overhead and immutable meshes.

## Regression gates and reproduction

Evidence-backed structural gates: exactly one sphere batch, 48 encoded bytes per
submitted sphere, 80 triangles per sphere, zero SAT work, no sleeping integration/
pair work, and unchanged stable IDs/material mass. These catch repeatable work/
storage regressions without inventing a shared-host millisecond SLA. Report
validation rejects missing/duplicate cases, incomplete/nonfinite timing samples,
failed sleep and changed batch/encoding counts. Timing statistics stay descriptive.

```
python scripts/benchmark_spheres.py --run --output artifacts/sphere-benchmarks
python scripts/benchmark_spheres.py --output reports/issue-235
cargo test --workspace --locked
python -m unittest discover -s scripts -p 'test_*.py'
```

The `--run` command builds/runs **release** probes. Stored evidence here is test
profile and must not be compared as release data. Weekly CI runs release probes,
validates the matrix and preserves reports with its normal benchmark artifacts.
Direct reproduction of test-profile evidence: set `SALIMON_SPHERE_CPU_REPORT`
and `SALIMON_SPHERE_RENDER_REPORT`, then `cargo test --locked -p salimon-client
-p salimon-renderer sphere_ -- --ignored`. Local codegen/debug overrides and the
environment restart are recorded in [environment.json](environment.json).

## Validation and limits

Retained workspace test executables: **475 tests pass**, 13 opt-in tests ignored
(the renderer structural workload test is included). The two new opt-in CPU
probes pass separately: 900 runtime samples and 1,200 renderer samples. The
updated runtime pile benchmark was rebuilt with Rust 1.89 directly against the
retained compiled dependencies after the environment restarted; no source build
settings were committed. All 75 Python tooling tests pass. Formatting and diff checks pass; direct
Clippy-driver checks of runtime/renderer test targets pass with warnings denied
against the retained dependencies. Source-build CI validates the actual Cargo workspace.

GPU execution was unavailable: the cloud session has no installed Vulkan/display
stack; package installation encountered restricted filesystem/account operations.
**Not measured:** average presented FPS, 1% lows, GPU frame/pass time, actual GPU
upload traffic, total process/GPU memory. Native hardware follow-up remains #236.
A matched settled old-hull 500-piece physics comparison was not practical because
its enlarged geometry defines a different pile; the preserved comparison is
renderer preparation at matched poses/mass. No native M1 or full-frame guarantee
is made, and CPU time is never converted into FPS.

Raw evidence: [runtime](cpu.json), [renderer](renderer.csv),
[validated summary](summary.json), [environment](environment.json).

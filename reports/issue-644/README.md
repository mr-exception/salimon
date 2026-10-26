# Issue #644 — measured source-cohesion review

Reviewed the four nominated owners against current source, the #638 audit and
pinned 30-PR co-change history. Extracted two independent responsibilities;
retained the two cohesive owners. [Metrics](metrics.json) include extracted files
rather than treating line reduction as lower total complexity.

| Reviewed source | Decision and evidence |
| --- | --- |
| physics `lib.rs` | Split exact sampled contact, warm impulses, friction/rolling response and response save validation into `response.rs` (539 lines/59 branch tokens). Root falls 1,861 → 1,335 lines and 193 → 134 branch tokens; it retains public data/surface contracts, gravity/integration and explicit substep/broad-phase traversal. These numerical response helpers form an independently testable owner through the frozen sixteen-iteration oracle. |
| physics `convex.rs` | Retain 1,605 lines. Roughly 920 lines precede its main test module; the rest tests hull/SAT/feature contracts. Hull construction, cached projections, support-feature clipping and contact geometry share the same immutable representation and exact axis/order invariants. No evidence justifies splitting that representation or widening its interfaces solely to reduce file size. |
| renderer `resource_mesh.rs` | Split pure validated GLB triangle decoding and stable indexed packing into `resource_mesh/geometry.rs` (94 lines/18 branch tokens). Parent falls 1,541 → 1,452 lines and 93 → 75 branch tokens; it retains public DTOs/asset table, CPU support facade, retained instance staging/culling/upload and GPU batch ownership. About 1,000 lines are related CPU/GPU tests, not another production owner. |
| ship `controller/assist.rs` | Retain 1,263 lines; only about 519 precede the main tests. One captured transition coordinates landing/takeoff/recovery admission, analytic eased paths, energy reservation and save validation. Splitting that state/order would scatter its invariants; related safety/partition tests account for most length. No flight source or arithmetic changed. |

The same history window is retained for comparisons. Moving functions changes
source paths but does not demonstrate fewer future conflicts. Revisit retained
owners when new function-level responsibility or test-isolation evidence exists,
not when an arbitrary length threshold is crossed.

The [body comparison](body-equality.json) confirms all 12 moved physics free
functions, three response methods and both geometry functions retain their
whitespace-normalized bodies. Only module/private visibility/import routing
changes. Public exports, serialized contact field order/shape, thresholds,
sixteen-iteration/fixed-point rules, traversal, allocations, shaders/assets and
7-float vertex/index layout remain unchanged. Physics/save/numerical regression
checks and real GPU paths establish behavior beyond the textual comparison.

## Verification

Linux x86_64, Rust 1.99.1; same prepared workspace toolchain/library environment
as #643. [Environment](environment.txt) records compiler, CPU quota and memory.

- `cargo test --workspace --locked`: **1,031 passed, 38 existing opt-in ignored**, zero failures. The retained exact-response/warm-continuation oracle passed.
- `cargo clippy --workspace --all-targets --locked -- -D warnings`: passed.
- `cargo build --workspace --locked`, `cargo fmt --all -- --check`: passed.
- `cargo test -p salimon-renderer --locked headless_resource -- --ignored`: **3 passed**, actual software Vulkan raster parity, pipeline and sphere-palette checks.
- `cargo test -p salimon-renderer --locked headless_five_hundred_shared_spheres -- --ignored`: **1 passed**, actual GPU readback, cached uploads and repeat pixels.
- `python3 -m unittest discover -s scripts -p 'test_*.py'`: quiet final rerun **129 passed**.
- `python3 scripts/check_docs.py --base codex/issue-643-renderer-composition`: zero failures.
- `python3 scripts/architecture_review.py --base codex/issue-643-renderer-composition --output artifacts/architecture/644-review.json`: passed; only the two retained broad-file prompts, no new directions/cycles. Active-PR inventory was unavailable in this local invocation.
- `git diff --check`: passed.
- Production resource-deposit route: **68/68 steps passed**, actual Vulkan surface capture at 1280×800 scale 1, with fixed-step protocol controls. [Evidence](evidence.json) records original PNG/hash, binary and context; the [compressed original result](resources/result.json.gz) retains every observation.

Native command:
`DISPLAY=:91 SALIMON_E2E_ISOLATE_INPUT=1 python3 scripts/salimon-test run scenarios/evidence/resource-deposits.json --binary /tmp/salimon-644-client --artifacts artifacts/644-resources --screenshot-command '["native"]' --summary`.
Set the prepared Mesa ICD/library environment described in #643 and the
[graphical validation guide](../../docs/validation.md#native-graphical-gates).
Actual test/build/lint/GPU/tooling logs are retained beside this report.

## Serial release CPU comparison

Built `cargo build -p salimon-physics --release --example tick_cost --locked`
on the parent and extracted source; retained both prebuilt executables. After
all builds, tests and rendering finished, ran:
`python3 reports/issue-644/compare_ticks.py /tmp/salimon-644-parent-tick-cost /tmp/salimon-644-current-tick-cost --output reports/issue-644/timing`.
The initial exploratory parent run during other work is excluded. The final
three paired runs alternate execution order and run serially, with no compilation,
rendering or competing measurement lane. Each row has 100 admitted measured
samples after ten warmups; CSVs retain the existing workload's median/p95/p99
summaries (not individual samples). [Summary and binary hashes](timing/summary.json)
retain all comparisons and verify identical substep/integrated/sleep counts.

| Objects / workload | Parent median ms | Current median ms | Ratio |
| --- | ---: | ---: | ---: |
| 500 settled | 0.082992 | 0.082771 | 0.997 |
| 500 fast drop | 6.616715 | 6.668076 | 1.008 |
| 500 active | 12.254814 | 12.315879 | 1.005 |
| 1,000 settled | 0.181522 | 0.180276 | 0.993 |
| 1,000 fast drop | 13.698208 | 13.772548 | 1.005 |
| 1,000 active | 25.882981 | 26.061299 | 1.007 |

Values are medians across the three run medians; p95/p99 summaries remain in the
CSVs. Changes are under 1% in these medians, with retained noisy tails. This is a
bounded identical-snapshot CPU comparison, not the historical moving contact
stress corpus, an end-to-end frame-budget pass or hardware approval. In particular,
1,000 active objects remain above 16 ms; that limitation is preserved.

Initial tooling run: 128/129 Python tests passed; the existing capture-helper
invalid-PNG fixture hit its 100 ms subprocess timeout during concurrent Rust
compilation rather than reaching PNG validation. [Original failure](initial-python.log)
is retained; the suite is rerun after compilation. The first extraction lint
found an unused import, removed before final verification. No threshold or
production policy was loosened to hide either failure.

## Limits and retained follow-up

Cloud software-Vulkan output is not actual Metal, hardware FPS or subjective
appearance signoff. Existing [#561](https://github.com/mr-exception/salimon/issues/561)
owns appearance, [#557](https://github.com/mr-exception/salimon/issues/557) physical
platform checks, and [#563](https://github.com/mr-exception/salimon/issues/563)
hardware budgets. The historical #592 contact-stress failures/limits are not
reclassified by this source extraction or the bounded tick-cost comparison.
No new tolerance/budget, baseline promotion, numerical optimization, asset change
or broad formatting pass is introduced. No merge performed by Codex.

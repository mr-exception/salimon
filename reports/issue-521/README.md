# #521 — Repeated scientific reference and partition cloud workloads

Fifth delivery in the requested stack, after [PR #588](https://github.com/mr-exception/salimon/pull/588).
Release reference and partition workloads pass on a reproducible Linux cloud
host. Seven fresh processes per workload retain all JSON/timings and independently
observed peak RSS. Fixture hashes and partition record/raw/pack sizes match the
retained Linux evidence. No scientific model-error or hardware budget is changed.

## Environment and measurement

[Context, exact revision/binary/fixture hashes](evidence/context.json),
[compiler/OS/C compiler](environment.txt), [release build](build-release.log),
[raw results](evidence/) and [all distributions/comparisons](evidence/summary.json)
identify source `d8ce33a`, rustc 1.99.1, Linux x86_64, AMD EPYC 9V74 80-Core host.
This executor has a **2-CPU cgroup quota and 8 GiB memory limit**; the host CPU
name/count does not mean dedicated access. Existing inherited toolchain/runtime
configuration is preserved. No graphics/backend/display/input is used or needed.

Examples are built once with the locked release profile, then run **serially**
with no concurrent local compilation, tests or graphical routes. Each reference
process validates five frozen cases and records seven samples × 1,000 iterations
for both the production-U02/frozen-source adapter and strict comparator: **49 raw
samples per workload across seven processes**. Each partition process runs the
unchanged 1/32/512-cell workload, yielding 4/128/2,048 records and identical
raw/packed byte counts. Cells beyond the first are explicitly synthetic replicas;
no full catalog capacity/performance forecast is approved.

The [Python driver](measure.py) and [small C supervisor](measure_rss.c) preserve
per-child monotonic launch-to-exit wall time, user/system CPU, Linux peak RSS in
KiB and voluntary/involuntary context switches. Inner benchmark timing excludes
process launch but retains its documented parse/allocation/setup boundaries.
Process RSS is a whole-executable high-water measurement, not per-cell live bytes,
GPU allocation or dedicated host RAM consumption. Cgroup/load snapshots and all
spikes are retained. Seven processes are a bounded distribution, not tail or FPS
certification.

## Matched Linux comparison

Historical values are retained unchanged in [#404](../issue-404/README.md) and
[#405](../issue-405/README.md): EPYC 9V45/rustc 1.99.1 on a different shared Linux
executor. Reference historical medians use seven samples; current medians pool
49 equal-size samples. Partition historical values are single-run diagnostics,
compared with seven current runs. Ratios are observations across hosts, not a
same-machine regression verdict or a performance acceptance threshold.

| Workload | Prior Linux ms | Current median ms | Current min–max ms | Ratio |
| --- | ---: | ---: | ---: | ---: |
| U02 + frozen-source adapter, 1,000 iterations | 4.4059 | 6.6032 | 6.2182–7.6930 | 1.50× |
| Strict comparator, 1,000 iterations | 0.1981 | 0.3325 | 0.3211–0.4603 | 1.68× |
| 1-cell encode, 4 records | 0.1298 | 0.1908 | 0.1805–0.1988 | 1.47× |
| 32-cell encode, 128 records | 0.7825 | 1.6724 | 1.5412–1.9173 | 2.14× |
| 512-cell encode, 2,048 records | 18.9147 | 33.2585 | 32.4079–35.0642 | 1.76× |

The raw summary also compares every index-open and reverse-read size; none is
omitted because it is slower. Historical fixture hashes all match. The
[workload source diff](workload-source-diff.txt) is empty against the merged U03
commit `b3f00c4` for comparator/adapter/frame/epoch/partition workload owners and
frozen reference/storage fixtures. The original pre-merge `adbe687` SHA is absent
locally after squash; `b3f00c4` is the retained merged delivery. No code was tuned
to force historical timing parity. Host/quota/scheduling and different source
context prevent attributing the timing difference to a game regression.

| Whole executable | Wall median (min–max) ms | RSS median (min–max) KiB |
| --- | ---: | ---: |
| Reference benchmark | 50.5205 (48.6034–51.2871) | 1,528 (1,496–1,580) |
| Partition benchmark | 51.2710 (49.8783–56.7128) | 4,812 (4,788–4,848) |

There is no comparable historical peak-RSS distribution; no memory-ratio claim is
made. Min/max/medians for every process metric and every raw sample remain in
`evidence/summary.json` and individual measurement JSONs.

## Comparator and observer controls

[Emitted candidates](candidates.json) run through the strict production comparator:
[all five cases pass](comparison-pass.json). A separately mutated Earth X sample
using kilometres as metres [fails](comparison-km-fault.json) with the required
[exit 1](comparison-km-fault.exit). Goldens/tolerances are never regenerated or
loosened. Fixture generation is demonstration input, not an independent solver.

The first measurement implementation launched children directly from Python and
reported the parent's inherited ~14,808 KiB RSS floor for both tiny executables.
[Those original runs](rejected-python-rss/) are preserved and **rejected as memory
evidence**, not silently relabeled. The final small C parent fixes the observation
boundary. An independent [32 MiB touched-page control](rss_control.c) reports
[33,152 KiB](rss-control.json), while a deliberately failing child returns
[exit 7](exit-control.json) through the supervisor. Both controls pass their
expected outcomes; they are not workload failures. Final commands all exit zero
except the two deliberately required nonzero controls.

## Reproduction and validation

From repository root; source `/workspace/.salimon-tools/env.sh` on this executor.
No new network data or graphics dependencies are needed after the locked build.
Use a new measurement directory so original raw samples are preserved.

```sh
cargo build --release --locked -p salimon-universe --example reference_bench --example reference_compare --example partition_bench
cargo test --locked -p salimon-universe
cargo clippy --locked -p salimon-universe --all-targets -- -D warnings
mkdir -p artifacts
cc -std=c11 -O2 -Wall -Wextra -Werror reports/issue-521/measure_rss.c -o artifacts/521-measure-rss
python3 reports/issue-521/measure.py --output artifacts/new-521 --repeats 7
# Optional observer admission check (expect approximately 32 MiB peak RSS):
cc -std=c11 -O2 -Wall -Wextra -Werror reports/issue-521/rss_control.c -o artifacts/521-rss-control
artifacts/521-measure-rss artifacts/new-521-rss-control.json "$PWD/artifacts/521-rss-control"
target/release/examples/reference_bench --emit-samples > artifacts/new-521-candidates.json
target/release/examples/reference_compare artifacts/new-521-candidates.json
python3 scripts/check_docs.py
python3 scripts/check_docs.py --base codex/issue-515-recovery-cloud
```

Release build, **69 universe tests**, universe all-target Clippy, strict passing
comparison, expected fault rejection, 14 serial measurement processes, C controls,
Python syntax, formatting, docs/source-impact and whitespace checks pass. No
runtime/gameplay test is newly claimed for this report-only measurement change;
the preceding stack retains its own native and workspace evidence.

## Documentation impact and remaining criteria

Updated [reference workload guide](../../docs/universe-reference-fixtures.md),
validation, maintenance map and [owning architecture](../../client/universe/architecture.md)
link actual current evidence and observer usage. [Universe invariants](../../client/universe/invariants.md)
and the [partition contract](../../docs/universe-storage.md) were checked and
remain unchanged: no algorithm/schema/frame/epoch/precision/budget or runtime
integration changes. Source/test mapping remains `reference.rs`, its tests,
`examples/reference_{bench,compare}.rs`, `support/reference_inputs.rs`,
`partitions.rs`, its tests and `examples/partition_bench.rs`. The required U35
same-PR documentation gate applies; this is measured cloud evidence, not a future
Universe implementation or model approval.

[#563](https://github.com/mr-exception/salimon/issues/563) receives optional
matched-hardware release CPU/RSS/whole-frame and GPU measurements, threshold and
hardware-budget decisions. Actual M1/Metal/GPU counters/frame rate remain
unrun/unavailable. No desktop is needed to establish these Linux CPU results.
Future #415–#420 integrators must still pass independent references and separately
approved model-error/validity budgets; U03 rounding tolerances grant none.
Full-game cloud/flight/render/restore gates remain #435–#439, with human
presentation/physical-device review #561/#557 and source permissions #537.

# #559 — Exact current-sphere grid-query stress

A focused broad-phase change deduplicates bucket membership with a retained
per-body epoch before exact AABB filtering and sorting. Grid width uses the
upper-quartile radius to reduce bucket enumeration for ordinary larger mixed
members; extreme outliers retain the existing exact fallback. Both choices
change candidate enumeration only. Ordered responses, corrections and their
requeries, sixteen passes, all elapsed time, damping, contact/sleep/mass rules
and population sizes remain unchanged. Epoch wrap is explicitly tested.

## Exact input and comparison boundary

The #538 cabin changes which fixture columns are supported. The matched lanes
therefore use detached pre-cabin geometry `2ecb271ff4af43ad102367cdea91c8cb08e375ec`,
whose character layout and benchmark fixture match retained #455 `ccbc2fda`.
The only runtime addition is the identical ignored measurement harness; both
physics builds contain identical per-query timing instrumentation, and only
the after grid has the optimization. No gameplay geometry is reverted in the
PR stack. See [reproduce.sh](reproduce.sh) and [collect.py](collect.py).

The initial production-adapter attempt revealed a few 1-ULP platform libm
radius differences. Those complete/failed/timed-out records remain separately
in `initial-adapter`. Final solver replay obtains actual production adapter
snapshots and restores archived mass/radius **bits** from the #455 inventory
before stepping the production stateful solver. IDs, resources, initial local
positions, velocities and original historical support geometry remain explicit.
The 18-entry archived cycle and its source hash are checked in under
`client/runtime/src/sphere_workloads/historical-inputs.json`. This is exact-input
CPU solver evidence, not a native foreground/frame measurement or a full
production-adapter cost comparison. The older adapter attempt is labelled as
such, not silently treated as bit-identical input.

Moving pools retain at least 100 actual moving intervals, restarting the same
fixture when it becomes quiet; quiet rows remain raw. Natural preparations use
production steps, at most 300, with no installed sleep flags. Each completed
step is flushed to JSONL, so every subprocess timeout retains its partial work.
Query time is included in contact time, not added to it. Timing instrumentation
and JSONL I/O between calls remain declared overhead. Timings are serial, with
no concurrent builds/render workloads. CPU software results do not establish
Metal behavior, hardware FPS or physical scanout.

## Historical failures preserved

[#455](../issue-455/README.md) retains 1,000-mixed moving 120s and natural 45s
timeouts, 492/500 asleep after 300 steps, 500-cap startup frame-tail relative
failure, and cap sleeping frame p95 above 17.5ms guidance. Before facing
controls also fail that guidance. These are historical failures, not passes
or attributed fixes. The new CPU measurements cannot repair historical native
frame cadence evidence. #563 owns actual hardware stress/budget decisions;
#557 owns physical platform checks. No population limit is imposed or approved.

## Results and verification

Final measured results, full raw records and checked revision are appended
below. A process completing its harness is distinct from natural-sleep success.

## Documentation impact

Physics architecture/invariants and runtime observation architecture describe
the search-only change, timing subset and exact-input boundary. Validation links
the serial runner. Supporting domain ownership and Universe U01–U34 behavior
are unchanged. Tests map the query implementation to exact all-pairs states,
correction-created contacts, refreshed bounds and explicit epoch rollover.

## Serial cloud measurements

Checked source `f804fc4`, Rust1.90.0 Linux x86_64 release. Exact binary/source hashes, CPU quota and frozen source patch: [environment.json](environment.json), [fixture-source.patch](fixture-source.patch). The checked root and compiled fixture harness hashes match exactly. No GPU/rendering is involved.

| Moving pool | Samples before/after | Total p95 ms before/after | Query p95 ms before/after |
| --- | ---: | ---: | ---: |
| 500-cap-moving | 100/100 | 10.239/9.792 | 0.897/0.817 |
| 500-mixed-moving | 100/100 | 198.014/88.572 | 108.601/12.984 |
| 1000-cap-moving | 100/100 | 65.465/68.634 | 5.512/7.386 |
| 1000-mixed-moving | 33/44 | 16353.351/7609.431 | 11366.553/868.446 |

The 500-mixed query p95 falls 88%, with total solver p95 down 55% in this
single serial comparison. This is not a universal improvement: 1,000-cap total
and query p95 increase; raw spikes and every sample remain in [before](before/)
and [after](after/). The 1,000-mixed rows are **partial 120s timeouts**, below
100 samples, so their table values describe incomplete traces and are not
accepted full-pool distributions. There is no 60FPS or hardware-budget pass.

Natural cap controls both sleep fully, then retain 100 samples. Their total
p95 before/after is 0.141/0.117ms (500) and 0.345/0.487ms (1,000). Do not
attribute the latter activation-only change to query work. Natural 500-mixed
fails identically at **492/500 after300 steps**. Natural 1,000-mixed times out
at45s in both lanes (19/25 completed preparation steps); no sleeping distribution
exists. Initial adapter attempts retain their separate timeouts, including
500-mixed stopping at296 steps with492 asleep.

[Exact-prefix comparison](comparison.json) verifies all overlapping population
records, all deterministic per-step counters, and every completed terminal
state. The archived500-mixed identity/resource/source and effective mass/radius
bits match exactly. Prefixes include 33 moving and19 preparation steps in the
incomplete1,000-mixed lanes; no missing suffix is invented.

Remaining automatic contact-solver/100-sample/preparation criteria transfer to
[#592](https://github.com/mr-exception/salimon/issues/592), with the exact replay
and no rule weakening. Hardware frame/cadence and stress-budget decisions
remain #563; physical-platform checks #557. Closing #559 through review records
this focused delivery and explicit transfer, not success on the failed stress
criteria. No owner budget, baseline or population cap is approved.

The first stacked-source verification reused the historical checkout's Cargo
target directory and failed to resolve the newer placement module from a stale
ship artifact. That failed log is retained. Final verification uses a new,
isolated `CARGO_TARGET_DIR=/workspace/salimon/target-stack`; no stale-artifact
failure is waived. Timing binaries were built serially in the historical
checkout, with identical dependencies in both lanes; this cache failure happened
later when switching the target back to the newer stacked source.

Final isolated gates PASS: `cargo test --release --workspace --locked` —
**895 passed, 0 failed, 32 opt-in ignored**; `cargo clippy --workspace --all-targets
--locked -- -D warnings`; debug and release workspace builds. Raw logs are in
[validation](validation/). The new exact sorted-bounds/duplicate-bucket/epoch-wrap
regression passes alongside all-pairs, correction-created, activation and runtime
continuation tests. Formatting and patch checks pass. The ignored stress harness
was run separately as documented, with its failed outcomes retained; passing
ordinary tests do not convert those into passes.

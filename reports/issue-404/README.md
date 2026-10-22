# U03 — scientific reference fixtures and profiling (#404)

Portable delivery on 2026-10-22, stacked after [U04 PR #520](https://github.com/mr-exception/salimon/pull/520)
(commit `0791e96`). Merge #520 first. [Reference contract](../../docs/universe-reference-fixtures.md),
[architecture](../../client/universe/architecture.md#scientific-reference-harness--u03-404),
[invariants](../../client/universe/invariants.md).

Delivered a strict bounded reference/candidate wire contract, immutable Suite,
complete exact-ID/frame/epoch/scale comparison and Euclidean SI position/velocity
error reports. An offline CLI requires candidate data and exits nonzero for any
failed case or malformed input. Missing/duplicate/extra coverage and nonfinite or
overflowing values cannot silently pass. Expected values are never updated by
candidate output.

## Golden evidence and budget boundary

Five frozen cases: two DE441 Horizons Earth399 barycentric geometric ICRF vectors
at 2000-Jan-01/02 midnight TDB; exact rotating/transport frame arithmetic;
submillimeter camera-relative GPU narrowing at million-cell anchor; and two-day
constant space motion with split galactic subtraction. Goldens are independently
transcribed SI vectors/analytic results. The test/example adapter parses original
KM-S data and runs production U02 math; it is not a U08 importer or orbital solver.
[Fixture hashes](fixture-sha256.txt) pin references and source context.

[CLI passing result](comparison-pass.json) passes all five cases: zero position
errors, maximum velocity conversion difference 3.64e-12 m/s. A separately mutated
Earth x coordinate divided by 1000 is [rejected](comparison-km-fault.json), with
[exit status 1](cli-fault-result.txt). Unit regressions also reject swapped axes,
wrong frame/epoch/scale, velocity errors, lost/extra/duplicate cases, NaN, overflow,
invalid budgets and unsupported/malformed wire. A 3-4-5 norm boundary passes;
(4,4,0) fails a 5 m norm budget even though each component is below 5 m.

Per-regime baseline tolerances retain their basis in each case: local/distant
cell-relative results allow 0.00025 m (rounded outward from sqrt(3) times worst
U02 local f64 spacing); JPL conversion allows 0.00025 m / 1e-10 m/s for printed
rounding and SI/f64 conversion; small integer/dyadic arithmetic uses exact or
1e-12 regression envelopes. These do not assert DE441 covariance/absolute truth
or approve future integrator/model drift. Future model/validity policies remain
#415–#420. No real orbital integration or flight-solver pass is claimed.

## Linux CPU diagnostics

[Environment](environment.txt): shared Linux x86_64 executor, AMD EPYC 9V45 host,
rustc 1.99.1, release profile; visible memory is executor context, not dedicated
physical machine sizing. [Raw seven-sample workload output](reference-bench.json)
retains all elapsed times, min/median/max and preflight errors.

| Workload | Iterations/sample | Median elapsed | Min/max |
| --- | --- | --- | --- |
| Production U02 + frozen-source adapter (parse/allocation included) | 1000 | 4.406 ms | 4.366/4.523 ms |
| Strict five-case comparator | 1000 | 0.198 ms | 0.198/0.227 ms |

The [U04 report](../issue-405/README.md) records the preceding same-environment
1/32/512-cell storage diagnostic workload; no unchanged measurements were repeated
as a new native claim. Reference fixture/loop overhead is measured, not whole-game
render/physics work. No FPS threshold or statistically supported tail guarantee
is installed by these tiny CPU workloads.

## Validation

- `cargo test -p salimon-universe --locked`: 30 pass.
- `cargo run -p salimon-universe --example reference_bench --release --locked`:
  pass; seven raw timing samples/workload retained above.
- `cargo run -p salimon-universe --example reference_bench --release --locked -- --emit-samples`:
  pass; demonstration candidates generated from production U02/frozen-source adapter.
- `cargo run -p salimon-universe --example reference_compare --release --locked -- artifacts/reference-samples.json`:
  pass; injected km-as-m CLI invocation fails as required.
- `cargo build --workspace --locked`: pass.
- `cargo test --workspace --locked`: 810 pass, 0 fail, 28 existing opt-in checks
  ignored (native/GPU/performance prerequisites remain explicit).
- `cargo clippy --workspace --all-targets --locked -- -D warnings`: pass.
- `cargo fmt --all -- --check`, `git diff --check`: pass.
- `python3 scripts/check_docs.py`: 0 failures; source-impact check against the
  preceding stack branch after commit: 0 failures.

## Documentation and remaining acceptance

Behavior/source/test mapping is in the reference contract and owning architecture.
Crate README/AI/architecture/invariants, root routing, validation and approved-design
status are updated together. No runtime, world, persistence or gameplay contract
is changed. Production source reuse remains #513.

**Reference Apple M1 profiling was not run**: this environment is Linux. Exact
remaining workload/context/raw-evidence acceptance is transferred to
[native follow-up #521](https://github.com/mr-exception/salimon/issues/521), which
must land after this delivery and remains part of future integrated native review.
Closing #404 through its PR records portable delivery and tracked scope transfer,
not M1 approval. Full-game/render/physical-flight gates remain #435–#439. No GPU
launch, playback, backend, catalog publication or native 60 FPS signoff is claimed.

# Scientific reference fixtures and CPU workloads — U03 (#404)

`salimon-universe::reference` is a portable comparison harness, not an ephemeris,
gravity/trajectory integrator or runtime navigation service. The frozen
[version-1 suite](../client/universe/fixtures/references-v1.json) combines actual
JPL reference vectors with independent analytic coordinate/time cases. It supports
future solver evaluation without pretending a solver already exists.

## References and baseline numerical budgets

| Regime / case | Golden reference | Position / velocity norm allowance |
| --- | --- | --- |
| Local contact / rotating frame | Exact 90-degree axes, integer translation and omega-cross-radius transport | 1e-12 m / 1e-12 m/s, finite arithmetic regression envelope |
| Local contact / distant camera | Million U02 cells, local x offset 2^39-1 m, exact 2^-12 m displacement through GPU narrowing | 0.00025 m / exact zero velocity |
| System flight / Earth, two dates | DE441 Horizons Earth399, Solar System barycenter 500@0, geometric ICRF, TDB, 2000-Jan-01/02 midnight, KM-S | 0.00025 m / 1e-10 m/s, printed decimal/SI/f64 conversion envelope |
| Interstellar / linear space motion | Million-cell anchor, (3,-4,12)m/s for exactly two TDB days, compared relative to original anchor | 0.00025 m / 1e-12 m/s |

U02 worst cell-local f64 spacing is 2^-13 m. A 3-axis spacing norm is below
0.000212 m; 0.00025 m rounds that outward. It is a bound on the nearby relative
result after distant-anchor cancellation, **not** on flattening an interstellar
position into global f64. The JPL printed coordinate resolution and SI conversion
are bounded by the same position allowance at these magnitudes; velocity allowance
covers decimal/f64 conversion. These are test-level numerical baselines, not
DE441 covariance, real-world body-position certainty or approved future orbital
drift/contact/gameplay tolerances. No mass/gravity/orbit model is inferred.
U14–U19 (#415–#420) must supply independent solver/model accuracy and validity
policies when implemented. Missing radial motion or measured uncertainty must
never be fabricated from these fixtures.

JPL source bytes, query/frame/epoch/model and SHA256 are frozen in the
[source manifest](../catalogs/sources/manifest-v1.json) and described by the
[source contract](universe-sources.md). Goldens are separately transcribed SI
vectors; the example/test adapter parses the frozen KM-S rows and evaluates
production U02 transforms, GPU narrowing and split epoch motion against them.
That adapter recognizes only these two epochs/context; it is separate from the [U08 scientific fixture importer](universe-observed-ingestion.md).
U08 tests compare its production adapter directly to these independent goldens. The two rows are reference orbital states, not an orbital
integration pass. Production reuse approval remains #513.

## Comparator wire and failure behavior

A strict <=4 MiB `salimon.universe.references` schema-1 JSON envelope contains
1–4096 cases. Each has an `expected` Sample, `regime`, `budget`, `budget_basis` and
`reference`. Samples contain exact `id`, `frame`, validated split `epoch`, three
SI `position_meters` and `velocity_mps` values. Budgets are finite/nonnegative
Euclidean norm limits in meters and meters/second; every state component must be
finite. The immutable Suite sorts exact case IDs and rejects duplicates, empty
metadata, unsupported versions/unknown or duplicate fields and invalid epochs.

Candidate envelope: `salimon.universe.samples`, schema version 1, `samples` array
with the same Sample shape. Every reference must appear exactly once. Extra,
missing or duplicate IDs, different frames/epochs/time scales and nonfinite or
overflowing differences are errors, never reduced coverage or implicit transforms.
Comparison returns every case's norm errors, budget and pass/fail in sorted order.
Tolerance boundaries are inclusive; a 3D norm cannot pass merely because each
component is below the limit. CLI exits nonzero on either malformed data or any
failed tolerance while retaining comparison JSON for numerical failures.

## Reproduce portable evidence

From the repository root, after the standard native Rust build prerequisites:

```sh
cargo test -p salimon-universe --locked
cargo run -p salimon-universe --example reference_bench --release --locked
cargo run -p salimon-universe --example reference_bench --release --locked -- --emit-samples > artifacts/reference-samples.json
cargo run -p salimon-universe --example reference_compare --release --locked -- artifacts/reference-samples.json
cargo run -p salimon-universe --example partition_bench --release --locked
```

Create `artifacts/` first if absent. `--emit-samples` exports production-U02 and
frozen-source-adapter results for demonstration; it is not solver output or an
independent science pass. For actual solver validation supply that solver's full
candidate envelope to `reference_compare`; expected values are never regenerated
or replaced by the comparator. Unknown arguments reject.

The reference CPU benchmark reports seven raw samples, each 1000 iterations, and
min/median/max for (1) production U02 plus frozen-source parsing/allocation and
(2) strict comparison. It validates a preflight, uses black_box and records
OS/architecture/build profile. The [U04 workload](universe-storage.md) adds
1/32/512-cell encode/index-open/reverse-read diagnostics. Retain exact revision,
rustc/OS/CPU/context and fixture hashes beside raw JSON; repeat on matched hardware
before interpreting a change. No timing thresholds, invented FPS or extrapolated
native speed are installed by these CPU harnesses.

[Portable report](../reports/issue-404/README.md) preserves Linux output and injected
failure evidence. [Cloud measurements #521](../reports/issue-521/README.md)
retain seven serial release processes per workload, raw timings, peak RSS,
fixture/binary hashes and matched-size historical Linux comparisons. The report's
`measure.py` uses its small C `measure_rss.c` supervisor to avoid inheriting the
Python parent's peak-RSS floor; reproduction and observer controls are included.
Actual M1 measurements remain unrun
unless separately collected. Hardware budget decisions belong to #563 under the
[verification workflow](verification-workflow.md). Whole-game cloud rendering and
flight/restore integration remain #435–#439; human presentation, physical platform
and hardware FPS qualification remain #561/#557/#563. These workloads do
not depend on graphics, host wall clocks, production data downloads or a backend.

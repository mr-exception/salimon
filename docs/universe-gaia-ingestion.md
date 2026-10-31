# Gaia DR3 ingestion — U07 (#408)

`client/universe/src/ingestion/gaia.rs` accepts local Gaia DR3 CSV through a
`BufRead` boundary. Source bytes must first pass the [U06 manifest/hash and reuse
gates](universe-sources.md). The adapter does no downloads and confers no production
reuse approval. #513 still owns approved full snapshots and source permissions.
The two pinned Gaia rows are **synthetic**, not measured Gaia science evidence.

## Selection and measurement contract

`stream` reads one CSV record at a time (1 MiB maximum, UTF-8, strict column widths,
unique required headers, RFC4180 quoting/newlines). Unknown columns survive in
`source_fields`. IDs are positive exact decimal u64 strings in `gaia_dr3`; no float
conversion or name-based identity. Invalid numeric syntax, nonfinite values,
negative/orphan errors, invalid flags and malformed CSV stop the operation.
Consumers must discard partial sink output on error.

The explicit `QualityPolicy` defaults to RUWE <=1.4 and rejection of
`duplicated_source=true`. This is a reproducible development selection, not a
claim that a universal scientific quality cut has been approved. Missing RUWE is
retained and counted separately. Missing RA/Dec/reference epoch, RA outside
[0,360), Dec outside [-90,90], or epoch outside [1,10000) reject that row and are
counted. Required headers match the U06 snapshot; errors and quality columns not
in that snapshot are optional. A source without parallax remains usable as a sky
observation; no 3D location is fabricated.

Gaia `ra_error`/`dec_error` are milliarcseconds even though RA/Dec values are
degrees. The typed `ra_error_mas`/`dec_error_mas` fields keep source errors
separate from the degree-valued coordinates, without inferring scalar RA-angle
covariance or transforming Gaia's astrometric error convention. Raw columns remain intact.
Typed observations retain RA/Dec in degrees, parallax and its standard error in
mas, pmra (including cos(dec))/pmdec in mas/year, radial velocity in km/s, their
optional published standard errors, and the Julian reference year. These source
units are explicit; there is no implicit ICRF-to-galactic rotation, epoch-scale
conversion, missing-motion zero, or inverse-parallax distance. Negative parallax
and incomplete velocities remain observed values/nulls. Distance inference is
U09, frame/space-motion composition is U02/U14. All original columns and exact
source record keys remain with dataset-level release/reference provenance.

## Canonical matching and publication

`StellarCrossMatch` accepts caller-reviewed exact source-to-canonical IDs, rejects
ambiguous assignments, and otherwise retains the Gaia ID. No angular/name
heuristic or arbitrary nearest neighbor is implemented. `import` deduplicates
identical selected source rows, refuses conflicting repeats, sorts by source ID,
and retains **all** measurements for many-to-one matches without averaging or
silently preferring one. Catalog bytes and observation order are input-order
independent. Selection summaries distinguish input, selected, quality/astrometry
rejected, missing quality/parallax/motion, repeated and canonical counts.

The collector publishes validated schema-1 systems/stars with unknown mass/radius;
its separate typed observation sidecar carries astrometry and cross-match source
IDs. A system's namespace is `system_` plus the canonical star namespace, with the
same exact key; ID-length/global collisions fail catalog validation. This simple
single-star grouping is not a multiple-star hierarchy solver. The decoder is
streaming; the convenience collector stores selected rows/IDs in memory and is
intended for bounded partition-builder batches. It is not a full-Gaia in-memory
builder. U04 currently serializes only canonical catalog fields, **not this
sidecar**; integrating astrometry into the production U12 pack remains required.
Neither the schema-1 wire nor current playable world/saves change.

## Checks and resource evidence

```sh
cargo test -p salimon-universe --locked
cargo run -p salimon-universe --example gaia_bench --release --locked
python scripts/check_docs.py
```

Owning regressions in `src/ingestion/gaia/tests.rs` cover exact golden IDs/errors,
missing data/negative parallax, quality summaries, reviewed matches, input order,
duplicate conflict and malformed input/sink failures. They run in normal workspace
PR CI. The shared CSV tests cover escaped quotes, quoted newlines, malformed rows
and record-size bounds. No new dependency or runtime/network path is added.

`gaia_bench` reports ingestion throughput and JSON/zlib6 bytes per synthetic star
including system/star records, all astrometry, raw columns and provenance. The
2/10,000-row workloads vary IDs/sky positions; their repetitive numeric values
are a compression sensitivity, not a representative real Gaia distribution.
The conditional 1.8-billion-object projection and 5/10 GB capacities are explicit
arithmetic at that density. They exclude extra quality columns, partition/index
costs and nonstellar content. Do not use them to approve the 2–5/10 GB pack target;
full reviewed data, release-specific quality selection and actual U12 codec
measurements remain necessary under [#524](https://github.com/mr-exception/salimon/issues/524). [Report](https://github.com/mr-exception/salimon/blob/d003bd58d506e04e3df97744f29f68449b6e2b3e/reports/issue-408/README.md) records
measured results and follow-up ownership. Native integration remains U34 and the
reference hardware gates; this offline ingestion does not establish native signoff.

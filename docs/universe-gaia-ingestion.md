# Gaia DR3 ingestion — U07 (#408)

`client/universe/src/ingestion/gaia.rs` accepts local Gaia DR3 CSV through a
`BufRead` boundary. Source bytes must first pass the [U06 manifest/hash and reuse
gates](universe-sources.md). The adapter does no downloads and confers no production
reuse approval. Private acquisition/use is owner-authorized by #524 (2026-10-27); #659 owns future public/commercial source review.
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
measurements remain necessary for a future full-Gaia claim. The current
[nearby catalog](nearby-celestial-catalog.md) is #524's explicitly bounded scope. [Report](https://github.com/mr-exception/salimon/blob/d003bd58d506e04e3df97744f29f68449b6e2b3e/reports/issue-408/README.md) records
measured results and follow-up ownership. Native integration remains U34 and the
reference hardware gates; this offline ingestion does not establish native signoff.


## Real DR3 verification tooling (#524)

`python scripts/acquire_gaia_sources.py --output NEW_DIRECTORY` freezes an exact
8,192-row `random_index` sample and Proxima/Barnard golden IDs from AIP's DR3 TAP
mirror. Original VOTable, units/metadata, queries, hashes and a string-preserving
CSV transcription remain together. TAP errors/OVERFLOW, incomplete counts,
duplicate/changed IDs and unexpected fields reject. The scope is explicitly a
measurement sample, not a full-release selection. Public redistribution status
remains unreviewed; private authorization does not change provider terms.

`python scripts/verify_gaia_sources.py --sources catalogs/sources/gaia-524-v1 --ps catalogs/sources/observed-526-v1/ps.csv` independently checks hashes, original
TABLEDATA/CSV equality, the complete random-index interval and exact DR3 host
associations against the frozen NASA PS snapshot. Published epoch/parallax/error
goldens are checked without nearest-neighbor or name-only matching. Ambiguous
caller associations still fail U07 validation.

`cargo run -p salimon-universe --example gaia_verify --locked -- catalogs/sources/gaia-524-v1 artifacts/gaia-measurement` runs the actual offline U07/U12
adapters and measures the installed catalog partitions, source/astrometry
sidecars, literal-ID manifest index, five original decode samples and source-field
error/coverage distributions. Every selected original column survives the sidecar.
Pack v2 compresses sidecars losslessly; `gaia_verify` additionally records encoded
and decoded sizes, decoded SHA256s and five original read/verify/inflate timings.
The output measures the production storage/reader route, not a new scientific
selection or changed precision. Exact source strings and missing values remain
intact. See [pack compatibility and bounds](universe-pack.md).
The earlier all-usable-DR3 request was replaced on 2026-11-01 by the owner's
[1 GB nearby selection](nearby-celestial-catalog.md). Neither the old random
sample nor the new regional publication establishes full-Gaia coverage or the
historical 2–5/10 GB galaxy budget. No scientific precision is reduced for size.

## Bounded nearby publication (2026-11-01)

[Coverage, local data location and expansion](nearby-celestial-catalog.md) define
the current #524 scope. `acquire_nearby_sources.py` freezes the complete all-sky
parallax >=10 mas query in 8192-row `(parallax DESC, source_id ASC)` pages, with
initial/final counts and an empty tail. Original XML and CSV use deterministic
`votable_gzip`/`csv_gzip` archives; source hashes bind stored bytes, and original
response hashes bind inflated bytes. `verify_nearby_sources.py` checks release
units/datatypes, every original CSV string, query/boundary/continuation semantics,
exact golden associations and the locally selected PS/JPL companions. With
`--pack`, it compares every published Gaia/planet source field and checks the
combined frozen-source/installed-pack 1 GB budget. Rejected Gaia rows remain
frozen inputs. `nearby_verify` exercises the production offline builder and
filesystem reader, all cell/sidecar decodes and exact-ID goldens in release mode.

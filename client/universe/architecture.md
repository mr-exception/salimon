# Canonical universe schema — U01 (#402)

`salimon-universe` is a portable leaf crate using Serde/JSON and pinned pure Rust flate2/CRC32 storage and sha2 digest dependencies. It is not yet a runtime dependency. `salimon-world`
continues owning the compressed six-body playable fixture, terrain/resource-v1
identity and existing saves. Catalog data must never implicitly grant player
navigation knowledge. No GPU/window/network, source download, real production catalog, orbital integration, OS clock,
playable world replacement or backend is introduced. U11 adds only the selected
world's portable surface query below. U04 adds consumer-supplied Read/Seek
IO and an in-memory prototype encoder, not a production pack builder.

## Source and API

| Module | Ownership |
| --- | --- |
| `identity.rs` | Private validated `CatalogId`; exact `namespace:key` from stable source keys |
| `records.rs` | Typed input DTOs for systems, stars, planets/moons/individual minor bodies and statistical populations; per-field provenance, uncertainty and validity |
| `validation.rs` | Referential integrity, physical scalars, source registration, parent cycles and canonical definition/seed dependency validation |
| `lib.rs` | Immutable publication, sorted identity lookup, schema/version envelope and bounded fixture JSON |
| `fixtures/schema-v1.json`, `src/tests.rs` | Independent frozen wire fixture and portable regression/failure contracts |

`Catalog::new` consumes input DTOs, sorts datasets/records by identity, validates
and publishes an immutable revision. Only shared references are exposed. IDs
preserve source key case and never derive from display names, list position,
system time, runtime random state or a default language hasher. Namespace is
lowercase ASCII; key is case-sensitive ASCII; both use letters/digits/`._-` and
one separating colon, at most 160 UTF-8 bytes. Builder-defined canonical keys are
committed publication identities; the schema validator does not select/generate them; U10 synthesis now owns fictional publication keys.

Dataset IDs and record IDs are globally disjoint. Every observed/inferred field
references registered versioned source metadata; original observed record keys
survive cross-match. Fiction names builder/revision/rationale. Identity-level
provenance does not override field-level evidence: a fictional body can have
inferred quantities, each explicitly tagged. Missing quantities remain `None`;
unknown uncertainty/validity stays unknown, never fabricated certainty or zero.
Mass is kg and radius meters, positive finite when present. Published asymmetric
bounds are finite/nonnegative, cannot cross zero mass/radius, and have optional
confidence in (0,1]. Validity retains declared Julian-day endpoints and explicit
UTC/TT/TDB scale as **metadata only**; U02 owns split epoch/frame arithmetic and U08
owns ephemerides/fallback policy. No cross-scale arithmetic occurs here.

Records reference an existing system. Body/population parents must be individual
system/star/body records in the same system; parent graphs must be acyclic. Moon
parents are bodies; planet parents are stars or systems (including future binary
systems). Statistical populations retain kind, estimated count, descriptor and
canonical definition reference, not allocated imaginary member identities.

## Versioned canonical definition references

Each fictional planet/moon requires a positive definition version, lowercase
64-hex SHA-256 **reference**, and independent named `terrain`, `resources` and
`visual` seed domains. Domains bind `u64` seed, positive algorithm version,
subdefinition digest and explicit upstream domain dependencies. At most 64 named
domains are admitted; duplicate/missing/cyclic dependencies reject. Seed equality
alone is not a dependency; domain names separate interpretation. The schema validator does
not verify full-definition content or evaluate algorithms. U10 now derives seeds
and hashes actual generation recipe/domain inputs; full canonical physical
definitions and evaluation are provided for the selected world by U11 below;
other worlds remain downstream. A seed/digest reference is
not advertised as a complete planet. Published revisions are immutable; changes
require explicit validated version/migration work rather than mutating a running
catalog. This is not owner approval of geography changes or a save migration.

## Schema and boundaries

Envelope format `salimon.universe.catalog`, schema version 1, separate catalog
revision. `Catalog::decode` bounds bytes before JSON parsing, inspects schema,
rejects unknown versions/fields/duplicate JSON fields and then validates before
publication. No implicit migration/unknown-field dropping. Future schema changes
must explicitly bump version and add compatibility/migration tests before use.
The checked-in v1 fixture is a golden compatibility contract, not real science.

JSON is limited to 16 MiB and intended for review/fixtures/small builder inputs;
there is no claim of GB-pack throughput/size or a production storage format.
U04 now owns the bounded binary prototype below; U06–U12 own official manifests,
licensed ingestion, inference, canonical generation and pack publication; U02 delivers hierarchical frames/epoch. U05 owns new-run save/catalog binding. Schema
extension can add future galaxy hierarchy without reinterpreting source IDs.
The real-scale playable migration and native gates remain those blocked issues.

## Validation

Run `cargo test -p salimon-universe --locked`, workspace build/tests, all-target
Clippy and fmt. Tests cover exact deterministic identity/order, golden schema-1
bytes, mixed provenance/missing measurements, referential errors/cycles,
nonfinite/uncertain/invalid data, domain dependency constraints, unsupported
versions, malformed/oversized JSON and immutable explicit revision replacement.
No hardware signoff is needed for this isolated portable schema; downstream cloud
integration retains #436–439, with human presentation/platform/budget criteria in
#561/#557/#563 under the [verification workflow](../../docs/verification-workflow.md).
See [approved design](../../docs/universe-expansion-approved-design.md)
and [canonical world policy](../../docs/canonical-planet-generation.md).

## Hierarchical frames and epoch — U02 (#403)

`coordinates.rs` owns validated galactic positions: integer 2^40-meter cells plus
centered local f64 offsets. Do not flatten them to global f64 meters. Relative
subtraction uses integer cell differences before combining offsets; GPU conversion
is available only after camera subtraction. Local cell spacing is at most 0.123 mm;
large differences remain approximate distances, not precision-preserving origins.

`FrameTree` evaluates one system at one exact epoch. Typed system/body/local IDs,
validated right-handed orthonormal axes and instantaneous parent-relative poses,
linear velocities and angular velocities compose through the common ancestor.
Velocity includes omega cross radius. Local-to-body paths never visit the galaxy
anchor. `SystemAnchor` carries galactic position, fixed orientation and explicit
SI space velocity (proper/tangential and radial components supplied by importers),
with linear epoch propagation. Missing observed motion must not be invented.
Actual orbit/orientation evaluation and catalog astrometry conversion are U08/U14;
frame trees reject stale epochs, unknown frames, duplicates, cycles and nonfinite
states. The tree is not an ephemeris or current runtime/world replacement.

`epoch.rs` owns validated split Gregorian midnight days/nanoseconds, years 1–9999,
and explicit UTC/TT/TDB labels. TT/TDB durations use uniform SI calendar seconds;
there is no relativistic player clock or implicit scale conversion. Leap-second
input and cross-day UTC arithmetic reject until an explicit leap-table adapter
exists. These errors preserve scientific uncertainty rather than silently using
POSIX UTC as TT. Calendar clocks are supplied by consumers; no OS clock is read.
The U01 wire schema remains version 1; new frame/epoch types are independent APIs.

U05 now consumes these IDs/epochs in the separate persistence universe envelope;
`Epoch::compare` orders same-scale split instants without UTC duration assumptions.
No catalog or world state is mutated by persistence.

## Source manifests — U06 (#407)

`sources.rs` is the typed, bounded version-1 `salimon.universe.sources` metadata
boundary: exact source IDs/release/query, official documentation, reuse status,
credit/review references, coverage/uncertainty and artifact byte counts/SHA256s.
It validates paths, duplicates, required metadata and production eligibility;
it performs no IO, hashing, scientific conversion or download. The offline CLI
[import contract](../../docs/universe-sources.md) verifies hashes and stages raw
bytes. U07/U08 own source-specific science ingestion/cross-match/units; U12 owns
production pack publication. Neither fixtures nor catalog truth grant chart knowledge.

## Binary partitions — U04 (#405)

`partitions.rs` owns deterministic container encoding, strict bounded index
validation, random-access Read/Seek cell decoding and the immutable source map.
`partitions/tests.rs` covers wire compatibility, corruption, hostile index
ranges/order, length expansion and payload/index agreement. The independently
Python/zlib-generated `fixtures/partition-v1.bin` freezes version-1 compatibility.
`examples/partition_bench.rs` measures the observed development sample and clearly
marked replicated sensitivity rows. See the canonical [wire/precision/budget
contract](../../docs/universe-storage.md). No cross-cell references, GB pack,
source reuse approval or runtime world replacement is supplied by this prototype.

## Scientific reference harness — U03 (#404)

`reference.rs` owns strict bounded reference/candidate wire decoding and validated
immutable suites. `reference/tests.rs` owns coverage/frame/epoch/norm/failure gates;
`fixtures/references-v1.json` freezes independently transcribed Horizons SI and
analytic goldens. `examples/support/reference_inputs.rs` is a frozen-source test
adapter plus production U02 workload, not a U08 importer/solver. The comparator
CLI and seven-sample CPU workload are `examples/reference_compare.rs` and
`examples/reference_bench.rs`. Read the [reference/budget contract](../../docs/universe-reference-fixtures.md).
No gameplay/model accuracy or M1 performance is inferred;
[cloud workload evidence #521](../../reports/issue-521/README.md) records repeated
release CPU/RSS measurements and retained Linux comparisons. Hardware budget
decisions remain #563; no numerical/model threshold changes.

## Gaia ingestion — U07 (#408)

`ingestion/gaia.rs` owns explicit quality selection, typed source-unit astrometry,
exact reviewed stellar cross-match and bounded-batch schema-1 canonicalization.
`ingestion/csv.rs` owns bounded local CSV decoding; Gaia tests live in
`ingestion/gaia/tests.rs`. `examples/gaia_bench.rs` measures synthetic JSON/zlib
storage sensitivity and throughput, not a production forecast. Read the
[selection, units, sidecar and scaling contract](../../docs/universe-gaia-ingestion.md).
No distance inference, full snapshot approval, multiple-star solver, runtime
migration or astrometry storage in U04's wire is implied.

Gaia angular values expose degrees; separately named RA/Dec source-error fields
retain mas without implicit covariance conversion. A mixed-unit regression
protects both typed and raw evidence.

## Observed Solar System and exoplanets — U08 (#409)

`ingestion/horizons.rs` owns bounded strict API-1.2 table-2 ICRF/SSB/TDB decoding,
SI vector/physical conversion, imported coverage and exact-state refusal policy.
`ingestion/exoplanets.rs` owns PS default-row typed quantities, asymmetric errors,
limits, original references and U07/Gaia host reconciliation. Their `tests.rs`
modules own actual Earth/Proxima plus synthetic boundary/matching regressions;
Earth states compare against independent U03 goldens. `examples/observed_import.rs`
builds the frozen source fixture deterministically, also checked twice in CI.
Read the [canonical units, identities and validity contract](../../docs/universe-observed-ingestion.md).
U01/U04 wire is unchanged: astrometry/ephemerides/orbits remain separate typed
observations requiring U12 packaging. Full reviewed coverage remains #526/#513;
U14 owns arbitrary-date model evaluation and lower-accuracy fallback, not U08.

## Missing-data inference — U09 (#410)

`inference.rs` owns typed single-star system requests, deterministic version-1
proposal distributions, qualitative confidence, source-dependency evidence and
bounded whole-system physical acceptance. `inference/observed.rs` adapts U08
measurements with explicit source-limit/conflict refusal. `inference/tests.rs`
and `examples/inference_sanity.rs` cover fixed-seed preservation, constraints,
Kepler references and distribution coverage. Read the [canonical inference
contract](../../docs/universe-inference.md) for exact formulas/ranges/limits.
Catalog schema-1 remains unchanged; age, partial orbits, model identity and
sampling evidence use separate typed sidecars requiring U12 packaging. Present
quantities are immutable; unsupported systems/inconsistent observations reject.
No calibrated posterior, full state, long-term stability proof, runtime migration
or native hardware signoff is claimed.

## Canonical fiction synthesis — U10 (#411)

`synthesis.rs` owns validated residual-density region inputs, versioned SHA-256
field lanes, deterministic stellar/body/population recipes and immutable bounded
publications. Cubes use fixed-order IEEE multiplication rather than the
platform-dependent precision of `f64::powi`; CI verifies debug/release hashes.
`synthesis/tests.rs` owns physical/source/identity/failure and fixed
hash contracts; `fixtures/fiction-request-v1.json` pins a development request.
`examples/fiction_build.rs` owns bounded local file IO and staged replacement;
`examples/fiction_sanity.rs` measures four fixed synthetic cohorts and raw/zlib
storage with repeated/reordered full-output comparisons. Read the [canonical
builder contract](../../docs/universe-fiction-builder.md) for formulas, units,
limits, namespace reservation, provenance and conditional distribution evidence.
U01/U04 wire is unchanged. Actual recipe bytes and domain inputs are hashed;
U11 below supplies the selected world's full inputs/surface query; other worlds'
full definitions and playable adapters remain downstream, and U12
must package the catalog plus sidecars. No empirical full-Galaxy fit, production
pack, runtime generation, chart reveal or native acceptance is supplied.
## Permanent starting world — U11 (#412)

`starting_system.rs` consumes a pinned U10 cohort and publishes exactly one
versioned immutable awakening definition. It owns the ID/naming freeze, split-cell
Earth-distance lower bound, physical/candidate screening, explicit full selected
world inputs, subsystem/world SHA-256s and portable single radial-height query.
`FrozenStart` exposes shared DTOs and validates bounded exact-manifest decoding;
unsupported/altered publications refuse without fallback selection.

The [canonical contract](../../docs/universe-starting-system.md) defines the
source/expanded-definition distinction, arithmetic, cold/airless assumptions,
palette overrides, named domains and fixed resource/arrival routes. The resource
domain declares terrain anchoring; visuals remain physically independent.
No new dependencies or U01 schema changes are needed. Prototype world remains
unchanged; full definitions for other worlds, U12 packaging, U14 states and U24
playable adapters remain separate. Catalog access never grants player knowledge.

Tests: `src/starting_system/tests.rs`; independent frozen manifest/request:
`fixtures/starting-world-v1.json`, `starting-candidates-v1.json`; reproduction:
`examples/starting_system.rs`. Run universe/workspace gates and repeated example
output; macOS/Windows CI runs debug/release U11 goldens. These are portable CPU
definitions and physical screens, not M1 rendering/contact/survival acceptance.

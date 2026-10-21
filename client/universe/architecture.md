# Canonical universe schema — U01 (#402)

`salimon-universe` is a portable leaf crate using only the workspace's existing
Serde/JSON dependencies. It is not yet a runtime dependency. `salimon-world`
continues owning the compressed six-body playable fixture, terrain/resource-v1
identity and existing saves. Catalog data must never implicitly grant player
navigation knowledge. No GPU/window/network/file IO, source download, real
catalog, orbit, clock, terrain evaluation, pack builder or backend is introduced.

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
committed publication identities; this crate does not select/generate them.

Dataset IDs and record IDs are globally disjoint. Every observed/inferred field
references registered versioned source metadata; original observed record keys
survive cross-match. Fiction names builder/revision/rationale. Identity-level
provenance does not override field-level evidence: a fictional body can have
inferred quantities, each explicitly tagged. Missing quantities remain `None`;
unknown uncertainty/validity stays unknown, never fabricated certainty or zero.
Mass is kg and radius meters, positive finite when present. Published asymmetric
bounds are finite/nonnegative, cannot cross zero mass/radius, and have optional
confidence in (0,1]. Validity retains declared Julian-day endpoints and explicit
UTC/TT/TDB scale as **metadata only**; U02 owns actual clocks/transforms and U08
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
alone is not a dependency; domain names separate interpretation. The crate does
not generate seeds, verify full-definition content against digests, run hashes,
or evaluate algorithms. Full canonical definitions/parameters, reproducible PRF
and actual digest verification belong to U04/U10/U11. A seed/digest reference is
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
U04 owns binary partitions/codecs/indexing; U06–U12 own official manifests,
licensed ingestion, inference, canonical generation and pack publication; U02
owns hierarchical frames/epoch. U05 owns new-run save/catalog binding. Schema
extension can add future galaxy hierarchy without reinterpreting source IDs.
The real-scale playable migration and native gates remain those blocked issues.

## Validation

Run `cargo test -p salimon-universe --locked`, workspace build/tests, all-target
Clippy and fmt. Tests cover exact deterministic identity/order, golden schema-1
bytes, mixed provenance/missing measurements, referential errors/cycles,
nonfinite/uncertain/invalid data, domain dependency constraints, unsupported
versions, malformed/oversized JSON and immutable explicit revision replacement.
No native signoff is needed for this isolated portable schema; downstream native
integration retains #436–439. See [approved design](../../docs/universe-expansion-approved-design.md)
and [canonical world policy](../../docs/canonical-planet-generation.md).

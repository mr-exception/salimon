# Universe schema maintenance

Read the active issue, root [AGENTS.md](../../AGENTS.md), this crate's
[architecture](architecture.md) and [invariants](invariants.md) before editing.
`identity.rs` owns stable source IDs; `records.rs` owns public input DTOs;
`validation.rs` validates publication; `lib.rs` exposes immutable catalog access.

Preserve the independent schema-1 fixture, missing/uncertain source distinctions
and existing playable world's IDs. Extend versioned wire contracts deliberately;
no runtime/renderer coupling, new scientific assumptions, migration, data import
or world evaluation is implied by a schema change. Run the architecture's checks
and update affected docs/report in the same PR.

U02: `coordinates.rs` owns split galactic cells, system/body/local transforms and
moving-frame velocity; `epoch.rs` owns split calendar instants and scale gates.
Read their architecture contract before changing numerical/time behavior.

U06 source metadata is in `sources.rs`; checksum verification and atomic raw-byte
staging are in `scripts/import_universe_sources.py`. Never promote fixture or
restricted/unreviewed source data into production by changing a checksum alone.

U04 `partitions.rs` owns the bounded prototype container and consumer-supplied
Read/Seek adapter. Read [storage contract](../../docs/universe-storage.md) before
changing offsets, codec/precision, limits, source map or schema versions. Run
partition regressions and the release example; retain conditional budget evidence.

U03 `reference.rs` owns strict independent reference comparison; frozen goldens,
unit regressions and example workload adapters have separate paths in the
[architecture](architecture.md#scientific-reference-harness--u03-404). Read the
[reference budget guide](../../docs/universe-reference-fixtures.md) before changing
tolerances or claiming solver/native coverage. Preserve failure and source context.

U07 `ingestion/gaia.rs` adds streaming local CSV decoding and bounded canonical
matching with typed source measurements; see the [Gaia contract](../../docs/universe-gaia-ingestion.md).
Production reuse/full-data acceptance remain gated on #513.

U08 `ingestion/{horizons,exoplanets}.rs` imports source-labeled Solar System states,
physical measurements and PS default-row planets; see the [observed ingestion
contract](../../docs/universe-observed-ingestion.md) before editing frame/epoch,
unit/error/limit or identity behavior. `observed_import` is a deterministic local
fixture example, not production packaging. Full source coverage remains #526.

U09 `inference.rs` owns bounded system completion; `inference/observed.rs` adapts
U08 measurements and `inference/tests.rs` owns preservation/failure regressions.
Read the [inference contract](../../docs/universe-inference.md) before changing
priors, confidence, rejection or deterministic identity. `inference_sanity` is
portable heuristic evidence, not calibrated science or production publication.

U10 `synthesis.rs` owns bounded offline fictional-system publication and SHA-256
lanes; `synthesis/tests.rs` owns identity/source/physical/golden regressions.
Read the [builder contract](../../docs/universe-fiction-builder.md) before changing
recipes or publication hashes. Examples `fiction_build` and `fiction_sanity` own
file IO and synthetic evidence; never call synthesis from runtime or replace
observed records. U11 now publishes the selected starting world's full definition;
other worlds' definitions and playable evaluation adapters remain downstream.

U11 `starting_system.rs` owns the exact permanent candidate/world digests, fixed
IDs, complete selected-world manifest and single radial surface query. Read the
[starting-system contract](../../docs/universe-starting-system.md). Do not reselect
on launch, regenerate a visited world or treat U10 recipe hashes as U11 world
digests. Visual overrides cannot change terrain or resources. Resource routes
must clear the authored ship disk and use the same terrain query. Golden changes
need explicit publication/migration review; native/playable integration is U24.

#513 owner scope is private development only. Python production staging additionally
requires a human review sidecar bound to the exact source manifest; see the
[source-use/evidence contract](../../docs/universe-sources.md#private-development-decision-and-production-evidence-513).
Rust source metadata/schema is unchanged; never treat metadata acceptance as
permission, full-snapshot coverage or approval of a production staging operation.

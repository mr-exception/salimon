# salimon-universe

Portable immutable canonical catalog schema and provenance foundation (U01).
See [architecture and API](architecture.md), [invariants](invariants.md), and
[maintenance](README.ai.md). The playable world still uses `salimon-world`.

`cargo test -p salimon-universe --locked`

U02 adds portable split galactic positions, hierarchical frame transforms and
calendar epochs. See [coordinate/time contract](architecture.md#hierarchical-frames-and-epoch--u02-403).

U04 adds a bounded independently compressed cell prototype, strict index/source
map and random-access reader. See [storage contract](../../docs/universe-storage.md).
Measure with `cargo run -p salimon-universe --example partition_bench --release --locked`.

U03 adds the [scientific reference comparator and CPU workloads](../../docs/universe-reference-fixtures.md).
Run `cargo run -p salimon-universe --example reference_bench --release --locked`;
reference Apple M1 measurements remain a separate gate.

U07 `ingestion/gaia.rs` adds streaming local CSV decoding and bounded canonical
matching with typed source measurements; see the [Gaia contract](../../docs/universe-gaia-ingestion.md).
Production reuse/full-data acceptance remain gated on #513.

U08 `ingestion/{horizons,exoplanets}.rs` imports source-labeled Solar System states,
physical measurements and PS default-row planets; see the [observed ingestion
contract](../../docs/universe-observed-ingestion.md) before editing frame/epoch,
unit/error/limit or identity behavior. `observed_import` is a deterministic local
fixture example, not production packaging. Full source coverage remains #526.

U09 adds [offline missing-data inference](../../docs/universe-inference.md) with
explicit model classes, deterministic proposals, provenance and conservative
physical screens. Run `cargo run -p salimon-universe --example inference_sanity --locked`
for fixed-seed histograms; this does not fit empirical population frequencies.

U10 adds the [offline canonical fictional-system builder](../../docs/universe-fiction-builder.md).
It preserves observed sources, publishes stable fictional IDs and compact minor
populations, and hashes complete generation sidecars. `fiction_build` accepts
reviewed region inputs; `fiction_sanity` measures synthetic cohorts/storage.
Other worlds' full definitions and U12 production publication remain downstream.

U11 freezes [Nacre Reach and Vesper](../../docs/universe-starting-system.md), with
distance/physical screening, full versioned starting-world surface/resource/visual
inputs and a reproducible manifest. Run `cargo run -p salimon-universe --example
starting_system --locked -- --audit` for candidate evidence. Portable definition
publication does not change playable startup; U24 owns that migration.


## Incremental private offline packs (#413)

See the [implemented contract](../../docs/universe-pack.md) for ownership, inputs, bounds, validation and remaining production/runtime limitations.


## Bounded read-only pack queries (#414)

See the [implemented contract](../../docs/universe-reader.md) for ownership, inputs, bounds, validation and remaining production/runtime limitations.

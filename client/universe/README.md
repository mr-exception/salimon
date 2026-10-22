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

# Bounded read-only pack queries (#414)

The `reader` module owns exact record/system ID and galactic-region queries, SHA256-verified cell/sidecar reads and a least-recently-used cache. Cache budget accounts Vec capacities of encoded catalog bytes; caller-owned decoded catalogs, the bounded manifest/index and a single <=256 MiB blob/decode scratch are explicitly outside that budget. Oversized cells bypass the cache. Background prefetch has eight queued requests, one worker and shared mutex ownership; queue saturation returns false and the latest failure remains observable. Production pack integration and real workload IO/RSS budgets remain unverified. This module does not attach catalog truth to player discovery or runtime.

This is portable private-development implementation, separate from playable runtime adoption. Production source approval remains #659 and full-data acceptance #524/#526. No hardware, scientific model accuracy or production completeness is inferred from fixture tests.

Run `cargo test -p salimon-universe --locked`, `cargo clippy -p salimon-universe --all-targets --locked -- -D warnings`, `cargo fmt --all -- --check` and `python3 scripts/check_docs.py`.

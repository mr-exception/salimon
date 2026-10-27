# Bounded read-only pack queries (#414)

The `reader` module owns exact record/system ID and galactic-region queries, SHA256-verified cell/sidecar reads and a least-recently-used cache. Cache budget accounts Vec capacities of encoded catalog bytes; caller-owned decoded catalogs, the bounded manifest/index and a single <=256 MiB blob/decode scratch are explicitly outside that budget. Oversized cells bypass the cache. Background prefetch has eight queued requests, one worker and shared mutex ownership; queue saturation returns false and the latest failure remains observable. The U12 pack service is integrated and measured by the filesystem/IO/RSS workload below. This module does not attach catalog truth to player discovery or runtime.

This is portable private-development implementation, separate from playable runtime adoption. Private real-data development is authorized; future distribution review remains #659 and full-release scientific data acceptance remains #524/#526. No hardware, scientific model accuracy or production completeness is inferred from fixture tests.

Run `cargo test -p salimon-universe --locked`, `cargo clippy -p salimon-universe --all-targets --locked -- -D warnings`, `cargo fmt --all -- --check` and `python3 scripts/check_docs.py`.

## Cloud workload acceptance

`cargo run -p salimon-universe --example reader_bench --release --locked -- PACK_DIR`
exercises the actual filesystem reader and prefetch worker for 20000 randomized
seeks over 1024 cells at each of 16/64/256 KiB cache budgets. It checks resident
encoded capacities on every request and records logical catalog IO, p50/p95/max
latency and Linux RSS at each quarter. `Reader::stats` exposes actual verified
catalog blob reads/bytes and cache hits for predictable IO accounting. Manifest
and index memory, bounded decode scratch and caller-owned returned catalogs are
explicitly separate from cache capacity. An eight-thread navigation regression
verifies shared-cache reuse without duplicate IO. Missing/corrupt cells and
prefetch failures retain their existing fail-closed coverage.

The recorded cloud workload completes with bounded cache and observed RSS; no
human verification or native performance claim is required. Hooking this reader
into playable real-scale streaming is #422 and subsequent integration work,
rather than a missing acceptance criterion of the read-only U13 service.

# Issue #414: Bounded read-only pack queries

The `reader` module owns exact record/system ID and galactic-region queries, SHA256-verified cell/sidecar reads and a least-recently-used cache. Cache budget accounts Vec capacities of encoded catalog bytes; caller-owned decoded catalogs, the bounded manifest/index and a single <=256 MiB blob/decode scratch are explicitly outside that budget. Oversized cells bypass the cache. Background prefetch has eight queued requests, one worker and shared mutex ownership; queue saturation returns false and the latest failure remains observable. Production pack integration and real workload IO/RSS budgets remain unverified. This module does not attach catalog truth to player discovery or runtime.

Validation results are recorded below. Linux x86_64, Rust 1.99.1; private fixtures only. No runtime/rendered change or hardware claim. Production source and full-snapshot approval remains #659, scientific/resource coverage #524/#526; those checks are unrun. This PR is a development implementation, not full production qualification.

- `cargo test -p salimon-universe --locked`: 76 passed, zero failures, including 1,000 randomized seeks across 32 cells with a 4 KiB resident budget.
- `cargo clippy -p salimon-universe --all-targets --locked -- -D warnings`: passed.
- `cargo fmt --all`: applied; docs/base-impact check passed with zero failures.
- Concurrent prefetch, missing/corrupt blob rejection and oversized-cell bypass pass. Full workload RSS/IO latency and runtime adoption are unrun.

## Cloud completion revision

Added observable verified catalog IO/cache-hit counters and an eight-thread
concurrent navigation regression. The actual filesystem/prefetch workload runs
20000 randomized seeks across 1024 cells at 16/64/256 KiB cache budgets (60000
queries total), checks residency at every query, and retains p50/p95/max latency,
logical IO and quarter-by-quarter Linux RSS in `cloud/414-bench.json`.

78 universe tests pass; all-target universe Clippy passes. These service-level
checks complete U13 cloud acceptance with the now-configurable U12 pack pipeline.
Later playable streaming integration remains #422; it is not a human gate or an
unfinished U13 API. Full-release input coverage remains separately #524/#526.

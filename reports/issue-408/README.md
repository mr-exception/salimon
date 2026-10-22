# Issue 408 — Gaia fixture ingestion and canonical stellar matching

Delivered portable local streaming CSV selection, exact Gaia source IDs, typed
astrometry/errors, complete original fields and observed dataset/record provenance.
Missing/negative parallax and incomplete motion remain explicit; no distance is
invented. Explicit reviewed-ID matching preserves each source observation,
canonicalizes order and rejects ambiguous/conflicting duplicates. The bounded
collector emits unchanged schema-1 systems/stars plus a separate observation
sidecar. This sidecar is not yet part of U04's binary wire or the playable world.

[Canonical contract](../../docs/universe-gaia-ingestion.md),
[owning architecture](../../client/universe/architecture.md#gaia-ingestion--u07-408)
and [invariants](../../client/universe/invariants.md) map behavior to source/tests.
All affected routing, contributor, source, approved-design status and validation
guides were updated in the same PR. Offline link/path lint checks maintained
links; source/test review checks units, deterministic identity and missing data.

## Measured validation

Linux x86_64 cloud environment, Rust 1.99.1, Python 3.12.14. Commands from repo root:

- `cargo test -p salimon-universe --locked`: **36 passed**, including six new
  CSV/Gaia contracts. Existing schema/partition/reference goldens still pass.
- `cargo clippy -p salimon-universe --all-targets --locked -- -D warnings`: pass.
- `cargo run -p salimon-universe --example gaia_bench --release --locked`: pass.
  The separate timed release executable run is preserved in [benchmark.json](benchmark.json)
  and [resource output](benchmark-resources.txt). It measures synthetic input,
  canonical records plus full source measurement/provenance debug sidecars.
- `cargo test --workspace --locked`: **816 passed, 28 intentionally ignored**, no failures.
- `cargo build --workspace --locked`: pass.
- `cargo clippy --workspace --all-targets --locked -- -D warnings`: pass.
- `cargo fmt --all -- --check`: pass.
- `python3 scripts/check_docs.py`: pass, zero findings.
- `python3 scripts/check_docs.py --base origin/main`: pass after committing;
  includes changed-source owning-guide coverage.

[Workspace output](validation.log) preserves actual tests/build/Clippy results.
Final science review separates degree-valued RA/Dec from explicitly mas-valued
source error fields. A mixed-unit regression protects both typed and raw values;
there is no implicit tangent-plane/covariance reinterpretation. Streaming
canonical counts remain explicitly unknown until collection. Focused universe
tests and all-target Clippy were rerun after this source-unit refinement.

The 10,000-row synthetic batch uses 389,884 compressed bytes (~38.99 bytes/star).
At an assumed 1.8 billion objects that density alone implies **70.18 decimal GB**,
above the approved initial budget, excluding additional fields/index/planet costs.
This is an explicit debug-format sensitivity and budget exception observation,
not an actual full-catalog forecast, representative Gaia benchmark or approval to
ship a larger pack. Raw timings/RSS in linked output apply to this cloud CPU and
include batch collection/JSON/compression, not a constant-memory whole-pack build.
The streaming decoder bounds individual records; the collector scales with rows.

## Acceptance and limits

Portable fixture ID/error/missing-data/selection/count/matching and resource
sensitivity criteria pass. Gaia inputs are synthetic and no production source
bytes were fetched. Actual Gaia release scientific matching, representative
source counts/memory/throughput and full-pack resource acceptance transfer to
[#524](https://github.com/mr-exception/salimon/issues/524), blocked on the existing
[#513](https://github.com/mr-exception/salimon/issues/513) source-use/full-snapshot
gate and U12 pack integration. Cross-match policy is explicit caller-reviewed
IDs, not an unvalidated nearest-neighbor sky solver. Mass/radius inference is U09.
No native/GPU/M1 checks were run: no runtime integration occurred, and native
signoff remains #435–#439/#521 as applicable. Closing #408 on PR merge records
portable delivery and this transfer, not waived real-data/native acceptance.

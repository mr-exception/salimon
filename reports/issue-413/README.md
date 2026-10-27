# Issue #413: Incremental private offline packs

The `pack` module owns a version-1 private manifest, SHA256-addressed blobs and incremental one-cell-at-a-time filesystem builder. Each catalog cell reuses the validated U04 dependency-closed codec. Exact named sidecar bytes are stored independently; source metadata is preserved without granting reuse permission. The builder checks global identities, conflicting sources and explicit mappings for removed IDs; failed publication retains the old manifest. Content blobs may remain as harmless unreachable data after failure. Per-blob bound is 256 MiB, manifest 16 MiB and 4096 cells: these are development limits, not measured Milky Way capacity. Production eligibility must be false. Full ingestion orchestration, representative real-data compression and the 10 GB publication gate remain incomplete pending #659/#524/#526.

Validation results are recorded below. Linux x86_64, Rust 1.99.1; private fixtures only. No runtime/rendered change or hardware claim. Production source and full-snapshot approval remains #659, scientific/resource coverage #524/#526; those checks are unrun. This PR is a development implementation, not full production qualification.

- `cargo test -p salimon-universe --locked`: 72 passed, zero failures.
- `cargo clippy -p salimon-universe --all-targets --locked -- -D warnings`: passed.
- `cargo fmt --all`: applied; `python3 scripts/check_docs.py`: zero failures.
- Full workspace and production/rendered checks are not run at this stage; this leaf module is not connected to runtime.

## Cloud completion revision

Supersedes the initial fixture-only CLI and draft assessment above. The CLI now
consumes an explicit build request with arbitrary frozen local catalogs/producer
bundles and additional full-world sidecars. Original source artifacts are streamed
through SHA256/size verification, source paths are confined to their root, bundle
observations/inference/generation are preserved, and combined revision selection
is explicit. Incremental reuse, identity migration and atomic old-manifest
preservation are exercised through the actual offline pipeline. Real observed
input is permitted for private development; #659 is future distribution review.

Cloud implementation acceptance maps to standalone packaging, complete selected
input/sidecar preservation, deterministic hashes, corruption/path refusal,
incremental reuse and explicit ID migration. Full-release scientific input
selection/coverage/size is already owned by #524/#526; it is not represented as
passed or a human gate on this builder PR. No new deferred code scope is created.

Fresh checks and measured storage/throughput are retained under `cloud/`.

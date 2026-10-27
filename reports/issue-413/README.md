# Issue #413: Incremental private offline packs

The `pack` module owns a version-1 private manifest, SHA256-addressed blobs and incremental one-cell-at-a-time filesystem builder. Each catalog cell reuses the validated U04 dependency-closed codec. Exact named sidecar bytes are stored independently; source metadata is preserved without granting reuse permission. The builder checks global identities, conflicting sources and explicit mappings for removed IDs; failed publication retains the old manifest. Content blobs may remain as harmless unreachable data after failure. Per-blob bound is 256 MiB, manifest 16 MiB and 4096 cells: these are development limits, not measured Milky Way capacity. Production eligibility must be false. Full ingestion orchestration, representative real-data compression and the 10 GB publication gate remain incomplete pending #659/#524/#526.

Validation results are recorded below. Linux x86_64, Rust 1.99.1; private fixtures only. No runtime/rendered change or hardware claim. Production source and full-snapshot approval remains #659, scientific/resource coverage #524/#526; those checks are unrun. This PR is a development implementation, not full production qualification.

- `cargo test -p salimon-universe --locked`: 72 passed, zero failures.
- `cargo clippy -p salimon-universe --all-targets --locked -- -D warnings`: passed.
- `cargo fmt --all`: applied; `python3 scripts/check_docs.py`: zero failures.
- Full workspace and production/rendered checks are not run at this stage; this leaf module is not connected to runtime.

# Incremental private offline packs (#413)

The `pack` module owns a version-1 private manifest, SHA256-addressed blobs and incremental one-cell-at-a-time filesystem builder. Each catalog cell reuses the validated U04 dependency-closed codec. Exact named sidecar bytes are stored independently; source metadata is preserved without granting reuse permission. The builder checks global identities, conflicting sources and explicit mappings for removed IDs; failed publication retains the old manifest. Content blobs may remain as harmless unreachable data after failure. Per-blob bound is 256 MiB, manifest 16 MiB and 4096 cells: these are development limits, not measured Milky Way capacity. Production eligibility must be false. Full ingestion orchestration, representative real-data compression and the 10 GB publication gate remain incomplete pending #659/#524/#526.

This is portable private-development implementation, separate from playable runtime adoption. Production source approval remains #659 and full-data acceptance #524/#526. No hardware, scientific model accuracy or production completeness is inferred from fixture tests.

Run `cargo test -p salimon-universe --locked`, `cargo clippy -p salimon-universe --all-targets --locked -- -D warnings`, `cargo fmt --all -- --check` and `python3 scripts/check_docs.py`.

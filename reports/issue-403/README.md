# Issue 403 — coordinates, moving frames and calendar epochs

Delivered split integer-cell/f64 galaxy anchors, typed system/body/local tree,
common-ancestor transforms and angular transport velocity, explicit system space
motion, relative-only GPU narrowing and split calendar epochs. U01 wire remains
unchanged. [Architecture](../../client/universe/architecture.md#hierarchical-frames-and-epoch--u02-403)
and [invariants](../../client/universe/invariants.md) define implemented bounds.

Linux Rust 1.99.1: `cargo test -p salimon-universe --locked` and all-target crate
Clippy pass; fmt and offline documentation lint pass. Full workspace tests and
all-target workspace Clippy pass. Workspace build passes after clearing a stale
renderer cache artifact (the initial cache-reuse build failed with E0463). Synthetic regression
fixtures include submeter precision at ~100 light-year anchors. Scientific error
fixtures are U03; actual ephemerides U14; playable/native integration #436–439.
UTC cross-day conversion requires a future explicit leap-table adapter and rejects
now; no scientific time conversion or real catalog motion is fabricated.

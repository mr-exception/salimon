# Repository Guidelines

## Notion Source of Truth

This repository implements the game described in the
[Salimon Notion space](https://app.notion.com/p/801c9c427af24e9b8d57b07572ef4119).
Use the connected Notion tools; [docs/notion.md](docs/notion.md) records the
workspace, documentation indexes, task database/data source, and specification
IDs. These links persist project context, not authentication or automatic sync.

Before each task, fetch the selected task from the
[Tasks database](https://app.notion.com/p/e2dadc570fff48b4899def8b0ee70a27), read its
`Order`, `Status`, `Description`, and `Acceptance Criteria`, then refresh the
[Phase 0 specification](https://app.notion.com/p/3d5b456853b981db968dca1901a270a2) and
[technical architecture](https://app.notion.com/p/3d5b456853b981078a82c68207f4444e).
Task numbers refer to the database's `Order` property. Consult linked domain
documents as needed. The hub also contains older unrelated notes; use the game
specifications under Documents and Business Specs for this repository.

Check Notion connection identity with `fetch` using `id: "self"`. If access is
unavailable, report it and request reconnection rather than substituting sibling
repositories or assuming cached requirements are current. Keep credentials out
of Git. Record unresolved behavior decisions in Project Q&A before changing
intended behavior. Update the existing task with progress and validation evidence;
mark it Done only after its acceptance criteria pass.

## Architecture and Scope

All Phase 0 code belongs under `client/`; `core/` contains only a README and has no
backend behavior. `client/runtime/` is the sole Cargo member at task 1. Other
client directories document future ownership. Keep rendering separate from
world, character, and ship state, and isolate OS concerns behind platform/runtime
boundaries. Use custom Rust and low-level libraries; no full game engine. Phase 0
excludes backend/networking, persistence, and production gameplay systems.

## Build, Test, and Style

Follow [README.md](README.md) for macOS setup. From the root, run:

```sh
cargo build --workspace --locked
cargo run --locked -p salimon-client
cargo fmt --all -- --check
cargo clippy --workspace --all-targets --locked -- -D warnings
cargo test --workspace --locked
```

The shell currently has no behavior tests. Use rustfmt defaults, Rust 2024, and
the shared Cargo lints. Future crates inherit workspace metadata/lints. Commit
`Cargo.lock`; exclude `target/`. Keep setup instructions and ownership documents
aligned with actual implementation.

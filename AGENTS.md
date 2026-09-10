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
backend behavior. `client/runtime/` owns the native `winit` lifecycle,
composition, redraw/update scheduling, surface recovery, and clocks;
`client/renderer/` owns `wgpu` resources, camera-relative GPU conversion,
reverse-Z depth, and presentation behind a narrow API. Task 4 activates
`client/world/` for portable `f64`-meter coordinates and the renderer-neutral
camera/precision prototype; Task 5 adds the immutable compressed Solar System
catalog and body-distance math there. Task 6 adds analytic textured spheres,
material LOD, and Sun lighting under `client/renderer/`, with per-body camera
inspection in world. The runtime maps world snapshots into
renderer DTOs; the renderer must not depend on world, character, or ship state.
The platform adapter is still reserved; `winit` integration may remain at the runtime boundary
until platform-specific behavior warrants extraction. Use custom Rust and
low-level libraries; no full game engine. Phase 0 excludes backend/networking,
persistence, and production gameplay systems. Task 3 owns the optional
diagnostics overlay.

## Build, Test, and Style

Follow [README.md](README.md) for macOS setup. From the root, run:

```sh
cargo build --workspace --locked
cargo run --locked -p salimon-client
cargo fmt --all -- --check
cargo clippy --workspace --all-targets --locked -- -D warnings
cargo test --workspace --locked
```

`cargo run` is interactive and continues until the native window closes. In
addition to automated gates, smoke-test all six textured Solar System spheres,
the visible scale-transition fixture, near-surface precision markers,
pause/restart controls, live resize,
minimize/restore, clean close, and relaunch on macOS. Inspect all five solid-body
approaches using keys 2–6 (4 is Earth); N pauses at 12 m. Use rustfmt defaults, Rust
2024, and the shared Cargo lints. Workspace crates inherit shared metadata/lints.
Commit `Cargo.lock`; exclude `target/`. Keep setup instructions, component
maintenance docs, ownership documents, and implementation aligned.

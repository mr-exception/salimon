# Repository Guidelines

## Project source of truth

Use [docs/README.md](docs/README.md) for project specifications and
[GitHub issues](https://github.com/mr-exception/salimon/issues) for tasks.
Before starting an issue, read its full description and **Blocked by** section;
do not start until every blocker is closed. The migrated Notion order is a planning
priority, not a substitute for explicit dependencies. Read the linked specifications
and the current code before implementation.

Record progress, validation evidence, and relevant commits on the GitHub issue.
Close it only after its acceptance criteria pass. Record unresolved product or
architecture decisions in [Project Q&A](docs/project-qa.md), then update the
relevant specification and issue. The legacy Salimon hub notes are archived in
[docs/legacy-salimon-hub.md](docs/legacy-salimon-hub.md) and do not define the
game.

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

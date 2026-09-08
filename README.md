# Salimon

Salimon is a space exploration game built around a custom Rust runtime. The
current milestone is **Phase 0 — Technical Feasibility Showcase**: a native macOS,
client-only prototype using a custom `wgpu` renderer, with Windows and web as
later targets. The reference performance machine is a MacBook Air M1.

This repository implements **task 1: Create Phase 0 repository shell**. The client
currently prints a startup message and exits successfully. Native windowing,
input, GPU initialization, and the render loop belong to task 2; there is no
playable scene yet.

## Source of requirements

The [Salimon Notion space](https://app.notion.com/p/801c9c427af24e9b8d57b07572ef4119)
holds the task list and documentation. Start with
[task 1](https://app.notion.com/p/3d5b456853b981e485dfcc5d8dc86c9f), the
[Phase 0 specification](https://app.notion.com/p/3d5b456853b981db968dca1901a270a2), and
[Technical Architecture & AI Maintenance](https://app.notion.com/p/3d5b456853b981078a82c68207f4444e).
[AGENTS.md](AGENTS.md) describes how future agents should access those sources;
[docs/notion.md](docs/notion.md) records the stable page and database identifiers.

## Repository boundaries

```text
salimon/
├── Cargo.toml          # Workspace metadata, members, and shared lints
├── Cargo.lock          # Committed dependency resolution
├── core/
│   └── README.md       # Future backend boundary; no Phase 0 implementation
└── client/
    ├── runtime/        # salimon-client executable and future orchestration
    ├── renderer/       # Future GPU resources and rendering
    ├── world/          # Future portable scene/domain state
    ├── character/      # Future first-person character behavior
    ├── ship/           # Future ship state and control
    ├── platform/       # Future native window/input/platform adapters
    ├── assets/         # Future source art and exported assets
    └── diagnostics/    # Future engineering metrics and overlay
```

Only `client/runtime/` is a Cargo package today. The other client directories
document ownership until their implementation tasks begin. See
[client/README.md](client/README.md) for the intended dependency boundaries.

## macOS setup

1. Install Apple's Command Line Tools if needed: `xcode-select --install`.
   Check the selected tools with `xcode-select -p`.
2. Install Rust through [rustup](https://rustup.rs/), then open a new terminal so
   `cargo` and `rustup` are on `PATH`.
3. From this repository, install or update the stable tools:

   ```sh
   rustup toolchain install stable --profile minimal --component rustfmt --component clippy
   rustc --version
   cargo --version
   ```

`rust-toolchain.toml` selects stable Rust and the formatting/linting components.
The workspace uses Rust 2024 and requires Rust 1.89 or newer. The stable channel
is intentionally not an exact compiler pin; record compiler versions when
reporting validation or performance. Task 1 was verified with Rust/Cargo 1.89.0
on native Apple Silicon macOS (`aarch64-apple-darwin`). Intel macOS is not yet
verified. The shell has no third-party dependencies and needs no credentials,
Notion access, or network connection to build once Rust is installed.

## Build and run

Run these commands from the repository root:

```sh
cargo build --workspace --locked
cargo run --locked -p salimon-client
```

Expected output:

```text
Salimon Phase 0: repository shell ready.
Native window and rendering will be added in task 2.
```

For an optimized native build:

```sh
cargo build --workspace --locked --release
cargo run --locked --release -p salimon-client
```

Cargo builds for the host architecture by default; outputs go to the ignored
root `target/` directory. No application bundle or installer exists yet.

## Development workflow

Read the current task and its linked Notion specifications before changing
behavior. Keep all Phase 0 implementation under `client/`; leave `core/` as a
documentation-only boundary.

```sh
cargo fmt --all -- --check
cargo clippy --workspace --all-targets --locked -- -D warnings
cargo test --workspace --locked
```

Use `cargo fmt --all` to apply formatting. The test command currently discovers
zero tests: this shell has no simulation or gameplay behavior. Its acceptance
checks are a successful native build, clean formatting/linting, the startup
smoke run, and the documented directory boundaries. Add behavior-specific tests
as implementation tasks introduce contracts and invariants.

Add future crates explicitly to the root workspace and inherit its package
metadata and lints. Keep `Cargo.lock` committed; validate normal changes with
`--locked`, and regenerate the lockfile intentionally when adding/updating
dependencies. Cargo's [workspace documentation](https://doc.rust-lang.org/cargo/reference/workspaces.html)
and [lockfile guide](https://doc.rust-lang.org/cargo/guide/cargo-toml-vs-cargo-lock.html)
describe these conventions.

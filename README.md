# Salimon

Salimon is a space exploration game built around a custom Rust runtime. The
current milestone is **Phase 0 — Technical Feasibility Showcase**: a native macOS,
client-only prototype using a custom `wgpu` renderer, with Windows and web as
later targets. The reference performance machine is a MacBook Air M1.

This repository implements **task 3: Add Phase 0 diagnostics overlay** on top of
the Task 2 native runtime and renderer bootstrap. The client opens a native,
resizable window, continuously renders a triangle through `wgpu`, and exposes an
optional engineering view for live runtime/renderer measurements. It is not yet
a playable scene.

## Source of requirements

The [Salimon Notion space](https://app.notion.com/p/801c9c427af24e9b8d57b07572ef4119)
holds the task list and documentation. Start with
[task 3](https://app.notion.com/p/3d5b456853b98136afd1dd612e9af67c), the
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
    ├── runtime/        # salimon-client lifecycle and composition executable
    ├── renderer/       # salimon-renderer GPU library and bootstrap scene
    ├── world/          # Future portable scene/domain state
    ├── character/      # Future first-person character behavior
    ├── ship/           # Future ship state and control
    ├── platform/       # Future native window/input/platform adapters
    ├── assets/         # Future source art and exported assets
    └── diagnostics/    # Engineering metrics and overlay rasterization
```

`client/runtime/`, `client/renderer/`, and `client/diagnostics/` are Cargo
packages. The other client directories document ownership until their
implementation tasks begin. The runtime drives native lifecycle and composes
typed observations; diagnostics aggregates and rasterizes the engineering view;
the renderer owns `wgpu` resources and presentation. See
[client/README.md](client/README.md) for the dependency boundaries.

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
reporting validation or performance. Task 2 was verified with Rust/Cargo 1.89.0
on native Apple Silicon macOS (`aarch64-apple-darwin`). Intel macOS is not yet
verified. The first build may need network access to download the locked `winit`
and `wgpu` dependency graph. Running the client needs no credentials or Notion
connection.

## Build and run

Run these commands from the repository root:

```sh
cargo build --workspace --locked
cargo run --locked -p salimon-client
```

The run command opens the **Salimon Phase 0** native window and continues until
the window is closed. A colored triangle on the clear background confirms that
the `wgpu` surface, render pipeline, command submission, and presentation path
are active.

Press **F3** to toggle the engineering diagnostics overlay. It is hidden by
default and updates at a throttled cadence while frame observations continue to
be collected. The view includes FPS/frame time, CPU-side encoding/submission
time, GPU pass time when timestamp queries are supported, scene and total
draw/object counts, optional GPU allocator totals, and memory-pressure warnings.
Rows for player/ship position, ship velocity/speed/thruster, and nearby-body
distance are explicitly `N/A` until their owning systems are implemented by
later tasks.

For an optimized native build:

```sh
cargo build --workspace --locked --release
cargo run --locked --release -p salimon-client
```

Cargo builds for the host architecture by default; outputs go to the ignored
root `target/` directory. No application bundle or installer exists yet.

### Native smoke check

Rendering and diagnostics require an interactive check on macOS in addition to
automated tests:

1. Launch the client and confirm the bootstrap triangle is visible.
2. Press F3 and confirm the diagnostics panel appears, values update, and GPU
   timing/memory show either a plausible value or an explicit unavailable state.
3. Press F3 again and confirm the panel disappears while the triangle remains.
4. Resize the window repeatedly, including to a very small size, and confirm
   rendering follows the new drawable size without a panic or validation error.
5. Minimize and restore the window, then confirm rendering and diagnostics resume
   without counting the paused interval.
6. Close the window, launch the client again, and confirm both shutdown and
   relaunch are clean.

This bootstrap instrumentation does not establish the later Phase 0 performance
target at fixed 1920x1080; Task 11 owns benchmark-scenario evidence on the
reference MacBook Air M1.

## Development workflow

Read the current task and its linked Notion specifications before changing
behavior. Keep all Phase 0 implementation under `client/`; leave `core/` as a
documentation-only boundary.

```sh
cargo fmt --all -- --check
cargo clippy --workspace --all-targets --locked -- -D warnings
cargo test --workspace --locked
```

Use `cargo fmt --all` to apply formatting. Automated tests cover logic that does
not require a live native surface; the native launch, drawing, overlay toggle,
resize, minimize, restore, and close behavior still require the smoke check
above. Task 3 does not add simulation or gameplay behavior.

Add future crates explicitly to the root workspace and inherit its package
metadata and lints. Keep `Cargo.lock` committed; validate normal changes with
`--locked`, and regenerate the lockfile intentionally when adding/updating
dependencies. Cargo's [workspace documentation](https://doc.rust-lang.org/cargo/reference/workspaces.html)
and [lockfile guide](https://doc.rust-lang.org/cargo/guide/cargo-toml-vs-cargo-lock.html)
describe these conventions.

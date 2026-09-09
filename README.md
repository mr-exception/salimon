# Salimon

Salimon is a space exploration game built around a custom Rust runtime. The
current milestone is **Phase 0 — Technical Feasibility Showcase**: a native macOS,
client-only prototype using a custom `wgpu` renderer, with Windows and web as
later targets. The reference performance machine is a MacBook Air M1.

This repository implements **task 4: Implement large-scale coordinate and camera
prototype** on top of the native runtime, renderer, and diagnostics bootstrap.
The client opens a native, resizable window and continuously moves an engineering
camera from a `120 Mm` far-space view to meter-scale surface markers placed near
a deliberately large absolute coordinate. It is a precision fixture, not yet a
playable Solar System scene.

## Source of requirements

The [Salimon Notion space](https://app.notion.com/p/801c9c427af24e9b8d57b07572ef4119)
holds the task list and documentation. Start with
[task 4](https://app.notion.com/p/3d5b456853b981258981dec8275426a0), the
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
    ├── renderer/       # salimon-renderer GPU library and presentation
    ├── world/          # Portable f64 coordinates and Task 4 camera fixture
    ├── character/      # Future first-person character behavior
    ├── ship/           # Future ship state and control
    ├── platform/       # Future native window/input/platform adapters
    ├── assets/         # Future source art and exported assets
    └── diagnostics/    # Engineering metrics and overlay rasterization
```

`client/runtime/`, `client/renderer/`, `client/world/`, and
`client/diagnostics/` are Cargo packages. The other client directories document
ownership until their implementation tasks begin. World owns portable coordinate
and camera state; the runtime drives native lifecycle and maps typed snapshots;
diagnostics aggregates and rasterizes the engineering view; the renderer owns
camera-relative GPU conversion, reverse-Z depth, `wgpu` resources, and
presentation. See
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
the window is closed. Colored cuboid proxies and scale markers show the automatic
far-space-to-near-surface camera transition. The fixture uses anonymous geometry
so Task 5 can own the compressed Solar System data and Task 6 can own planet
rendering without inheriting temporary content.

Press **P** to pause or resume the automatic transition, **R** to restart it at
the far endpoint, and **N** to jump to the exact 12 m near-surface dwell and
pause there for inspection. Only the initial physical key press is acted on;
repeats and releases are ignored.

Press **F3** to toggle the engineering diagnostics overlay. It is hidden by
default and updates at a throttled cadence while frame observations continue to
be collected. The view includes FPS/frame time, real prototype-update and
CPU-side render time, GPU pass time when timestamp queries are supported, scene
and total draw/object counts, optional GPU allocator totals, camera
position/altitude/transition state, and memory-pressure warnings. Rows for
player/ship position, ship velocity/speed/thruster, and nearby-body distance are
explicitly `N/A` until their owning systems are implemented by later tasks.

The coordinate/depth approach and its measured precision limits are documented
in [client/world/coordinate-strategy.md](client/world/coordinate-strategy.md).
In brief, CPU world positions stay in `f64` meters, the renderer subtracts the
camera origin in `f64` before narrowing relative offsets to GPU `f32`, and an
infinite-far reverse-Z `Depth32Float` projection avoids a far clipping plane.

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

1. Launch the client and confirm the colored far-space proxy approaches smoothly
   until the meter-scale surface pad and markers are visible.
2. Let the fixture traverse far and near scales; confirm there is no visible
   position jitter, premature far clipping, depth inversion, or coplanar flicker
   in the intentionally separated markers.
3. Press N and confirm the exact 12 m near-surface view and its markers remain
   stable while paused; press P to resume, then R and confirm the camera restarts
   at the far endpoint.
4. Press F3 and confirm the panel shows updating camera altitude/phase, CPU/GPU
   states, and truthful scene counts; hide it again without affecting the scene.
5. Resize repeatedly, including to a very small size, and confirm projection and
   depth follow the drawable size without a panic or validation error.
6. Minimize and restore, then confirm rendering/animation resume without a time
   jump or paused interval contaminating diagnostics.
7. Close the window, launch the client again, and confirm both shutdown and
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
above. Task 4 adds only a deterministic engineering camera/coordinate fixture;
it does not add gameplay, celestial-body data, planet rendering, or backend
behavior.

Add future crates explicitly to the root workspace and inherit its package
metadata and lints. Keep `Cargo.lock` committed; validate normal changes with
`--locked`, and regenerate the lockfile intentionally when adding/updating
dependencies. Cargo's [workspace documentation](https://doc.rust-lang.org/cargo/reference/workspaces.html)
and [lockfile guide](https://doc.rust-lang.org/cargo/guide/cargo-toml-vs-cargo-lock.html)
describe these conventions.

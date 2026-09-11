# Salimon

Salimon is a space exploration game built around a custom Rust runtime. The
current milestone is **Phase 0 — Technical Feasibility Showcase**: a native macOS,
client-only prototype using a custom `wgpu` renderer, with Windows and web as
later targets. The reference performance machine is a MacBook Air M1.

This repository includes **task 10: Scale spaceship to at least twice its current
size** and the completed first-person character and walkable ship shell.
The native client starts inside the custom Task 7
Salimon scout landed on Earth, with portable character/ship state, runtime-loaded
GLB geometry, walking, free mouse look, jumping, cockpit interaction, a landed-only
door, and radial surface traversal.

## Source of requirements

The [Salimon Notion space](https://app.notion.com/p/801c9c427af24e9b8d57b07572ef4119)
holds the task list and documentation. Start with
[task 10](https://app.notion.com/p/3d7b456853b981cabd1bd60a1d641b99), the
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
    ├── world/          # Compressed Solar System, f64 coordinates, and camera fixture
    ├── character/      # Portable first-person movement and gravity transitions
    ├── ship/           # Portable ship pose, cockpit authority, motion, and door state
    ├── platform/       # Future native window/input/platform adapters
    ├── assets/         # Editable source art and exported runtime assets
    └── diagnostics/    # Engineering metrics and overlay rasterization
```

`client/runtime/`, `client/renderer/`, `client/world/`, `client/character/`,
`client/ship/`, and `client/diagnostics/` are Cargo packages. World owns portable coordinate
and camera state; the runtime drives native lifecycle and maps typed snapshots;
diagnostics aggregates and rasterizes the engineering view; the renderer owns
camera-relative GPU conversion, reverse-Z depth, `wgpu` resources, and
presentation. Assets owns the custom Task 7 ship's procedural DCC source,
Blender-importable glTF, packaged GLB, texture, metadata, and validation tools;
the renderer loads that checked-in GLB without depending on ship state. See
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

The run command opens the **Salimon — Compressed Solar System** native window and
continues until the window is closed. It captures the cursor for mouse look and
starts inside the landed ship facing the cockpit. Use **WASD** to walk, the mouse
to look, and **Space** to jump. Press **E** near the cockpit to enter or leave
control instantly; press **E** near the aft door to open/close it while landed.
Walk backward through the open door to transition over 0.25 seconds to Earth-radial
gravity and inspect the ship exterior. There is no sprint or crouch. Press
**Escape** to release the captured cursor for window controls; click the game
view to capture it again.

Press **F2** to switch to the engineering Solar System precision tour. Six textured spheres show the static
compressed bodies during the automatic far-space-to-Earth-surface transition.
The Sun is explicitly visual-only; the other five bodies have disjoint `1.15R`
landing volumes. Three separate meter-scale markers remain near Earth's positive-Z
surface for precision inspection. The spheres have original generated materials,
unshadowed Sun illumination, and an emissive Sun; there are no atmospheres,
clouds, dynamic shadows, or post effects.

In the precision tour, press **P** to pause or resume the automatic transition, **R** to restart it at
the selected body's far endpoint, and **N** to jump to the exact 12 m near-surface dwell and
pause there for inspection. Only the initial physical key press is acted on;
repeats and releases are ignored.

Press **1–6** to restart the approach for **Sun, Mercury, Venus, Earth, Moon,
Mars**, respectively. Earth is selected at launch. Selection is an engineering
camera fixture; it does not move bodies or implement flight/landing gameplay.

Press **F3** to toggle the engineering diagnostics overlay. It is hidden by
default and updates at a throttled cadence while frame observations continue to
be collected. The view includes FPS/frame time, real world-update and
CPU-side render time, GPU pass time when timestamp queries are supported, scene
and total draw/object counts, optional GPU allocator totals, camera
position/altitude/transition state, and memory-pressure warnings. Rows for
player/ship position and ship velocity/speed/thruster report live Task 8
snapshots in gameplay view. The nearby-body row reports the closest
of six nonnegative camera-to-surface observations.

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

1. Launch the client at 1920×1080 and confirm it starts inside the enlarged
   landed ship facing the cockpit. Walk and mouse-look around the cockpit/cabin,
   jump, and check the invisible interior collision boundaries. Confirm the
   20.30 × 7.44 × 16.60 m ship has no severe 0.05 m near-plane clipping.
2. Use E for instant cockpit entry/exit and to open the aft door. Walk outside,
   confirm the 0.25-second gravity transition is smooth, inspect the custom
   exterior/material variation, confirm its lowest point rests on Earth without
   a visible gap or penetration, then re-enter and close the door.
3. Press F2 and confirm the far view contains distinct textured spheres
   for exactly Sun, Mercury, Venus, Earth, Moon, and Mars, with visibly distinct
   compressed sizes and no other celestial bodies.
4. Let the fixture traverse far and near scales; confirm there is no visible
   position jitter, premature far clipping, depth inversion, or coplanar flicker
   in the intentionally separated markers.
5. Press N and confirm the exact 12 m Earth-surface view and its three markers remain
   stable while paused; press P to resume, then R and confirm the camera restarts
   at the far endpoint.
   Repeat with keys 2, 3, 5, and 6 to inspect Mercury, Venus, Moon, and Mars
   through their complete approach/retreat. Check the lit curved silhouettes,
   smooth material filtering, and surface detail at N; use 4 to return to Earth.
6. Press F3 and confirm the panel identifies Earth as the nearest body at the
   12 m dwell, while camera altitude/phase, CPU/GPU states, and scene counts
   update truthfully; hide it again without affecting the scene.
7. Resize repeatedly, including to a very small size, and confirm projection and
   depth follow the drawable size without a panic or validation error.
8. Minimize and restore, then confirm rendering/animation resume without a time
   jump or paused interval contaminating diagnostics.
9. Close the window, launch the client again, and confirm both shutdown and
   relaunch are clean.

During the Task 10 interior/exterior pass at fixed 1920×1080 on the reference M1
MacBook Air, enable F3 and confirm the overlay never reports below 60 FPS. Record
that machine-specific evidence in the Notion task before treating the performance
criterion as manually verified.

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
above. Task 6 adds textured sphere presentation, Sun lighting, and per-body
inspection. [Sphere rendering](client/renderer/sphere-rendering.md) records the
precision technique, LOD budget, material source, and future terrain path.
Task 11 loads Task 7's GLB through a renderer-owned mesh path and keeps behavior in
separate character/ship crates. Validate and regenerate its model using the
commands in the [ship asset documentation](client/assets/ship/README.md).
Orbital simulation, gameplay, persistence, networking, and backend behavior remain
outside this implementation.

Add future crates explicitly to the root workspace and inherit its package
metadata and lints. Keep `Cargo.lock` committed; validate normal changes with
`--locked`, and regenerate the lockfile intentionally when adding/updating
dependencies. Cargo's [workspace documentation](https://doc.rust-lang.org/cargo/reference/workspaces.html)
and [lockfile guide](https://doc.rust-lang.org/cargo/guide/cargo-toml-vs-cargo-lock.html)
describe these conventions.

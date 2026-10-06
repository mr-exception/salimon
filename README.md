# Salimon

Salimon is a space exploration game built around a custom Rust runtime. The
current implementation extends the **Phase 0 — Technical Feasibility Showcase**
with mining, physical fragments, carrying and EVA: a native-first, client-only
prototype using a custom `wgpu` renderer. See [current scope](docs/project-phases.md).
Native build scripts target
macOS, Windows, and Debian-based Linux; web remains a later target. The reference performance machine is an Apple M1 iMac
(`iMac21,1`, model `Z12X002L9GR/A`) with 8 CPU cores, 8 integrated GPU cores,
and 16 GB unified memory.

The custom scout has a broad living cabin, panoramic side/rear windows,
a central Energy Core, warm materials and detailed aft thrusters. Its compact
nose and shorter cockpit glazing keep the exterior silhouette streamlined.
One center cockpit console carries three physical monitors showing live
metric speed, thruster percentage, Core stored/capacity energy, and nearby-body
surface distance plus approaching/receding/zero radial speed. Routes on both sides
let the player walk past the console into the cockpit nose. Loose fragments can
be dropped on the cabin deck and pile up; the separate cargo module is removed. See
the [ship asset guide](client/assets/ship/README.md).
The native client starts inside the custom
Salimon scout landed on Earth, with portable character/ship state, runtime-loaded
GLB geometry, walking, free mouse look, jumping, cockpit interaction, a landed/open-space
door, and radial surface traversal.

## Source of requirements

The [project documents](docs/README.md) and
[GitHub issues](https://github.com/mr-exception/salimon/issues) are the source of
truth. Start with the [current scope](docs/project-phases.md),
[technical architecture](docs/technical-architecture.md), and the active issue's
linked contracts. Check an issue's
**Blocked by** section before work. [AGENTS.md](AGENTS.md) describes the
contributor workflow. The checked-in [Phase 0 evaluation](docs/phase-0-evaluation.md)
records the go-with-revisions decision, benchmark evidence, limitations, and
required follow-up before Phase 1.

## Repository boundaries

```text
salimon/
├── Cargo.toml          # Workspace metadata, members, and shared lints
├── Cargo.lock          # Committed dependency resolution
├── models/            # Offline 3D authoring workspace and shared contracts
├── core/
│   └── README.md       # Future backend boundary; no backend implementation
└── client/
    ├── runtime/        # salimon-client lifecycle and composition executable
    ├── math/           # Dependency-free shared f64 component arithmetic
    ├── physics/        # Portable fragment motion/contact rules
    ├── renderer/       # salimon-renderer GPU library and presentation
    ├── world/          # Catalog, coordinates, resource generation, mining and carrying
    ├── character/      # Portable first-person movement and gravity transitions
    ├── ship/           # Portable ship pose, cockpit authority, motion, and door state
    ├── platform/       # Future native window/input/platform adapters
    ├── assets/         # Runtime exports and legacy editable sources
    └── diagnostics/    # Engineering metrics and overlay rasterization
```

`client/runtime/`, `client/renderer/`, `client/world/`, `client/character/`,
`client/ship/`, `client/math/`, `client/physics/` and `client/diagnostics/` are Cargo packages. World owns portable coordinate
and camera state; the runtime drives native lifecycle and maps typed snapshots;
diagnostics aggregates and rasterizes the engineering view; the renderer owns
camera-relative GPU conversion, reverse-Z depth, `wgpu` resources, and
presentation. Assets owns the custom scout's checked-in glTF interchange,
packaged GLB, texture and runtime metadata; `models/` owns its Blender source,
export adapter and validation tools. The renderer loads that checked-in GLB without depending on ship state. See
[client/README.md](client/README.md) for the dependency boundaries.

The [3D authoring workspace](models/README.md) defines shared contracts for future
Blender-authored ships, resources, items, structures, props, characters, and
vehicles. Editable new sources belong in `models/`; validated runtime exports
belong in `client/assets/`. The scout uses Blender source with shared
export/validation and a ship category adapter that also derives spatial Rust
layouts from authored proxies/markers. Blender is optional for authoring and
is never required by normal client builds.

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
reporting validation or performance. The dated
[reference evaluation](docs/phase-0-evaluation.md) records Rust/Cargo 1.89.0
on native Apple Silicon macOS (`aarch64-apple-darwin`); it does not certify
other machines or current revisions. The first build may need network access to
download the locked `winit`
and `wgpu` dependency graph. Running the client needs no external account connection.

## Build and run

For standalone native debug/release executables on macOS, Windows, and Debian
Linux, use `python3 scripts/build_game.py --profile release` (`python` on
Windows). See [native build documentation](scripts/BUILDING.md) for prerequisites,
output locations, failure handling, and the cross-platform CI build matrix.

Run declarative native E2E scenarios with `scripts/salimon-test suite` or
`scripts/salimon-test run scenarios/landed-earth.json`; see
[runner documentation](scripts/README.md) for the scenario format and JSON results.
Use `scripts/salimon-test run scenarios/cockpit-nose.json` to check the new
walking routes on both sides and unified console.

Run these commands from the repository root:

```sh
cargo build --workspace --locked
cargo run --locked -p salimon-client
```

The run command opens the **Salimon — Compressed Solar System** native window at
a requested physical 1920×1080 drawable size and continues until the
window is closed. It captures the cursor for mouse look and starts inside the
landed ship facing the cockpit. Use **WASD** to walk, the mouse to look, and
**Space** to jump. During open-space EVA, **WASD** translates relative to the
view (including pitch), **Space** ascends, and **Left Shift** descends. Release
translation input to stop assisted relative movement while retaining inherited
ship velocity. Press **E** near the cockpit to enter or leave
control instantly; press **E** near the aft door to open/close it while landed or in open space beyond the nearby-body threshold.
Press **M** to equip the handheld mining tool on the surface; aim at a deposit
within 4 m and hold **F** or left mouse to extract at 2 kg/s. Press M to stow.
Aim at a physical fragment within 3 m and press **E** to pick it up. You can
carry one world object at a time; equipped gear remains separate. Press **E**
again to release it from your hand. Gravity and contact make fragments fall and pile up.
Face the open door and walk forward through it to transition over 0.25 seconds
to the active solid body's radial gravity and inspect the ship exterior. There is no sprint or crouch. Press
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
player/ship position and ship velocity/speed/thruster report live domain
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

1. Launch the client and confirm its initial drawable is 1920×1080 and it starts inside the
   landed ship facing the cockpit. Inspect the warm cabin, walk both routes
   around the central Energy Core, and look through the side/rear windows and
   forward from behind the lowered chair. Follow the routes on both sides past the
   single center console into the shortened cockpit nose. Inspect the three
   monitors on that console: speed/power, Core energy, and nearby-body state.
   Check seated forward and downward visibility through the shortened glazing,
   and inspect the exterior silhouette without the former port cargo module.
   Check 0%, 1%, and 100%
   during flight; cross the 3 Mm nearby threshold; confirm approaching, receding,
   zero, and out-of-range states; and confirm readings remain live after leaving
   the seat. Jump and check the
   invisible interior collision boundaries. Confirm the shortened nose has no
   severe 0.05 m near-plane clipping.
2. Use E for instant cockpit entry/exit and to open the aft door. Face the door
   and hold W to walk outside; confirm movement stays outward throughout the
   doorway blend and does not pull you back inside. Also check sideways and
   backward crossings and re-entry while moving toward the cabin. Outside,
   push against both sides, the nose, and rear windows with the gate open:
   all remain solid. Slide around an aft corner and re-enter through the gate;
   confirm the jambs still require full body clearance.
   Confirm the 0.25-second gravity transition is smooth, inspect the custom
   exterior/material variation, confirm its lowest point rests on Earth without
   a visible gap or penetration, then re-enter and close the door.
3. From cockpit control, approach Mercury, Venus, Earth, Moon, and Mars from
   representative non-polar directions. Confirm `Press L to land` appears only
   inside `1.15R`; press L at high speed/poor orientation, leave the cockpit,
   and confirm landing still aligns and completes at the approached surface.
   Walk outside on each body and confirm radial full-sphere movement. Return,
   leave the door open, and confirm L reports `Close door before takeoff`; close
   it, start takeoff, leave the cockpit, and confirm the ship clears the landing
   volume before normal flight resumes without a loading screen.
4. Press F2 and confirm the far view contains distinct textured spheres
   for exactly Sun, Mercury, Venus, Earth, Moon, and Mars, with visibly distinct
   compressed sizes and no other celestial bodies.
5. Let the fixture traverse far and near scales; confirm there is no visible
   position jitter, premature far clipping, depth inversion, or coplanar flicker
   in the intentionally separated markers.
6. Press N and confirm the exact 12 m Earth-surface view and its three markers remain
   stable while paused; press P to resume, then R and confirm the camera restarts
   at the far endpoint.
   Repeat with keys 2, 3, 5, and 6 to inspect Mercury, Venus, Moon, and Mars
   through their complete approach/retreat. Check the lit curved silhouettes,
   smooth material filtering, and surface detail at N; use 4 to return to Earth.
7. Press F3 and confirm the panel identifies Earth as the nearest body at the
   12 m dwell, while camera altitude/phase, CPU/GPU states, and scene counts
   update truthfully; hide it again without affecting the scene.
8. Resize repeatedly, including to a very small size, and confirm projection and
   depth follow the drawable size without a panic or validation error.
9. Minimize and restore, then confirm rendering/animation resume without a time
   jump or paused interval contaminating diagnostics.
10. Close the window, launch the client again, and confirm both shutdown and
   relaunch are clean.

For performance evaluation, use a release build at fixed 1920×1080 on the
reference Apple M1 iMac and enable F3. Record FPS, average/p95 presented-frame
intervals, CPU/GPU timings, scenario, drawable size, machine, and revision in
the task report. The overlay reports rolling statistics and cannot prove a
per-frame 60 FPS floor. The dated [Phase 0 evaluation](docs/phase-0-evaluation.md)
did not establish that strict historical target; the smoke check validates
behavior, while performance claims require evidence for the measured revision.
See [validation policy](docs/validation.md#native-graphical-gates).

## Development workflow

Use the [ownership map](docs/maintenance-map.md) to route changes, follow the
[coding conventions](docs/coding-conventions.md), and run the affected checks in
the [validation matrix](docs/validation.md), including Python/model checks.
Store task outcomes and applicable images in [reports/](reports/README.md).

Read the current GitHub issue and its linked specifications before changing
behavior. Keep current client implementation under `client/`; leave `core/` as a
documentation-only boundary.

```sh
cargo fmt --all -- --check
cargo clippy --workspace --all-targets --locked -- -D warnings
cargo test --workspace --locked
```

Use `cargo fmt --all` to apply formatting. Automated tests cover logic that does
not require a live native surface; the native launch, drawing, overlay toggle,
resize, minimize, restore, and close behavior still require the smoke check
above. The renderer provides textured sphere presentation, Sun lighting, and
per-body inspection. [Sphere rendering](client/renderer/sphere-rendering.md) records the
precision technique, LOD budget, material source, and future terrain path.
The runtime loads the authored scout GLB through a renderer-owned mesh path and
keeps behavior in separate character/ship crates. Validate and regenerate its model using the
commands in the [ship asset documentation](client/assets/ship/README.md).
Mining, carrying, fragment motion/contact, ship flight and assisted landing,
airlock access, and EVA are implemented. Modified deposits and physical fragments
are retained in memory across local streaming during the current session.
Orbital simulation, disk/backend persistence, networking, survival, and production
energy management remain deferred.

Add future crates explicitly to the root workspace and inherit its package
metadata and lints. Keep `Cargo.lock` committed; validate normal changes with
`--locked`, and regenerate the lockfile intentionally when adding/updating
dependencies. Cargo's [workspace documentation](https://doc.rust-lang.org/cargo/reference/workspaces.html)
and [lockfile guide](https://doc.rust-lang.org/cargo/guide/cargo-toml-vs-cargo-lock.html)
describe these conventions.

# Client

All Phase 0 implementation lives here. Tasks 2–11 provide the `salimon-client`
runtime binary, `salimon-renderer` GPU library, `salimon-diagnostics`
metrics/overlay library, and portable `salimon-world` compressed Solar System,
coordinate/camera model, portable character and ship domains, and the custom
ship asset/runtime mesh path. Add future Cargo packages explicitly to the root workspace.

## Ownership

| Boundary | Current responsibility |
| --- | --- |
| `runtime/` | Native lifecycle, redraw/update scheduling, typed key routing, timing, and client composition |
| `renderer/` | `wgpu` resources, camera-relative conversion, reverse-Z scene/ship/overlay pipelines, measurements, and presentation |
| `world/` | Immutable six-body compressed Solar System, portable `f64` coordinates, camera state, geometry math, and precision markers |
| `character/` | First-person state, typed movement, fixed gravity, cockpit/doorway/surface traversal |
| `ship/` | Ship pose, flight/landing/takeoff state, cockpit authority, persistent motion, and landed-only door rules |
| `platform/` | Reserved for native input and OS-specific adapters |
| `assets/` | Validated runtime exports/metadata and legacy editable sources; new 3D sources belong in root `models/` |
| `diagnostics/` | Engineering metric aggregation, formatting, and RGBA overlay rasterization |

The current dependency direction is:

```text
salimon-client (runtime, winit lifecycle, clocks, snapshot mapping)
    ├── salimon-character (portable first-person movement and gravity frames)
    ├── salimon-ship (portable ship interaction and persistent motion state)
    ├── salimon-world (static Solar System, f64 coordinates, and camera prototype)
    ├── salimon-diagnostics (typed observations and optional overlay image)
    └── salimon-renderer (wgpu resources, generic overlay composition, presentation)
```

World keeps absolute body, camera, and marker coordinates in `f64` meters without
GPU or window types. Its static catalog contains exactly Sun, Mercury, Venus,
Earth, Moon, and Mars; the Sun is visual-only, and the five solid bodies expose
disjoint `1.15R` landing volumes. The runtime creates the window, routes lifecycle
and resize events, schedules redraws/updates, maps P/R/N/1–6 into typed world commands,
routes F2/F3 and gameplay input (including contextual L landing/takeoff), composes typed character/ship snapshots, and converts
domain state into renderer DTOs. The renderer subtracts the camera origin in `f64`, uploads only
camera-relative `f32` data, and presents with infinite-far reverse-Z depth.
Recoverable surface loss is handled by the runtime without discarding world
state. Its ship mesh loader consumes the custom checked-in GLB and remains
independent of ship behavior. Diagnostics remains observational, and no supporting crate owns the event
loop or unrelated authoritative gameplay state.

`winit` integration currently lives at the runtime boundary, which is permitted
for this native bootstrap. Move OS-specific behavior into `platform/` as that
behavior grows or another native target needs an adapter. Future domain modules
must consume typed input/presentation data rather than GPU or window types.
There is no dependency on `core/` in Phase 0.

Tasks 2–11 deliberately add no ECS, physics engine, WASM host,
orbital simulation, or backend scaffolding. Task 6 adds renderer-owned analytic
spheres, generated mipmapped textures, and Sun illumination. Task 4 camera
telemetry is explicitly separate from future player/ship state; Task 5 supplies
camera-to-body surface observations without presenting the camera as gameplay
state. As major components gain behavior, maintain the contracts, ownership
documentation, architecture, invariants, and relevant validation required by the
[technical architecture](../docs/technical-architecture.md).

Run the root [development workflow](../README.md#development-workflow) after
changes. Stable Rust, workspace lints, and the root lockfile apply to all client
crates.

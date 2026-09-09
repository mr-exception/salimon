# Client

All Phase 0 implementation lives here. Tasks 2–4 provide the `salimon-client`
runtime binary, `salimon-renderer` GPU library, `salimon-diagnostics`
metrics/overlay library, and portable `salimon-world` coordinate/camera
prototype. The remaining directories are documented ownership boundaries until
their implementation tasks begin. Add future Cargo packages explicitly to the
root workspace.

## Ownership

| Boundary | Current responsibility |
| --- | --- |
| `runtime/` | Native lifecycle, redraw/update scheduling, typed key routing, timing, and client composition |
| `renderer/` | `wgpu` resources, camera-relative conversion, reverse-Z scene/overlay pipelines, measurements, and presentation |
| `world/` | Portable `f64` coordinates, camera state, and renderer-neutral Task 4 precision fixture |
| `character/` | Reserved for first-person character state and movement |
| `ship/` | Reserved for ship state, cockpit control, flight, and landing/takeoff |
| `platform/` | Reserved for native input and OS-specific adapters |
| `assets/` | Reserved for editable source art and exported game-ready content |
| `diagnostics/` | Engineering metric aggregation, formatting, and RGBA overlay rasterization |

The current dependency direction is:

```text
salimon-client (runtime, winit lifecycle, clocks, snapshot mapping)
    ├── salimon-world (portable f64 coordinates and camera prototype)
    ├── salimon-diagnostics (typed observations and optional overlay image)
    └── salimon-renderer (wgpu resources, generic overlay composition, presentation)
```

World keeps absolute camera and fixture coordinates in `f64` meters without GPU
or window types. The runtime creates the window, routes lifecycle and resize
events, schedules redraws/updates, maps P/R/N into typed world commands, routes
F3, and converts the renderer-neutral world snapshot into the renderer's generic
scene DTOs. The renderer subtracts the camera origin in `f64`, uploads only
camera-relative `f32` data, and presents with infinite-far reverse-Z depth.
Recoverable surface loss is handled by the runtime without discarding world
state. Diagnostics remains observational, and no supporting crate owns the event
loop or unrelated authoritative gameplay state.

`winit` integration currently lives at the runtime boundary, which is permitted
for this native bootstrap. Move OS-specific behavior into `platform/` as that
behavior grows or another native target needs an adapter. Future domain modules
must consume typed input/presentation data rather than GPU or window types.
There is no dependency on `core/` in Phase 0.

Tasks 2–4 deliberately add no ECS, physics engine, WASM host, gameplay state,
celestial-body catalog, planet renderer, or backend scaffolding. Task 4 camera
telemetry is explicitly separate from future player/ship state; those inputs and
body distances remain unavailable until their owning tasks. As major components
gain behavior, maintain the contracts, ownership documentation, architecture,
invariants, and relevant validation required by the
[Notion architecture](https://app.notion.com/p/3d5b456853b981078a82c68207f4444e).

Run the root [development workflow](../README.md#development-workflow) after
changes. Stable Rust, workspace lints, and the root lockfile apply to all client
crates.

# Client

All Phase 0 implementation lives here. Tasks 2 and 3 provide the
`salimon-client` runtime binary, `salimon-renderer` GPU library, and
`salimon-diagnostics` metrics/overlay library. The remaining directories are
documented ownership boundaries until their implementation tasks begin. Add
future Cargo packages explicitly to the root workspace.

## Ownership

| Boundary | Current responsibility |
| --- | --- |
| `runtime/` | Native lifecycle, redraw scheduling, frame timing, F3 routing, and client composition |
| `renderer/` | `wgpu` resources, bootstrap/overlay pipelines, measurements, and presentation |
| `world/` | Reserved for portable scene data, coordinates, and world state |
| `character/` | Reserved for first-person character state and movement |
| `ship/` | Reserved for ship state, cockpit control, flight, and landing/takeoff |
| `platform/` | Reserved for native input and OS-specific adapters |
| `assets/` | Reserved for editable source art and exported game-ready content |
| `diagnostics/` | Engineering metric aggregation, formatting, and RGBA overlay rasterization |

The current dependency direction is:

```text
salimon-client (runtime, winit lifecycle, frame clock)
    ├── salimon-diagnostics (typed observations and optional overlay image)
    └── salimon-renderer (wgpu resources, generic overlay composition, presentation)
```

The runtime creates the window, routes lifecycle and resize events, schedules
redraws, maintains monotonic frame timing, routes F3, and maps runtime/renderer
measurements into diagnostics through typed values. It supplies the resulting
borrowed RGBA image through the renderer's narrow `new`/`resize`/`render`
boundary. Recoverable surface loss is handled by the runtime through that
boundary. Neither supporting crate owns the event loop or authoritative
world/game state.

`winit` integration currently lives at the runtime boundary, which is permitted
for this native bootstrap. Move OS-specific behavior into `platform/` as that
behavior grows or another native target needs an adapter. Future domain modules
must consume typed input/presentation data rather than GPU or window types.
There is no dependency on `core/` in Phase 0.

Tasks 2 and 3 deliberately add no ECS, physics engine, WASM host, gameplay state,
or backend scaffolding. Future domain tasks populate the diagnostics crate's
optional player, ship, and body-distance inputs; until then those values remain
explicitly unavailable. As major components gain behavior, maintain the
contracts, ownership documentation, architecture, invariants, and relevant
validation required by the
[Notion architecture](https://app.notion.com/p/3d5b456853b981078a82c68207f4444e).

Run the root [development workflow](../README.md#development-workflow) after
changes. Stable Rust, workspace lints, and the root lockfile apply to all client
crates.

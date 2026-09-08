# Client

All Phase 0 implementation lives here. Task 2 provides the `salimon-client`
runtime binary and the `salimon-renderer` library. The remaining directories are
documented ownership boundaries until their implementation tasks begin. Add
future Cargo packages explicitly to the root workspace.

## Ownership

| Boundary | Current responsibility |
| --- | --- |
| `runtime/` | Native application lifecycle, redraw scheduling, frame timing, and client composition |
| `renderer/` | `wgpu` device/surface resources, bootstrap pipeline, and presentation |
| `world/` | Reserved for portable scene data, coordinates, and world state |
| `character/` | Reserved for first-person character state and movement |
| `ship/` | Reserved for ship state, cockpit control, flight, and landing/takeoff |
| `platform/` | Reserved for native input and OS-specific adapters |
| `assets/` | Reserved for editable source art and exported game-ready content |
| `diagnostics/` | Reserved for engineering metrics, profiling, and the optional overlay |

The current dependency direction is:

```text
salimon-client (runtime, winit lifecycle, frame clock)
    └── salimon-renderer (wgpu resources and presentation)
```

The runtime creates the window, routes lifecycle and resize events, schedules
redraws, maintains monotonic frame timing, and invokes the renderer through its
narrow `new`/`resize`/`render` boundary. Recoverable surface loss is handled by
the runtime through that boundary. The renderer does not own the event loop or
authoritative world/game state.

`winit` integration currently lives at the runtime boundary, which is permitted
for this native bootstrap. Move OS-specific behavior into `platform/` as that
behavior grows or another native target needs an adapter. Future domain modules
must consume typed input/presentation data rather than GPU or window types.
There is no dependency on `core/` in Phase 0.

Task 2 deliberately adds no ECS, physics engine, WASM host, diagnostics overlay,
or backend scaffolding. Task 3 owns visible FPS and profiling diagnostics. As
major components gain behavior, maintain the contracts, ownership documentation,
architecture, invariants, and relevant validation required by the
[Notion architecture](https://app.notion.com/p/3d5b456853b981078a82c68207f4444e).

Run the root [development workflow](../README.md#development-workflow) after
changes. Stable Rust, workspace lints, and the root lockfile apply to all client
crates.

# Client

All Phase 0 implementation lives here. `runtime/` currently contains the
`salimon-client` Cargo binary; the other directories are documented ownership
boundaries, not implemented libraries. Add Cargo packages or ordinary Rust
modules as their tasks introduce behavior, keeping the root workspace explicit.

## Ownership

| Boundary | Responsibility when implemented |
| --- | --- |
| `runtime/` | Application lifecycle and orchestration of the client modules |
| `renderer/` | GPU resources, rendering, materials, and presentation |
| `world/` | Portable scene data, coordinates, and world state |
| `character/` | First-person character state and movement |
| `ship/` | Ship state, cockpit control, flight, and landing/takeoff |
| `platform/` | Native window/input and OS-specific adapters |
| `assets/` | Editable source art and exported game-ready content |
| `diagnostics/` | Engineering metrics, profiling, and optional overlay |

The runtime composes capabilities through explicit typed interfaces. Domain
modules must not depend on GPU/window types; the renderer consumes presentation
data rather than owning authoritative world or gameplay state. Keep platform
calls behind adapters so domain source can later compile for native and web
targets. There is no dependency on `core/` in Phase 0.

Task 2 will add native windowing and `wgpu` initialization. Task 1 deliberately
has no renderer dependency, ECS, physics engine, WASM host, or backend scaffolding.
As major components gain behavior, add the contracts, ownership documentation,
architecture, invariants, and relevant validation required by the
[Notion architecture](https://app.notion.com/p/3d5b456853b981078a82c68207f4444e).

Run the root [development workflow](../README.md#development-workflow) after
changes. Stable Rust, workspace lints, and the root lockfile apply to all future
client crates.

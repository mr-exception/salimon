# Technical architecture and AI maintenance

## Implemented architecture

Salimon uses a custom Rust runtime with low-level `winit` and `wgpu`, without a
full game engine. Phase 0 is a native, client-only prototype. Build scripts and
CI stage macOS, Windows and Debian-based Linux executables; graphical CI runs
on Linux software Vulkan. Native macOS is the reference playable/performance
target. Web/server reuse remains a future capability, not a shipped target.

The [feature maintenance map](maintenance-map.md) routes work to source, tests,
invariants and scenarios. The [client guide](../client/README.md) defines package
dependencies; each crate's architecture/invariants defines its local contracts.

| Boundary | Implemented ownership |
| --- | --- |
| `client/runtime/` | Native lifecycle, input, clocks, domain composition, automation, resource presentation and cross-domain fragment contact simulation |
| `client/world/` | Static six-body compressed catalog, `f64` coordinates/camera tour, resource contracts, generation, extraction/session deltas, fragments and one-object carrying |
| `client/ship/` | Pose, cockpit authority, direct flight, uncancellable landing/takeoff, airlock interlocks and live telemetry |
| `client/character/` | Portable movement, collision, gravity, cockpit transitions and open-space/nearby-body EVA |
| `client/renderer/` | GPU resources, GLB loading, sphere/ship/image pipelines, reverse-Z, instruments and measurements through renderer-neutral DTOs |
| `client/diagnostics/` | Observational metrics, formatting and overlay rasterization |
| `models/` and `client/assets/` | Offline authored sources/contracts and checked-in validated runtime exports respectively |
| `client/platform/` and `core/` | Reserved adapter/backend boundaries; no backend implementation |

Runtime depends on supporting crates and maps snapshots into presentation DTOs;
supporting crates do not call runtime. Character has no ship-crate dependency;
ship may consume world contracts. Renderer does not depend on authoritative
world, character or ship state. Native window/input types stay at the composition
boundary; GPU types stay in renderer. Typed Rust contracts currently provide
module boundaries, not dynamically loaded WASM components.

Implemented gameplay includes surface walking, ship flight/assists, airlock/EVA,
resource mining, one-fragment carrying, planet/ship transfer and custom fragment
motion/contact. Mining session deltas and physical fragments survive nearby
streaming **in memory for the current session**. This is not disk/backend
persistence. No orbital simulation, multiplayer, production survival or energy
management is implemented. Core energy is a bounded telemetry fixture.

## Accepted constraints

- No full game engine. Keep low-level libraries and explicit domain ownership.
- Portable domain source uses typed commands, validated state and snapshots;
  platform capabilities and presentation do not leak into domain contracts.
- CPU absolute coordinates remain `f64` meters. Subtract camera origin in `f64`
  before narrowing to GPU `f32`; never subtract universe-scale `f32` positions
  in WGSL. Renderer uses infinite-far reverse-Z `Depth32Float`, clear 0 and
  greater-depth comparison with an explicit near plane. See the
  [coordinate strategy](../client/world/coordinate-strategy.md).
- World/runtime axes are right-handed +Y up. Ship local +X is forward and -Z
  starboard. Surface gravity uses the selected body's local radial direction,
  not global +Y. Blender source is +Z up; export converts once. See
  [asset contracts](../models/contracts.md).
- The static catalog and tour are compressed validation fixtures, not realistic
  orbital simulation. Follow [catalog invariants](../client/world/invariants.md)
  rather than treating tour tuning as a product requirement.
- Preserve deterministic identities/update semantics, one-object carrying and
  uncancellable assists. Changes must update owning invariants and tests.
- Profile before optimizing or adding shared-memory coupling. Linux software
  rendering does not establish the reference Apple M1 iMac's hardware budget.

## Authored asset boundary

The scout migration (#79–#83) is complete. Blender
[`source.blend`](../models/assets/ships/salimon-scout/README.md) owns editable
geometry/materials; the ship category adapter uses shared export/validation.
The GLB is the runtime artifact, with matching glTF/bin interchange, generated
spatial sidecar and character layouts. Required proxy/marker nodes are nonvisual
metadata, while gameplay policy remains in Rust. Preserve named groups, axes,
monitor UVs, door metadata, budgets and licenses. Read the
[scout regeneration guide](../client/assets/ship/README.md#source-and-regeneration)
for commands and generated files; normal builds never require Blender.

The current scout removes the dedicated cargo module, shortens the nose/glazing
and uses three monitors on one center console with walking routes on both sides.
Loose fragments use the cabin deck. Historical cargo and migration evidence in
[reports/](../reports/README.md) describes earlier revisions. Modular ship
assembly and resource/tool Blender presentation work remain issue-scoped plans;
they are not implemented by this documentation update.

## Future candidates and deferred capabilities

These directions require their own issue, explicit contracts and validation;
they are not dependencies or scaffolding requirements for current work:

| Direction | Status and decision boundary |
| --- | --- |
| Web/WASM and WIT Component Model | Portable Rust reuse and versioned capability interfaces where useful; no component host or WIT boundary currently shipped |
| ECS | Standalone `bevy_ecs` may be evaluated; no canonical ECS or Bevy engine adoption |
| Physics | Rapier remains the preferred candidate for evaluation; current fragments use custom deterministic motion/contact |
| Workers/scheduler/event bus | Evaluate for measured simulation needs; no worker infrastructure required by current native composition |
| Streaming/storage | Larger spatial hierarchies and storage interfaces remain future work; IndexedDB is the browser cache direction, native storage needs a platform implementation |
| Networking/server | Authoritative multiplayer server and shared deterministic domain reuse are future architecture |
| Survival/economy/crafting/NPCs | Product directions, not implemented crates or interfaces |

Future public component boundaries should declare capabilities, errors,
compatibility and versioning. Shared buffers need profiling and documented
ownership; unrestricted cross-module shared memory is not the default. Do not
add a framework or runtime service solely because it appears in this table.

## Maintenance and history

Use [AGENTS.md](../AGENTS.md) for GitHub issue/blocker workflow and precedence,
[coding conventions](coding-conventions.md) for implementation/report rules, and
[validation](validation.md) for current checks. Each major implemented crate
maintains its public Rust contract, `README.ai.md`, `architecture.md`, invariants
and focused tests. Update affected contracts, consumers and guides together.

Project specifications and [Project Q&A](project-qa.md) capture product decisions.
[Phase 0 evaluation](phase-0-evaluation.md) retains the dated go-with-revisions
assessment and machine-specific benchmark limits. Older task reports and
[legacy hub notes](legacy-salimon-hub.md) are historical context; GitHub issues
replace the former Notion task database and worker-selection instructions.
The maintenance goal is a focused change whose owners, contracts and useful
checks can be identified without reading unrelated domains.

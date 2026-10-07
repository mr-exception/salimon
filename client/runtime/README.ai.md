# Runtime AI Maintenance Guide

## Purpose

`salimon-client` is the native composition executable. Its reasoning scope is
application lifecycle, event routing, redraw scheduling, frame timing, and
coordination of client capabilities. It is not a gameplay domain.

## Read before changing

1. Fetch the active GitHub issue, its linked contracts, and current technical
   architecture as required by the root [AGENTS.md](../../AGENTS.md).
2. Read [architecture.md](architecture.md) and [invariants.md](invariants.md).
3. Read the renderer's [maintenance guide](../renderer/README.ai.md) before
   changing runtime/renderer interaction.

## Ownership

The runtime owns the `winit` application handler, native window lifetime,
renderer orchestration, resize and redraw routing, surface-loss recovery policy,
monotonic frame/update clocks, typed input routing, and client composition. Its
outbound dependencies are `salimon-character`, `salimon-ship`, `salimon-world`,
`salimon-physics`, `salimon-renderer`, and `salimon-diagnostics`.

Do not add GPU pipelines/resources, authoritative world or ship state, backend
behavior, persistence, or networking here. Keep new platform-specific behavior
small in `app/native.rs`. Introduce a separate platform boundary only when a
real second implementation requires it.

## Source routing

Start in `app.rs` for lifecycle, recovery, update order or automation dispatch.
Use its private `app/input.rs`, `interaction.rs`, `frames.rs`, `scene.rs`,
`diagnostics.rs` and `native.rs` for the corresponding translation helpers and
local regression tests. See the [module ownership table](architecture.md#internal-composition-modules).
Keep cross-domain update sequencing visible in `app.rs`; a helper extraction must
not reorder it or move domain rules into runtime mapping.

## Change checklist

- Preserve the lifecycle and timing invariants.
- Translate native keys/mouse motion into typed domain commands; never pass
  `winit` events to portable code.
- Keep camera state through renderer rebuilds while resetting the monotonic
  update interval across lifecycle discontinuities.
- Keep the renderer call surface narrow and typed.
- Keep contextual gameplay prompts in the runtime-owned action-bar state. Map
  typed domain messages to text here, keep transient expiry out of gameplay
  domains, and send only borrowed RGBA pixels plus typed screen/world placement
  to the renderer. Object anchors retain absolute `f64` coordinates; authored
  ship markers and selected resource poses are canonical sources.
- Map current ship snapshot speed/thruster data into `CockpitInstruments` on
  every gameplay frame, including after cockpit exit. Preserve ship-domain
  telemetry semantics and keep screen drawing in the renderer.
- Field-map Core energy and optional nearby-body distance/radial telemetry from
  the same snapshot. Do not select bodies or derive velocity in runtime.
- Map all six world bodies to spheres, the Sun to a point light, and the three
  noncanonical markers to separate cuboids; preserve absolute `f64` centers
  and sphere radii. Keep material-style selection at the composition boundary.
- Keep diagnostics observational: map typed snapshots at the composition root
  and report real camera-to-body surface distances plus live typed player/ship
  values in gameplay view.
- Add or update deterministic tests for non-GUI logic.
- Keep the E2E JSON reader isolated from game state; execute its commands on the
  native event thread and preserve production gates for action commands.
- Run all root build/format/lint/test gates.
- Perform the root native smoke check for lifecycle or rendering changes.
- Update this guide and the component architecture/invariants when ownership or
  behavior intentionally changes.

F grab/drop priority and independent keyboard/mouse held state are runtime
composition in `app.rs`/`app/input.rs` and `mining.rs`; see runtime invariants.
Automation must share that route. Mining input/presentation is runtime composition; keep target validation, rate,
source mass mutation, and session state in `salimon_world::mining`.

## Shared maintenance rules

Follow the root [coding conventions](../../docs/coding-conventions.md),
[validation matrix](../../docs/validation.md) and
[feature map](../../docs/maintenance-map.md). Update affected contracts/guides
with behavior changes and record completion evidence under root `reports/`.

Loose-object physical rules belong in `salimon-physics`. Runtime selects session
entities/environment, supplies geometry and sequences the solver; keep restitution,
gravity, contact response and substeps out of the native adapter. See the
[ownership decision](../../docs/technical-architecture.md#physical-object-simulation-decision-114).

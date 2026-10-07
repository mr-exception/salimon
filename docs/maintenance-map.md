# Feature maintenance map

Start with the affected row, then read that crate's `README.ai.md`,
`architecture.md` and `invariants.md`. Paths below are repository-root-relative;
Rust tests are generally inline `#[cfg(test)]` modules. Run focused checks while
iterating and the [applicable validation gates](validation.md) before completion.

| Feature | Owner and source entry points | Contracts/tests | Native scenarios |
| --- | --- | --- | --- |
| Lifecycle, resize, recovery, clocks | runtime `src/app.rs`, `app/native.rs`, `frame_clock.rs`, `update_clock.rs`, `main.rs` | runtime invariants; inline clock/native tests | `landed-earth.json`, `orbit-earth.json`; root native smoke for resize/minimize/close |
| Rendering, depth, lighting, instruments | renderer `src/lib.rs`, `spheres.rs`, `ship_mesh.rs`, `cockpit_instruments.rs`, `overlay.rs`, adjacent WGSL | renderer invariants, `sphere-rendering.md`, inline tests and CPU/shader layout | `lower-cockpit-windows.json`; evidence variants and visual tour |
| Shared vector arithmetic | math `src/lib.rs`; character/ship and runtime fragment/carrying consumers | math architecture/invariants; arithmetic/handedness/precision tests; domain normalization/quaternion tests | Existing movement, flight and fragment scenarios |
| Coordinates, catalog, landing geometry | world `src/lib.rs` facade; `coordinates.rs`, `catalog.rs`, `geometry.rs`, `proximity.rs` | world invariants, `solar-system-layout.md`, `coordinate-strategy.md`, inline catalog/coordinate/geometry/proximity tests | `orbit-earth.json`; F2/N/1–6 precision tour |
| Precision fixtures and engineering camera | world `src/precision.rs`, `camera.rs`; root public reexports in `lib.rs` | world coordinate strategy/invariants; inline precision and deterministic camera-tour tests | F2/N/1–6 precision tour |
| Flight, assists, cockpit authority, airlock | ship `src/lib.rs` facade, `state.rs`, `orientation.rs`, `controller/{mod,flight,steering,assist,door,cockpit,telemetry}.rs`; runtime `app.rs`, `app/input.rs`, `app/interaction.rs`, `app/scene.rs`, `action_bar.rs` compose | ship architecture/invariants, inline motion/assist/door tests | `orbit-earth.json`, `space-airlock.json`, `lower-cockpit-windows.json` |
| Walking, gravity, hull collision, EVA | character `src/controller/{mod,interior,doorway,surface,eva,camera}.rs`, `collision.rs`, `layout.rs`, `queries.rs`; runtime `app.rs`, `app/frames.rs` compose ship snapshots | character architecture/invariants; mode-local collision/gravity/EVA tests and `controller/collision_tests.rs`; generated `spatial_contracts.rs` | `cockpit-nose.json`, `space-airlock.json`, `moving-eva.json`, `nearby-eva.json` |
| Resource identities/generation/streaming | world `src/resources.rs`, `resource_distribution.rs`, `resource_generation.rs`, `mining.rs` | `resource-contracts.md`, world invariants; inline generation/session tests | `resource-deposits.json`, `resource-streaming.json` |
| Mining and resource UI | world `src/mining.rs`, `resource_fragments.rs`; runtime `mining.rs`, `resource_context.rs`, `resource_presentation.rs`, `action_bar.rs`; renderer `held_item.rs` / `held_item.wgsl` | world extraction/mass tests; runtime input/presentation and scenario contracts | `mining.json`, `resource-loop.json`; evidence `mining-tool.json` |
| Carrying, transfer, fragment motion/contact | world `src/carrying.rs`, `mining.rs`, `resource_fragments.rs`; runtime `app.rs` F priority, `mining.rs` held input, `carrying.rs`, `fragment_physics.rs` adapters; physics `src/lib.rs` rules | world identity/mass/one-object invariants; physics motion/contact/step tests; runtime frame/placement/session tests | `carrying.json`, `fragment-transfer.json`, `resource-loop.json` |
| Automation, staging, packaged input | runtime `src/automation.rs`, `e2e.rs`; `scripts/salimon_test.py`, `build_game.py`, `packaged_smoke.py` | runtime protocol/scenario tests; `scripts/test_*.py`; scripts README/BUILDING/PACKAGED_SMOKE | default suite, `--group resource-collection`, `--group ship-eva`; packaged smoke |
| Asset authoring and spatial contracts | `models/tools/`, `models/assets/ships/salimon-scout/{export,validate,spatial_contracts}.py`; renderer `ship_mesh.rs`; generated character layouts | models contracts/manifest, scout preservation metadata, `models/tests/`, asset README | cockpit/door/lower-window routes and applicable evidence variants |
| Observational diagnostics | diagnostics `src/lib.rs` facade/DTOs, `aggregation.rs` window/cadence/statistics, `format.rs` rows/units, `raster.rs` font/RGBA; runtime `app/diagnostics.rs` snapshot/frame mapping | diagnostics architecture/invariants and owner-local tests | F3 native smoke; never authoritative gameplay state |

All `src/` paths in the table are under the named `client/<crate>/` unless stated
otherwise. Scenario paths are under `scenarios/`; screenshots are in corresponding
`scenarios/evidence/` routes. Evidence proves a recorded revision, not future
behavior or hardware performance.

## Broad source files

`runtime/src/app.rs` coordinates native events, update ordering, domain snapshots,
automation dispatch and renderer recovery. Private `runtime/src/app/{input,interaction,frames,scene,diagnostics,native}.rs`
own key translation, interaction gates/prompts, frame adapters, renderer DTO
mapping, observational diagnostics and window/cursor helpers respectively.
Helper tests live with these owners; update order remains in `app.rs`.
Clocks, E2E protocol, mining/carrying,
fragment frame/session adapters and resource presentation have separate internal modules.
Physical-object rules live in `physics/src/lib.rs`, not the native composition layer.
Route changes to these modules before growing the event-handler composition.

`character/src/lib.rs` is the public facade. Typed controls and environmental
contracts live in `input.rs` and `state.rs`. `controller/mod.rs` owns private
state and update dispatch; `controller/{interior,doorway,surface,eva,camera}.rs`
own mode behavior and look/snapshots, with adjacent regression tests.
`layout.rs`, `collision.rs`, `queries.rs` and `math.rs` own shared geometry,
sweep/sliding, sight/placement and character vector policy. Controller collision
regressions and shared fixtures are in `controller/collision_tests.rs` and
`controller/test_support.rs`. Generated anchor/engine layouts remain at their
existing paths and come from the scout exporter. Keep portable movement here
and native events in runtime.

`ship/src/lib.rs` is the public facade and constants. `ship/src/state.rs` owns
pose/snapshot DTOs; `orientation.rs` owns ship-local quaternion/normalization
policy. `controller/mod.rs` owns private authoritative state, construction,
update dispatch and snapshot composition. Child `controller/{flight,steering,assist,door,cockpit,telemetry}.rs`
own direct flight/collision, ramped rotations, captured assist paths/interlocks,
airlock permissions/animation, authority/messages and instrument observation.
Existing regression tests live alongside those owners, including all-body assist
timing and quaternion boundaries. Ship may consume world/math contracts; it does
not own renderer meshes, native input or character movement.

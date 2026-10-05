# Feature maintenance map

Start with the affected row, then read that crate's `README.ai.md`,
`architecture.md` and `invariants.md`. Paths below are repository-root-relative;
Rust tests are generally inline `#[cfg(test)]` modules. Run focused checks while
iterating and the [applicable validation gates](validation.md) before completion.

| Feature | Owner and source entry points | Contracts/tests | Native scenarios |
| --- | --- | --- | --- |
| Lifecycle, resize, recovery, clocks | runtime `src/app.rs`, `frame_clock.rs`, `update_clock.rs`, `main.rs` | runtime invariants; inline clock/app tests | `landed-earth.json`, `orbit-earth.json`; root native smoke for resize/minimize/close |
| Rendering, depth, lighting, instruments | renderer `src/lib.rs`, `spheres.rs`, `ship_mesh.rs`, `cockpit_instruments.rs`, `overlay.rs`, adjacent WGSL | renderer invariants, `sphere-rendering.md`, inline tests and CPU/shader layout | `lower-cockpit-windows.json`; evidence variants and visual tour |
| Coordinates, catalog, landing geometry | world `src/lib.rs` | world invariants, `solar-system-layout.md`, `coordinate-strategy.md`, inline catalog/precision tests | `orbit-earth.json`; F2/N/1–6 precision tour |
| Flight, assists, cockpit authority, airlock | ship `src/lib.rs`; runtime `app.rs`, `action_bar.rs` compose | ship architecture/invariants, inline motion/assist/door tests | `orbit-earth.json`, `space-airlock.json`, `lower-cockpit-windows.json` |
| Walking, gravity, hull collision, EVA | character `src/lib.rs`; runtime `app.rs` composes ship snapshots | character architecture/invariants; inline collision/gravity/EVA tests; generated `ship_anchors.rs`, `thruster_collision.rs` | `cockpit-nose.json`, `space-airlock.json`, `moving-eva.json`, `nearby-eva.json` |
| Resource identities/generation/streaming | world `src/resources.rs`, `resource_distribution.rs`, `resource_generation.rs`, `mining.rs` | `resource-contracts.md`, world invariants; inline generation/session tests | `resource-deposits.json`, `resource-streaming.json` |
| Mining and resource UI | world `src/mining.rs`, `resource_fragments.rs`; runtime `mining.rs`, `resource_context.rs`, `resource_presentation.rs`, `action_bar.rs`; renderer `held_item.rs` / `held_item.wgsl` | world extraction/mass tests; runtime input/presentation and scenario contracts | `mining.json`, `resource-loop.json`; evidence `mining-tool.json` |
| Carrying, transfer, fragment motion/contact | world `src/carrying.rs`, `mining.rs`, `resource_fragments.rs`; runtime `carrying.rs`, `fragment_physics.rs` | world identity/mass/one-object invariants; runtime frame/placement/contact tests | `carrying.json`, `fragment-transfer.json`, `resource-loop.json` |
| Automation, staging, packaged input | runtime `src/automation.rs`, `e2e.rs`; `scripts/salimon_test.py`, `build_game.py`, `packaged_smoke.py` | runtime protocol/scenario tests; `scripts/test_*.py`; scripts README/BUILDING/PACKAGED_SMOKE | default suite, `--group resource-collection`, `--group ship-eva`; packaged smoke |
| Asset authoring and spatial contracts | `models/tools/`, `models/assets/ships/salimon-scout/{export,validate,spatial_contracts}.py`; renderer `ship_mesh.rs`; generated character layouts | models contracts/manifest, scout preservation metadata, `models/tests/`, asset README | cockpit/door/lower-window routes and applicable evidence variants |
| Observational diagnostics | diagnostics `src/lib.rs`; runtime snapshot mapping | diagnostics architecture/invariants and inline tests | F3 native smoke; never authoritative gameplay state |

All `src/` paths in the table are under the named `client/<crate>/` unless stated
otherwise. Scenario paths are under `scenarios/`; screenshots are in corresponding
`scenarios/evidence/` routes. Evidence proves a recorded revision, not future
behavior or hardware performance.

## Broad source files

`runtime/src/app.rs` coordinates native events, update ordering, domain snapshots,
render mapping and automation dispatch. Clocks, E2E protocol, mining/carrying,
fragment contacts and resource presentation already have separate internal modules.
Route changes to these modules before growing the event-handler composition.

`character/src/lib.rs` contains typed input/state, frame/camera math, walking and
collision, gravity transitions, cockpit and EVA behavior plus regression tests.
Its generated anchor/engine layouts come from the scout exporter. Keep portable
movement here and native events in runtime.

`ship/src/lib.rs` contains pose/state, input/control authority, direct flight,
landing/takeoff assists, door interlocks and instrument telemetry plus tests.
It may consume world contracts; it does not own renderer meshes or character
movement. Split future responsibilities when needed, rather than imposing file
size limits or moving tests solely to reduce line count.

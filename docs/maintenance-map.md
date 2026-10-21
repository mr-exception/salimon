# Feature maintenance map

Start with the affected row, then read that crate's `README.ai.md`,
`architecture.md` and `invariants.md`. Paths below are repository-root-relative;
Rust tests are generally inline `#[cfg(test)]` modules. Run focused checks while
iterating and the [applicable validation gates](validation.md) before completion.

| Feature | Owner and source entry points | Contracts/tests | Native scenarios |
| --- | --- | --- | --- |
| Durable active run | persistence `src/storage.rs`; runtime `src/checkpoints.rs`, `session/persistence.rs`, `app.rs` | [Save contract](save-contract.md); fault injection, 60s/event scheduling, native restart/fallback, 500 fragments | Normal play startup/exit; E2E/benchmark isolated; expanded signoff #375/#376 |
| Lifecycle, resize, recovery, clocks | runtime `src/app.rs`, `app/native.rs`, `frame_clock.rs`, `update_clock.rs`, `main.rs` | runtime invariants; inline clock/native tests | `landed-earth.json`, `orbit-earth.json`; root native smoke for resize/minimize/close |
| Rendering, depth, lighting, instruments | renderer `src/lib.rs`, `spheres.rs`, `ship_mesh.rs`, `cockpit_instruments.rs`, `overlay.rs`, adjacent WGSL | renderer invariants, `sphere-rendering.md`, inline tests and CPU/shader layout; single `Monitor_Wide` contract and cached 1536×256 dashboard | `lower-cockpit-windows.json`, `cockpit-nose-starboard.json`; `evidence/cockpit-wide-monitor{,-space}.json` and visual tour |
| Shared vector arithmetic | math `src/lib.rs`; character/ship and runtime fragment/carrying consumers | math architecture/invariants; arithmetic/handedness/precision tests; domain normalization/quaternion tests | Existing movement, flight and fragment scenarios |
| Terrain definitions/contact/streaming | world src/terrain/ and mining.rs; runtime terrain.rs, terrain_patches.rs, resource_presentation.rs; renderer terrain.rs/terrain.wgsl; character state.rs/controller/surface.rs; ship terrain_landing.rs; physics lib.rs/sleep.rs | [terrain contract](../client/world/terrain-contract.md), [integration/budgets/validation](terrain-validation.md); golden, slope/jump, footprint, contact/sleep, border/topology/precision/cache regressions | terrain suite: Earth/Moon/Mars plain/crater, touchdown, mining/carrying/streaming and steep-site rejection; reference terrain-render-smoke |
| Coordinates, catalog, landing geometry | world `src/lib.rs` facade; `coordinates.rs`, `catalog.rs`, `geometry.rs`, `proximity.rs` | world invariants, `solar-system-layout.md`, `coordinate-strategy.md`, inline catalog/coordinate/geometry/proximity tests | `orbit-earth.json`; F2/N/1–6 precision tour |
| Precision fixtures and engineering camera | world `src/precision.rs`, `camera.rs`; root public reexports in `lib.rs` | world coordinate strategy/invariants; inline precision and deterministic camera-tour tests | F2/N/1–6 precision tour |
| Flight, assists, cockpit authority, airlock | ship `src/lib.rs` facade, `state.rs`, `orientation.rs`, `controller/{mod,flight,steering,assist,door,cockpit,telemetry}.rs`; runtime `session.rs`, `session/interaction.rs`, `app/input.rs`, `app/interaction.rs`, `app/scene.rs`, `action_bar.rs` compose | ship architecture/invariants, inline motion/assist/door tests | `orbit-earth.json`, `space-airlock.json`, `lower-cockpit-windows.json` |
| Walking, gravity, hull collision, EVA | character `src/controller/{mod,interior,doorway,surface,eva,camera}.rs`, `collision.rs`, `layout.rs`, `queries.rs`; runtime `session.rs`, `session/frames.rs` compose ship snapshots | character architecture/invariants; mode-local collision/gravity/EVA tests and `controller/collision_tests.rs`; generated `spatial_contracts.rs` | `cockpit-nose.json`, `space-airlock.json`, `moving-eva.json`, `nearby-eva.json` |
| Fabricator recipes, storage, queue, tray and consumables | fabrication `src/{mass,recipes,machine,inventory}.rs`; runtime `session/{fabrication,crafting_ui,consumable_wheel}.rs`, `app/input.rs` | fabrication architecture/invariants; `tests/fabricator.rs`; integer/atomic/FIFO/power/mass regressions | Runtime scenarios are #347; native signoff #348 |
| Resource identities/generation/streaming | world `src/resources.rs`, `resource_distribution.rs`, `resource_generation.rs`, `mining.rs` | `resource-contracts.md`, world invariants; inline generation/session tests | `resource-deposits.json`, `resource-streaming.json` |
| Equipment toolbar state/input | runtime `equipment.rs`, `session.rs`, `app/input.rs`, `automation.rs` | five-slot/loadout/selection tests, carrying-lockout and view-gating tests; runtime invariants | `carrying.json` five-slot selection, empty-slot mining rejection and F pickup/drop; synchronized evidence captures equipped/empty/carrying/post-drop states |
| Equipment toolbar HUD | runtime `equipment.rs` presentation mapping / `app.rs` redraw; renderer `equipment_toolbar.rs`, `overlay.rs`, `lib.rs` | renderer DTO/raster/cache/DPI/stacking tests; runtime mapping tests | `evidence/equipment-toolbar.json`; native resize/F2 |
| Mining and resource UI | world `src/mining.rs`, `spherical_mining.rs`, `sphere_session.rs`, `resource_fragments.rs`; runtime `equipment.rs` equip authority, `mining.rs`, `reticle.rs`, `resource_context.rs`, `resource_presentation.rs`, `action_bar.rs`; renderer `held_item.rs` / `held_item.wgsl`, `overlay.rs` / `overlay.wgsl` | world extraction/mass tests; runtime input/presentation and scenario contracts | `mining.json`, `mining-emission.json` (continuous cap/release), `resource-loop.json`; evidence `mining-tool.json`, `world-prompts.json` |
| Carrying, transfer, fragment motion/contact | world `src/carrying.rs`, `mining.rs`, `resource_fragments.rs`; runtime `session.rs` F priority, `mining.rs` resource/cargo owner and tool input, `carrying.rs`, `fragment_physics.rs` adapters; physics `src/lib.rs` / `src/broad_phase.rs` / `src/convex.rs` / `src/sleep.rs` rules | world identity/mass/one-object invariants; physics motion/contact/step tests; runtime frame/placement/session tests | `carrying.json`, `fragment-transfer.json`, `resource-loop.json` |
| Automation, staging, packaged input | runtime `src/automation.rs`, `e2e.rs`; `scripts/salimon_test.py`, `build_game.py`, `packaged_smoke.py` | runtime protocol/scenario tests; `scripts/test_*.py`; scripts README/BUILDING/PACKAGED_SMOKE | default suite, `--group resource-collection`, `--group ship-eva`; packaged smoke |
| Asset authoring and spatial contracts | `models/tools/`, `models/assets/ships/salimon-scout/{export,validate,spatial_contracts}.py`; renderer `ship_mesh.rs`; generated character layouts | models contracts/manifest, scout preservation metadata, `models/tests/`, asset README | cockpit/door/lower-window routes and applicable evidence variants |
| Observational diagnostics | diagnostics `src/lib.rs` facade/DTOs, `aggregation.rs` window/cadence/statistics, `format.rs` rows/units, `raster.rs` font/RGBA; runtime `app/diagnostics.rs` snapshot/frame mapping | diagnostics architecture/invariants and owner-local tests | F3 native smoke; never authoritative gameplay state |

All `src/` paths in the table are under the named `client/<crate>/` unless stated
otherwise. Scenario paths are under `scenarios/`; screenshots are in corresponding
`scenarios/evidence/` routes. Evidence proves a recorded revision, not future
behavior or hardware performance.

## Broad source files

`runtime/src/session.rs` owns portable gameplay commands, snapshots and update order.
`runtime/src/app.rs` coordinates native events, presentation snapshots,
automation dispatch and renderer recovery. Private `runtime/src/app/{input,interaction,scene,diagnostics,native}.rs`
own key translation, prompt mapping, renderer DTO mapping, observational
diagnostics and window/cursor helpers respectively. Portable frame adapters and
interaction gates live in `session/`.
Helper tests live with these owners; update order remains in `session.rs`; frame/interaction gates live in `session/`.
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

Ship/character/fragment render interpolation: `client/runtime/src/session/presentation.rs`,
`fragment_physics.rs` and `app.rs`; tests live with presentation and the fragment
adapter. See the [coherent presentation contract](../client/runtime/architecture.md#bounded-gameplay-ticks-and-fragment-interpolation-159).

Frame/size boundaries: see the [typed boundary inventory](technical-architecture.md#position-boundaries-and-resource-dimensions-311).
Character `points.rs` / `state.rs` own the narrow position DTO/conversion API;
runtime anchor storage/presentation and physics/renderer mesh-scale consumers must
be migrated together. World owns analytic fragment radius/diameter.

## Fabricator wave (#336–#348)

Canonical behavior and planned typed ownership: [accepted Fabricator contracts](spaceship-building-and-survival.md#accepted-fabricator-contracts-2026-10-19-336). World owns raw identities/generation; fabrication owns recipes/storage/queue/tray and typed consumable slots; runtime owns physical transactions, Core funding, UI/input, inventory transfer and mass adapters. #338 implements the [portable crate](../client/fabrication/architecture.md); later issues integrate it.

## Fabricator session verification (#347)

`session/fabrication.rs` owns intake, Core funding, tray transactions and mass;
`automation.rs` maps read-only machine/jobs/tray and both typed slot arrays into
inspection and admits `crafting` actions only while the production modal is open.
`fabrication_scenarios.rs` covers mined new resources through the same intake,
FIFO, output and receiver adapters. Existing fabrication domain and session tests
cover overflow/rejected intake, mixed batches, nonrefundable cancel, blackout,
blocked tray, mass, flight and takeoff. `scenarios/fabricator-contract.json`
protects initial snapshot/closed-modal admission. Q consumption coverage is #377.


## Whole-game persistence (#363)

The [canonical save contract](save-contract.md) inventories authoritative state,
restore order, clocks, validation and schema evolution. `salimon-persistence` owns
the platform-independent versioned envelope and deterministic stable-ID map codec.
Domain adapters follow in #364 and session/Fabricator composition in #365; disk
checkpoints remain #366. Renderer caches are never save authority.

The #364/#365 adapters now implement the typed whole-session in-memory save
boundary: runtime `session/persistence.rs`, domain character/ship persistence
modules, world `mining_save.rs`, physics `SleepSaveState` and fabrication
`save.rs`. Machine/inventory/physical identities validate before atomic session
publication; derived mass, power request and caches regenerate. Persistence
regressions live in runtime session/persistence and fabrication_scenarios.
Filesystem durability and native lifecycle checkpoints remain #366.

## Survival meters (#367)

Portable meter/active-clock/offline-fatigue authority lives in
[`salimon-survival`](../client/survival/architecture.md). Native/session persistence
integration is implemented by #371; the session now advances and saves survival.

Damage, regeneration, treatment-only wound debt and data-driven timed/persistent/
exposure status effects (#368) use the survival crate's [single pipeline](../client/survival/architecture.md#damage-and-status-pipeline-368).

Q-wheel #377: fabrication `inventory.rs` owns exact typed slot consumption;
runtime `session/consumable_wheel.rs` owns modal/held state; native pointer/focus
mapping lives in `app.rs`/`app/input.rs`; renderer `consumable_wheel.rs` owns the
shared image/hit geometry. Tests cover slot identities, input/focus/repeat,
world-input isolation, continuing ship simulation and protocol resource-to-Q
consumption. Consumption uses #370/#371 effects/lifecycle; pause coordination is #381/#382.
Native visual/input signoff remains #376.
Survival movement/interaction restrictions (#369): `survival/src/restrictions.rs`
owns exact threshold percentages, validated tuning and typed permissions;
`runtime/src/session/survival.rs` applies them to shared commands/ticks,
`character/src/controller/` scales voluntary locomotion, and ship's
`controller/cockpit.rs` owns explicit pilot revocation. Tests: survival
`tests/restrictions.rs`, runtime `session/survival/tests.rs`, character locomotion
and ship revocation regressions. Run workspace tests/build/Clippy/fmt. See the
[canonical policy](../client/survival/architecture.md#movement-and-interaction-restrictions-369).
Clock/save integration is implemented by #371; HUD/wheel/effects remain #373/#377/#370.

Consumable survival effects (#370): `survival/src/consumables.rs` owns validated
milli-point recipe recovery records; its `pipeline.rs` executes timed food/healing
with the existing deterministic effect clock. `fabrication/src/inventory.rs`
provides exact slots shared with #377; runtime `session/consumables.rs` commits
exact identity plus prepared benefits and `automation.rs` exposes inspection/
portable requests. Coverage: survival `tests/consumables.rs`, runtime
`session/consumables/tests.rs`, existing inventory/save/status regressions. Run
workspace tests/build/Clippy/fmt; full-save clock integration is #371 and native
wheel/presentation is #377. [Canonical data/semantics](../client/survival/architecture.md#consumable-recovery-definitions-370).


Survival lifecycle (#371): runtime `session/lifecycle.rs`, `session/persistence.rs`,
`session.rs`, `checkpoints.rs` and `app.rs`; persistence envelope `src/lib.rs` and
`src/storage.rs`. The [save/lifecycle contract](save-contract.md) owns
v1 migration, startup publication, terminal freeze, archive/restart ordering and
failure guarantees. Tests live in `session/lifecycle/tests.rs`, checkpoint launch/
restart/publication-failure modules and storage restart fault injection. Run
workspace tests/build/Clippy/fmt, native startup/restart smoke; reference signoff
remains #376. Journey distance sampling is implemented by #372.


Journey statistics (#372): runtime `session/journey.rs`, tick composition in
`session.rs`, observational `automation.rs`, background admission in `app.rs` and
terminal publication in `checkpoints.rs`; storage owns versioned archive paths.
See [metric and migration](save-contract.md#journey-statistics-372). Regressions:
`session/journey/tests.rs`, app background ticks, checkpoint archive/repair/version
and persistence restart fault injection.


## Survival HUD (#373)

Runtime `survival_hud.rs` owns typed observational mapping and presentation
feedback/cache, with bounded raster/layout in `survival_hud/raster.rs`; `app.rs`
owns H/[ / ] focus-safe dispatch and overlay priority. Survival's immutable
`effect_snapshots` supplies captured rules/stacks/timers. Renderer exposes its
existing fitted toolbar message band; existing inventory/equipment art is unchanged.
See the [canonical HUD contract](../client/runtime/architecture.md#survival-hud-and-status-details-373).
Tests cover actual consumables, exact penalties, visual timing/cache, all supported
viewport/DPI layouts, full effect pagination and native input policy. #376 retains
native visual signoff; #381/#377 extend the existing menu/wheel admission later.


## Journey Ended (#374)

Runtime `journey_ended.rs` + `journey_ended/raster.rs` own the report screen and
physical pointer layout. `session/lifecycle.rs::ended_run` is shared with durable
archives; `app.rs::restart_terminal` composes the existing verified restart for
keyboard/mouse/native automation. Renderer only exposes its generic screen margin.
See the [terminal contract](save-contract.md#journey-ended-presentation-374).
Tests: `journey_ended/tests.rs`, native-host input/restart regressions and existing
checkpoint/restart fault injection. Reference native visual/input acceptance is #376.

## Starting supplies (#379)

Fabrication `machine.rs::starter_with_supplies` owns recipe-derived grants and
allocator reservation; runtime `session.rs::new_run`, normal startup `app.rs`
and `session/lifecycle.rs` compose new runs and mass. Restore is unchanged.
See the [canonical contract](save-contract.md#starting-supplies-379).
Tests: fabrication `tests/starting_supplies.rs`, runtime
`session/starting_supplies_tests.rs`, existing checkpoint/restart fault injection.

Gameplay pause (#381): `client/runtime/src/session/pause.rs` owns transient modal
authority and clock resets. Native routing is `app.rs`/`app/input.rs`; regressions
cover native/background and portable full-save freeze.

Condition cues/accessibility (#383): `client/runtime/src/condition_feedback.rs`,
`condition_feedback/settings.rs` and `condition_feedback/audio.rs`; renderer owns
only a generic fullscreen overlay before ordinary HUDs.

Four-food expansion (#378): fabrication `recipes.rs` owns the three locked food
records; survival `consumables.rs`, `effects.rs`, `pipeline.rs`, `lib.rs` and
`restrictions.rs` own validated modifiers, exact fatigue remainder and penalties.
Runtime `session/survival.rs` and `survival_hud.rs` consume the same derived policy.
Tests: survival `tests/foods.rs`, fabrication starter catalog regression, runtime
`session/consumables/tests.rs` craft/Core/exact-consume/full-save scenarios. See the
[canonical catalog and provisional settings](../client/survival/architecture.md#four-food-catalog-378).

Native current-sphere validation (#313) routes through runtime
`benchmark_fixture.rs` (`current-cap`/`current-mixed`), shared with the portable
`sphere_workloads.rs` matrix; use the
[benchmark contract](../scripts/FRAGMENT_BENCHMARKS.md#native-current-sphere-profiles-313).

Scout lighting/reference cameras (#463): renderer `ship_mesh.rs/.wgsl` owns bus
lighting; runtime `scout_reference.rs` owns E2E-only camera mapping. Shared poses
and eight power/view checkpoints live under `scenarios/visual/scout-*`; offline
`models/assets/ships/salimon-scout/preview_lighting.py` reads the same poses without
saving source assets. See [capture/limits](scout-presentation.md).

Scout spatial soundscape (#465): runtime `scout_audio.rs`, `scout_audio/synth.rs`
and `scout_audio/tests.rs`; shared OS playback `native_audio.rs`; persisted mix
controls remain in existing `condition_feedback/settings.rs`. See
[soundscape contract/budgets](scout-presentation.md#soundscape-boundary-465).
Do not duplicate #383 survival cues or simulate machinery in the audio adapter.

Scout machinery feedback (#464): runtime `scout_feedback.rs`, `app.rs` mapping
and `automation.rs` inspection; renderer `machinery.rs`, `ship_mesh.rs/.wgsl`.
Tests include portable pause/restore/power/blocked/progress/yaw and opt-in
`ship_feedback_tests.rs` production shader snapshots with raw timing evidence.
See [feedback contract](scout-presentation.md#machinery-feedback-boundary-464);
native walkthrough and budget acceptance remain #489.

Pause saves (#382): runtime `checkpoints/control.rs`, `checkpoints.rs` and `app.rs`
own request/status/UTC, asynchronous verified IO and focused menu/exit admission.
Fault-injected worker/store and native dispatch regressions cover the contract.
See [save contract](save-contract.md#pause-checkpoint-controls-382); native #376.

Survival onboarding (#380): `runtime/src/session/tutorial.rs`, success hooks in
`session/{consumables,consumable_wheel,fabrication,pause}.rs`, native H/T routing
and action-band fallback in `app.rs`; version-4 composition in
`session/persistence.rs`. Tests: `session/tutorial/tests.rs`, persistence and
existing native admission regressions. See the [onboarding contract](../client/runtime/architecture.md#persistent-survival-onboarding-380).

Survival balance #384: runtime `src/survival_balance.rs` owns six bounded portable
scenarios and opt-in JSON reports. See [evidence and assumptions](survival-balance.md);
owner/native route and tuning acceptance remains #496.

Funded impulse #493: ship `propulsion.rs`, `energy.rs`, controller flight/throttle/
telemetry/assist/persistence; runtime session/native B, schema migration, scene,
audio/machinery mapping; renderer cockpit atlas. Tests: ship `impulse_tests.rs`,
runtime `session/propulsion_tests.rs`, input/cache and inertial/landing scenarios.
See [current math](../client/ship/architecture.md#funded-core-impulse-and-engine-switch-493); native review #498.

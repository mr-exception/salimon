# Feature maintenance map

[Architecture baseline and ownership audit](architecture-baseline.md) records measurable refactoring priorities (#638).

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
| Modular structural foundation | ship `src/structure/{mod,grid,persistence,assembly,tests}.rs`; `src/mass.rs` namespace | [Structure architecture](../client/ship/architecture.md#portable-structural-foundation-541), ship invariants; headless six-face, multi-deck, removal, damage, identity, mass and save tests | No live runtime composition in #541; placement/construction integration remains #542–#550 |
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
and `scout_audio/tests.rs`; shared continuous CPAL output `native_audio.rs` / `native_audio/mix.rs`; persisted mix
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

## Persistence/survival cross-system verification (#375)

Runtime `session/persistence_scenarios.rs` executes the versioned
[`scenarios/portable/persistence-survival.json`](../scenarios/portable/persistence-survival.json)
DSL and full-source disk/restore/continuation matrix. Its `rates.rs` covers meter
partition/save boundaries and exhausted Q behavior. `checkpoints/scenarios.rs`
exercises the real worker/launch/restart composition with before/torn/after
publication faults, including every archive/restart write. The two extended
`persistence_extended` tests are opt-in locally and run nightly. See
[commands and operation reference](../scenarios/portable/README.md) and
[acceptance traceability](https://github.com/mr-exception/salimon/blob/d003bd58d506e04e3df97744f29f68449b6e2b3e/reports/issue-375/README.md). Existing #377–#384 owner
tests remain part of workspace CI; this matrix composes their production owners.

## Scout UI readability (#466)

[Canonical surface matrix](scout-presentation.md#scout-ui-readability-contract-466):
renderer `ui_style.rs` is the shared bitmap/palette owner; `cockpit_instruments.rs`
owns bounded readings/region caches; `ship_mesh.rs` owns physical screen U/V
interpretation. Runtime `action_bar.rs`, `app/interaction.rs`,
`session/crafting_ui.rs` and `survival_hud/raster.rs` retain their existing text,
input and layout roles. Run workspace tests/build/Clippy/fmt, including glyph,
contrast, line-fit, three-screen orientation and modal/interaction regressions.
The ignored `scout_ui_raster_evidence` / `scout_prompt_raster_evidence` tests emit
CPU pixels with `SALIMON_SCOUT_UI_DIR` set to an **absolute** evidence directory.
Existing `scout_feedback_gpu_evidence` supplies opt-in synthetic GPU views.
[Delivery report](https://github.com/mr-exception/salimon/blob/d003bd58d506e04e3df97744f29f68449b6e2b3e/reports/issue-466/README.md); native M1 acceptance #502;
scanner readability #503 uses the delivered #430 hologram. Physical/subjective approval remains separate.


## Scout Archive (#467)

Owner: `client/runtime/src/session/archive.rs`; tests in `session/archive/tests.rs`.
Persistence: `session/persistence.rs` and schema-6 save envelope. Native keys:
`app/input.rs`; contextual target/prompt: `session/interaction.rs`, `app/interaction.rs`.
Geometry: renderer `archive_terminal.rs`; label: `cockpit_instruments.rs`; shader:
`ship_mesh.wgsl`. Contract: [runtime](../client/runtime/architecture.md#scout-archive-467).
Run workspace build/tests/fmt/Clippy; retain native visual/input limits separately.

## Canonical universe schema (#402)

`client/universe/src/{identity,records,validation,lib}.rs` owns U01; its
[architecture](../client/universe/architecture.md) and
[invariants](../client/universe/invariants.md) define the immutable contract.
Tests: `client/universe/src/tests.rs`, frozen `fixtures/schema-v1.json`.
Run `cargo test -p salimon-universe --locked` and workspace gates. U02 frames,
U04 binary storage, U06–U12 data/build and U05 save binding are future consumers;
no existing world IDs or render/native boundaries are changed.

## Documentation maintenance policy (#440)

Canonical owner: [documentation maintenance gate](documentation-maintenance.md).
Contributor entry points: root AGENTS/coding conventions, PR template and Universe
issue form. Same-PR owning guides/links and explicit no-impact reasons are required
before readiness/review. Automated validation owner is #442; final audit is #441.
Do not duplicate those scopes or claim automatic semantic accuracy.

## Universe coordinates and epoch (#403)

Owner: `client/universe/src/coordinates.rs`, `epoch.rs`; regressions:
`coordinates/tests.rs`. [Architecture](../client/universe/architecture.md#hierarchical-frames-and-epoch--u02-403)
and [invariants](../client/universe/invariants.md) own numerical/scale boundaries.
Run universe and workspace Rust gates plus offline documentation validation.

## Universe new-run saves (#406)

Owner: `client/persistence/src/universe.rs`; tests: `src/universe/tests.rs`.
Persistence now depends on the portable universe leaf for validated IDs, split
positions and epochs. Existing `SaveManager` protects catalog incompatibility as
a fatal recovery error. Runtime retains its separate runtime envelope (now schema 26). See
[save contract](save-contract.md#future-universe-new-run-contract--u05-406),
[persistence architecture](../client/persistence/architecture.md) and
[invariants](../client/persistence/invariants.md).

## Universe source inputs (#407)

`client/universe/src/sources.rs` owns typed source metadata;
`scripts/import_universe_sources.py` verifies frozen local raw bytes and stages
a deterministic bundle. `catalogs/sources/manifest-v1.json` separates synthetic
Gaia fixtures from actual JPL/exoplanet snapshots. [Source contract](universe-sources.md),
[architecture](../client/universe/architecture.md#source-manifests--u06-407) and
[invariants](../client/universe/invariants.md) govern reuse/precision boundaries.
Production reuse/full snapshots wait for #659; science ingestion remains U07/U08.

## Data Recovery Module (#468)

Runtime `session/recovery.rs` owns Archive work; Fabricator data provides the
controller recipe and typed tray output; ship funds low-priority surplus joules
and accounts installed mass; renderer only styles the existing package. Runtime
schema 7 persists source and rejects duplicate ownership. See [canonical contract](data-recovery-module.md),
[runtime architecture](../client/runtime/architecture.md#data-recovery-module-468)
and [invariants](../client/runtime/invariants.md). Tests: `session/recovery/tests.rs`
and ship auxiliary energy tests; native acceptance #515, future leads #470.

## Universe binary partitions (#405)

Owner: `client/universe/src/partitions.rs`; tests: `partitions/tests.rs`, independent
wire fixture `fixtures/partition-v1.bin`; workload: `examples/partition_bench.rs`
(all relative to `client/universe/`). Read the [storage contract](universe-storage.md),
[owning architecture](../client/universe/architecture.md#binary-partitions--u04-405)
and [invariants](../client/universe/invariants.md). Bounded prototype only: no GB
builder/cross-cell references/runtime cache or production reuse approval. U12/U13
retain those boundaries. Run universe/workspace gates and the release workload.

## Universe scientific reference fixtures (#404)

Owner: `client/universe/src/reference.rs`, tests: `reference/tests.rs`, goldens:
`fixtures/references-v1.json`, examples: `reference_compare.rs`, `reference_bench.rs`
with `support/reference_inputs.rs` (paths within `client/universe/`/its examples).
See [reference wire/budget/workload guide](universe-reference-fixtures.md),
[architecture](../client/universe/architecture.md#scientific-reference-harness--u03-404)
and [invariants](../client/universe/invariants.md). Strict SI/epoch/frame comparison
and CPU diagnostics only; no solver/world migration or scientific model approval.
Repeated cloud reference/RSS measurements are recorded in [the #521 report](https://github.com/mr-exception/salimon/blob/d003bd58d506e04e3df97744f29f68449b6e2b3e/reports/issue-521/README.md);
full-game cloud gates remain #435–#439. Human hardware budgets remain #563.

## Universe Gaia ingestion (#408)

Owner: `client/universe/src/ingestion/gaia.rs`, shared `ingestion/csv.rs`; tests:
`ingestion/gaia/tests.rs`; workload: `client/universe/examples/gaia_bench.rs`.
See the [Gaia selection/units/cross-match contract](universe-gaia-ingestion.md),
[architecture](../client/universe/architecture.md#gaia-ingestion--u07-408) and
[invariants](../client/universe/invariants.md). Synthetic local ingestion and
bounded canonical batches are implemented; production approval/full snapshots
remain #659, inference U09, astrometry pack integration U12. No playable migration.

## Universe observed ingestion (#409)

Owners: `client/universe/src/ingestion/horizons.rs`, `exoplanets.rs`; tests in their
sibling module directories; fixture example `client/universe/examples/observed_import.rs`.
See [units/identity/validity contract](universe-observed-ingestion.md),
[architecture](../client/universe/architecture.md#observed-solar-system-and-exoplanets--u08-409)
and [invariants](../client/universe/invariants.md). CI compares two offline builds.
Actual frozen Earth/Proxima and synthetic matching work; full reviewed release
coverage remains #526/#513, packaging U12, arbitrary-date models U14. No runtime
world, scanner knowledge, save or rendering migration is implied.

## Universe missing-data inference (#410)

Owner: `client/universe/src/inference.rs`, `inference/observed.rs`; tests:
`inference/tests.rs`; fixed-seed workload: `client/universe/examples/inference_sanity.rs`.
Read the [inference contract](universe-inference.md), [architecture](../client/universe/architecture.md#missing-data-inference--u09-410)
and [invariants](../client/universe/invariants.md). Run universe/workspace gates,
sanity report twice and docs checks. Priors are heuristic; U12 owns packaging,
U14 full-state evaluation; native and production-source gates remain separate.

## Canonical fictional systems U10 (#411)

Owner: `client/universe/src/synthesis.rs`; regression tests:
`synthesis/tests.rs`; pinned request: `fixtures/fiction-request-v1.json`; offline
file adapter: `examples/fiction_build.rs`; cohort/storage workload:
`examples/fiction_sanity.rs` (all under `client/universe/`).
See the [builder contract](universe-fiction-builder.md), [architecture](../client/universe/architecture.md#canonical-fiction-synthesis--u10-411)
and [invariants](../client/universe/invariants.md). Source imports and observed
U09 completion remain separate. U11 below publishes the selected world; other
worlds' full definitions, U12 production packs and U14
complete states retain their boundaries; runtime/world/renderer are unchanged.

## Universe permanent starting system (#412)

Owner: `client/universe/src/starting_system.rs`; tests:
`src/starting_system/tests.rs`; request/world goldens: `fixtures/starting-*-v1.json`;
audit adapter: `examples/starting_system.rs` (paths within `client/universe/`).
Read the [selection/world contract](universe-starting-system.md),
[architecture](../client/universe/architecture.md#permanent-starting-world--u11-412)
and [invariants](../client/universe/invariants.md). Run universe/workspace gates,
repeat the manifest example and compare the golden; cross-platform CI also runs
debug/release tests. Candidate/world hashes must not change silently. U12 merges
and packages; U14 supplies states; U23/U24 integrate terrain/arrival/resources.
No runtime, prototype save, player chart or native signoff changes are implied.

Scout machinery verification (#489): runtime `scout_walkthrough.rs` supplies two
validated E2E initial fixtures; `e2e.rs` and `scripts/salimon_test.py` select them.
Subsequent intake/menu/power/work/collection uses production session/window routes.
`reports/issue-489/walkthrough.py` retains GPU motion frames plus authoritative
state, with two additional close cameras consumed by `scout_reference.rs`.
See [evidence/limits](https://github.com/mr-exception/salimon/blob/d003bd58d506e04e3df97744f29f68449b6e2b3e/reports/issue-489/README.md); subjective review #561,
physical platform/input #557, hardware budgets #563; delivered scanner integration #503/#430 is cloud-verified separately.

Recovery cloud verification (#515): `scout_walkthrough.rs` owns declared initial
materials, `automation.rs` exposes read-only recovery source and the R alias,
`session/recovery/tests.rs` owns exact concurrent funding/migration/disk coverage.
Renderer `ship_mesh.rs` normalizes the generic tray package's per-face artwork UVs;
its geometry regression and the matched native before/after images cover the
correction. See [the report](https://github.com/mr-exception/salimon/blob/d003bd58d506e04e3df97744f29f68449b6e2b3e/reports/issue-515/README.md) and renderer architecture.

## Blueprint placement (#542)

- Authority: `client/ship/src/structure/placement.rs`, with focused regressions in
  `structure/placement/tests.rs`.
- Composition/catalog/save/input: runtime `session/construction.rs`,
  `catalogs/blueprints.json`, `session/persistence.rs`, `app/input.rs` and automation.
- Rendering: `app/scene.rs` maps hollow oriented cuboids; renderer `lib.rs` and
  `bootstrap.wgsl` own validated quaternion upload/projection.
- Integration: `scenarios/evidence/blueprint-placement.json`, portable construction
  restore/transaction tests, native input admission and actual wgpu captures.


Modular interior presentation (#575): [contract](modular-interior-fixtures.md),
[renderer](../client/renderer/src/fixtures.rs),
[runtime mapping](../client/runtime/src/session/fixtures.rs),
[template source](../models/assets/fixtures/scout-interior-kit/README.md).
Physical attachment/construction/damage/aging authority remains with their
existing domain owners; this renderer layer consumes observations only.


Exact contact stress (#592): physics `iteration_oracle.rs`, fixed-point response
loops and shared `salimon-math` arithmetic inlining; runtime `sphere_workloads.rs`
provides diagnostic terminal prefixes. Reproduce with the isolated historical
fixture/report, never by reverting gameplay geometry or weakening sleep/contact
rules. Hardware cadence/frame budgets remain #563 and physical checks #557.

## Physical mounts (#576)

Source: `client/ship/src/attachments.rs`, `client/runtime/src/session/mounts.rs`, world carrying and character installed obstacles. Checks: `session::mounts`, ship attachment tests and `scenarios/evidence/physical-mounts.json`. See [the canonical contract](physical-mounts.md).


## Expanded cabin mechanisms (#577)

The optional schema-12 cabin source, manual reversible fixtures, supported postures, visible restraints and native controls are implemented in [the maintained cabin contract](cabin-mechanisms.md). Historical schema-10 habitation remains the migration baseline; lighting preferences (#580), sound (#583), human comfort (#561) and hardware budgets (#563) retain their separate owners.


## Persistent cosmetic groups (#579)

Five room groups and exact furniture overrides now use the [bounded finish contract](room-finishes.md). Schema 13 preserves histories independently of divider topology; physical material replacement, future constructed room geometry (#545) and actual damage (#549) retain their owners. Shared instance tint factors are now bounded to [0,4] and cosmetic RGB is clamped before lighting; warning/functional surfaces retain authored values.


## Instant direct-material Core upgrades (#548)

The owner-authorized capacity tiers, exact receipt accounting, separate fuel/material controls and schema-14 save behavior are implemented in [Core upgrades](core-upgrades.md). Ordinary construction/commissioning and blackout throttle rules remain separate. No free charge or upgrade power/timer is introduced.


## Construction material ownership (#543)

See [construction material transactions](construction-materials.md) for exact original receipts, atomic physical refunds, schema 15, mass and the owner-approved construction economy. Paid work and completed geometry follow [construction progress](construction-progress.md) and [modular structure](modular-construction.md).


## Room and fixture lighting (#580)

The [maintained lighting contract](room-lighting.md) describes implemented presets, independent overrides, effective bus/condition gates, shutter/Sun contribution and schema 16. Actual future damage/room creation remain #549/#545; human approval #561 and hardware budgets #563.


## Tactile audio and physical partitions (#583)

The [maintained acoustics contract](scout-acoustics.md) describes actual mechanism/attachment events, finite cargo/privacy separation, differentiated floor contact and bounded continuous CPAL filtering. E2E mapping stays silent; actual output captures and human listening remain separately identified.


## Automatic construction (#544)

See [automatic construction contract](construction-progress.md) for active Core-funded timers, safety/pause admission, simple interior completion, schema 17, exact save/refund behavior, production evidence completed [modular structure](modular-construction.md) and [optional module commissioning](non-core-modules.md).


## Completed modular structure (#545)

See the [canonical completed construction contract](modular-construction.md) for the ship assembly/shell and runtime installed/work owners, character traversal, per-cell/face identity, material mass transfer, schema-18 reconstruction/migration, partition editing and the production room evidence scenario. Gameplay damage/salvage remains #549; offline construction remains #550.


## Construction flight readiness (#546)

See [construction flight safety](construction-flight-safety.md) for schema-19 Stop/Resume, actual hull and attachment admission, cockpit warnings and airborne stop latches. Source: runtime `session/construction/flight.rs`, ship `structure/construction.rs`; regressions: runtime `session/construction/flight/tests.rs`. Run workspace and construction checks plus the production `construction-interlocks.json` evidence scenario.


## Optional module commissioning (#547)

See [non-Core module contract](non-core-modules.md) for the Archive processor, G/E/T/RMB maintenance, immediate activation downtime, real commissioning faults, paid restoration, schema 20, source/test mapping and deferred Jump Drive follow-up #633.


## Construction damage and salvage (#549)

See the [canonical damage and schema-21 contract](construction-damage.md) for runtime `session/construction/damage.rs`, ship lost-cell geometry, world sealed salvage allocation, controls, exact source/mass/identity boundaries and focused tests/evidence routes. Run workspace tests/build/Clippy/fmt plus documentation validation.


## Offline construction (#550)

See [the maintained chronology and schema-22 contract](construction-offline.md): runtime `session/offline.rs`, ship `controller/offline.rs`, launch/checkpoint staging and existing work owners. Run offline/checkpoint/Recovery and ship tests, workspace build/tests/Clippy/fmt, the two documented wgpu routes and docs/base-impact validation. Future #433 supplies verified universe events; no scanner or orbit implementation is duplicated here.


Cosmetic aging #581: [source/identity, successful use, schema 23, rendering budgets and exact verification](surface-age.md). Damage remains condition-driven; clean repairs restore normal geometry and preserve age.


Automation domain operations now live in `client/runtime/src/automation/` (#639).
The envelope/reader stays in `automation.rs`; native commands, observations and
legacy key aliases have separate owners. Domain handlers call the existing
production session API; protocol compatibility and scenario regressions live in
`automation/tests.rs`.

Domain input binding owners are in `client/runtime/src/session/input/` (#640);
`app/input.rs` retains physical translation and native admission only.

Domain save mapping/version rules now live in runtime `session/<domain>/save.rs` (#641);
central persistence retains the explicit layout and atomic publication. See the
[save ownership contract](save-contract.md#runtime-domain-mapping-ownership-641).

For session composition (#642), use `session/init.rs`, `commands.rs`,
`interaction/actions.rs`, `input/state.rs`, `policy.rs` and the explicit
`tick.rs` scheduler with power/character/resource stage owners.


## Presentation extraction (#643)

GPU initialization, resize and frame work: renderer `src/initialize.rs`,
`src/surface.rs`, `src/frame.rs`; DTOs/resources remain in `src/lib.rs`.
Native redraw/overlays: runtime `src/app/presentation.rs`; prompt priority/anchors:
`src/app/presentation/prompts.rs`; Scout observation mapping:
`src/app/scene.rs::gameplay_ship`. Tests remain with renderer camera/layout
contracts and runtime scene/input/session owners. Production evidence routes:
`scenarios/evidence/lower-cockpit-windows.json` and
`scenarios/evidence/construction-interlocks.json`.


## Contact and resource geometry review (#644)

Physics scheduling/data: `client/physics/src/lib.rs`; sampled contact, warm
impulses and normal/friction/rolling response: `client/physics/src/response.rs`.
Exact response regression: `client/physics/src/iteration_oracle.rs`; broad-phase
and sleep tests retain their owners. Resource GLB validation/index packing:
`client/renderer/src/resource_mesh/geometry.rs`; GPU staging/culling/cache remains
in its parent. Convex/SAT and ship assist owners were reviewed and retained.
See [physics](../client/physics/architecture.md#contact-response-owner-644),
[resource](../client/renderer/architecture.md#resource-geometry-owner-644) and
[assist](../client/ship/architecture.md#assist-cohesion-review-644) ownership.


## Incremental private offline packs (#413)

See the [implemented contract](universe-pack.md) for ownership, inputs, bounds, validation and remaining production/runtime limitations.


## Bounded read-only pack queries (#414)

See the [implemented contract](universe-reader.md) for ownership, inputs, bounds, validation and remaining production/runtime limitations.


## Time-addressable ephemeris models (#415)

See the [implemented contract](universe-ephemerides.md) for ownership, inputs, bounds, validation and remaining production/runtime limitations.


## Dominant-source celestial gravity (#416)

See the [implemented contract](universe-gravity.md) for ownership, inputs, bounds, validation and remaining production/runtime limitations.


## Gravity-aware ship trajectory (#417)

See the [implemented contract](universe-trajectory.md) for ownership, inputs, bounds, validation and remaining production/runtime limitations.


## Real-scale surface flight (#418)

See the [surface-flight contract](universe-surface-flight.md) for rotating/orbiting support,
Core-funded physically bounded landing/takeoff, production source/tests and
remaining playable-world adoption. Current compressed gameplay is unchanged.


## Real-scale atmospheric drag (#419)

See the [atmosphere contract](universe-atmosphere.md) for planet-specific density,
co-rotating gas/wind, atomic online/offline drag integration and physically paid
assist compensation. Model validity, tests and playable-adoption limits are explicit.


## Real-scale moving contact (#420)

See the [continuous-contact contract](universe-contact.md) for moving solids/terrain,
certified earliest-event queries, atomic gravity/drag interception, body-relative
consequences, funded-work receipts, cloud tests and adoption/precision limits.


## Predictive spatial streaming (#422)

See the [streaming contract](universe-streaming.md) for worker ownership,
nonblocking runtime polling, explicit residency limits, stale-result rejection
and immutable quality-independent content. Startup uses `SALIMON_UNIVERSE_PACK`;
loaded catalog truth does not grant player discovery or replace gameplay yet.


## Canonical terrain streaming (#424)

See the [canonical terrain contract](universe-terrain.md) for the exact U11
evaluator, separate frozen identity/codec, planet-wide quadtree, stitching and
CPU/GPU bounds. Compressed six-body fixtures remain unchanged. Gameplay adoption
and normal alien-world startup belong to #425/#426.


## Canonical surface integration (#425)

The [canonical gameplay contract](universe-gameplay.md) maps current body/frame,
resource/physics/Fabricator integration and schema-24 definition binding to owners
and tests. It preserves the frozen terrain and explicit historical fixtures.

Canonical new journeys (#426) use `session/canonical.rs::new_canonical_run`: frozen Vesper ID, published traveler offset [14,0], conservative authored-hull clearance and a view facing the landed ship. Fresh terminal restarts preserve run serial/seed policy and grant existing supplies once; restore never grants them. Flat fixtures remain explicit; normal launch rejects prototype saves under #427. Oxygen depletion stays deferred. Tests and native evidence: `reports/issue-426/README.md`; setup `canonical-journey`.


## Navigation forecasts (#421)

See the [bounded prediction contract](universe-navigation.md): `ship/src/prediction.rs` (split-coordinate coasting gravity) and `ship/src/preview.rs` (actual playable power/contact model), `runtime/src/session/navigation.rs` disposable cache, `app/scene.rs` / `automation/inspection.rs` observation mapping, and renderer `cockpit_instruments.rs`. Run ship/runtime navigation and renderer tests, all-target Clippy/fmt/docs, runner tests and `scenarios/evidence/universe-navigation.json` in production software-Vulkan. No autopilot, source-energy mutation, chart reveal or certified arrival/error bound is introduced.


## Scanner contacts (#428)

`client/universe/src/scanner.rs` owns bounded player discovery, exact sampled
celestial states, three-hour freshness and durable encounter history. Unknown
catalog entries are never revealed implicitly; batches validate before mutation.
See the [scanner contract](universe-scanner.md) for ownership, clock semantics, tests and downstream integration.


## Scanner delivery (#429)

Universe `scanner/pulses.rs` owns fixed five-second cadence and analytical funded linear-path detection. Runtime `session/scanner.rs` composes ordinary Core funding after construction, samples current body-local state and exposes read-only automation. Unknown orbital phase/velocity is never invented; body-local and galactic observations cannot be mixed.
See the [scanner contract](universe-scanner.md).


## Scanner delivery (#431)

Scanner contact checkpoints validate catalog revision, exact IDs/frames, timestamp ordering and bounds before atomic reconstruction. Runtime schema 25 saves canonical scanner parameters, clock/remainder and encounter history; legacy canonical saves migrate empty, current missing/malformed sources reject. Closed-app launch advances freshness once without inventing offline detections.
See the [scanner contract](universe-scanner.md).


## Scanner delivery (#430)

Renderer `scanner_hologram.rs` owns bounded depth-tested ship-local 3D projection geometry; runtime `session/scanner/presentation.rs` maps only fresh same-frame discoveries to ship pose. The physical console projector shows logarithmic distance, true 3D direction/elevation and selected contacts. M cycles contacts only in an admitted canonical cockpit; power loss suppresses projection, not history.
See the [scanner contract](universe-scanner.md).


## Scanner delivery (#469)

Universe `scanner/chart.rs` owns player-known observations, inferred/transmitted leads with explicit uncertainty, and immutable voluntary route annotations. Runtime `session/chart.rs` owns cockpit P/M/Enter controls and schema-26 pair validation with scanner history. Catalog truth never seeds the chart; observed knowledge outlives contact TTL and source frames are preserved without guessed transforms.
See the [scanner contract](universe-scanner.md).


## Canonical geology (#612)

World `terrain::geology` owns versioned terrain-bound province/history/substrate
queries. The returned physical sample is the exact existing terrain; geology
never displaces ground or reshuffles resource identities. See the
[geology contract](planetary-geology.md) for codec, profiles, compatibility and tests.
Vesper remains a homogeneous silicate interpretation with unchanged published
terrain/resources; rendering and later environmental systems retain their owners.


## Geological surface materials (#613)

Follow the [geological material contract](geological-materials.md) for terrain-bound
profile mapping, immutable material cache keys, the 52-byte vertex layout,
filtered stratification and original software-Vulkan evidence. Runtime owns
geology-to-DTO mapping; renderer owns shading only. Vesper and resource palettes,
physical queries, geometry and save identities are preserved.


## Regional climate and weather (#615)

Follow the [climate contract](planetary-climate.md) for immutable geology-bound climate inputs,
strict versioned publication and stateless TT epoch queries. World owns pressure,
temperature, coherent storm/cloud/precipitation metadata and body-fixed tangent wind;
presentation cannot rewrite them. Vacuum has no airborne effects. Existing terrain,
resource identities, saves and drag physics remain unchanged. Weather visuals are #616.


## Active geological sources (#617)

The [active geology contract](active-geology.md) owns explicit heat/mechanism inputs,
stable terrain-bound sites, strict publication restore and shared-TT source levels.
Spatial queries cannot create/rekey sources. Vacuum permits ballistic plumes and
contact coupling, never atmospheric wind/sound. Source radii do not add colliders.


## Rare biospheres and ecological regions (#618)

Follow the [ecology contract](planetary-ecology.md) for independent fixed life decisions,
geology/climate-bound potential, continuous realms and stable patch/grammar IDs.
Suitability never grants life; observed bodies retain no procedural life claim
and Vesper stays sterile. Strict catalog/save bindings reject incompatible
publications. Organism meshes, physical instances and harvesting remain #619–#621.


## Hybrid astronomical sky (#423)

The [implemented sky contract](universe-sky.md) maps the universe source/streaming,
runtime projection and renderer pass/test owners. Full ephemeris-frame adoption,
full source coverage and human/hardware release qualification remain separate.


## Optical atmosphere implementation (#614)

See [planetary optical atmosphere](planetary-atmosphere.md) for source/test ownership,
physical inputs, bounded depth-aware scattering and exact fixture/native limits.
Clouds/weather presentation remains #616; human/hardware qualification is separate.


## Coherent weather presentation (#616)

[Implemented weather visuals](planetary-weather-visuals.md) route authoritative
shared-time samples through runtime lighting and fixed production renderer
fields. Acoustic integration remains #627 and hardware/human qualification separate.


## Organism family presentation (#619)

[Procedural organism families](planetary-organisms.md) map the world grammar,
stable cosmetic surface tiles, runtime adapter and bounded renderer/test owners.
Physical colliders/landing safety are #620; biological material response is #622.


## Physical organisms and clearance (#620)

[Physical organism placement](physical-organisms.md) owns stable source/part
bindings, root/exclusion rules, common physics sweeps and live character/ship
clearance. Vesper remains sterile; mutable harvesting/removal remains #621.


## Scanner readability (#503)

See the [Scout presentation contract](scout-presentation.md). Renderer `ScannerReadout` owns bounded text/units and `ui_style` owns the face and functional palette, including linear 3D inks. Runtime maps only scanner-owned power and sampled same-frame contacts to the existing action bar. Presentation cannot reveal catalog truth, mutate discovery, change five-second cadence or TTL, allocate a second HUD or bypass cockpit/modal gates.


## Scanner cloud integration (#438)

See the [scanner acceptance contract](universe-scanner.md#cloud-scanner-acceptance-438). The existing action bar places fresh scanner status top-right at 1280+ physical pixels to leave the projector visible at simulated 2x scale; compact windows and chart mode retain their existing bottom placement. Renderer owns generic `TopRight` image placement; runtime chooses placement from viewport width without changing scanner, input, collision or saved authority. The required scanner suite and exact frozen-frame/pixel review preserve failure evidence and physical/hardware qualification limits.

## Biological material response (#622)

Follow [organism materials](planetary-organisms.md#biological-materials-and-weather-response-622)
for runtime source/weather mapping, renderer instance/shader layout, root/bounds
regressions and opt-in production Vulkan reference captures. Verify workspace
build/tests/Clippy/fmt, script and documentation checks. Human art review #561,
physical platform #557 and hardware #563 remain separately unverified.

## Geological landmarks (#623)

[Landmark contract](planetary-landmarks.md) maps world publication/relief tests,
actual runtime mesh/LOD export and production renderer reference checks. Run
landmark/terrain tests, workspace gates and `universe-terrain.json` captures;
source/site identity never comes from render LOD or player chart state.

## Natural signature sites (#624)

[Biological signatures](biological-signature-sites.md) maps world publication,
rarity/mixed-source tests, runtime physical/render endpoint equality and opt-in
source export to actual production Vulkan reference captures. Preserve source
IDs, root/exclusion rules, true physical scale and explicit player discovery.


Ordinary-play isolation (#427): [canonical runtime and prototype-save gate](universe-gameplay.md#ordinary-canonical-play-427).


## Suit observations (#625)

See [suit measurements and field marks](suit-observations.md) for source/validation
ownership, F10/F11/F12, configuration, schema 27 and preserved scanner boundaries.


## Personal planetary chart (#626)

[Planetary chart](planetary-chart.md) maps personal history, physical inspection,
native text/keys, uncertainty presentation and schema-28 validation to owners/tests.


## Planetary audio (#627)

[Source and verification contract](planetary-audio.md): runtime `environment_audio.rs`
feeds the existing native continuous mixer from local world definitions. Run its
portable regressions, production callback export and native environment-audio scenario.


## Biological harvesting (#621)

[Source, frame, shape, transaction and schema contract](biological-harvesting.md).
World `environment/harvest.rs` and `biological_session.rs` own source/mass rules;
runtime `session/biology.rs` composes input, current anatomy and checkpoint restore.
Run world/runtime harvest tests, full gates and both native biological scenarios.


## Canonical render acceptance (#436)

See the [cloud render acceptance contract](universe-terrain.md#cloud-integration-acceptance-436) for the required
`universe-render` suite, native frozen/revisit pixel checks, canonical stitched
seam tests and bounded raw frame/draw observations. Measurements remain
observational; missing GPU counters stay null and software timings do not
establish hardware FPS.

Planetary environment combined verification: `scripts/planetary_environment.py`,
`scenario_suites.py` and `scenarios/evidence/planetary-night.json`; runtime
`scout_reference.rs` owns physical night-camera placement. The [matrix guide](../scripts/README.md#planetary-environment-matrix-628)
links production path coverage and hardware/perception limits.

Flight acceptance #437: see [the reproducible matrix](universe-flight-verification.md) for nine GPU routes, production orbital/contact/session contracts and separate X11 input coverage.


## Selected observed-source coverage (#526)

See the [coverage contract](observed-source-coverage.md) for JPL/PS frozen inputs,
`ingestion/horizons.rs` physical formats, `pack/observed.rs` expected targets and
star/parent mapping, acquisition and independent verification scripts, exact
commands and scientific/release boundaries. Full Gaia acceptance remains #524;
this private observed pack does not grant player knowledge or publish a release.


## Canonical offline propagation (#432)

[Source and frame contract](universe-offline-propagation.md): ship `src/offline.rs`, regressions `src/offline/tests.rs`. Ordinary canonical motion publication and saved-journey integration are implemented; timed/scanner composition and native progress follow in #433/#434.

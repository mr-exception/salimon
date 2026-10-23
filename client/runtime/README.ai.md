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
behavior or networking here. Persistence orchestration uses the dedicated
`checkpoints.rs` boundary. Keep new platform-specific behavior
small in `app/native.rs`. Introduce a separate platform boundary only when a
real second implementation requires it.

## Source routing

Start in `app.rs` for native lifecycle, recovery, scheduling and evidence IO.
Use `session.rs` for typed commands/snapshots, gameplay state and exact cross-domain
update order; `session/frames.rs` and `session/interaction.rs` own portable adapters.
Native key translation stays in `app/input.rs`; scene, diagnostic and prompt
mapping stay in `app/`. See the [ownership table](architecture.md#internal-composition-modules).
E2E/benchmark initializers and portable regressions construct `GameplaySession`
directly. `automation::execute_session` exercises production commands without a
native shell; `execute` adds native-only operations/observations.

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
  and nominal catalog radii. Gameplay far-envelope radii and bounded terrain DTOs
  follow the canonical terrain integration guide; keep material selection here.
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

F grab/drop latching and left-mouse mining held state are runtime
composition in `session.rs`, translated by `app/input.rs`; `mining.rs` holds the resource owner and separate tool input; see runtime invariants.
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

`fragment_physics/geometry_evaluation.rs` is an opt-in geometry audit and CPU
control experiment, not gameplay fixtures or replacement collision geometry.
Run the commands and interpret limitations in the
[benchmark guide](../../scripts/FRAGMENT_BENCHMARKS.md#resource-geometry-evaluation-160).

Equipment toolbar state lives in `equipment.rs`, composed in `session.rs`. Keep
world-session carrying checks and native/automation view gates aligned. Its
selection is the sole equipment authority for mining, held presentation and reticle.
Slot changes clear mining input; physical pickup clears selection and held mining.

Map toolbar contents and optional selection through `equipment::presentation`
for gameplay redraws only. Renderer owns rasterization and message stacking;
keep the toolbar independent of contextual action text and tool equip policy.

For toolbar changes, run the carrying baseline/evidence route: it exercises all
five selections, left-mouse mining rejection for every empty slot, equipped pickup,
lockout and post-drop explicit selection. See [runner coverage](../../scripts/README.md#toolbar-gameplay-coverage-132).

`session/presentation.rs` owns render-only ship/character history and frame mapping;
see the coherent presentation contract below. Native tick/backlog admission lives in `update_clock.rs` and `app::advance_native`;
all domains consume the same completed tick. Fragment history lives
in the resource adapter and must never enter authority/interaction state. Changes
need stalled-clock, immediate release, local-frame and measurement aggregation
regressions. See [the tick contract](architecture.md#bounded-gameplay-ticks-and-fragment-interpolation-159).

## Terrain integration (#179–#181)

Follow the canonical [integration/budgets/validation guide](../../docs/terrain-validation.md)
and [world definition contract](../world/terrain-contract.md). Keep local rules
and their consumers aligned; flat deterministic fixtures are explicit.

## Durable active-run orchestration (#366)

Runtime `checkpoints.rs` owns normal-play startup restore, a bounded save worker,
60-game-second/discrete-event scheduling, persistent native-title warnings and
verified clean-exit capture. Domain state stays on the event thread; owned bytes
cross to the worker. `salimon-persistence::storage` owns verification and native
filesystem durability. Startup renderer creation preserves restored clock backlog;
E2E and benchmark launches remain isolated from user saves. See the
[canonical save contract](../../docs/save-contract.md#durable-active-run-366)
for paths, recovery/failure policy, Windows behavior and future lifecycle owners.

Survival action gates and voluntary movement composition (#369) live in
`session/survival.rs`; use its shared policy for new native/protocol interactions.
See [canonical restrictions](../survival/architecture.md#movement-and-interaction-restrictions-369).

Item effects/removal compose in `session/consumables.rs` (#370): use the exact
slot/job/unit selection seam from native wheel or protocol adapters. Validate and
prepare benefits before removing inventory; never consume a slot first and try
to apply potentially fallible effects afterward. See the recovery contract.


#371 composes schema-2 survival source, explicit v1 migration, verified launch-only
fatigue credit, terminal freeze and archive/restart publication. See the canonical
[save/lifecycle contract](../../docs/save-contract.md). Restart must publish an
archive and both fresh fallback generations before exposing new gameplay; failures
remain failures. Maintain lifecycle, offline rollback, publication-failure and
restart fault-injection tests with changes to this boundary.


Journey accounting and final location live in `session/journey.rs`; read the
[metric/save contract](../../docs/save-contract.md#journey-statistics-372) before
changing frame sampling, background timing or archived summaries. Preserve the
journey, background tick and archive version/recovery regressions.


Survival HUD/status details (#373): start in `survival_hud.rs` (observational DTOs,
feedback/tween/cache), `survival_hud/raster.rs` (bounded layout/RGBA), and native
H/[ / ] admission in `app.rs`. See the [canonical HUD contract](architecture.md#survival-hud-and-status-details-373).
Keep UI state out of full saves, use typed effect/restriction snapshots, and use
the renderer-owned toolbar reservation when fitting overlays. Preserve the HUD,
real-consumable, focus/modal and raster/layout regressions.


Journey Ended (#374): `journey_ended.rs` and its `raster.rs` own report DTO/layout/
pixels; `session/lifecycle.rs::ended_run` also supplies the archive. Native input
and the sole verified restart path stay in `app.rs`. Read the
[terminal contract](../../docs/save-contract.md#journey-ended-presentation-374),
preserve highest overlay priority, stale-pointer invalidation and failed-restart
terminal authority. Regressions live in `journey_ended/tests.rs` and `app/input_tests.rs`.

Starting supplies (#379): normal launch/restart use `GameplaySession::new_run`;
restore and isolated fixtures use empty `Default`. Fabrication allocates grant
identities and exact recipe reserves; runtime reconciles mass. Read the
[canonical contract](../../docs/save-contract.md#starting-supplies-379) and preserve
`session/starting_supplies_tests.rs` launch/craft/reload/restart coverage.

#378 extends the existing catalogs with four food definitions and captured food
modifiers; see the owning architecture for provisional tuning and regressions.
Keep advanced food recipes locked and preserve exact-slot consumption; no new
progression acquisition or native Q-wheel input is introduced.

For #313 reference validation, `benchmark_fixture.rs` owns the shared
`current-cap` / `current-mixed` 500/1,000 initialization used by native runs and
`sphere_workloads.rs`. Preserve the legacy sphere and onboard layouts; see the
[canonical benchmark guide](../../scripts/FRAGMENT_BENCHMARKS.md#native-current-sphere-profiles-313).

Deterministic runner children explicitly isolate desktop input/focus from protocol
holds and fixed-step time (#313); direct native `Game` probes retain OS input.
See the [harness boundary](architecture.md#deterministic-e2e-desktop-input-isolation-313)
before interpreting a scenario pass as native input/lifecycle evidence.

Scout machinery #464: read the [canonical feedback boundary](../../docs/scout-presentation.md#machinery-feedback-boundary-464). Runtime maps snapshots in `scout_feedback.rs`; renderer owns `machinery.rs`/ship roles/shader. Preserve pause/restore and power/work gating tests; run the opt-in offscreen GPU evidence check for shader changes.

Pause saves (#382) use the shared worker/store without advancing authoritative
clocks. Saved/UTC require verified completion. Resume cancels exit intent; failure
requires retry/cancel/explicit risk exit. See [contract](architecture.md#pause-checkpoint-controls-382).

Tutorial #380 lives in `session/tutorial.rs`; record progress only at successful
action boundaries. It is durable per-run state, not UI cache. T skips; native
reference review remains #376. See the onboarding section in architecture.md.

Engine #493: B input uses `Command::ToggleEngines` and ship-owned authority;
`session/persistence.rs` migrates v1-v4 force records to schema 5. Keep visual/audio
feedback conditional on explicit engine state, with native review #498. Portable
`flight_regression` tests also replay landing-cutoff and cockpit-monitor evidence
JSON through production commands, ignoring only screenshot markers. Keep
screenshot flight timings aligned with the current engine and assert assist
admission before advancing toward landing; native periodic runs verify capture
and pixels. Software-Vulkan corpus renewal is tracked in #518.

Engine cloud verification #498 adds a clear-space disk regression in
`session/propulsion_tests.rs`: the real two-generation store and normal
`Checkpoints::open_at` launch preserve engine-off, selected throttle and momentum.
Refueling cannot enable the switch; a fresh command is required. This tests the
existing save/engine contract without changing source ownership or gameplay.
The [report and native driver](../../reports/issue-498/README.md) retain exact
commands, GPU instruments, realistic-flight routes and human/hardware limits.


Scout UI #466: prompt and crafting/status glyphs reuse renderer `ui_style`.
See the [canonical surface matrix](../../docs/scout-presentation.md#scout-ui-readability-contract-466);
retain contextual/expiry and modal admission tests, and keep power/interaction
rules out of palette/raster changes. Scanner integration is separately #503.


Archive #467: follow [the source/persistence/input contract](architecture.md#scout-archive-467).
Record identity/read selection is source; the modal is transient. No log mention
grants a navigation fix. Preserve schema-6 transaction, modal, ray and restart tests.

## Data Recovery Module (#468)

`session/recovery.rs` owns per-run module identity, captured config and paid
record progress. `archive.rs` composes R start/stop and bounded evidence text;
`fabrication.rs` moves the one component tray identity into the installed mass
ledger. Ordinary ship/Fabricator tick settlement precedes low-priority Core
funding; recovery only spends surplus above an emergency floor. Schema 7
validates installed/pending/tray identity and reconstructs mass without charging.
See [canonical contract](../../docs/data-recovery-module.md). No offline work,
free coordinates or reconstruction of irrecoverable bytes. Native review #515.

For #455 inactive-history attribution, run the independently selectable sleeping
lane in the [benchmark guide](../../scripts/FRAGMENT_BENCHMARKS.md#independent-inactive-history-measurement-455).
It preserves the existing fixture and records CPU adapter costs, never native FPS.

Native cargo review #477 uses an opt-in test-only checkpoint exporter. The
[validation guide](../../docs/validation.md#normal-clock-cargo-review-fixture-477)
records its explicit fixture/normal-launch boundary. It changes no production
input, save schema, physics or interpolation policy.

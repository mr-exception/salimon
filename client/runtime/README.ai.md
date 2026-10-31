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

Starter Scout movement/carrying scenarios cross the authored cargo bulkhead
through its central opening. `flight_regression.rs` replays the production
command paths; [starter-layout](../../scenarios/evidence/starter-layout.json),
[cockpit-nose](../../scenarios/evidence/cockpit-nose.json) and
[fragment-transfer](../../scenarios/fragment-transfer.json) retain the actual
walking, Archive-targeting and post-flight retrieval routes. Runtime consumes
character-owned generated fixture bounds; it does not derive collision geometry
from rendered meshes. Cargo-door/bin mechanics are implemented by #539;
habitation interactions remain #540.

Start in `app.rs` for native lifecycle, recovery, scheduling and evidence IO.
Use `session.rs` for typed contracts/domain fields, `session/commands.rs` for
command admission and `session/tick.rs` for exact cross-domain update order; `session/frames.rs` and `session/interaction.rs` own portable adapters.
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
- For the explicit engineering camera, map all six fixture bodies to spheres,
  the Sun to a point light, and the three
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

Starting supplies (#379): normal launch/restart use `GameplaySession::new_canonical_run`;
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
The [report and native driver](https://github.com/mr-exception/salimon/blob/d003bd58d506e04e3df97744f29f68449b6e2b3e/reports/issue-498/README.md) retain exact
commands, GPU instruments, realistic-flight routes and human/hardware limits.


Scout UI #466: prompt and crafting/status glyphs reuse renderer `ui_style`.
See the [canonical surface matrix](../../docs/scout-presentation.md#scout-ui-readability-contract-466);
retain contextual/expiry and modal admission tests, and keep power/interaction
rules out of palette/raster changes. Scanner integration #503 maps authoritative observations to renderer `ScannerReadout`
and reuses this action bar, preserving lowercase SI glyphs.


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

## Blueprint tool and placeholders (#542)

`session/construction.rs` composes ship-owned placement with actual authored
character geometry, individual physical cargo and protected starter escape/service
lanes. `catalogs/blueprints.json` declares seven furniture/wall/door/hull/deck/room/
module entries. Its costs/durations are **proposed estimates**, with
`approved_economy=true` following the explicit owner decision on 2026-10-24.
#543 accepts physical materials; timed progress still belongs to #544.
#537 owns economy approval before #543/#544 integration. Fine pitch is per-entry
data (currently 0.5 m), not a mandatory global furnishing rule.

C opens/closes the tool, 1–7 select, R rotates, left click places, E inspects the
aimed project, right click cancels the aimed reachable reservation. Escape closes
before pausing. Native and protocol inputs use the same session commands and
native focus/repeat/synthetic admission remains in the shell. Opening stows the
mining tool; closing never re-equips it. Existing essential carrying/escape paths
are protected. The initial coarse structural graph is an explicit support model;
it does not move authored anchors or generate physical hull. Exterior reservations
also reject the conservative existing hull, wings and engine volumes. Actual hull
modification/solid generation remain #545.

`app/scene.rs` maps hollow colored edges into ordinary oriented cuboid instances:
green clear preview, red declined preview, cyan placed project. These render-only
edges never become solid colliders or installed mass. Requirements/status stay
visible together with transient feedback in the action bar. Whole-session schema 8
preserves project source and rejects malformed restore atomically, while tool UI
resets. See the canonical [save contract](../../docs/save-contract.md#blueprint-placeholder-schema-8-542).
Portable and native routes use `scenarios/evidence/blueprint-placement.json`.


## Physical cargo door and open bins (#539)

Character `cargo.rs` supplies the authored aperture, opposed sliding-leaf bounds
and hollow bin shells. Walking retains conservative bin envelopes. Loose cargo
and pickup rays use the actual 4 cm side walls, 12 cm bottom and open top; no
slot inventory, identity replacement, mass exemption or zero-G pin is introduced.
Runtime `session/cargo.rs` composes manual door motion and swept player/object
clearance, independent of the aft ramp, power and flight state. Aim at the
port-side door handle and press E; reverse with a second press. Partial openings
remain conservatively impassable. Door position is saved in schema 9 and
legacy captures migrate open; see the [save contract](../../docs/save-contract.md#cargo-door-schema-9-539).


## Base habitation (#540)

The bunk's two authored panels use the generated
`habitation_geometry.rs` hinge/swept bounds, sourced from
[the category definition](../../models/assets/ships/salimon-scout/habitation-geometry.json).
The renderer maps a finite stowed/deployed fraction to rigid rotation and normals;
runtime admits manual motion only after full player/belonging clearance. Static
poses cost no repeated clearance scan. Reserved walking clearance is unchanged.
Galley E opens the existing exact-identity consumable wheel, with no generated
food or additional inventory; hygiene remains an inspectable base fixture.
No water/hygiene meter, sleep-time advancement or additional survival benefit is
introduced. Expanded sitting/lying, restraints and privacy mechanisms are implemented by #577; see [cabin mechanisms](../../docs/cabin-mechanisms.md). Schema 10 saves source motion; see
[the save contract](../../docs/save-contract.md#habitation-schema-10-540).


## Interior fixture composition (#575)

`session/fixtures.rs` maps existing saved habitation state to the versioned
[interior layout](../assets/ship/interior-layout-v1.json), preserving the stable
berth ID and ship-rooted f64 transform. `app.rs` supplies borrowed fixture DTOs;
renderer resolves/batches presentation and never owns collision, installed mass,
construction, cosmetic age or damage authority. Native debug inspect includes
actual fixture draw/memory/upload counters; `window_state` reports true focused/
occluded flags. Schema 10 is unchanged. Future room/module owners supply their
real parent IDs and local poses through this boundary. See
[architecture and budgets](../../docs/modular-interior-fixtures.md).


Continuous audio verification: `native_audio/mix/tests.rs` covers callback-size
invariance and reset/mute/stale control; original 60 Hz stereo fixtures and real
device telemetry are opt-in. Linux builds require ALSA development headers.


Capture verification (#568): native `inspect` with `scope: capture` returns only
readback status after the usual E2E envelope/admission checks. It is observational
and never builds the world snapshot. Ordinary inspection is unchanged; old
executables may ignore the optional scope and return the full snapshot. The
runner restores the real window before GPU capture and logs that OS request;
it does not fake focus, disable occlusion or grant physical input qualification.
See [runner protocol](../../scripts/README.md#bounded-native-capture-polling-568).

## Physical mounts (#576)

Use `session/mounts.rs` for G/K/J, save source and collider synchronization; ship owns attachment rules and world owns the one-object slot. See [the canonical contract](../../docs/physical-mounts.md).


Persistent finishes #579: [canonical ownership, schema 13, bounded render factors and controls](../../docs/room-finishes.md). Ship owns room/item cosmetic identity; runtime owns nearby physical control admission; renderer applies the explicit nonfunctional surface allowlist. Cosmetic changes never edit physical material/age/damage.


Instant Core upgrades #548: [canonical input, source, mass and schema-14 contract](../../docs/core-upgrades.md). Ship owns validated capacity-only transactions; world returns original caller-owned reserved entities through the one carry slot; runtime maps the data catalog/physical controls and reconstructs pending+installed material mass. No timer, joule grant, Core replacement or flight command.


Construction receipts/refunds #543: [canonical transaction, mass, controls and schema-15 contract](../../docs/construction-materials.md). Funding respects the explicit economy approval gate; progress/geometry retain #544/#545 owners.


Room and fixture lights #580: [canonical controls, source, effective output and schema-16 contract](../../docs/room-lighting.md). Master/blackout/inoperable lamps suppress output without changing preferences; critical machine displays remain independent.


Tactile sound/acoustics #583: [canonical source-event, partition, callback-budget and evidence contract](../../docs/scout-acoustics.md). Actual travel/attachments trigger cues; restore audio epochs are transient. E2E exposes silent preview mapping and keeps device playback suppressed.


## Automatic construction (#544)

Follow the [canonical progress/completion contract](../../docs/construction-progress.md). Ship owns validated paid nanoseconds and ppm; runtime owns actual Core funding, current safety and atomic interior/structural installation. No unfinished collider, free progress, duplicate mass or unsupported module completion. Schema 18 restores exact work and validated modular edits without elapsed-time credit; see the completed modular structure contract below.


## Completed modular structure (#545)

Follow the [canonical assembly, traversal, identity, mass and schema-18 contract](../../docs/modular-construction.md). Completed cell parts and face edits regenerate shared render/collision geometry; unpublished projects supply no support or colliders. Preserve the existing gate, per-cell identities, atomic restore and single-owner material ledger.


## Construction flight readiness (#546)

Follow the [canonical Stop/Resume, physical securing, takeoff and schema-19 contract](../../docs/construction-flight-safety.md). The durable stop latch never clears on landing; actual hull/attachment state governs readiness. Preserve Core/engine/cockpit/ramp authority and inertial momentum.


## Optional module commissioning (#547)

`session/construction/modules.rs` owns physical processor tiers, activated downtime, maintenance funding/work and actual commissioning faults. `session/recovery.rs` requests real tier-dependent Core watts. Schema 20 validates module/receipt ownership atomically; cancellation never restores activated equipment without paid restoration. Follow the [canonical controls, tuning, source/test and deferred Jump Drive contract](../../docs/non-core-modules.md).


## Cosmetic surface age (#581)

Follow the [persistent exposure, schema-23 and clean-repair contract](../../docs/surface-age.md). Ship owns bounded integer exposure; runtime admits physical identity/use and verified offline time; renderer consumes observations only. Recolor/repair retain age, replacements use fresh IDs, and no cosmetic state grants damage or collision authority.


## Domain automation ownership (#639)

`automation.rs` owns the line reader, common validated envelope, deadlines,
paused/modal admission and protocol responses. `automation/native.rs` owns
native-window/capture observations and commands; `automation/inspection.rs`
retains the observational snapshot. Registered handlers in `automation/`
(`survival`, `crafting`, `construction`, `lifecycle`, `clock`, `movement`, `input`)
own operation parsing and typed production calls. Add operations with their
owner; update central registration only for a new owner. An unrecognized handler
returns `None` without effects; recognized validation failures return the same
protocol error. The native event thread remains the only executor. Legacy
aliases live in `automation/keys.rs`; tests retain the original scenario corpus.


## Domain input bindings (#640)

`app/input.rs` translates physical keys into `session/input::Key` and owns native
focus/capture/repeat/synthetic-event admission. `session/input/` owns portable
construction, lighting, ship, equipment and interior bindings. Modal owners
(Archive, wheel, crafting) retain explicit priority: construction admission,
Escape/pause, Archive, Q/T, wheel, crafting, then ordinary commands. A domain
binding change normally edits its owner; a new physical key also needs native
translation. Native and automation dispatch through the same typed boundary.
Input reset remains centralized in `GameplaySession::clear_held_controls`.


## Domain save adapters (#641)

`session/persistence.rs` retains the explicit serialized section layout and
ordered decode/validate/reconstruct/publish transaction. Domain `save.rs` modules
own historical-version admission, source-specific reconstruction and contextual
rules (survival, construction, cargo/habitation, mounts, fabrication, Archive,
Recovery, lighting, finishes, Core upgrades, surface age, tutorial and offline
clock). `session/persistence/physical.rs` owns the physical composition adapter;
`legacy.rs` isolates historical propulsion compatibility. No schema or field
layout changed. Keep unknown-field rejection and publish only after all owners
and cross-domain invariants pass. Generic persistence codec/storage remain
independent of gameplay domains; there is no dynamic save registry.


## Session composition owners (#642)

Held input lives in `session/input/state.rs`; external observers receive Copy
snapshots through `movement()`/`steering()`, while mutation stays within session
composition. `session/policy.rs` groups host automation step, interpolation and
physics measurement flags; copy this single nonserialized value through restore,
offline reconciliation and restart. `session/init.rs` distinguishes empty
reconstruction defaults from supply-granting new runs. Command admission and
interaction details live in `session/commands.rs` and `interaction/actions.rs`.

`session/tick.rs` is the explicit scheduler: admit pause/terminal/offline and
survival time; begin render history; advance age/camera, steering/Fabricator/ship,
crash interruption, Recovery then construction; use the completed ship frame for
character/cargo/habitation; advance mining then loose physics; record journey,
synchronize carrying/mass, validate UI focus, freeze terminal state and complete
render history. Stage bodies live in `tick/power.rs`, `character.rs`, `resources.rs`
and `presentation.rs`. Do not register unordered callbacks or move power consumers
across these boundaries. Domain-specific behavior normally changes its stage or
owner, without editing the central session file. New domains/shared contracts
still require explicit composition review; this is not a generic plugin system.


Native presentation (#643): redraw/overlay composition lives in
`src/app/presentation.rs`, contextual prompt policy in its `prompts.rs` child,
and Scout visual fields in `src/app/scene.rs::gameplay_ship`. Read the
[presentation boundary](architecture.md#native-presentation-composition-643)
before editing lifecycle/input in `app.rs`.


## Predictive spatial streaming (#422)

See the [streaming contract](../../docs/universe-streaming.md) for worker ownership,
nonblocking runtime polling, explicit residency limits, stale-result rejection
and immutable quality-independent content. Startup uses `SALIMON_UNIVERSE_PACK`;
loaded catalog truth does not grant player discovery; ordinary gameplay uses the separate canonical Vesper publication.


## Canonical terrain streaming (#424)

See the [canonical terrain contract](../../docs/universe-terrain.md) for the exact U11
evaluator, separate frozen identity/codec, planet-wide quadtree, stitching and
CPU/GPU bounds. Compressed six-body fixtures remain unchanged. Ordinary canonical gameplay and alien-world startup are implemented by #425/#426.


## Canonical surface gameplay and schema 24 (#425)

Follow the [canonical surface contract](../../docs/universe-gameplay.md):
selected body/terrain context, immutable U11 deposits, shared physical ground,
conserved fragment/cargo/Fabricator ownership and exact U05 definition binding.
Historical fixtures remain explicit; normal new-run startup uses the canonical constructor.

Canonical new journeys (#426) use `session/canonical.rs::new_canonical_run`: frozen Vesper ID, published traveler offset [14,0], conservative authored-hull clearance and a view facing the landed ship. Fresh terminal restarts preserve run serial/seed policy and grant existing supplies once; restore never grants them. Flat fixtures remain explicit; normal launch rejects prototype saves under #427. Oxygen depletion stays deferred. Tests and native evidence: `reports/issue-426/README.md`; setup `canonical-journey`.


## Bounded navigation prediction (#421)

Follow the [prediction/telemetry contract](../../docs/universe-navigation.md): ship owns cloned real-scale coasting and current-controller previews, runtime owns disposable observation caching, renderer owns instrument DTO/rasterization only. Preserve explicit model assumptions, bounded horizon, sampled contact/arrival semantics, empirical convergence (not a certified error bound), visible validity/age and nonmutation of controls, power, source saves and simulation clocks.


## Scanner delivery (#429)

Universe `scanner/pulses.rs` owns fixed five-second cadence and analytical funded linear-path detection. Runtime `session/scanner.rs` composes ordinary Core funding after construction, samples current body-local state and exposes read-only automation. Unknown orbital phase/velocity is never invented; body-local and galactic observations cannot be mixed.
See the [scanner contract](../../docs/universe-scanner.md) for source/test mapping, units and limitations.


## Scanner delivery (#431)

Scanner contact checkpoints validate catalog revision, exact IDs/frames, timestamp ordering and bounds before atomic reconstruction. Runtime schema 25 saves canonical scanner parameters, clock/remainder and encounter history; legacy canonical saves migrate empty, current missing/malformed sources reject. Closed-app launch advances freshness once without inventing offline detections.
See the [scanner contract](../../docs/universe-scanner.md) for source/test mapping, units and limitations.


## Scanner delivery (#430)

Renderer `scanner_hologram.rs` owns bounded depth-tested ship-local 3D projection geometry; runtime `session/scanner/presentation.rs` maps only fresh same-frame discoveries to ship pose. The physical console projector shows logarithmic distance, true 3D direction/elevation and selected contacts. M cycles contacts only in an admitted canonical cockpit; power loss suppresses projection, not history.
See the [scanner contract](../../docs/universe-scanner.md) for source/test mapping, units and limitations.


## Scanner delivery (#469)

Universe `scanner/chart.rs` owns player-known observations, inferred/transmitted leads with explicit uncertainty, and immutable voluntary route annotations. Runtime `session/chart.rs` owns cockpit P/M/Enter controls and schema-26 pair validation with scanner history. Catalog truth never seeds the chart; observed knowledge outlives contact TTL and source frames are preserved without guessed transforms.
See the [scanner contract](../../docs/universe-scanner.md) for source/test mapping, units and limitations.


## Geological surface materials (#613)

Follow the [geological material contract](../../docs/geological-materials.md) for terrain-bound
profile mapping, immutable material cache keys, the 52-byte vertex layout,
filtered stratification and original software-Vulkan evidence. Runtime owns
geology-to-DTO mapping; renderer owns shading only. Vesper and resource palettes,
physical queries, geometry and save identities are preserved.


## Hybrid astronomical sky (#423)

See the [sky contract](../../docs/universe-sky.md) for bounded presentation,
split-origin/source validation, explicit frame/epoch limits and production
verification. Sky visibility grants no scanner/chart knowledge or simulation authority.


## Optical atmosphere (#614)

The [atmosphere contract](../../docs/planetary-atmosphere.md) maps immutable climate
optics, runtime light inputs and depth-limited production scattering. Vacuum
remains empty; presentation never changes gas physics, weather or canonical identity.


## Coherent weather presentation (#616)

See [weather visuals](../../docs/planetary-weather-visuals.md): shared-TT source
fields, fixed body-frame maps, bounded optical rendering and exact source/quality
separation. Vacuum stays empty; no terrain, hydrology, weather truth or sound is invented.


## Organism family grammar and presentation (#619)

Follow the [organism contract](../../docs/planetary-organisms.md) for stable
family/part identities, cosmetic tile sampling, neutral segment mapping, bounded
instanced geometry and source-driven GPU evidence. Physical population remains #620.


## Physical organisms and static clearance (#620)

The [physical organism contract](../../docs/physical-organisms.md) maps canonical
placement/part/baseline identity, physics-owned swept boxes, character/ship typed
adapters, runtime residency and matching renderer evidence. Quality never changes
physical populations. Explicit obstruction safety overrides ordinary assist continuation.

## Biological material response (#622)

[Organism materials](../../docs/planetary-organisms.md#biological-materials-and-weather-response-622)
map stable species traits and explicit shared-time weather to bounded renderer
instances. Keep root/physical anatomy, collision envelopes and quality population
invariant; shader flex stays inside the canonical bounds. Renderer owns shading,
runtime owns source/weather mapping; neither changes world identities.

## Geological site identity (#623)

[Landmark source and validation](../../docs/planetary-landmarks.md) binds sparse
sites to actual published terrain relief. Keep physical source, renderer vertices,
immutable site IDs and save/reference binding consistent across LOD and eviction.
No chart/scanner discovery, duplicate terrain surface or invented cave geometry.

## Natural signature sites (#624)

[Biological signature contract](../../docs/biological-signature-sites.md) maps rare
site metadata onto exact physical organisms and geological/activity provenance.
No source geometry, collision/LOD anatomy, identity or player knowledge changes;
strict publication binding survives reconstruction and rejects mismatched sources.


Ordinary-play isolation (#427): [canonical runtime and prototype-save gate](../../docs/universe-gameplay.md#ordinary-canonical-play-427).


## Suit observations (#625)

Follow the [measurement, frame, error and schema-27 contract](../../docs/suit-observations.md).
Canonical player knowledge stores estimated spots/bearings with explicit uncertainty;
validated reload cannot reveal catalog POIs or invent a target range.


## Personal planetary chart (#626)

Follow the [chart, uncertainty, inspection and schema-28 contract](../../docs/planetary-chart.md).
Personal edits/targets never mutate canonical sites, suit observations or scanner history.


## Planetary environmental audio (#627)

Follow the [observational source, media, lifecycle and accessibility contract](../../docs/planetary-audio.md).
`environment_audio.rs` maps actual local definitions into five reserved continuous
callback channels; audio has no simulation/save authority. Preserve vacuum silence,
physical contact gates and independent environment settings.


## Biological harvesting and schema 29 (#621)

Follow the [yield, atomic transfer, geometry and persistence contract](../../docs/biological-harvesting.md).
World owns explicit source designations, revision/depletion journals and physical
publication; runtime composes F/tool input, current anatomy and source-bound restore.
Schema 29 preserves missing/partial parts and natural object identity. Vesper
remains sterile; positive sources are explicit fictional geometry fixtures.

Combined planetary environment evidence: see the [matrix runner](../../scripts/README.md#planetary-environment-matrix-628).
`scout_reference.rs` owns the canonical night camera and ground-clearance test;
positive atmosphere/life evidence remains explicitly fictional component fixtures.

Flight acceptance #437: see [the reproducible matrix](../../docs/universe-flight-verification.md) for nine GPU routes, production orbital/contact/session contracts and separate X11 input coverage.


Canonical offline motion #432: [source, schema-30 and verification contract](../../docs/universe-offline-propagation.md). Runtime stores the model epoch/frame, composes the same ship step online/offline, disables engines on closing/launch and refuses unverified motion publication.


Offline timed/scanner integration #433: [chronology and analytic idle pulses](../../docs/universe-offline-propagation.md#offline-timed-systems-and-scanner-433). `session/scanner/offline.rs` owns the stationary certified fast path; active motion/work retains the actual scheduler.


Native verified resume #434: [asynchronous worker, real progress and current-region admission](../../docs/universe-offline-propagation.md#native-progress-and-verified-resume-434). `launch.rs` transfers verified bytes; UI/thread-local query caches remain on the main thread.


Current U01–U33 truth and explicit fixture boundaries are mapped in the
[U36 consistency audit](../../reports/issue-441/README.md). Canonical sky/light,
streaming observer/velocity, physical environment and preview share the saved
motion source; suit field-mark timestamps remain personal active-time metadata.

The internal `hall_prototype.rs` adapter is admitted only by its explicit E2E
scenario and never serialized. World owns layout/collision; runtime maps existing
renderer DTOs. See [the prototype contract](../../docs/hall-prototype.md) before
changing its gate, camera, controls or capture path.

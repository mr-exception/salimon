# Runtime Invariants

1. Window-bound renderer state is created only when the application lifecycle
   permits a native window and is never rendered before initialization.
2. Zero-width or zero-height drawable sizes are not used to configure or render
   a GPU surface.
3. Every accepted nonzero resize reaches the renderer before the next presented
   frame.
4. Frame and update intervals come from monotonic clocks. Native gameplay retains
   observed elapsed time as backlog and advances all domains in bounded 16 ms
   ticks; diagnostics may clamp display intervals. The renderer never advances
   simulation time. Lifecycle exclusions are explicit; see the tick contract.
5. Recoverable surface loss rebuilds or reconfigures rendering state. A transient
   surface acquisition failure is delayed and does not crash or hot-loop the
   application.
6. A close request or destruction of the sole window exits the event loop
   cleanly. Unrecoverable renderer failures are reported before termination.
7. The runtime may orchestrate renderer/platform capabilities but must not own
   rendering implementation, authoritative domain state, or backend behavior.
8. World, character, and ship code must not receive raw `winit` events or
   `wgpu` resources from the runtime.
9. The engineering overlay is hidden by default and toggles only on an initial
   physical F3 press, never on release or key repeat.
10. Diagnostics observes typed runtime, renderer, and future domain snapshots;
    it never becomes authoritative state or changes simulation behavior.
11. Suspension, occlusion, zero-sized drawables, and renderer reconstruction
    reset the rolling timing window before presentation resumes.
12. Native input is translated into typed portable commands; `winit` event types
    never enter the world crate.
13. Renderer reconstruction preserves portable camera state. Lifecycle gaps
    reset the update clock so the prototype cannot jump forward on resume.
14. Each world snapshot maps exactly six spheres plus three separate noncanonical
    marker cuboids. Sphere centers/radii remain `f64`. The Sun maps to emissive
    material and the point-light position; every solid maps to a lit material.
    Gameplay may add bounded terrain patches and lower-envelope presentation radii
    under the canonical terrain rules; nominal catalog radii remain unchanged.
15. Diagnostics body observations preserve catalog order/names and report finite,
    nonnegative camera-to-surface distances (nominal in flat fixtures) for all six bodies.
16. Gameplay starts inside the ship landed on Earth; F2 preserves access to the
    earlier precision-tour fixture without changing domain state.
17. Runtime contextual E routing changes cockpit authority or requests a door
    action, but never owns the resulting character/ship behavior. Cockpit entry
    requires the character to be inside the ship as well as in range and aimed;
    the same gate presents `[E] Use cockpit`. Exterior and doorway characters
    cannot gain cockpit authority through a spatial interaction hit.
18. Escape releases cursor capture so native window controls remain reachable;
    clicking the game view restores mouse-look capture.
19. Gameplay interaction zones match the authored cockpit and exit-door markers;
    runtime composition does not apply a second scale to the baked ship mesh.
20. Cockpit W/S, A/D, and Left/Right Arrow state maps to typed pitch, yaw, and
    roll without changing the independent character mouse-look state; A turns
    left and D turns right in the rendered cockpit view.
21. Up/Down Arrow forwards real press/release edges to ship-owned signed
    throttle and its 100 ms simulation-time repeat cadence. OS repeat and
    synthetic focus presses are ignored. Exit, focus/lifecycle/view changes clear
    held controls while preserving selected throttle.
22. L is translated to the ship domain only on an initial press; the runtime
    cannot cancel or steer an active assisted landing/takeoff sequence.
23. Character surface traversal receives the body frame stored by landed or
    assisted ship state, so every solid body uses the same character gravity.
24. Every visible ship receives current snapshot speed, thruster, Core energy,
    and optional nearby-body distance/radial data through renderer-owned
    `CockpitInstruments`; leaving the cockpit does not freeze or hide these
    values. Runtime field-maps rather than derives flight telemetry or simulates
    monitor values. Contextual messages remain in the window title.
25. Applicable cockpit, door, fragment and deposit prompts carry current absolute
    world anchors. Existing eligibility/obstruction gates select them; renderer
    projection/scene depth determines presentation visibility. Carried-object
    drop guidance follows the carried pose. Global flight/tool guidance and
    three-second blocked feedback remain screen-space. Precision tour hides
    gameplay prompts.
26. E2E scenario setup requires an explicit launch flag and occurs before the
    native event loop. It changes only initial controller state and update-clock
    policy. A ready line follows successful renderer initialization; ordinary
    launches retain the default state and monotonic clock.
27. The versioned JSON automation channel starts only after E2E renderer readiness.
    Its reader never touches game state; commands run on the native event thread.
    Only explicit fixed steps advance E2E simulation, and gameplay actions retain
    production interaction and controller gates. Ordinary launches do not read stdin.

- Handheld extraction requires gameplay view, Surface location, an equipped tool,
  held mining input, and a valid unobscured world target. It never uses E.
- Inspection and presentation apply the same world-owned session mass deltas.
- Focus loss, cursor release, view switch, and stowing clear held mining input.
- Fragment presentation/inspection read world-owned physical entities; neither
  can mutate mass or allocate output. Stowing does not remove fragments.

- Authored handheld presentation is visible only in gameplay on the surface
  while equipped; active feedback requires held input and a valid target. The
  separate screen-space reticle remains available when stowed.

- Gameplay aiming uses a screen-space centered dot when the tool is stowed and
  `+` when equipped, selected from the actual equip state every redraw. Camera
  pitch/yaw, location and drawable resize do not shift it. Precision tour hides
  it. No world-space aim cuboid is emitted; interaction/mining rays are unchanged.

- Loose-fragment simulation preserves session iteration order, excludes the carried
  ID and objects at least 125 m from the player, and preserves IDs/mass/material during pose writeback; physics updates the normalized orientation. Ship anchors/velocities use ship-local metres;
  surface objects use absolute metres; orientation/angular motion follows the same frame. Physical response is owned by
  `salimon-physics`; runtime supplies geometry and frame conversions only.

- F initial presses grab an aimed reachable world object, or release the carried
  object. A miss never starts mining; repeated or synthetic native presses cannot
  repeat the action. E never changes carrying.
- Only left mouse controls held mining. F press/repeat/release never activates
  the tool or sustains extraction after mouse release. Successful pickup cancels
  mining. Focus loss, cursor release, view switch and stow clear input and F latch.
- Automation `grab_drop` and legacy `pickup`/`drop` aliases use the production F
  route. Mining uses the shared left-mouse route; the old `mine` key is rejected.

- Equipment toolbar state has exactly five slots, initially mining tool then four
  empty slots, and zero or one selected slot (initially none). Empty selection
  targets are valid; repeated selection of the same slot never toggles it off.
- Gameplay 1–5 selects slots only on real initial native presses with cursor
  captured; releases, repeats and focus replay do not select. Precision-tour
  1–6 inspects bodies without changing equipment state.
- Successful physical pickup clears toolbar selection immediately; a failed pickup
  preserves selection and mining input. World-session carrying blocks all slot
  selection; dropping never restores selection. Explicit selection after drop
  never resumes a previous mining hold.
- Toolbar loadout/selection maps directly to renderer DTOs in gameplay and is
  hidden in precision tour. Toolbar presentation cannot change equip behavior;
  slot 1 is the sole mining equip authority; empty/absent selection stows it.
- Mining owns no independent equipped boolean. Targets, extraction, held tool,
  reticle and prompts all consume the toolbar decision. Switching slots cancels
  held input immediately; pickup cancels mouse mining without releasing the
  consumed F latch. M and the legacy equip automation key are unavailable.

- The carrying baseline/evidence route protects selection and mining together:
  all four empty slots must prevent left-mouse extraction at a reachable deposit, while
  explicit slot 1 selection restores the held tool and extraction eligibility.
  Evidence checkpoints observe equipped, empty-selected, carrying-deselected
  and post-drop-deselected states without changing gameplay actions.

- Resource gameplay dimensions use the shared world `resource_size` policy;
  enlarged bounds never substitute for density-derived solid material volume.
  See [resource size contract](../world/resource-contracts.md#gameplay-size-policy-144).

- Continuous mining reserves at most one attached identity, default cap 10 kg,
  and releases on input/aim/cap/depletion/interruption. Attached entries follow
  tool front-left and are excluded from loose physics/pickup. Released entries
  retain ID/mass/source; continued hold creates a new identity next tick.
  Legacy facet-emission helpers remain only for old fixtures/benchmarks.
  Sphere visuals use the shared #232 mesh and one nonphysical growth preview.
  Released sphere radius drives contact, pickup, carry and drop geometry under
  the world-owned resource-size contract (#233–#234).

- Benchmark populations are opt-in initial conditions from production extraction;
  subsequent updates, carrying and support queries retain production behavior.
  Benchmark observations never change IDs, mass, poses or collision ordering.
- Native wall-clock benchmark results and fixed-step correctness/CPU costs remain
  distinct. Raw benchmark frame intervals preserve spikes; missing GPU/sleep data
  is null/explicit rather than synthesized. Requested settled state must be checked
  against observed moving-object counts.
- Native benchmark warmup/collection requires focused, known-visible, non-minimized,
  unoccluded window observations. Loss before retained intervals resets warmup;
  loss during collection fails, including suspension/surface loss. Results retain
  PID/lifecycle admission evidence. Successful present-call intervals do not certify
  physical display scanout; CPU-only measurements require no native readiness.

- Fragment activation follows stable IDs at the portable physics boundary.
  Sleeping objects stay in session presentation/picking/collision; omission wakes
  dependent contacts and streaming return rebuilds support. Ship-local rigid
  movement preserves sleep and f64 world writeback. See the [activation adapter](architecture.md#fragment-activation-adapter-156).

## Terrain integration (#179–#181)

Follow the canonical [integration/budgets/validation guide](../../docs/terrain-validation.md)
and [world definition contract](../world/terrain-contract.md). Keep local rules
and their consumers aligned; flat deterministic fixtures are explicit.

- Cargo mass composition drains physical mutation events after carrying/physics
  synchronization. Centers use current authored ship bounds independent of door
  animation; carried and loose objects share one identity. Stationary frames need no extra cargo scan; changed containment frames
  reclassify retained centers under the #306 invalidation contract.

- Core power snapshot gates character/fragment artificial gravity and blanks the
  physical monitor. No planetary fallback applies inside the unpowered cabin.

## Unified input admission (#302)

Native gameplay presses require focused, non-occluded Gameplay view and cursor
capture. Initial real edges only are accepted; release edges always clear holds.
Automation explicitly bypasses desktop admission only and shares view/domain
gates. View, focus/capture loss and lifecycle resets clear movement, steering
including its ramp, mining/F and held throttle, preserving selected throttle.
See the [canonical dispatch policy](architecture.md#gameplay-input-dispatch-302).

- Exact identical fragment poses generate no redundant physical journal event.
  Pickup/drop, growth/removal and actual pose/frame changes retain their events.
  A changed containment frame reclassifies retained centers so a moving ship
  cannot miss a stationary object's boundary transfer. An unchanged settled tick
  performs zero journal-driven containment/mass visits.
- Adapter snapshot/state/ID buffers and the solver's six per-body arrays retain
  capacities; initialization/iteration order and interpolation history are unchanged.

## Gameplay-session boundary (#309)

- `GameplaySession` composes portable controllers, resource/cargo state, typed
  commands and snapshots. It has no window/GPU handles, protocol reader, evidence
  IO or benchmark recorder. Supporting domains and renderer never depend on it.
- Native and automation admission translate to the same session commands. Releases
  still clear both movement and steering meanings across view/location changes.
  Native lifecycle resets clear holds/history without changing selected throttle.
- E2E fixtures, benchmark initialization and headless protocol regressions construct
  the production session directly. Every complete gameplay tick follows the order
  recorded in architecture; fractional/backlogged elapsed time is never discarded
  except by explicit lifecycle exclusion.
- `ResourceGameplay` owns the world session and fragment/cargo adapter caches;
  `MiningTool` owns only held mouse/F input. IDs, source mass and mutations remain
  world-owned; ledger and contact rules remain ship/physics-owned.
- Action-bar rasterization and transient expiration remain shell presentation.
  Session feedback is consumed immediately after dispatched native/protocol input;
  expiration follows completed simulation time, including explicit automation steps.

## Coherent gameplay presentation (#310)

- Render-only ship/character history uses the same clamped backlog alpha as
  loose fragments. Interior camera/objects use the presented ship frame;
  exterior positions retain `f64` world coordinates until renderer rebasing.
- Live mouse look, telemetry, interaction/mining decisions and physics remain
  current-state observations. Rendering never advances or writes simulation.
- Lifecycle/view discontinuities, teleports and location/flight-frame transitions
  bypass incompatible history. Doorway blending bypasses it throughout.
- New/removed/transferred/carried/released objects never reuse incompatible
  object history; current ship/camera attachments use presented frames.
- E2E presentation bypasses all interpolation. Normal-clock recordings and
  explicit human/native review remain required for smoothness/capture/latency
  signoff; portable schedules cannot establish it. See the [reference review](../../reports/issue-477/README.md).

## Fabricator physical input (#339)

The session owns a portable Fabricator. E inserts the one held world fragment
through a universal interior port, within 3 m and 30 degrees with unobstructed
sight. Core/cockpit/door targets retain priority; blackout does not gate intake.
The port uses the generated ship-local Fabricator input marker (#342).
Successful insertion removes the source exactly once and drains its physical
journal into mass, motion, anchors and presentation caches immediately. Full
reserves and invalid milligram masses leave both domains unchanged. The whole
accepted object is consumed, with overflow discarded. F pickup/drop is unchanged.

## Fabricator mass transitions (#341)

The runtime updates two dedicated cached ledger keys for integer raw reserves
and the one physical tray item. Every machine command and completion synchronizes
immediately; zero contributions remove their keys. World fragment consumption
drains the same physical journal before the input command returns, so carrying
never creates a second cargo identity. Enqueue removes committed ingredients
from accounted mass deliberately, collection removes tray mass, and cancellation
never refunds. Queue/in-progress/consumable inventory contribute zero. Existing
loose cargo and Core-consumed identity accounting retain their owners. Runtime
UI/inventory adapters must use execute_fabricator and validate receiving slots
before collection; no consumable collection UI is introduced by this issue.

Fabricator modal input is exclusive: opening clears held gameplay controls;
repeat/synthetic native presses never enqueue twice. Range/interior/occlusion are
revalidated on simulation ticks; leaving the monitor closes the modal. Display
reads never step or mutate crafting. Add/cancel use atomic domain commands.

### Consumable HUD (#346)

Runtime maps the existing three food and two medical slots into a separate
renderer DTO row. Equipment selection remains keys 1–5. M/N do not consume
items in gameplay; #377 owns Q-wheel selection. Survival effects use the shared #370 transaction.

### Fabricator automation (#347)

Inspection exposes stable ordered raw bins/jobs, tray identity and the existing
3/2 inventory slots without advancing simulation. `crafting` actions invoke only
the same open-modal action handler as native input; they cannot insert free
resources, open a remote machine or bypass transactional validation. Portable
session regressions run in PR CI; graphical and native signoff remain separate.


## Logical save boundary (#364)

The [canonical save inventory](../../docs/save-contract.md) defines schema-1
source state and excluded caches. Domain-owned save state is validated before
restore; runtime composes temporary owners and rebuilds mass, immutable terrain
queries and presentation/query/solver caches without advancing simulation.
Stable IDs, depletion, carrying, motion, Core/assist progress and logical
contact/sleep/dwell state survive. No input latch, OS handle or GPU state is
restored. An attached mining sphere becomes a detached existing physical entity,
without allocating a new ID or losing mass. Whole-session/Fabricator publication
is composed in #365; disk checkpoints remain #366.

## Whole-session saves (#365)

`FabricationSaveState` validates recipe definitions, ordered complete bins, queue
IDs/captured recipes/committed quantities/progress, allocator high-water marks,
tray and exact inventory slots together. Cross-owner item identities cannot be
both collected and on the tray. Runtime's `SessionState` is the concrete payload
inside `SaveGame`; `save_bytes` and transactional `restore_bytes` integrate all
physical state, equipment and exact elapsed/backlog clocks. Stored mass and
recipe watts are reconstructed without charging or refunding. UI/input/focus,
feedback and render caches reset; interactions are recomputed from restored
world state. The native host's automation policy stays host-owned. No disk IO,
autosave scheduling or offline simulation is implied; #366 owns durable storage
and shell lifecycle integration. See [canonical save contract](../../docs/save-contract.md).

## Hold-Q consumable wheel (#377)

Exact-identity survival consumption, continuing world/vital clocks and pause/death/
focus/restore cancellation follow the [canonical wheel contract](architecture.md#hold-q-consumable-wheel-377).

## Durable active-run orchestration (#366)

Runtime `checkpoints.rs` owns normal-play startup restore, a bounded save worker,
60-game-second/discrete-event scheduling, persistent native-title warnings and
verified clean-exit capture. Domain state stays on the event thread; owned bytes
cross to the worker. `salimon-persistence::storage` owns verification and native
filesystem durability. Startup renderer creation preserves restored clock backlog;
E2E and benchmark launches remain isolated from user saves. See the
[canonical save contract](../../docs/save-contract.md#durable-active-run-366)
for paths, recovery/failure policy, Windows behavior and future lifecycle owners.

- #369 movement/interaction modifiers and exhausted-pilot revocation consume
  typed survival snapshots at shared command/tick boundaries, with no native or
  automation bypass. See the canonical survival restriction contract.
- #371 ticks and saves survival on admitted active intervals; only fatigue recovers
  offline through the verified launch transaction. Emergency consumption permission never shares nonessential cooldowns.
- Exhaustion cannot pause ongoing Fabricator jobs, Core, ship or fragment physics;
  it interrupts held mining and clears manual pilot input/thrust without braking.

- #370 consumption identifies exact `(slot, job, unit)` and prepares validated
  recipe effects before removing one item. Repeated/stale clicks cannot consume
  a replacement identity; failed use preserves both owners, full vitals permit
  waste, and exhaustion alone never denies emergency food/medicine.
- The shared transaction respects Gameplay/crafting/terminal gates and changes no
  ship mass. Native focus/Q-wheel input remains #377; full-save/clock composition
  is owned by #371. M/N are not reintroduced.


## Survival lifecycle (#371)

- Schema 2 requires complete validated survival/effect/catalog/config/run state.
  Schema 1 migrates physical/Fabricator state once with full new vital meters.
- Restore never ticks or credits recovery. Native launch credits only fatigue in
  a temporary candidate and verifies its checkpoint before live publication.
- Terminal state truncates the lethal tick, clears inputs and backlog, rejects
  world commands and freezes all authoritative domains. IO/UI may still operate.
- Restart archives the terminal summary and verifies two fresh fallback slots
  before publishing fresh gameplay. A failed archive/write never silently revives
  a terminal run or acknowledges an unverified restart.
- Terminal IO retries use host time, independent of frozen gameplay timers.


Journey (#372): active duration comes from the survival clock. Combined physical
travel uses ship-origin and independent character tick paths, with cabin-local
coordinates removing passenger double counting. Saved samples never bridge a
teleport/restore/camera change. Occlusion/zero drawable suppress presentation while
bounded simulation and checkpointing continue; OS suspension remains an explicit
clock exclusion. Death archives before restart. See the canonical
[journey contract](../../docs/save-contract.md#journey-statistics-372).


Survival HUD (#373) is observational: icon/thin-bar meters carry no permanent
numeric text. Exact penalty labels consume the existing basis-point authority.
Animations/status details cannot mutate meters, clocks, inventory or saves.
H/[ / ] uses focused initial gameplay admission; focus/capture loss and competing
modals close/suppress details. Crafting > diagnostics > survival overlay priority
and shared renderer toolbar reservations prevent collisions. See the
[HUD contract](architecture.md#survival-hud-and-status-details-373) for thresholds,
small-window behavior, timing and pending native signoff.


Journey Ended (#374) displays the exact typed archive source. It has one Restart
Game action, admitted by initial focused Enter/R or a fresh physical pointer hit.
All other terminal gameplay/view/status commands remain blocked. The overlay
supersedes ordinary HUDs, including equipment/reticle, without stepping authority.
Failed/missing verified storage cannot expose a living run; success publishes both
fresh generations before resetting host presentation/input. See the
[terminal contract](../../docs/save-contract.md#journey-ended-presentation-374).

- #379 starting supplies enter only through explicit new-run construction, before
  verified first-checkpoint/restart publication. Whole-owner restore preserves
  spent items/ingredients and old empty saves; focus, resume and ship entry never
  replenish. Reserve mass is reconciled before observation; inventory retains
  its existing deliberate zero ship-mass contribution.


## Gameplay pause (#381)

Escape in gameplay opens the pause menu and releases the cursor. Enter or Escape
resumes; F1 explicitly releases the cursor without pausing. P/R/N camera commands
belong only to the F2 precision tour. Pausing closes crafting/status overlays and
clears held movement, steering, throttle keys and mining. Requested ship throttle
is preserved. Full simulation, Core, fabrication, loose cargo, survival effects,
journey accounting and autosave game-time intervals freeze at the session boundary.
UI/rendering and checkpoint IO continue. The transient pause flag is not saved.
Both native/background and portable automation tick paths exclude paused elapsed
time and clear pending backlog on entry/resume; focus/minimize/suspend cannot
resume a paused session or credit offline fatigue. Resume requires fresh controls.
The pause menu takes priority over diagnostics, HUD, reticle and equipment.
Save Now/Save & Exit use the verified #382 worker; Q-wheel input must respect
`GameplaySession::is_paused()` and the shared consumption gate.


## Optional condition presentation (#383)

`condition_feedback.rs` samples immutable survival snapshots. Health <25 gives a
faint edge vignette and a finite two-beat cue; hunger <20 gives one warning (the
existing survival HUD owns its pulse); fatigue <20 gives a quiet breathing cue
and walking-only camera roll, capped at 0.003 radians at zero fatigue. The center
aim ray and all authority remain unchanged. Three-point hysteresis rearms cues;
there is no endless sound loop. Exponential fades use active time. Pause, crafting,
tour, terminal, unfocused and occluded presentation suppress effects immediately.
The renderer stretches a cached 128x128 generic image before HUDs, preserving
center readability. No postprocessing or gameplay-domain dependency is added.

Pause -> S opens keyboard accessibility: C camera motion, A condition audio, V
vignette; S returns and Enter/Escape resumes. Version-1 preferences live in
`settings.json` beside the active-run directory (or `SALIMON_SETTINGS_FILE`) and
survive restart/new runs. Save errors stay visible; corrupt/unknown settings use
defaults with a warning. Settings do not enter run checkpoints.

`condition_feedback/audio.rs` synthesizes original quiet one-second PCM WAV cues
(22050 Hz mono, 16-bit, peak <=1500) beside the settings file. No third-party
samples/licenses or Cargo/native audio link dependency. Playback uses macOS
`afplay`, Windows PowerShell SoundPlayer, or optional Linux `paplay`/`aplay`. A
bounded queue and one child process are polled without blocking the frame; menu
suppression kills/reaps playback. Missing/failed players log an explicit warning;
missing players also show an accessibility warning. Actual OS playback and sensory
quality/performance require reference-device verification (#376).

Current-sphere benchmark/E2E profiles (#313) share the CPU workload's production
extraction, counts, material/mass cycle and supported layered cells. These are
opt-in initial conditions only; they never force sleep, keep motion alive or
change production damping/contact/mass rules. Raw accounting reports actual
contact radii alongside mass/identity; the minimum radius does not clamp mass.

Deterministic E2E isolation (#313) is explicit and limited to `--e2e`: desktop
input/focus must not rotate the scripted camera, clear protocol holds or discard
fixed-step backlog. Native OS-input probes keep normal delivery/reset semantics.
Isolated scenario evidence cannot establish native input or lifecycle acceptance.

Scout audio (#465) observes snapshots only. Four queued transients/eight mixed
voices and one scene playback child are hard bounds; existing survival audio
has its own one-child bound and uses the same platform spawn seam. Master mute
suppresses both. Focus/pause/occlusion/suspend/terminal/time/run discontinuities
clear scene events; E2E/benchmarks never spawn scene players. Surface/EVA has no
transmitted ship sound in the absence of an atmosphere model. Settings stay
outside run saves, and failed playback never changes gameplay.

Scout machinery feedback (#464) is stateless and observational. Real power,
funded work/progress and ramped yaw gate the cues; retained throttle alone cannot
light an unpowered/landed drive. Active phase freezes during pause/death and
restoration never replays prior cues. Existing interpolated pose, ramp/cargo
transforms and collision authority are preserved.

Pause saves (#382) use the shared worker/store without advancing authoritative
clocks. Saved/UTC require verified completion. Resume cancels exit intent; failure
requires retry/cancel/explicit risk exit. See [contract](architecture.md#pause-checkpoint-controls-382).

## Survival onboarding (#380)

Only successful production events complete persistent tutorial objectives; their
order is independent of prompt order. Skip/progress never grant items, alter
meters, capture movement or change clocks. New runs start fresh; restore preserves
progress and skip. Native admission rejects repeat/synthetic/unfocused keys.
Existing interaction feedback and modal/status overlays take priority over guidance.

Engine switch #493: B is an initial admitted powered-cockpit action. Paused,
wheel/crafting, terminal, non-pilot and active assist gates cannot toggle it.
Save migration never changes velocity/energy or grants cargo. Renderer/audio
observe the explicit switch, not merely selected zero throttle.


Portable verification (#375) must validate the complete save source, not infer
restoration from matching HUDs. Rejected restores and failed publication cannot
publish candidate gameplay or acknowledge a successful exit. A complete write
followed by an IO error may be recovered on the next launch, but cannot have
reported success in the failed host. Offline comparisons permit only fatigue and
its receipt/watermark to change; effects, power, crafting and motion freeze.
Test-only checkpoint/reload operations never enter the live automation protocol.
See [scenario isolation and commands](../../scenarios/portable/README.md).


Scout UI #466 reuses one renderer-owned face/palette. Prompt shape/anchors/expiry,
reticle dimensions/center and all modal/interaction gates retain their owners.
Readable Fabricator status text and queue-selection chevrons never enqueue,
consume, change recipe unlocks or advance simulation. Native review is #502;
future scanner UI #503 is blocked by #430.


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

## Modal presentation priority (#348)

Crafting, Archive, pause/settings, terminal and Q-wheel overlays suppress world
action prompts and the gameplay reticle while they own the screen. Opening a
modal does not clear prompt state or change interaction authority; closing it
restores ordinary presentation from current state. Native evidence lives in
[the Fabricator report](../../reports/issue-348/README.md).

Archive cloud evidence (#505) initializes position only at opt-in E2E startup.
Read-only protocol source/text cannot mark a record read; E navigation retains
production gates. Save Now retains exact selection/read source without the modal;
verified restart resets source and modal. See runtime architecture and the
Archive persistence tests for the shared transaction boundary.

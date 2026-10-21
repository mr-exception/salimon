# Canonical whole-game save contract (schema 6)

> This is the **current compressed-world runtime save schema**. Future real-scale universe saves (#406) must additionally pin the [published multi-seed planet definitions](canonical-planet-generation.md) by stable catalog/body/version/digest and save mutable world deltas separately. The current gameplay/world seed and terrain mode cannot stand in for the future shared-world canonical record. Under the approved policy, future fictional-planet upgrades require explicit versioned save/world-delta migration, preservation or validated remapping of physical anchors, and recoverable failure handling; scientific terrain detail on **any published real observed solid body** may not displace saved positions after its first authoritative real-scale surface publication. Future fictional-planet changes to geology/biome-dependent resource layouts likewise require explicit stable-ID, depletion and positional migration; the historical compressed prototype does not establish the real-body surface freeze. This note does not modify schema 3 or imply backend multiplayer exists.

`salimon-persistence::SaveGame<T>` is the versioned envelope. New captures use
version 3. Decode accepts versions 1 and 2 for explicit runtime migration:
retain the complete physical/Fabricator/equipment payload, initialize newly
introduced survival meters at 100 with no effects or retrospective active time,
and initialize the run/journey source. A version-1 payload must lack survival;
version 2 must contain complete typed survival state. V2 keeps its cumulative
journey distance and initializes empty sampling history without inventing travel.
Version 3 requires journey tracking source (which can have no samples on a fresh
run). Missing/null v2/v3 survival or missing v3 tracking,
invalid meters/effect registries/configuration, unknown fields and unsupported
versions reject before publication. Migration is checkpointed before exposure.

## Authoritative inventory

| Owner | Required logical state | Regenerated/excluded state |
| --- | --- | --- |
| Survival/run | Meters/config, fractional drain/starvation/regeneration, full captured effect rules/stacks/partial ticks/lifetimes, wounds/death source/terminal flag, active survival duration, offline UTC watermark, consumable catalog, movement/interaction tuning, run serial, cumulative journey distance and last valid framed physical samples | Launch receipt, held inputs/UI/capture, movement scale and presentation history; derived permissions are repaired |
| Envelope/runtime clock | Unix-second recording time, completed simulation Duration, exact admitted Duration backlog | Instant, wall-clock frame timer, profiling durations; offline world advancement is zero |
| Character | Location/frame including doorway blend, local/world position, yaw/pitch, vertical/interior/EVA velocities, nearby surface, EVA basis, gravity/ramp state | Jump key edge reset; camera matrices regenerated |
| Ship | Pose/quaternion, linear velocity, steering ramp, propulsion/configuration, flight/landed state, full assist/bounce progress, selected signed throttle, door state/fraction, cockpit authority, terrain/footprint, Core config/charge/fraction/reservation/consumption, conversion source and progress, Fabricator demand | Held throttle/steering intent reset, transient message and last-funded interval reset; instruments regenerated |
| World/mining | Seed in runtime, terrain definition mode/version, sparse deposit depletion journal and extracted total, all fragment stable IDs/source/material/mass/pose, allocator high-water mark, sealed growth state, carry identity, attached mining sphere | Fragment index and physical change journal rebuilt. Active attached sphere detaches on restore because no mining key is restored. Generated deposits/terrain patches recreated from seed and immutable versioned definitions; there are no editable terrain deltas in this revision |
| Runtime physics | Ship-frame membership/local anchors, linear/angular fragment motion and frame orientation, gravity mode; contact support/sleep/dwell state by stable identity | Render interpolation, broad phase, solver scratch, GPU hulls/terrain patches and query/placement caches rebuilt; retain logical contact state and regenerate geometry |
| Ship mass | Physical containment and stored/tray contributions, one contribution per identity | Cached ledger rebuilt/reconciled from source state; never count carried cargo twice or count queued/collected consumables |
| Fabricator | Recipe definitions/IDs/unlocks, bins, FIFO committed job recipes/IDs/counts/current progress, queue capacity, next ID, occupied tray item | No refund/re-enqueue/recharge during restore; status/demand derived from jobs and Core |
| Inventory/equipment | Exactly three food/two medical single-item slots with stable job/unit identities; five equipment slots and selection | No held controls or automatic consumption; carrying selection lock revalidated |
| Session/presentation | Gameplay seed, gameplay/precision camera mode and camera controller, deterministic simulation state | Crafting/radial modals, hover, feedback, aim commands, pending OS events, held keys/buttons, focus/capture, window/GPU, automation reader, measurement flags and caches excluded |

## Validation and restore order

Decode into temporary owned state; check envelope version, size and structural
integrity; validate finite SI values, unit quaternions, capacities, stable ID
uniqueness/references and allocator bounds before publishing. A failed restore
must leave the current session unchanged. Reconstruct immutable world/terrain
configuration, ship and character, world entities/depletion, frame/motion/contact
state, Fabricator/inventories/equipment, mass and power adapters, then exact
simulation/backlog clocks. Clear native/input/modal state and invalidate
presentation/query caches. Do not call gameplay interactions or advance a tick
while restoring. Re-snapshot must equal the normalized restored state. This
boundary performs no filesystem IO, checkpoint scheduling or offline fatigue
calculation; those are later persistence/survival issues.

Disk durability is implemented by #366, described below. Logical round-trip tests
and native filesystem restart tests establish separate boundaries.

## Implemented whole-session boundary (#364–#365)

Runtime `session/persistence.rs::SessionState` is the concrete typed payload.
`GameplaySession::save_bytes(now_unix_seconds)` exports it through `SaveGame`;
`restore_bytes` validates and constructs temporary domain owners before one
assignment. `FabricationSaveState` keeps machine/inventory validation together.
Propulsion mass is excluded from wire data and rebuilt from the physical cargo,
reserves and tray source state. Tests cover subsequent deterministic simulation,
not only byte round-trip, including sleeping contacts and mid-assist/mid-craft.
Normal-play startup/exit and automatic checkpoints are wired by #366; manual
Save Now UI belongs to #382.

## Durable active run (#366)

`salimon-persistence::storage` owns a portable `CheckpointStore` and
`SaveManager`; `NativeStore` owns filesystem operations and an exclusive OS file
lease (released automatically after crash). Runtime `checkpoints.rs` captures
validated `SessionState` on the session thread and sends owned bytes to one
bounded worker. The worker revalidates full reconstruction, writes/flushes a
same-directory temporary, renames, flushes the directory on Unix and reads back
exact bytes before acknowledging success. Two alternating slots retain the newest
verified checkpoint while publishing the next. Windows removes only the inactive
slot before rename because replacement rename differs there; the current slot
remains intact. CRC32 covers generation, length and payload; it detects accidental
corruption, not malicious modification. Reads and payloads are size bounded.

Recovery validates both slots, integrity, schema and domain reconstruction, then
chooses the highest generation (never wall time). A corrupt latest slot falls
back with a persistent warning in the native window title. If neither slot is
valid, start a fresh run with a warning. Unsupported schema or unreadable files
fail launch explicitly, preventing silent overwrite by an incompatible version.
Unpublished temporary files are ignored. Another running game using the same
save directory is rejected. Failed captures/writes keep the verified generation,
log the error, warn visibly and retry after five game seconds; exit failure returns
an unsuccessful application status.

Normal play restores before window/gameplay startup and preserves the saved
admitted clock backlog through first renderer initialization. It checkpoints on
the first update, every 60 seconds of simulated active time, and after discrete
cargo/fragment creation/removal/carry, landing/takeoff, door/power, Fabricator
reserve/FIFO/completion/tray, inventory and equipment changes. While IO is busy,
events coalesce to the next complete snapshot; clean exit drains IO and verifies
the final snapshot. Serialization/capture still runs on the event thread; durable
IO, readback and reconstruction verification run on the worker. No asynchronous
mutation of gameplay owners is allowed. E2E/benchmark launches never read/write
the user's active run. No closed-app world advancement occurs.

Default directory: Linux `$XDG_DATA_HOME/salimon/active-run` (fallback
`~/.local/share/salimon/active-run`), macOS
`~/Library/Application Support/salimon/active-run`, Windows
`%LOCALAPPDATA%/salimon/active-run`. `SALIMON_SAVE_DIR` overrides it. There is one
active run, represented by two recovery generations, with no multi-slot UI.
Existing files can be backed up/moved when deliberately starting a fresh run.

## Survival lifecycle and launch transaction (#371)

The session ticks survival on every admitted game interval while alive, including
ordinary crafting/tour menus. It advances physical domains only for the admitted
portion before a lethal boundary; death clears held controls/mining/crafting,
stops manual pilot authority and discards pending catch-up. Further authoritative
simulation and world commands are frozen. Terminal state is a discrete checkpoint
event; the worker writes it into both generations before acknowledging, so its
verified backup is terminal too. Failed writes remain failures: a death that cannot
reach storage is not claimed durable, and visible save warnings/retry remain.

Normal-play launch first restores into a temporary candidate. Only fatigue is
credited from the checkpoint UTC to launch UTC, with the survival domain's six-hour
configured cap and rollback/high-watermark protection. Hunger, health, effects,
world and active duration never advance offline. The worker's SaveManager verifies
and publishes credited state/watermark before the candidate is exposed. Failed
credit publication leaves the caller's live session unchanged and fails startup.
The receipt is launch-only: focus, minimize/resume, save and renderer changes do
not grant recovery. Crash recovery conservatively treats time since the newest
verified checkpoint as closed time; see the survival clock contract.

A new native run is also checkpointed before play. Default save paths/lease and
60-active-second/discrete-event/exit policy above are retained. Checkpoint failures
while terminal use a host-time retry delay because the gameplay clock is frozen;
no game-domain time advances to retry IO. E2E/benchmarks remain isolated from the
user's save directory. Direct portable restore is a reconstruction operation and
never grants offline credit; only the launch transaction does.

## Terminal archive and restart (#371)

The native Journey Ended screen (#374) displays the typed archived report and a
single Restart Game action. An initial focused Enter/numpad Enter/R press or left
click on the button requests the same verified restart transaction. Repeats,
synthetic replay and living runs cannot restart through this path. Runtime prepares a fresh
deterministic Earth world, clears all old objects/inventory/jobs/effects/inputs, then grants new-run supplies,
starts meters at 100 and increments the saved run serial. Terrain mode and world
seed retain their deterministic new-run policy. New-run supplies follow the [#379 contract](#starting-supplies-379).

The existing save worker first checkpoints the terminal run into both slots,
then durably publishes an immutable idempotent `journeys/v2/run-<serial>.json` summary,
then verifies the fresh run in both slots. Only success replaces the live terminal
session. Failure never grants gameplay with an unverified fresh state; retry
retains the terminal source. After success both fallback generations belong to
the fresh run. Interruption before success can recover a terminal run or the
already verified pristine fresh run, never an old living generation after the
terminal pair was verified. A first terminal write that fails can only preserve
the prior verified checkpoint; no stronger disk-failure guarantee is claimed.

Archive summaries contain their version, serial, seed, typed death cause, active
survival duration, final absolute position and typed journey counter source.
Byte-identical retry is idempotent; conflicting historical reports reject without
overwrite. Native archive scans choose unused run serials when starting fresh
after missing/damaged saves. Temporary archive files are not active reports.
Version-2 summaries are published on the death checkpoint, clean exit and terminal
launch recovery, before any restart request. Launch repairs a missing report after
an interrupted terminal publication. Restart verifies the same report idempotently.
Version-1 reports stay immutable in `journeys/run-<serial>.json`; both namespaces
reserve serials, including an interrupted v1 restart restored by a newer client.
Archive failure leaves verified terminal generations and retries through the same
visible failure policy. Report version 2 includes final named location and samples.

#375 owns expanded cross-system suites and #376 native reference signoff; #381
owns a true pause menu and #382 manual Save Now controls. Survival, HUD, Journey Ended and starting supplies are implemented through
#371/#373/#374/#379; native Q-wheel consumption UI remains #377.


## Journey statistics (#372)

`session/journey.rs` owns the cumulative distance and latest valid ship/character
samples. Active duration remains the exact `Survival::active_time` authority,
including ordinary menus and background running, stopping at the lethal boundary.
It is not wall-clock age or the legacy world elapsed clock. Offline recovery
changes neither duration nor distance.

Distance is the sum of ship-origin displacement and character displacement during
each admitted physical tick. Ship flight/coasting/landing/takeoff counts even
while the player is outside. Cabin movement uses native ship-local f64 positions;
an idle passenger contributes zero extra distance, including a turning cabin.
Character endpoints are taken before/after the controller using the same completed
ship frame. Outside/EVA and doorway crossings use absolute f64 endpoints in that
frame. An EVA traveller and the independently moving ship each contribute their
own path. Surface walking, gravity and inherited EVA drift count as physical travel.
The metric is sampled path length (tick chords), not net spawn-to-current distance.

Only movement inside the production physics/controller tick enters accounting.
Out-of-tick fixture/assist commands, cockpit snaps, teleport, restore, streaming
and camera precision-tour positions cannot bridge old samples into a fake leg.
Physical assist motion during a tick counts normally. Render-origin shifts never
enter the source. Last valid positions persist with explicit ShipLocal/World tags;
they record the last completed physical sample, not an instruction to bridge a
later rebase/restore. Fresh runs have no samples. Finite checks protect persisted
statistics; f64 subtraction precedes norms and cabin motion avoids solar-coordinate
round-trips. This adds no arbitrary maximum-speed teleport heuristic.

Final location derives from the saved physical character position, nearest catalog
surface (catalog order breaks ties), stable body ID and nominal altitude. Names
are `Near <body>` within the shared inclusive 3,000,000 m envelope, otherwise
`Space near <body>`; neither claims terrain contact. Cause is the typed survival
death source, never inferred from location or last movement. Automation inspection
exposes duration, distance and the same location DTO. #374 owns its native layout.

Occluded or zero-sized windows stop presentation and clear held controls but keep
the same bounded physical tick/checkpoint scheduler running at 16 ms wakeups.
Focus loss alone also keeps time running. OS suspension and renderer reconstruction
retain the existing explicit lifecycle gap exclusion; this is distinct from a
running minimized/unfocused app. Native desktop delivery remains #376 signoff.


## Journey Ended presentation (#374)

`GameplaySession::ended_run()` is the single typed report builder used by both
archive serialization and `journey_ended.rs` presentation. UI reads the existing
typed cause, active duration, combined distance and final nearest-body context;
it never guesses death cause or derives travel from render positions. Display
rounding does not change archived source precision. The report is available only
for terminal runs, including terminal launch recovery.

The terminal overlay takes priority over crafting, diagnostics and survival HUD;
reticle, held tool, inventory/equipment and action messages are hidden. The frozen
world remains behind a dark backdrop and readable final-transmission card.
Enter/R and pointer activation share `ClientApplication::restart_terminal()`.
All ordinary world/view/status/diagnostic commands are rejected during terminal
input dispatch; the existing session gate also rejects consumption/mining/flight.

Failed publication retains terminal authority, shows a persistent failure footer
and leaves Restart Game usable. Missing verified storage never permits a free
restart. Successful verification replaces both active slots (rather than deleting
a verified fallback prematurely), clears terminal/HUD/action/presentation caches,
terrain streaming and inputs, and leaves cursor capture off until fresh input.
No revive or resume action exists. Restart IO retains the existing synchronous
verified transaction; fresh gameplay is never exposed while writes are pending.

Pointer positions are physical pixels and match the exact raster button rectangle.
Focus/capture loss, cursor leave and resize/DPI/lifecycle resets invalidate stale
pointer coordinates. The renderer-owned `overlay_available_size` defines fitting;
the normal terminal image exactly fits that margin so it is not further scaled.
RGBA allocation is bounded at 64 MiB: very large surfaces retain a centered card
without allocating an unbounded backdrop. Normal resize/DPI/aspect layouts remain
centered and cached. Native OS-input/Metal validation remains #376.

## Starting supplies (#379)

Normal play constructs `GameplaySession::new_run` before the verified first
checkpoint; terminal restart uses the same constructor before its existing
archive/two-generation publication transaction. It grants two basic rations in
food slots one/two, one medkit in medical slot one, and the exact integer recipe
ingredients for one additional ration in Fabricator reserves. The fabrication
owner derives item mass/category and reserve quantities from its validated
starter recipes. Initial grants consume no game time, Core energy or ingredients;
they are new-run endowments, not completed player crafting operations.

The two rations use `(job 1, units 1/2)` and the medkit `(job 2, unit 1)`;
subsequent player jobs start at 3. These are ordinary typed output identities,
persisted with the existing allocator high-water mark and cross-owner validation.
No parallel inventory, seed flag, recipe unlock or save-schema change is added.
Stored reserves contribute 0.5 kg at current recipe tuning; inventory contributes
zero under the existing collected-consumable mass exception. Enqueue, output and
collection retain ordinary ingredient/power/tray/mass rules.

Restore replaces inventories/reserves exactly, including vacancies and old saves
with no supplies. It never calls the grant constructor. Loading, offline recovery,
ship entry, focus/cursor release, resume and renderer reconstruction cannot grant
items. Restart candidates contain one grant; publication failures retain terminal
authority and retries rebuild that same candidate rather than appending supplies.
Isolated engineering/E2E/benchmark fixtures keep their explicitly empty `Default`
starting state; portable new-run E2E tests opt into the production constructor.


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

## Pause checkpoint controls (#382)

The paused menu offers Enter/Escape Resume, N Save Now, S Settings and X Save &
Exit. R retries failures, C cancels exit and Q explicitly exits without saving
only after a failed exit checkpoint. Resume cancels exit intent. Menu keys require
initial focused admission; repeats/synthetic/unfocused input cannot request IO.
The existing center panel uniformly fits its drawable; Settings keep their keys.

`checkpoints/control.rs` owns bounded coalescing, verified UTC receipts and exit
intent. Manual/automatic captures share the same worker, atomic store and active
run. Polling continues while paused/minimized; a newer request cannot be satisfied
by an older in-flight snapshot. Retry bypasses frozen game-time backoff. Save &
Exit and living-gameplay window close await verification or explicit failed-save
risk choice. Final exit does not start a second write after either choice.
Terminal archives/restart and 60-second/event autosaves retain their owners.
No manual request ticks gameplay or credits offline fatigue. Native OS-input,
DPI/appearance and M1 signoff remain separately tracked by #376.

## Future Jump Drive and shared-clock save additions

The [approved future travel contract](jump-drive-and-interstellar-travel.md) uses one 1:1 real-world universe clock for every online/offline player, **no relativistic time dilation**, ordinary powered flight plus real-time engine-off coasting, and an owner-triggered Core-powered Jump Drive. The engine change (#493) now maintains deterministic impulse from **actually funded joules** (1 N·s/J baseline) over exact power/clock boundaries without adding a second kinetic-energy charge. The engine switch/coefficient are implemented in schema 5; Jump Drive and shared-world clock fields are still future.

A later, explicitly versioned save/online server model must include the installed Jump Drive identity/revision, parameter N (initial **20%**), effective impulse coefficient, physically mounted/upgrade parts, true engine on/off state **distinct from selected throttle**, position/orientation/velocity frame+epoch, and sufficiently precise Core amounts. Pending quantum transitions require stable idempotency transaction IDs, captured pre-jump position, mass, velocity, heading, six-field normalized light-second distance, quoted/reserved energy, validated target state and passenger/cargo identities. On recovery **commit exactly once or roll back/refund exactly once**; never duplicate an owner ship or charge twice. Do not let an unverified position materialize or let quantum staging create an alternate time line.

Offline catch-up remains deterministic on one shared clock and **does not autonomously fire jumps**. The code's current compressed-world zero-offline-world-duration implementation must be explicitly migrated by the existing universe offline-propagation issues; these future plans do not claim such propagation is implemented today.

## Tutorial progress and schema 4 (#380)

Schema 4 adds required `state.tutorial` with seven validated objective bits and
per-run skipped flag. Captures use version 4; versions 1–3 migrate dismissed,
without granting supplies or inferring completed actions from an empty inventory.
Missing/unknown objective state rejects before publication. UI/input/cache remain
transient; only durable progress/skip participates in discrete checkpoint events.
Verified death restart resets onboarding through `new_run`. The existing
physical/Fabricator/survival/journey migrations and storage guarantees remain.

## Funded-impulse engine migration and schema 5 (#493)

Schema 5 requires physical ship `engines_enabled` and propulsion
`impulse_ns_per_joule`. Runtime migrates valid versions 1–4 before typed domain
decode: validate and retire both legacy force capacities, install coefficient 1
and default legacy engine-on, preserving velocity, energy, physical cargo and
all other state. Current-schema missing/retired/unknown fields reject atomically.
Tutorial schema-4 progress remains required for v4/v5; earlier tutorial migration
stays dismissed. No inventory/resource grant or simulation tick occurs on restore.
The switch participates in discrete checkpoint events; held B/keys are transient.

## Portable cross-system verification (#375)

The [portable scenario DSL](../scenarios/portable/README.md) composes real session
encoding/validation, NativeStore checkpoints and the normal verified launch path
at explicit UTC. Full-source equality and post-restore continuation cover landed,
moving, outside-EVA, cargo, blackout/Core and terrain states with partial
fabrication, inventory, tutorial, active healing/damage and fixed-tick fractions.
An extended 500-fragment case preserves depletion across stream reconciliation,
physical identities, derived mass and continuation.

Checkpoint worker fault injection covers failed writes before publication, torn
bytes and complete writes followed by an error. Death/restart tests inject at
each of the two terminal copies, archive and two fresh copies, retain the terminal
visible session on error, recover valid durable state and converge to exactly one
archive/new run. CRC-valid invalid-domain saves must fall back; future schemas
must abort launch without replacing visible state. Existing native-store tests
retain crash-temp-file, lease and platform durability coverage. See the
[acceptance matrix and raw results](../reports/issue-375/README.md).


## Scout Archive schema 6 (#467)

Schema 6 adds required `state.archive`: inner version 1, selected stable record ID
and bounded unique read IDs. Runtime validates this with the full session before
publishing. Save/close/relaunch retains selection/read progress and original local
record definitions; reader open/held input remain transient. V1–V5 explicitly
initialize an unread Archive and preserve their existing survival/tutorial/engine
migrations. No recovery progress or charted destinations are invented. Unknown
versions/IDs, duplicate read IDs and missing current Archive source reject before
replacing a live session. Discrete read/selection changes use existing atomic
checkpoint scheduling. See [source and input](../client/runtime/architecture.md#scout-archive-467).

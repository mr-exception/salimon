# Canonical whole-game save contract (schema 3)

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

The minimal native terminal notice accepts an initial real focused R press;
repeat, synthetic replay and living runs cannot restart through that path. Full
Journey Ended layout/action/report presentation is #374. Runtime prepares a fresh
deterministic Earth world, clears all old objects/inventory/jobs/effects/inputs,
starts meters at 100 and increments the saved run serial. Terrain mode and world
seed retain their deterministic new-run policy. New-run starter supplies are #379.

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
owns a true pause menu and #382 manual Save Now controls. This change activates
survival before the final HUD/Q-wheel/starter-supply wave (#373/#377/#379); native
consumption UI is not claimed delivered by the lifecycle PR.


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

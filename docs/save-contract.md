# Canonical whole-game save contract (schema 1)

`salimon-persistence::SaveGame<T>` is the only versioned envelope. Runtime's
concrete state payload composes domain-owned typed state; read-only observation
snapshots are not sufficient. Version 1 has no predecessor: older/future versions
fail explicitly, never silently reseed. A future migration must dispatch on the
header before decoding a payload and validate its result. Unknown fields and
missing required state fail; new authoritative extensions require a schema bump,
a typed field, owner validation and a round-trip fixture. No unregistered opaque
extension payloads are accepted.

## Authoritative inventory

| Owner | Required logical state | Regenerated/excluded state |
| --- | --- | --- |
| Envelope/runtime clock | Unix-second recording time, completed simulation Duration, exact admitted Duration backlog | Instant, wall-clock frame timer, profiling durations; offline world advancement is zero |
| Character | Location/frame including doorway blend, local/world position, yaw/pitch, vertical/interior/EVA velocities, nearby surface, EVA basis, gravity/ramp state | Jump key edge reset; camera matrices regenerated |
| Ship | Pose/quaternion, linear velocity, steering ramp, propulsion/configuration, flight/landed state, full assist/bounce progress, selected signed throttle, door state/fraction, cockpit authority, terrain/footprint, Core config/charge/fraction/reservation/consumption, conversion source and progress, Fabricator demand | Held throttle/steering intent reset, transient message and last-funded interval reset; instruments regenerated |
| World/mining | Seed in runtime, terrain definition mode/version, sparse deposit depletion journal and extracted total, all fragment stable IDs/source/material/mass/pose, allocator high-water mark, sealed growth state, carry identity, attached mining sphere | Fragment index and physical change journal rebuilt. Active attached sphere detaches on restore because no mining key is restored. Generated deposits/terrain patches recreated from seed and immutable versioned definitions; there are no editable terrain deltas in this revision |
| Runtime physics | Ship-frame membership/local anchors, linear/angular fragment motion and frame orientation, gravity mode; contact support/sleep/dwell state by stable identity | Render interpolation, broad phase, solver scratch, GPU hulls/terrain patches and query/placement caches rebuilt; retain logical contact state and regenerate geometry |
| Ship mass | Physical containment and stored/tray contributions, one contribution per identity | Cached ledger rebuilt/reconciled from source state; never count carried cargo twice or count queued/collected consumables |
| Fabricator | Recipe definitions/IDs/unlocks, bins, FIFO committed job recipes/IDs/counts/current progress, queue capacity, next ID, occupied tray item | No refund/re-enqueue/recharge during restore; status/demand derived from jobs and Core |
| Inventory/equipment | Exactly three food/two medical single-item slots with stable job/unit identities; five equipment slots and selection | No held controls or automatic consumption; carrying selection lock revalidated |
| Session/presentation | Gameplay seed, gameplay/precision camera mode and camera controller, deterministic simulation state | Crafting/radial modals, hover, feedback, aim commands, pending OS events, held keys/buttons, focus/capture, window/GPU, automation reader, measurement flags and caches excluded |
| Future survival owner | Health/hunger/fatigue/status timers/effects/journey counters must become required typed fields when implemented | Only fatigue may recharge while closed. Health/hunger/DoT/healing/world freeze. No survival implementation or default state invented here |

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

Survival/death does not exist yet: #371 integrates terminal-state checkpoints and
restart, #372/#374 own journey archives and terminal UI. Those owners must include
their new typed state and discrete events and retire both old generations on
restart so fallback can never revive the previous run. #382 owns verified Save
Now feedback; #375/#376 own expanded survival/graphical/reference-hardware checks.

## Pending pause controls (#382)

The same active-run worker now supports manual save request coalescing, verified
status/timestamps and save-before-exit failure choices; see runtime
[preparation contract](../client/runtime/architecture.md#pending-pause-checkpoint-controls-382-preparation).
No second slot or offline recovery is introduced. Pause-menu UI is not yet wired:
#381 full freeze and its #371 prerequisite remain blocking work. #382 stays open
and its preparation PR stays draft until that integration and native UI validation.

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

Disk durability, verification/fallback and periodic/event/exit checkpoints belong
to #366. A round-trip here proves the logical state boundary, not durable storage.

## Implemented whole-session boundary (#364–#365)

Runtime `session/persistence.rs::SessionState` is the concrete typed payload.
`GameplaySession::save_bytes(now_unix_seconds)` exports it through `SaveGame`;
`restore_bytes` validates and constructs temporary domain owners before one
assignment. `FabricationSaveState` keeps machine/inventory validation together.
Propulsion mass is excluded from wire data and rebuilt from the physical cargo,
reserves and tray source state. Tests cover subsequent deterministic simulation,
not only byte round-trip, including sleeping contacts and mid-assist/mid-craft.
The boundary is ready for #366; no startup/exit/manual-disk UI is wired yet.

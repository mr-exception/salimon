# Portable save contracts and durable checkpoints

See [canonical save contract](../../docs/save-contract.md). The envelope owns
schema/version/clock validation and bounded encoding; domains own payload rules.
`storage.rs` owns portable two-generation verification/recovery and the native
filesystem adapter. Publication cannot destroy the newest verified generation.
CRC32 includes generation/length/payload. Unknown schema rejects before overwrite.
Native directories have an exclusive crash-safe OS lease; stale temporary files
are ignored. Runtime owns scheduling and session capture, never this crate.
No window/GPU state or offline simulation is saved. Run
`cargo test -p salimon-persistence --locked` (fault injection, corruption, schema,
size, atomic publication, filesystem restart and lease checks).


#371 composes schema-2 survival source, explicit v1 migration, verified launch-only
fatigue credit, terminal freeze and archive/restart publication. See the canonical
[save/lifecycle contract](../../docs/save-contract.md). Restart must publish an
archive and both fresh fallback generations before exposing new gameplay; failures
remain failures. Maintain lifecycle, offline rollback, publication-failure and
restart fault-injection tests with changes to this boundary.


#372 advances captures to schema 3 with explicit v1/v2 migration in runtime.
Terminal publication verifies both fallback slots before archiving. Report versions
use separate immutable namespaces (legacy `journeys/`, current `journeys/v2/`);
allocator high-water scanning covers both. Runtime supplies the version and opaque
bytes; storage does not interpret gameplay JSON. See the canonical
[journey contract](../../docs/save-contract.md#journey-statistics-372).

#380 advances captures to schema 4 for per-run tutorial progress/skip. Runtime
explicitly migrates versions 1–3 dismissed; missing schema-4 source rejects.

#493 advances captures to schema 5 for explicit engine on/off and the fictional
impulse coefficient. Runtime migrates v1-v4 force records without a simulation
tick; see the current canonical save contract.


#467 advances captures to schema 6, requiring versioned local Archive identities,
read progress and selection. Runtime initializes old v1–v5 Archives unread;
current payload omissions/unknown identities reject transactionally. See
[Archive save contract](../../docs/save-contract.md#scout-archive-schema-6-467).

## Universe new-run envelope (#406)

`universe.rs` owns the independent version-1 future universe save contract, using
`salimon-universe` IDs/frames/epochs. It does not replace the runtime envelope (now schema 26, with a canonical definition binding).
Catalog/definition incompatibility is fatal during recovery; corruption alone can
fall back. Domain delta validators are mandatory. See the canonical
[universe save contract](../../docs/save-contract.md#future-universe-new-run-contract--u05-406)
and `src/universe/tests.rs` for wire, identity and interrupted-publication coverage.

## Runtime schema 7 (#468)

New compressed-world captures require runtime-owned Recovery config/module/paid
progress; explicit v1-v6 migration initializes empty hardware/work and preserves
prior recipe sources while adding the blueprint. Current universe envelope is
independent and unchanged. See [save contract](../../docs/save-contract.md#recovery-module-schema-7-468).

## Runtime schema 8 (#542)

Current compressed-world captures require validated blueprint placeholder source.
Versions 1–7 migrate empty without ticking; UI/preview state is transient. The
independent universe envelope remains version 1. See the canonical
[placeholder save contract](../../docs/save-contract.md#blueprint-placeholder-schema-8-542).


## Runtime schema 9 (#539)

New runtime captures require independent cargo door position/direction. Older
captures migrate to the previously open aperture without advancing time. World
fragments retain original IDs, motion, mass and ship-local anchors. The independent
universe envelope is unchanged. See [migration and validation](../../docs/save-contract.md#cargo-door-schema-9-539).


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
introduced. Sitting/lying, privacy mechanisms and expanded tactile presentation
remain #577. Schema 10 saves source motion; see
[the save contract](../../docs/save-contract.md#habitation-schema-10-540).

## Physical mounts (#576)

Version 11 attachment validation cannot mutate the active session on failure or grant new fixtures to historical saves. See [the canonical contract](../../docs/physical-mounts.md).

Whole-session schema 12 adds the optional runtime-owned cabin mechanisms and posture source. The storage envelope/atomic generation protocol is unchanged; see [cabin save semantics](../../docs/cabin-mechanisms.md#save-and-verification).


Persistent finishes #579: [canonical ownership, schema 13, bounded render factors and controls](../../docs/room-finishes.md). Ship owns room/item cosmetic identity; runtime owns nearby physical control admission; renderer applies the explicit nonfunctional surface allowlist. Cosmetic changes never edit physical material/age/damage.


Instant Core upgrades #548: [canonical input, source, mass and schema-14 contract](../../docs/core-upgrades.md). Ship owns validated capacity-only transactions; world returns original caller-owned reserved entities through the one carry slot; runtime maps the data catalog/physical controls and reconstructs pending+installed material mass. No timer, joule grant, Core replacement or flight command.

Schema 15 adds exact construction funding receipts/progress; legacy unfunded placeholders migrate empty and cross-owner material IDs reject before publication. See [construction material contract](../../docs/construction-materials.md).


Room and fixture lights #580: [canonical controls, source, effective output and schema-16 contract](../../docs/room-lighting.md). Master/blackout/inoperable lamps suppress output without changing preferences; critical machine displays remain independent.


## Completed modular structure (#545)

Follow the [canonical assembly, traversal, identity, mass and schema-18 contract](../../docs/modular-construction.md). Completed cell parts and face edits regenerate shared render/collision geometry; unpublished projects supply no support or colliders. Preserve the existing gate, per-cell identities, atomic restore and single-owner material ledger.


## Construction flight readiness (#546)

Follow the [canonical Stop/Resume, physical securing, takeoff and schema-19 contract](../../docs/construction-flight-safety.md). The durable stop latch never clears on landing; actual hull/attachment state governs readiness. Preserve Core/engine/cockpit/ramp authority and inertial momentum.


#547 advances the envelope to schema 20 for runtime-owned optional module commissioning, maintenance receipts/work and durable downtime. Versions 1–19 migrate without granting modules or time; historical envelopes cannot smuggle module source. See [the module/save contract](../../docs/non-core-modules.md).


## Construction damage (#549)

Follow the [canonical damage, physical salvage and schema-21 contract](../../docs/construction-damage.md). Surviving work preserves paid time and receipts; lost cells and destroyed projects cannot respawn or duplicate salvage. Core identity is protected. Fresh debris IDs, exact material shares, collision/containment and installed mass publish together; restore reconstructs source without reallocating debris.


## Offline construction (#550)

Follow the [canonical schema-22 clock, energy, safety and atomic launch contract](../../docs/construction-offline.md). Runtime owns chronological work and verified-event admission; ship owns closed power without impulse or new engine/assist authority. Saved scene/cargo and survival active sources remain frozen; timed service, Fabricator, Recovery and eligible construction settle before verified publication. Universe/scanner/trajectory event production remains #433.


## Cosmetic surface age (#581)

Follow the [persistent exposure, schema-23 and clean-repair contract](../../docs/surface-age.md). Ship owns bounded integer exposure; runtime admits physical identity/use and verified offline time; renderer consumes observations only. Recolor/repair retain age, replacements use fresh IDs, and no cosmetic state grants damage or collision authority.


## Canonical surface gameplay and schema 24 (#425)

Follow the [canonical surface contract](../../docs/universe-gameplay.md):
selected body/terrain context, immutable U11 deposits, shared physical ground,
conserved fragment/cargo/Fabricator ownership and exact U05 definition binding.
Historical fixtures remain explicit; new-run startup is #426.


## Scanner delivery (#431)

Scanner contact checkpoints validate catalog revision, exact IDs/frames, timestamp ordering and bounds before atomic reconstruction. Runtime schema 25 saves canonical scanner parameters, clock/remainder and encounter history; legacy canonical saves migrate empty, current missing/malformed sources reject. Closed-app launch advances freshness once without inventing offline detections.
See the [scanner contract](../../docs/universe-scanner.md) for source/test mapping, units and limitations.


## Scanner delivery (#469)

Universe `scanner/chart.rs` owns player-known observations, inferred/transmitted leads with explicit uncertainty, and immutable voluntary route annotations. Runtime `session/chart.rs` owns cockpit P/M/Enter controls and schema-26 pair validation with scanner history. Catalog truth never seeds the chart; observed knowledge outlives contact TTL and source frames are preserved without guessed transforms.
See the [scanner contract](../../docs/universe-scanner.md) for source/test mapping, units and limitations.

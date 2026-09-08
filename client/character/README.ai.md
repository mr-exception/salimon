# Character AI Maintenance

Read `architecture.md` and `invariants.md` before edits. Keep this crate portable:
no `winit`, `wgpu`, renderer, diagnostics, or ship-crate dependency. Native input
translation and domain composition belong to `client/runtime`.

Important regression areas are diagonal-speed normalization, edge-triggered
jumping, closed/flying doorway containment, continuous ramp contact, distance-based up blending, tip-only entry and occupied sweep rejection,
camera-relative W/S/A/D doorway crossings and direction-based surface re-entry,
instant cockpit transitions, horizontal mouse-delta direction in both walking
and cockpit views, the default forward cockpit-window sightline, central Core
and Core collision and sliding, the port cockpit route into the nose,
pilot-chair and unified console/monitor collision, forward-hull containment,
window sill clearance,
ceiling and lintel containment, rear doorway body clearance, safe cockpit exit,
exterior hull containment with the gate open or closed, corner sliding and gate
approach from outside, vertical separation from the hull,
forced gate loss during traversal or after a stopped exit, repeated open/close
crossings at the center and both jambs, closed-gate contact in rotated frames at
the catalog's 1e12 m anchor,
and shared-query surface movement (constant-radius in flat fixtures). Add a focused unit test whenever one
of these rules changes. Surface camera and movement direction must continue to
share the same yaw-derived local tangent basis rather than using fixed ship/world
axes.

## Shared maintenance rules

Follow the root [coding conventions](../../docs/coding-conventions.md),
[validation matrix](../../docs/validation.md) and
[feature map](../../docs/maintenance-map.md). Update affected contracts/guides
with behavior changes and record completion evidence under root `reports/`.

## Source map

`src/lib.rs` preserves the public re-exports and constants. Read
`src/controller/mod.rs` for private controller state and update dispatch, then
`controller/interior.rs`, `doorway.rs`, `surface.rs`, or `eva.rs` for the active
movement mode. `controller/camera.rs` owns look and snapshots. Regression tests
stay with these mode owners; `controller/collision_tests.rs` checks movement
through shared proxies and `controller/test_support.rs` contains test fixtures.
`state.rs` owns frame/snapshot/location contracts and private position variants;
`input.rs` owns typed controls and planar normalization. `layout.rs` derives
body-clear proxies from authored geometry, `collision.rs` owns sweep/slide helpers,
`queries.rs` owns public sight/placement queries, and `math.rs` owns character
normalization/projection policy. Generated `spatial_contracts.rs` remains exporter-owned.

## Terrain integration (#179–#181)

Follow the canonical [integration/budgets/validation guide](../../docs/terrain-validation.md)
and [world definition contract](../world/terrain-contract.md). Keep local rules
and their consumers aligned; flat deterministic fixtures are explicit.


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
introduced. Sitting/lying, privacy mechanisms and expanded tactile presentation
remain #577. Schema 10 saves source motion; see
[the save contract](../../docs/save-contract.md#habitation-schema-10-540).


## Completed modular structure (#545)

Follow the [canonical assembly, traversal, identity, mass and schema-18 contract](../../docs/modular-construction.md). Completed cell parts and face edits regenerate shared render/collision geometry; unpublished projects supply no support or colliders. Preserve the existing gate, per-cell identities, atomic restore and single-owner material ledger.


## Canonical surface gameplay and schema 24 (#425)

Follow the [canonical surface contract](../../docs/universe-gameplay.md):
selected body/terrain context, immutable U11 deposits, shared physical ground,
conserved fragment/cargo/Fabricator ownership and exact U05 definition binding.
Historical fixtures remain explicit; ordinary new-run startup uses the canonical constructor (#426).


## Physical organisms and static clearance (#620)

The [physical organism contract](../../docs/physical-organisms.md) maps canonical
placement/part/baseline identity, physics-owned swept boxes, character/ship typed
adapters, runtime residency and matching renderer evidence. Quality never changes
physical populations. Explicit obstruction safety overrides ordinary assist continuation.

# Character Architecture

## Internal ownership

The crate root is a stable public facade; callers still import the same types,
functions and constants from `salimon_character`. Internal modules introduce no
new crate dependencies or native/platform types.

| Source under `src/` | Responsibility |
| --- | --- |
| `state.rs`, `points.rs` | Ship/surface frames, narrow world/ship-local point DTOs, location/snapshot and private position state |
| `input.rs` | Portable movement flags and normalized planar controls |
| `controller/mod.rs` | Private mutable controller fields, default state, inspection and one-mode update dispatch |
| `controller/interior.rs` | Ship-local walking/jumping and instant cockpit transitions |
| `controller/doorway.rs` | Continuous sloping ramp support, edge sliding and gate recovery |
| `controller/surface.rs` | Radial walking, shared yaw tangent basis and eye-height projection |
| `controller/eva.rs` | Independent inherited drift, 3D assist, nearby-body influence and re-entry/contact |
| `controller/camera.rs` | Mouse look and snapshots for all modes |
| `layout.rs`, `collision.rs` | Shared authored dimensions/proxies and hull/fixture/appendage sweep/sliding |
| `queries.rs` | Public ship-local floor placement and sight obstruction |
| `math.rs` | Domain normalization fallback/threshold, rejection, interpolation and tangent selection |
| `spatial_contracts.rs` | Unmodified exporter-generated spatial contracts |

Controller mode modules are children of the controller owner so private fields
remain private. A private `Step` samples the existing input edge and environmental
frames once per update; dispatch selects exactly one movement mode. It retains
the bounded 100 ms fallback integration slices (runtime supplies 16 ms ticks) without discarding elapsed duration.
Tests live with mode/query/math owners; shared test-only frames and constructors
live in `controller/test_support.rs`, and hull/appendage controller regressions
live in `controller/collision_tests.rs`. Shared geometry remains independent of
controller mutation so both traversal and placement/sight consume its proxies.

## Movement and presentation


The controller stores cockpit, ship-local interior, world-space ramp/doorway,
surface or EVA position. Interior movement is camera-relative and enters the
fully deployed ramp through the body-clear doorway. Ramp support follows the
authored sloping plane with normalized 3D walking speed; lateral movement slides
at the full-body edge limit. Gravity/camera up blends by distance along the ramp,
never by an elapsed completion timer. A stopped walker stays supported indefinitely.
At the tip, surface contact preserves the ship-local planar point and adopts
radial gravity. Ground entry requires inward motion across the tip, so side and
under-hull approaches cannot jump to the cabin. Jump input cannot bypass ramp
contact or its side edges. The protocol retains `DoorwayBlend` and
`doorway_blend_fraction` names for ramp location/progress compatibility.

Hull, wing, engine and landing foot/leg proxies retain their existing finite
height/body expansion and sweep/slide policy. A closed gate stays solid. Runtime
supplies the moving-ramp flag; its swept footprint is reserved during motion and
the open ramp's side edges block ground walkers. Forced environmental gate loss
still resolves a doorway overlap to the nearer safe side, but normal runtime
closing is rejected before state mutation whenever a body occupies the sweep.

The ship interior uses a body-radius-inset walking envelope and simple planar
collision proxies for the central Core pedestal, pilot chair,
one centered console with three monitors, and the solid forward hull beside the cockpit.
Movement resolves forward and sideways axes separately, preserving edge sliding
and a clear side route past the chair and console without a physics engine or mesh
collision. The outer cockpit hull proxies retain solid side walls; a narrower
nose floor extends beyond the broad deck and body-expanded shoulder proxies
keep the player on it. The side bounds account for the
projecting window sills. The aft bulkhead stays solid outside the body-clear
doorway aperture, even when the door is open. Jump height is bounded by the
lowest ceiling fixtures and a lower local bound at the door lintel. Spawn and
cockpit exit share a clear starboard aisle position.

`MovementInput` is the typed platform boundary. `ShipFrame` and `SurfaceFrame`
are read-only environmental inputs. `CharacterSnapshot` is the presentation and
diagnostics output. The runtime maps these types to/from ship and renderer DTOs;
neither dependency points back into character.

The seated cockpit snapshot starts with only a slight downward pitch so its authored
cockpit anchor and forward ray clear the console and solid nose while crossing the
glazing. The 1.80 m body / 1.75 m eye-height contract and ship-local anchors
follow the wider 4.00 m-tall asset. Mouse look remains unrestricted and
independent of ship orientation. On a solid-body surface, the controller derives
one camera-relative tangent basis from yaw and the local radial up vector; both
the camera snapshot and WASD movement consume that basis so forward and strafing
remain view-relative at every sphere orientation.

Open-space state additionally retains the exit velocity and orientation basis.
`advance_with_motion` receives the already-advanced ship frame and its velocity;
world drift integrates independently each update. A normalized flight-assist
control velocity adds 3D view-relative translation without modifying inherited
velocity. Input release removes controlled movement only. Re-entry clears EVA
control state and rebases yaw/pitch to preserve the world look direction when
adopting the ship frame. The stationary `advance` adapter supplies zero velocity.

Nearby-body influence is a runtime-selected optional `SurfaceFrame`, leaving the
portable character independent of the world catalog. Detached state reports
`NearbyBody` while influenced. Updates integrate drift plus half-step radial
acceleration, then update drift velocity once. Surface contact adopts existing
walking; influence exit retains drift. Influence changes preserve world position
and rebase the view to radial up without resetting look direction. Interior
re-entry clears the external influence. The runtime evaluates the shared world
selector before and after movement so authoritative mode matches boundary crossing.

## Shared vector primitives

Compatible `f64` component arithmetic comes from the dependency-free
`salimon-math` leaf crate. Normalization, frame and quaternion policies stay
with their owning domain. See the canonical
[decision and inventory](../../docs/technical-architecture.md#shared-math-decision-115).

## Authored spatial geometry (#111)

`spatial_contracts.rs` is the single generated raw geometry/anchor module.
`layout.rs` applies player-radius clearance, conservative planar projection,
shoulder/gate composition and camera policy; collision, sight and floor placement
consume that layout. Public anchor exports and floor-height APIs are preserved.
The [scout mapping](../../models/assets/ships/salimon-scout/README.md#geometry-and-gameplay-policy)
identifies component ownership, coarse envelope semantics and remaining gameplay
policy. Builds use checked-in artifacts and never run Blender.

The exported `EXIT_DOOR_MARKER_DEPTH_METERS` derives the aft-face depth from
the authored door marker and collider bounds for consumers presenting interaction
guidance. It does not alter traversal, collision or interaction eligibility.

## Carrying and equipment composition

Character movement and location do not own equipment selection. Runtime composes
one-world-object carrying with the five-slot toolbar: successful pickup clears
selection, carrying blocks numeric selection, and release requires an explicit
new selection. Failed pickup does not stow equipment. See the
[runtime contract](../runtime/architecture.md#carrying-and-equipment-regression-contract-131)
and carrying/resource-loop scenarios; no character movement rule changes.

## Raised landing supports (#168)

Generated `LANDING_COLLIDERS` supply finite foot/column boxes to exterior walking,
visibility and placement queries. Hull contact retains its original finite Y
extent, permitting walking under the raised fuselage. The centered aft gate connects to the generated boarding ramp support and
retains closed-door collision down to the landing foot plane.

## Terrain integration (#179–#181)

Follow the canonical [integration/budgets/validation guide](../../docs/terrain-validation.md)
and [world definition contract](../world/terrain-contract.md). Keep local rules
and their consumers aligned; flat deterministic fixtures are explicit.

## Onboard object containment (#187)

`ship_contains_object` classifies a physical center against authored cabin/nose
bounds and floor/ceiling. It has no door-animation or furniture exclusion and is
independent of floor-placement clearance. Runtime converts world positions in
f64 using the current ship frame; handling modes never alter this rule.

## Core power integration (#188)

Runtime supplies `set_ship_gravity_enabled` from authoritative Core power.
Disabled artificial gravity preserves interior vertical motion and contact/hull
constraints without planetary fallback. Jump requires powered gravity; full
cabin inertial dynamics remain deferred. External surface/EVA gravity is unchanged.

## Interior zero-G (#191)

On outage, interior character movement uses ship-local view-relative three-axis
acceleration (WASD, Space up, Shift down), retaining walking/jump momentum.
Released controls coast; controlled speed is bounded by the existing 3.8 m/s
movement speed without reducing faster inherited momentum. Body-clear fixture/wall/deck/ceiling contacts remove blocked
velocity components. Exterior planetary gravity never enters this path. Power
return preserves position and vertical velocity before ordinary deck gravity
settles the player. Airlock EVA inherits ship plus local drift; seated occupancy
remains mechanically fixed. Held fragments follow existing carrying rules in
both powered and weightless interior modes.

Loose cargo's unpowered surface preserves velocity without ambient damping and
adds conservative radius-based ceiling containment; horizontal hull/fixture
containment and precise deck support remain unchanged. Surface changes invalidate
sleep support through existing snapshot matching. No acceleration-induced cabin
forces, crew inertial simulation or new contact engine is introduced.

## Typed position boundary (#311)

`ShipFrame::project_world(WorldPoint)` and `unproject_local(ShipLocalPoint)`
retain the existing raw-array transform arithmetic while rejecting reversed frame
arguments. Runtime uses them for persistent cargo and ship-relative presentation;
local controller geometry retains the raw adapters. No validation, quaternion or
normalization policy changes. See the [canonical inventory](../../docs/technical-architecture.md#position-boundaries-and-resource-dimensions-311).

## Wall Fabricator (#342)

The generated Fabricator box expands by the existing player radius for interior
walking, zero-G and loose-object floor placement. Both Core aisles and the aft
door route remain clear. Three generated markers own input, screen and tray
positions. Ordinary sight queries treat the module as solid; the port query
omits only its own conservative envelope because the input face is recessed
behind the tray. Runtime still checks interior authority, front face, range and aim.

Fabricator surface targeting uses `fabricator_sight_obstruction(eye, target)`
for authored port/monitor/tray faces; it excludes only the module's conservative
fixture envelope, preserving cabin solids, deck and ceiling occlusion. Runtime
owns face hit regions and interaction range; walking collision is unchanged.

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

## Voluntary movement multiplier (#369)

`set_movement_scale` admits a bounded transient multiplier for voluntary movement
in every walking mode and EVA assist, and for zero-G cabin acceleration/control
speed. It never scales elapsed time, gravity, jump impulse or inherited velocity.
Existing faster zero-G momentum is preserved. The field is excluded from saved
source state and resets to one on restoration; the host reapplies current derived
survival restrictions before stepping. Character has no survival dependency;
thresholds/permissions remain in the survival domain and runtime composition.

## Starter envelope (#538)

The exporter owns seven new starter fixture bounds and a separate cargo header.
`layout.rs` expands the fixtures by the existing body radius. The central doorway
remains traversable; bulkhead sides and habitation/storage envelopes block walking
and cargo placement. `interior_ceiling` applies the header to powered jumping and
zero-G movement, while sight uses its actual 3D bounds. The independent cargo door now uses the #539 obstruction and save contract below.

Blueprint admission reads `ship_installation_obstacles` and the conservative
hull envelope from the same authored layout used by movement. These read-only
bounds do not install construction colliders or change character dimensions.


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

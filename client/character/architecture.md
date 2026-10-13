# Character Architecture

## Internal ownership

The crate root is a stable public facade; callers still import the same types,
functions and constants from `salimon_character`. Internal modules introduce no
new crate dependencies or native/platform types.

| Source under `src/` | Responsibility |
| --- | --- |
| `state.rs` | Typed ship/surface frames, location/snapshot and private position state |
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
the original capped integration seconds and uncapped doorway elapsed duration.
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

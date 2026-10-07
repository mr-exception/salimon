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
| `controller/doorway.rs` | Gate clearance/recovery and exact 250 ms gravity transition |
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


The controller stores one portable position state: cockpit, ship-local interior,
world-space doorway blend, or world-space surface. Ship-local positions move with
the ship without copying ship simulation into this crate. Doorway and surface
states use world coordinates so the gravity transition and radial projection are
explicit. Interior and doorway movement share the same normalized, yaw-relative
ship-plane direction; the doorway converts both planar axes into world space.
During the blend, lateral movement slides along the body-clear doorway limits.
Surface re-entry uses the movement tangent's component along ship-forward, so
walking away or parallel to the door cannot trigger an entering blend.
Surface walking also collides with a conservative, body-expanded cabin/nose
envelope. Overlapping aft wall proxies leave only the actual gate aperture;
closed or unlanded gates fill that opening. Entry requires inward movement from
the exterior into this aperture with vertical hull overlap. Collision slides
along the exterior and solves surface height without changing the resolved
ship-local planar coordinates, so radial projection cannot push a walker back
through the hull. The hull proxies have finite height, and source-derived wing
boxes block exterior movement at wing height. Each engine has source-derived body and raised-fin boxes with separate
height checks and body-radius expansion. These ship-local boxes follow the landed
ship frame and leave the aft gate clear. A completed entry gravity blend stays in the doorway
until the player reaches the cabin, preserving slow and diagonal crossings.
Doorway movement rechecks the landed/open rule before each update. When the gate
becomes impassable, the blend ends immediately and an overlapping character
resolves to the nearer of the interior and exterior stopping planes. Existing
surface state always stays outside: a short or stopped exit may leave it within
the gate volume, so closure clears that overlap before the usual movement sweep.
The recovery is restricted to the aperture and vertical hull overlap to preserve
side-wall sliding and movement below the ship. Surface recovery solves eye height
at the resolved ship-local planar coordinates, retaining body clearance.
The following movement sweep retains the exact resolved local stopping plane;
decoding it from quantized world coordinates could falsely treat contact as an
existing penetration. Aperture-overlap recognition allows a few world-coordinate
ULPs at the catalog's 1e12 m anchor, without widening the passable aperture.
Closure restores the doorway's exact lateral bounds before choosing the next
state, so a rounded jamb contact cannot become an interior aft-wall overlap.

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

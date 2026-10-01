# Character Architecture

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
through the hull. The hull proxies have finite height and exclude the broad wing
bounds. Each engine has source-derived body and raised-fin boxes with separate
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
collision proxies for the central Core pedestal, cabin furniture, pilot chair,
three console/monitor assemblies, and the solid forward hull beside the cockpit.
Movement resolves forward and sideways axes separately, preserving edge sliding
and clear routes on both sides of the chair without a physics engine or mesh
collision. The outer cockpit hull proxies prevent bypassing a side console
through the exterior shell, while the forward envelope follows the actual deck
edge instead of globally excluding the cockpit. The side bounds account for the
projecting window sills. The aft bulkhead stays solid outside the body-clear
doorway aperture, even when the door is open. Jump height is bounded by the
lowest ceiling fixtures and a lower local bound at the door lintel. Spawn and
cockpit exit share a clear starboard aisle position.

`MovementInput` is the typed platform boundary. `ShipFrame` and `SurfaceFrame`
are read-only environmental inputs. `CharacterSnapshot` is the presentation and
diagnostics output. The runtime maps these types to/from ship and renderer DTOs;
neither dependency points back into character.

The seated cockpit snapshot starts with only a slight downward pitch so its Task
10 anchor and forward ray clear the console and solid nose while crossing the
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

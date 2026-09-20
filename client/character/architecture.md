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
through the hull. The proxies have finite height and exclude the broad asset
bounds for wings/engines. A completed entry gravity blend stays in the doorway
until the player reaches the cabin, preserving slow and diagonal crossings.

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

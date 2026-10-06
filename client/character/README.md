# Character

`salimon-character` owns the portable first-person controller. It starts
at the redesigned ship's player-start contract inside the landed ship, consumes typed
WASD/jump/mouse-look input, applies the shared 9.81 m/s² gravity strength, and
produces a renderer-neutral camera snapshot.

The controller uses ship-local floor gravity while inside, clamps movement to
simple invisible interior bounds, and slides around the central Core pedestal,
pilot chair, unified cockpit console/monitors, and forward
hull proxies.
The enlarged room has a 9.20 m interior width with walking routes on both sides
of the Core. A 0.24 m body radius keeps the player clear of fixtures, projecting
window sills, and rear window bulkheads. Jumping keeps the player's head below
the ceiling lights and the lower door lintel. The aft transition is restricted to the actual
2.80 m doorway, and a closed doorway remains solid. An
open landed doorway begins an exact 0.25-second up-vector blend before the
controller changes to radial surface walking around the active solid body
supplied by `SurfaceFrame`. Doorway movement retains
the interior's camera-relative WASD directions, including strafing. Re-entry
requires actual movement toward the cabin, independent of the pressed key.
Outside walkers collide with a conservative cabin/nose envelope, including the
side hull and rear windows, whether the gate is open or closed. Only the landed
open gate admits entry; exterior contact allows sliding around the hull while
preserving body clearance and surface eye height.
Closing the gate cancels an active gravity blend and keeps the player on the
nearer physical side with body clearance. A surface walker still overlapping
the gate after a short exit is moved clear of the closed door before further
movement. Reopening restores the usual passage and starts a fresh blend.
Surface movement is projected
back to the same spherical radius each update, so the portable rule works around
the full body rather than only near the initial landing point.

The player body is 1.80 m tall with a 1.75 m standing eye height. The floor,
player start, cockpit camera, doorway crossing, and invisible bounds follow the
authored ship geometry. Spawn and
cockpit exit use the clear starboard aisle at `[0.50, 1.9973, -2.20]`, outside
the Core and the lowered pilot chair. Walking can pass the chair on its port
side, continue past the unified center console, then turn into the narrow
nose. The broad deck allows body centers through X `6.32 m`; the nose floor
continues to X `7.85 m`, with body centers through X `7.61 m` and within
`±0.86 m` of the centerline. The single console collision footprint is X
`3.66–6.44 m`, Z `−3.34–0.24 m`, including the player's radius. Seating
remains a contextual transition.

Cockpit entry and exit are instant. The default seated view is slightly pitched
down while retaining a clear forward sightline through the authored cockpit
glazing; mouse look remains independent and unrestricted. This crate does not decide whether
the ship is landed, whether the door may open, or how ship motion changes; the
runtime passes those facts through `ShipFrame`/`SurfaceFrame` DTOs.

See [README.ai.md](README.ai.md), [architecture.md](architecture.md), and
[invariants.md](invariants.md) before modifying controller behavior.

## Space airlock access (#35)

An open flying airlock exits into `Space`, retaining the current world eye
position and ship-up camera orientation without planetary projection or floor
gravity. Camera-relative 3D movement supports stationary and moving-ship exit/re-entry;
closed-gate, rear-window, hull and thruster collisions remain active. Closing
around a player within the aperture resolves them outside with body clearance.
Re-entry through the open aperture restores interior floor gravity.

## Inertial EVA (#37)

`advance_with_motion` accepts the current ship transform and world linear
velocity. The stationary-frame `advance` entry point remains available to
portable walking callers/tests. Exit captures the current world velocity and
ship orientation; detached position integrates inherited world motion without
following subsequent ship steering or thrust. EVA view orientation retains its
exit basis and independent mouse yaw/pitch. Normalized WASD follows the complete
view direction, Space ascends and Left Shift descends. Flight assist commands
3.8 m/s translation relative to the inherited drift, returning control velocity
to zero when input is released. This assist does not cancel inherited velocity.
`eva_velocity` reports world motion while detached. Open-gate re-entry restores
ship-local movement and gravity and preserves the world view direction.
The runtime selects nearby solid bodies using player surface distance and the
world-owned inclusive 3,000,000 m threshold. `NearbyBody` retains the airborne
position and drift velocity and adds fixed 9.81 m/s² radial gravity once per
update. The view adopts radial up; 3D assisted controls remain available during
descent. Body contact clamps to eye height and adopts the existing surface walk.
The selected surface remains authoritative even if the ship is near another
body. Leaving influence disables gravity while retaining current world motion;
re-entry restores interior gravity and clears influence.

`ship_floor_placement` supplies conservative cabin and nose deck support using
the same Core and console bounds as traversal. Interior sight queries use walls,
Core, deck, ceiling, and the gate rather than treating the whole cabin as
a solid exterior box. These renderer-neutral queries also keep loose resource
motion on the deck and validate pickup sightlines; no resource types enter the character crate.

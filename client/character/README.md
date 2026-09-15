# Character

`salimon-character` owns the portable Phase 0 first-person controller. It starts
at the redesigned ship's player-start contract inside the landed ship, consumes typed
WASD/jump/mouse-look input, applies the shared 9.81 m/s² gravity strength, and
produces a renderer-neutral camera snapshot.

The controller uses ship-local floor gravity while inside, clamps movement to
simple invisible interior bounds, and slides around the central Core pedestal,
port sofa, starboard worktop, pilot chair, cockpit consoles/monitors, and forward
hull proxies.
The enlarged room has a 9.20 m interior width with walking routes on both sides
of the Core. A 0.24 m body radius keeps the player clear of fixtures, projecting
window sills, and rear window bulkheads. Jumping keeps the player's head below
the ceiling lights and the lower door lintel. The aft transition is restricted to the actual
2.80 m doorway, and a closed/flying doorway remains solid. An
open landed doorway begins an exact 0.25-second up-vector blend before the
controller changes to Earth-radial surface walking. Surface movement is projected
back to the same spherical radius each update, so the portable rule works around
the full body rather than only near the initial landing point.

The Task 10 playtest revision defines a 1.80 m player body with a 1.75 m standing
eye height. The floor, player start, cockpit camera, doorway crossing, and
invisible bounds match the redesigned 20.30 × 4.00 × 20.00 m ship. Spawn and
cockpit exit use the clear starboard aisle at `[0.50, 1.9973, -2.20]`, outside
the Core and the lowered pilot chair. Walking can enter the cockpit on either
side of the chair, while body-expanded object proxies keep the player out of the
chair, monitor consoles, walls, and exterior hull. Seating remains a contextual
transition.

Cockpit entry and exit are instant. The default seated view is slightly pitched
down while retaining a clear forward sightline through the Task 9 glazing; mouse
look remains independent and unrestricted. This crate does not decide whether
the ship is landed, whether the door may open, or how ship motion changes; the
runtime passes those facts through `ShipFrame`/`SurfaceFrame` DTOs.

See [README.ai.md](README.ai.md), [architecture.md](architecture.md), and
[invariants.md](invariants.md) before modifying controller behavior.

# Character

`salimon-character` owns the portable Phase 0 first-person controller. It starts
at the Task 7 player-start contract inside the landed ship, consumes typed
WASD/jump/mouse-look input, applies the shared 9.81 m/s² gravity strength, and
produces a renderer-neutral camera snapshot.

The controller uses ship-local floor gravity while inside, clamps movement to
simple invisible interior bounds, and keeps the closed/flying doorway solid. An
open landed doorway begins an exact 0.25-second up-vector blend before the
controller changes to Earth-radial surface walking. Surface movement is projected
back to the same spherical radius each update, so the portable rule works around
the full body rather than only near the initial landing point.

Cockpit entry and exit are instant. The default seated view is slightly pitched
down while retaining a clear forward sightline through the Task 9 glazing; mouse
look remains independent and unrestricted. This crate does not decide whether
the ship is landed, whether the door may open, or how ship motion changes; the
runtime passes those facts through `ShipFrame`/`SurfaceFrame` DTOs.

See [README.ai.md](README.ai.md), [architecture.md](architecture.md), and
[invariants.md](invariants.md) before modifying controller behavior.

# Construction flight safety (#546)

Physical consequences (`hull_affecting` and `flight_eligible`), rather than
blueprint category names, govern flight permission. Structural expansion and
hull-affecting work require landed state. The portable placement authority also
recognizes docked permission; the playable client has no docking state yet.

With C open, aim at a reachable unfinished project and press **T** to stop it
immediately. Progress, exact receipts and mass remain unchanged. Press T again
to explicitly resume after current safety validation. No extra Start action is
required for a new fully funded project. Stopped work spends no construction
energy, and the stop latch survives checkpoints, reload, blackout and landing.

An unfinished exterior project blocks takeoff until stopped or cancelled. The
cockpit shows a specific reason and the C/aim/T remedy. A stopped project is
**secured** only if its site is undamaged, the current completed hull matches its
canonical connected geometry, and its physical attachment remains supported.
Otherwise it is **unsecured** and takeoff remains blocked. Securing is computed
from current authority every time, never trusted from a saved/UI boolean.
Current partial projects have only non-solid reveal edges: they do not cut the
existing sealed hull or publish temporary colliders. Stop can therefore secure
an intact supported site immediately; it does not magically seal a damaged hull.

The readiness check compares the actual graph, faces, connections and completed
hull condition with validated completed project sources. Missing/damaged completed
cells, removed exterior faces or lost attachment refuse takeoff. Allowed shared
internal partition edits are included in the canonical source and remain valid.
Completed geometry, ramp and door constraints retain their owners. Cancellation
uses the [exact physical refund transaction](construction-materials.md), removing
only unfinished work and preserving completed neighbors.

Airborne exterior work is immediately latched stopped before any work debit,
including forced displacement. It neither teleports the ship nor clears velocity,
throttle or controls. Landing does not resume it. Safe interior work remains
eligible during coasting. The existing actual Core power, engine-switch and
cockpit-authority gates remain enforced by the ship controller; optional
Fabricator/Jump Drive availability does not itself ground the ship. General
module downtime/commissioning follows the [non-Core module contract](non-core-modules.md) and actual damage/salvage by #549;
this change adds no pretend damage source or module-state flags.

Whole-session schema **19** adds `Work.stopped`. Versions 1–18 migrate with the
latch false; an older envelope containing an active latch rejects atomically.
Completed-and-stopped work rejects as contradictory. Checkpoint event tracking
includes latch transitions. Restore never advances construction or grants energy.

Runtime `session/construction/flight.rs` owns physical admission and Stop/Resume;
ship `structure/construction.rs` owns the durable validated latch. Native T and
protocol `construction_stop_resume` share the same builder command. Runtime maps
readiness to typed cockpit text shown in the bottom-center flight guidance and
window title. Instrument telemetry retains its existing owner.
Run construction tests, workspace gates and the production wgpu evidence scenario
`scenarios/evidence/construction-interlocks.json`. Human presentation, physical
device qualification and hardware budgets remain #561, #557 and #563.


## Optional module commissioning (#547)

See [non-Core module contract](non-core-modules.md) for the Archive processor, G/E/T/RMB maintenance, immediate activation downtime, real commissioning faults, paid restoration, schema 20, source/test mapping and deferred Jump Drive follow-up #633.

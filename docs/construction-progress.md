# Automatic construction (#544)

A fully supplied placeholder begins automatically on the next active simulation
tick, without a Start action, held tool, builder UI, player proximity or continuing
line of sight. The existing [material/refund contract](construction-materials.md)
remains authoritative. Catalog durations remain 10/20/30/120/90/300/60 seconds.
`client/runtime/catalogs/construction-power.json` supplies positive construction
watts per blueprint: 1 kW stool, 2 kW partition/door, 5 kW hull/deck, 10 kW room,
3 kW utility module. These are explicit initial power tuning, separate from the
owner-approved material costs and durations, and require no extra resource.

## Funding, timing and safety

Runtime settles ordinary ship/Fabricator work, then recovery, then projects in
stable ID order. Construction uses `fund_auxiliary_work`, preserving both captured
flight reservations and the existing recovery emergency/mobility reserve. Its
power is included in total observed auxiliary draw. Only actual debited joules
buy time. A fully funded tick credits its exact integer nanoseconds; an energy
limited tick rounds paid time down to a nanosecond (at most one nanosecond of
charged energy is uncredited). Partial ticks stop at the blueprint's duration.
The existing integer-ppm progress drives proportional physical refunds.

Every tick repeats ownership, current flight eligibility, structural support,
installed/cabin/mount/protected-route, player and individual cargo occupancy
checks. Player distance/aim is deliberately irrelevant to automatic work.
Insufficient energy, invalid safety or a damaged project pauses without spending
or resetting progress. Clearing the condition resumes automatically. Project
condition is a saved seam for #549; this change invents no damage source. Exterior
stop/takeoff latches follow [construction flight safety](construction-flight-safety.md). Normal pause and terminal state freeze work;
restart clears the old run's construction. No closed-game elapsed time is credited
here: chronological construction/offline reconciliation remains #550/#433.

## Completion and presentation

Stools and partitions have a complete simple physical installation adapter.
Immediately after successful final safety validation and the last paid interval,
the existing stable project commits its completed flag once. Its original receipt
mass remains in one ConstructionMaterials ledger contribution, without spawning
another resource object or adding duplicate mass. Shared ship-local solid boxes
feed visible stool seat/legs or partition geometry, character and fragment/carry
collision, placement and cabin-motion clearance. Unfinished work has no collider.
Completed source survives save/load; it cannot be cancelled as unfinished work.
Safe dismantling/damage is separate #549 work, not a refund shortcut.

Structural hull/deck cells and four-cell/two-deck rooms now complete through the
[modular structure adapter](modular-construction.md). Its final graph/connector
clearance preflight precedes the last paid interval. Internal doors also complete into manual, saved pocket doorways with
occupancy-gated closure. Utility modules complete through the [commissioning contract](non-core-modules.md). Legacy two-cell room reservations remain cancellable but cannot
complete into an unsafe stair footprint. These unsupported types stop one
microsecond before completion, spend no more energy there and never become solid.

Hollow amber edges reveal upward from paid progress; completed simple geometry
replaces them atomically. The builder shows percent and explicit pause reason.
Active construction uses a restrained spatial machine sound through the existing
bounded Fabricator synthesis voice, coalescing with Fabricator audio when both
run. Pause/blackout/inactive lifecycle gates suppress audio. Rendering/audio own
no progress or safety. Human appearance/listening remains #561; physical devices
#557 and actual hardware budgets #563.

## Save and verification

Whole-session schema 17 adds validated elapsed nanoseconds, damage and completed
source. Legacy #543 progress reconstructs at its original ppm precision on its
first paid tick; current elapsed/progress mismatches reject atomically. Historical
schemas cannot smuggle work/completion fields. Completion and damage transitions
trigger discrete checkpoints; continuous progress uses periodic/manual/exit saves.
Restore reconstructs collisions without advancing work or spending energy.

Run `cargo test -p salimon-client --locked session::construction`, ship and
workspace gates. The production protocol/wgpu route is
`scenarios/evidence/construction-build.json`; it explicitly seeds one carried
1 kg iron fragment, feeds with E, checks half-progress and completion, and tests
that unfinished cancellation cannot remove the completed stool. It is a fixture,
not evidence of a mining journey or physical OS input.


## Optional module commissioning (#547)

See [non-Core module contract](non-core-modules.md) for the Archive processor, G/E/T/RMB maintenance, immediate activation downtime, real commissioning faults, paid restoration, schema 20, source/test mapping and deferred Jump Drive follow-up #633.

# Completed modular structure (#545)

The C builder's hull, deck and two-deck room blueprints now complete through the
same [paid construction transaction](construction-progress.md). A room is one
project, one material supply and one automatic build, with no Start action.
The two-deck room reserves four 3 m cells (6 × 6 × 3 m), retains the approved
40 kg iron cost and 300-second duration, and includes a 20-tread staircase,
upper-deck hatch and landing. Hull/deck cells retain their existing cost/time.
Utility-module commissioning follows the [non-Core module contract](non-core-modules.md). Internal doors complete into
manual pocket doors: with C open, aim and press E to open/close. Closing checks
the player and physical cargo before changing the saved state; the leaves
retract into their frame pockets. No powered module is fabricated here.

## Placement and access

The first structural addition uses the existing aft cargo gate: its volume must
include cell (-3, 0, 1) on the explicit Scout grid, with its forward edge at
X=-8.5 m and its lower doorway spanning Z=-1.5…1.5 m. The builder's quarter-turn
rotation supports aiming a room backwards from this connection. A short enclosed
connector joins that face to the existing gate. The authored Scout hull, original
Core, cockpit, fixtures and precision anchors are unchanged; side attachments
cannot silently cut a new opening through an authored wall.

Further additions require a horizontal face connection to a completed room at
the same base deck height. Reservations alone provide no support. Existing
fixtures, cargo, player occupancy and protected routes still reject placement.
Adjacent completed rooms acquire a shared 1.4 m wide, 2.2 m clear doorway;
internal cells of one room share an open volume. The original gate still controls
passage to the Scout. The outermost lower cell also supplies an exterior doorway and 20-step boarding
stair. Orange handles use the existing E gate control; its saved open/closed
state drives both the original and outer gate proxies. Closing checks both
passages. Surface walking can leave and board again at the stair foot without
being captured from below the hull. Surface/EVA bodies also sweep against the
new exterior panels. The original ramp mechanism retains its existing owner.
Further aft expansion reroutes the outer boarding stair only when its old route
is unoccupied; the new stair foot must match terrain within 0.3 m.

Before the last paid interval, runtime stages the whole resulting graph and
checks new solid geometry, including the connector, against the player and all
physical fragments. A failed check spends no final joule and publishes nothing.
There are no early colliders, usable decks or room mass contributions. Completion
publishes all cells, room membership and collision together. Powered walking,
jumping, body clearance, stair ascent/descent and zero-gravity drift use these
completed volumes. Walking cannot snap through the stair hatch to a lower deck.
The same boxes drive production wgpu instances, carry/loose-object collision,
room containment and cargo support. Exposed corners have stepped clipped panels
and a tapered cap, with real matching collision rather than cosmetic silhouettes.
Power/pressure network hookups and decompression simulation remain future work.

## Source, identity and editing

Ship `structure/{assembly,shell}.rs` owns validated atomic assembly and deterministic
cell/face geometry. Runtime `session/construction/{installed,boarding,doors,work}.rs` composes
catalog/source, physical safety and paid publication. `evidence.rs` owns only
explicit native fixture setup. Character
`modular.rs` and `controller/modular.rs` consume only host-supplied completed
volumes/solids; they grant no construction authority. Renderer consumes ordinary
oriented box instances without owning topology or work.

Cell parts use stable IDs `project_id * 1024 + 1024 + sorted_cell_index`, with
at most 128 cells per project; the separate mass-free room ID uses offset 2047.
Faces have stable `(part_id, direction)` identities, and each part retains its
own collision and mass. Completion order does not allocate or renumber these
identities. Existing structural damage/detachment and domain save APIs can address
one cell independently; gameplay damage, salvage and detachment follow the [damage contract](construction-damage.md).

With C open, aim at an ordinary shared room partition and press **Delete** to
remove it. Reach, line of sight and owner/survival gates apply. Only a horizontal
shared doorway between completed cells is eligible: exterior panels, floors,
stairs and the authored hull cannot be removed this way. The frame and room
connectivity remain, as does its material mass; this is an opening edit, not
salvage or an unfinished-work refund. Saved edits regenerate both visible and
collision geometry and trigger a checkpoint. Later expansion preserves existing
cell IDs and updates shared boundaries atomically.

## Save and mass

Whole-session schema 18 retains original completed project sources, manual door
states and validated partition edits. Restore rebuilds the deterministic per-cell graph, room
membership, geometry and collision without time credit, new IDs or energy spend.
Structural material receipts transfer out of `ConstructionMaterials` at completion
into disjoint `Structure(part_id)` ledger entries; no room or face adds duplicate
mass. Original receipt identities remain reserved. Completed room reservations allow supported interior furnishings to be placed
inside them. Fine horizontal snapping preserves the actual completed deck
elevation; equipment still needs real support and clear final geometry.
Loose cargo in completed rooms
retains ship-local identity, support, mass and motion across reload/ship movement.

Historical schema-17 two-cell room reservations are recognized verbatim and are
still cancellable with their original receipts/refunds. They cannot complete
into an unsafe narrow staircase: cancel and place the current four-cell blueprint.
No migration silently enlarges an occupied reservation or charges extra material.
Installed structural work or partition edits smuggled into older schema envelopes
reject. Malformed geometry/edits and unsafe authored-hull overlap reject before
replacing a live session. The independent universe save envelope is unchanged.

## Verification

`cargo test -p salimon-client --locked construction` covers final-tick publication,
unattended single-intake work, closed/open gate traversal, stairs, jump/gravity,
zero-gravity drift, late connector obstruction, adjacent expansion, shared doorway
editing, checkpoints, save rejection and upper-deck cargo identity/mass/support.
Ship/character and workspace tests/build/Clippy remain required.

`scenarios/evidence/construction-room.json` seeds only a reservation, one carried
40 kg sphere, open gate and review camera. E supplies that sphere through actual
intake; protocol steps pay the full 300 seconds. Retain real wgpu screenshots of
reservation/progress/completion both decks, the clipped exterior shell, and surface exit/reboarding. The fixture is explicit setup,
not evidence of mining a 40 kg piece or physical OS input. See the
[issue report](../reports/issue-545/README.md) for actual results and limitations.
Human presentation review remains #561; physical devices #557 and hardware/Metal
budgets #563. Software Vulkan images do not establish hardware frame rate.


## Optional module commissioning (#547)

See [non-Core module contract](non-core-modules.md) for the Archive processor, G/E/T/RMB maintenance, immediate activation downtime, real commissioning faults, paid restoration, schema 20, source/test mapping and deferred Jump Drive follow-up #633.


Gameplay damage, safe dismantling, surviving-site recovery and physical debris now follow the [damage contract](construction-damage.md). Offline progression remains #550.

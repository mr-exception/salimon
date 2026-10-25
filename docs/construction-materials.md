# Construction material transactions (#543)

`ship::structure::funding` owns integer-milligram receipts on each placeholder.
With the builder open (C), aim at the existing project and press E to contribute
the one carried fragment. Owner, reach, sight and actual carry clearance gates
apply. Only a required resource with a whole-milligram mass fitting the remaining
cost is accepted. Oversized pieces are rejected unchanged; there is no 20 kg
Fabricator cap, splitting, remote cargo debit or separate Start control.

The catalog's `approved_economy` gate must be true before any intake. The owner
explicitly approved the proposed costs/durations in this chat on 2026-10-24;
`client/runtime/catalogs/blueprints.json` now enables that approved catalog.
The exact values and decision are recorded in [Project Q&A](project-qa.md).
A fully supplied project is ready for automatic Building, without another action.
Core-funded active work and simple interior completion now follow the
[automatic construction contract](construction-progress.md); completed structural geometry follows [modular structure](modular-construction.md). The monotone `set_progress` seam retains exact refunds.

RMB cancels a reachable unfinished project. At progress p, return
`floor(delivered_milligrams * (1-p))` once per resource, distributing in original
receipt order. Underfunded projects have p=0 and return everything. Original
IDs, deposit provenance and resource identity survive. Zero-mass returns are
omitted; integer rounding belongs to consumed material. At 100% unfinished
cancellation rejects. Retained receipts account for supplied matter once while
work is active; consumed cancellation matter leaves the ship's material ledger.

Before cancellation, runtime finds clear nearby deck positions for every refund,
checking hull, sight, door, cabin, player and other cargo. Insufficient space
leaves the entire project and all receipts unchanged. World batch return validates
all identities/materials before inserting any physical objects. Cancellation
then removes the reservation; repeats cannot produce another refund.

Schema 15 adds funding receipts/progress. Older placeholders migrate empty;
funded state in a historical envelope rejects. Restore checks catalog costs,
unique IDs across projects/Core/world and allocator bounds before publishing.
The ConstructionMaterials ledger replaces carried-fragment mass at intake,
and physical returned identities replace it at cancellation. Save/load never
advances progress or replays an intake.

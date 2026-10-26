# Construction damage and physical salvage (#549)

`session/construction/damage.rs` composes typed physical impacts, safe deliberate
removal and surviving-site recovery. Ship owns topology/receipts and exact paid
work; world owns fresh sealed debris IDs; the existing physical-fragment solver
owns debris motion/contact. Neither rendering nor the evidence protocol owns
material, graph or work authority.

## Damage and recovery

An impact addresses one project or one independently identified structural cell.
A non-destructive hit interrupts that site's future work without changing its
receipts, paid nanoseconds or identity. Completed cell condition is independent
of the neighboring cells. Red wire bounds show damaged survivors. With C open,
aim within 5 m and press H to mechanically resecure surviving construction.
This only clears interruption: it creates no material, lost cell or paid time.
Damaged utility processors remain unavailable and require the existing physical
G/E restoration or continuation/commissioning transaction before their bonus returns.

The existing crash transition interrupts surviving construction. There is no new
weapon or player damage hotkey. Localized destructive producers supply the typed
`Impact` with an explicit surviving-material fraction in integer ppm. The E2E-only
`construction_impact` operation injects that same physical boundary for repeatable
verification; it is fixture input, not proof of a combat encounter.

A destructive hit transfers surviving actual material to loose physical debris.
The hit cell uses the supplied survival fraction; disconnected unhit cells keep
100% of their material. This initial physical salvage representation is raw-material
fragments, rather than a second ship or an intact rigid wreck mesh. Fragments use
the ordinary pickup, gravity/contact, Core intake and persistence paths; there is
no protected inventory, lock or remote recovery grant. Unsupported completed
furnishings are also released as physical salvage. Unfinished reservations whose
support disappeared pause until support returns or the owner cancels them.

Each cell receives an exact integer-milligram share of every material, with
remainders assigned to the first original sorted cells. Impact loss floors once
per cell/resource. Original receipts remain provenance and reserved identities;
new debris gets fresh monotonically allocated IDs. Destruction, cell-loss
publication, collider/containment refresh and mass transfer commit together.
Repeated destruction rejects without another object or allocator change.

Lost cell IDs remain tombstones. Surviving cells retain their original IDs and
geometry, including exposed openings; damage does not add free replacement walls.
Partial hull loss prevents takeoff. Remove the remaining damaged assembly safely
and build a fresh project to replace it. A wholly destroyed reservation no longer
blocks a replacement; its project ID and original receipts cannot be reused.
Original authored hull/Core IDs are rejected by the construction-damage boundary.

## Cancellation and dismantling

RMB on unfinished work still returns exactly `(1-p)` of each delivered resource,
with the established per-resource integer floor. Damage survival is a separate
physical input and never substitutes for this voluntary cancellation rule.

With C open, X dismantles an aimed **completed** object/cell while landed.
It refuses a structural disconnect, unsupported installed machinery and occupied
player/cargo space. Deliberate dismantling has no impact loss and releases the
existing material as physical salvage. It cannot cancel an unfinished project,
remove authored structure, resurrect a destroyed site or replace the Core.

## Persistence and verification

Whole-session schema 21 adds validated lost/damaged cell identities and destroyed
project tombstones. Schemas 1–20 migrate with no new losses or time credit;
older envelopes cannot smuggle these fields. Restore regenerates only surviving
parts, their shared visible/collision geometry and disjoint mass entries. It never
replays salvage allocation. Discrete damage/repair/removal changes checkpoint.

Run `cargo test -p salimon-client --locked construction`, ship/world and workspace
checks. Tests in `session/construction/damage/tests.rs` cover partial interruption,
exact cancellation, actual survival vs cancellation, replacement, detachment,
protected identities, stable reload/mass, safe removal, module restoration,
malformed saves and exhausted allocator atomicity. Production wgpu routes:
`scenarios/evidence/construction-damage.json` and
`scenarios/evidence/construction-detachment.json`.

[Completion evidence](../reports/issue-549/README.md) records actual results.
Human presentation review remains #561, physical platform/input checks #557,
and hardware/Metal budgets #563. Offline chronology follows [the schema-22 contract](construction-offline.md); universe/scanner
integration remains #433. No new combat system, decompression simulation or
reference-hardware performance claim follows from the damage boundary.


Closed-game power and eligible construction/maintenance now follow [the schema-22 offline contract](construction-offline.md). Native launch advances service/conversion, Fabricator/Recovery and eligible work while preserving the saved scene and active survival clock. Future universe/scanner/trajectory events remain #433.

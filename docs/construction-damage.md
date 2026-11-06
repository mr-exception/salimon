# Construction damage and physical salvage (#549)

> **Corpus migration status (#791): Historical native implementation.** The current [React overhead 2D world](current-client-status.md) uses a separate **cockpit-only 3D radar**. Any older Rust, wgpu, 3D assets, first-person view or issue-delivery statements below concern the [archived native tree](https://github.com/mr-exception/salimon/tree/8f112bb6e6be9f813174efb47d706a8601930d04) rather than shipped browser features. Keep approved ship layout and state invariants, but render with an exposed top-down Scout cutaway and ship-local footprints. Native physical fragments, first-person cameras and localized hull damage do not describe shipped 2D gameplay; current damage scope is ship-wide. See the [migration register](2d-migration-register.md) for readiness and ownership.


> **Future 2D scope correction (2026-11-06):** Historical native **localized combat damage and detachment** below remain deferred; current 2D combat damage/destruction is **whole-vessel only**. However, the owner has approved **future deliberate component-by-component ship dismantling** (walls, rooms, modules and cells), not all-at-once deletion. A bound [Stasis Pod](stasis-pod-and-transfer.md) lost while its character is **offline** causes terminal death, regardless of their last physical boarding position; a living player can move their binding via approved transfer. The native C/X controls and physical fragment mechanics are archival, **not** present in the React game. Do not treat this page as a shipped implementation.

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

## Future 2D salvage reconciliation

- The newly approved **Stasis Pod** is a normal separately addressable module for build, removal and terminal pod-loss effects. Per-part deliberate dismantling must respect remaining structure, cell/room identities, occupancy and module dependencies.
- **Not a mass hull delete:** remove one supported fixture/module/wall/structural part at a time; a multi-cell room may lose valid individual components.
- **TBD:** exact dismantle resource yield and tools, final protected Core-root decommission, pod-support removal ordering, hazard to live occupants and what becomes of cargo if a support cell disappears.
- Keep completed-part salvage separate from cancellation of a partially funded/building placeholder; do not reuse native fragments or Rust APIs as browser implementation evidence.

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

[Completion evidence](https://github.com/mr-exception/salimon/blob/f1cd0b3e878fac487ce898547a0c8c35aa40519b/reports/issue-549/README.md) records actual results.
Human presentation review remains #561, physical platform/input checks #557,
and hardware/Metal budgets #563. Offline chronology follows [the schema-22 contract](construction-offline.md); universe/scanner
integration remains #433. No new combat system, decompression simulation or
reference-hardware performance claim follows from the damage boundary.


Closed-game power and eligible construction/maintenance now follow [the schema-22 offline contract](construction-offline.md). Native launch advances service/conversion, Fabricator/Recovery and eligible work while preserving the saved scene and active survival clock. Future universe/scanner/trajectory events remain #433.


Cosmetic aging #581: [source/identity, successful use, schema 23, rendering budgets and exact verification](surface-age.md). Damage remains condition-driven; clean repairs restore normal geometry and preserve age.

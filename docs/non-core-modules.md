# Non-Core module commissioning (#547)

The existing `utility-module` blueprint now installs an optional **Archive utility
processor**. It occupies the declared 1 × 1 × 1 m ship-local envelope, with the
existing 0.5 m fine snap and actual deck/mount, player, cargo, authored-service-route
and installed-geometry checks. A completed room can supply the deck. It does not
replace the fabricated Data Recovery Module: that module still owns Archive
recovery and its two saved record jobs.

The [owner's scope decision](https://github.com/mr-exception/salimon/issues/547#issuecomment-6400459515)
authorizes balanced utility defaults and defers Jump Drive tuning. The following
initial choices preserve the approved 3 kg / 60 s installation and the existing
3 kW construction power. Faster recovery draws proportionally more actual Core
power, preserving joules per record and the emergency/flight reserve. Multiple
processors use the best available tier without stacking.

| Operation | Iron | Active time | Build power | Commissioned effect |
| --- | --- | --- | --- | --- |
| Install tier 1 | 3 kg | 60 s | 3 kW | Up to 6.25 MW recovery instead of 5 MW (1.25× throughput) |
| Upgrade tier 1 → 2 | 2 kg | 90 s | 3 kW | Up to 7.5 MW recovery (1.5× throughput) |
| Restore previous tier after activated cancellation | 1 kg | 30 s | 3 kW | Restores the previous tier, with no free upgrade |

All contributed material remains in the single physical mass ledger. Successful
maintenance material remains installed; restoration is an explicit material and
work cost. The durable maintenance receipt collection is bounded to 128 receipts;
intake rejects at capacity without taking the carried item. This is initial
utility tuning, not Jump Drive or survival balance approval.

## Controls and downtime

Use **C**, select **7**, place, then **E** with one fitting carried iron fragment
at a time. Full funding starts automatic installation. Initial unfinished work
has no physical module collider or capability. Completion publishes one solid
case through the existing shared render/collision geometry.

With C open, aim at the completed case and press **G** to plan its upgrade.
Placement/planning and partial deposits leave the old processor usable. The final
required deposit **immediately disables the entire processor**, before the next
simulation tick. E feeds maintenance, **T** stops/resumes its saved work latch,
and **RMB** cancels unfinished maintenance. All these controls retain normal
view/modal/focus, owner, reach and occlusion admission. Maintenance is independent
of the builder being open or the player remaining nearby.

Cancellation returns exactly the existing `(1-p)` quantity of every contributed
resource using original receipt IDs/provenance and integer-milligram rounding.
Clear physical refund positions are checked against existing constructed objects
as well as the cabin/player/cargo. A failed return leaves all source unchanged.
Cancelling before activation keeps the previous module available. Cancelling
after activation preserves its physical case, stable project ID, old tier and
installed material, but **keeps it unavailable**. G then plans the separate
restoration above. There is no magical rollback or immediate old-tier activation.
Completed work cannot be cancelled as unfinished work.

## Actual commissioning and safety

Only paid work advances. Pause/terminal state, no surplus power, stopped or damaged
work and unsafe geometry pause maintenance without changing receipts or elapsed
nanoseconds. The same actual Core auxiliary funding and mobility/flight reserve
used by ordinary construction apply. Closed-game maintenance follows [the schema-22 offline contract](construction-offline.md).

Commissioning checks current power, original Core-connected hull, actual deck
support, damage and player/cargo/fixture clearance. The module's own installed
case is excluded from that final occupancy test. A failed commissioning retains
all paid material/work and reports `Damage`, `NoPower`, `Connection` or `Clearance`;
capability remains unavailable. Resolve the actual condition and press G to retry
without repaying completed work. Damaged maintenance can also be cancelled and
replaced by a funded restoration. Authoritative damage producers and structural salvage follow [the damage contract](construction-damage.md).

This processor is non-invasive inside work and can run while coasting. Its outage
does not itself ground the ship. Existing Core, engine, cockpit, ramp and
[structural takeoff interlocks](construction-flight-safety.md) remain authoritative.
Neither installation nor upgrade changes throttle, engine enablement, velocity or
cockpit authority. No operation automatically starts recovery, engines or a jump.

## Ownership, save and verification

Runtime `session/construction/modules.rs` owns saved tier, maintenance receipts,
work, availability and faults. The existing ship `Funding`/`Work` own exact intake,
refund and paid-time arithmetic. `session/recovery.rs` requests the selected actual
watts; Core retains debit authority. `work.rs` supplies the physical case through
ordinary scene instances and character/fragment collision; renderer has no module
or progress authority. Maintenance uses the original blueprint footprint rather
than a second placement or protected inventory.

Whole-session schema **20** adds module source keyed by the original placeholder
ID. Earlier v1–v19 migrate with no commissioned modules; existing incomplete
utility reservations retain receipts/progress and finish through ordinary paid
work. Historical envelopes containing module source reject. Restore validates
registered blueprint/owner, tier/phase, exact receipts, paid progress, allocator
bounds and unique ownership across original materials, maintenance, world and
Core upgrades before publishing any session. Save/load grants neither time nor
energy and never commissions a faulted module automatically. Discrete activation,
cancellation, fault and commissioning changes trigger checkpoints; running work
uses the existing periodic/manual/exit checkpoints.

Tests: `cargo test -p salimon-client --locked session::construction`, including
`modules/tests.rs` for real recovery joules, partial funding, downtime, 50% refund,
restoration, damaged/outage/stopped work, commissioning failure, mass, migration
and exact reload. Run workspace build/tests/fmt/Clippy and runner/document checks.
Production wgpu routes are `scenarios/evidence/construction-module*.json`; their
explicit seeded material/prior-work setup is documented in the
[completion report](https://github.com/mr-exception/salimon/blob/f1cd0b3e878fac487ce898547a0c8c35aa40519b/reports/issue-547/README.md). Human appearance, physical
platform and hardware/Metal budgets remain #561, #557 and #563 respectively.

## Deferred Jump Drive

The canonical initial N=20%, reusable physical-module/no-consumable-cell rule and
Core-joule cost in [the Jump Drive contract](jump-drive-and-interstellar-travel.md)
remain unchanged. No Jump Drive recipe, footprint, N-tier reduction or jump
operation is invented here. [Follow-up #633](https://github.com/mr-exception/salimon/issues/633)
retains its physical install/commissioned-N acceptance after owner tuning and
real-scale flight/navigation prerequisites. It must reuse these downtime,
physical-material and commissioning contracts with its actual module-specific
footprint and safety; the utility processor is the completed example.


Gameplay damage, safe dismantling, surviving-site recovery and physical debris now follow the [damage contract](construction-damage.md). Offline progression remains #550.


Closed-game power and eligible construction/maintenance now follow [the schema-22 offline contract](construction-offline.md). Native launch advances service/conversion, Fabricator/Recovery and eligible work while preserving the saved scene and active survival clock. Future universe/scanner/trajectory events remain #433.

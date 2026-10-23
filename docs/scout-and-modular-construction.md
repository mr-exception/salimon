# Scout art direction and modular ship construction — approved design (2026-10-22)

Verification uses the [cloud/human workflow](verification-workflow.md)
(owner decision 2026-10-23). Routine implementation and integration acceptance
run in cloud; human spatial/art review and actual platform qualification are
separate. This supersedes blanket M1-only delivery gates without changing the
approved construction rules or measured asset budgets.

**Status:** owner-approved **future target and implementation contract**, **NOT a statement that these features already exist in the playable client**. This document consolidates the detailed Scout/art-direction and 3D construction design review. Delivery is tracked by [#538–#551](https://github.com/mr-exception/salimon/issues/538) (individual links at the end). The current asset dimensions, anchors, 100-tonne dry-mass baseline and rendering budgets remain authoritative *for currently implemented code* until deliberately changed and reverified.

**Priority and supersession:** This contract takes precedence over older future-design claims that the starter Scout should have no dedicated cargo room, that construction uses only Z-axis snapping, that Core upgrades need staged worksite power, or that cancelling a partially built object needs stage-by-stage salvage. Historic implemented-state descriptions must remain explicitly distinguished from this approved future target. Existing canonical propulsion, Core fueling, Fabricator, survival, universe, narrative and Jump Drive contracts are unchanged except where this document expressly says otherwise.

## Goals and experience

- **One permanent Scout:** the player's first and final player-owned ship. It is a familiar, functional home/workshop/vehicle at the start and physically grows through upgrades. Do not require any building tools to use the starter.
- **Grounded cinematic low-poly:** a warm, credible, lightly lived-in interior contrasting with the exposed alien world. Prioritize readable silhouettes and functional machinery, lighting, feedback and silence over ornament/polycount.
- **Real physical interactions:** cargo has individual object identity, mass and geometry, with the permanent rule that a character carries **one world object at a time**. No conventional backpack, magical cargo ledger, virtual crate, invisible ship-wide construction supply or arbitrary auto-transfer.
- **Meaningful but simple expansion:** place a transparent preview/placeholder of an object, room or module, carry and feed its required materials directly, automatically build when fully supplied, and use the completed result after a short animation.
- **Safety even before detailed simulation:** physical hull integrity, exterior construction and critical-module downtime constrain takeoff before detailed vacuum/pressure or aerodynamic simulation is implemented.

## Starter Scout: spatial and art target

The starting ship comes fully playable with:
- Forward cockpit with walkable access, a wide readable flight monitor, existing usable lower forward glazing and flight controls.
- Main cabin containing the existing Energy Core, wall-integrated Fabricator and physical Archive terminal, with their existing working interactions; keep adequate access/carrying clearance.
- Compact wall-integrated habitation: fold-away bunk, small food/water galley and basic hygiene/toilet/washing installation. These need honest visual/physical representation and can have simple interactions, but **do not claim future health, hunger, hygiene, pressure or water systems are playable** until those systems are implemented.
- **Dedicated aft physical cargo bay**, separated from the main cabin by a **full-height structural bulkhead and wide, closable internal doorway**. This restores dedicated cargo space to the future starter, superseding the previous removal decision, without requiring a protruding cargo pod.
- Existing aft exit/boarding ramp retained as the exterior loading path; central passage continues from ramp through cargo to the systems and cockpit. The internal door is distinct from the exterior ramp/airlock.
- Open physical bins or simple racks for individually retrievable loose fragments and objects. No abstract types/stacking, protected containers, teleport-to-fabricator or loss of mass. A rack may physically restrain its contents if designed to do so; an open bin is not an invisible physics suppressor.
- Core-powered artificial gravity in normal operation and current weightless drift/contact behavior in blackout; don't invent independently simulated pressure zones for the internal cargo partition.
- **No Spatial Jump Drive installed initially.** It is a later, real, persistent, upgradeable machine constructed inside a suitably sized engineering area. Current Jump Drive contract starts at N=20%.

**Geometry caveats:** The current Scout is a manually authored GLB with precision anchors/colliders and an approximately 2.8-m-wide aft ramp. Aim for comparable usable internal cargo-door clearance, but door frame, hull enlargement, actual dimensional budget and grid/origin alignment must be validated. The final silhouette need not be rectangular or cube-only. Current GLB triangle/primitive/material ceilings cannot be assumed to accommodate extra rooms without architecture/budget review. Preserve existing gameplay and asset constraints until an implementation explicitly migrates them.

## Future structural grid and room construction

- **Exactly 3 m × 3 m × 3 m** ship-local structural grid along X (fore/aft), Y (up/down) and Z (port/starboard), anchored to the permanent Core-bearing ship. The starter has one main deck; future ship construction supports decks both above/below, horizontal extension and room volumes spanning many cells.
- Face-connect adjacent structural cells; merely touching an edge/corner does not constitute a structural connection. Track the six cardinal neighbor directions and whether each cell can trace a valid connection to the permanent Core-bearing structure.
- Floor, ceiling, hull face, wall, doorway, stairs/ladders/hatches and large equipment footprints can span/combine cells. Internal partitions may be removed without destroying a viable structural frame. A room can be designed as one **multi-cell placeholder/project** instead of one project per cell.
- Structural grid controls connectivity and main geometry. Fine snapping for doors, furniture and fixture placement can be separate, data-driven (the earlier **0.5 m** proposal remains a suggestion, **not an owner-approved mandatory dimension**).
- External hull shell can use tapered/clipped low-poly shapes and meaningful bevels; structural cells do not force rectangular architecture.
- **Normal deliberate removal is refused if it would disconnect any remaining ship section.** Damage/combat can break connections, create independently physical detached wreckage, damage systems and future pressure boundaries. Structural connectivity is distinct from traversal, power and future pressure graphs.
- Room construction changes actual ship silhouette, walkable volume, mass and collisions after completion; validate reachable doorways, cargo/player occupancy and cross-deck traversal.
- Existing precisely-authored cockpit seat, Core, Fabricator, Archive and ramp coordinates are **not rounded to the new grid** without a validated asset/runtime migration.

## Simplified player building loop (authoritative UX)

1. Select an unlocked blueprint for an object, furniture, wall, door, room/deck/hull extension or ordinary ship module.
2. Position its visibly translucent **placeholder** at a valid location on the applicable ship snap grid, with footprint, shape, structural connectivity, safe access, owner authority and collision/overlap validation.
3. Place it. **The placeholder itself is the construction site and physical resource receiver.** No extra robotic construction station, central controller, manually-held tool timer or separate Start button.
4. Bring world resources/components physically **one object at a time** and feed the placeholder. Show per-material requirements and received/remaining amounts. Reject incompatible/unaccepted materials. Do not pull materials automatically from the starter cargo bay or Fabricator's internal storage. Avoid waste/duplication with deterministic physical remainder or clear over-size rejection.
5. As soon as **all required materials** are present and safety validation passes, start automatic construction. No further player action. Use a restrained, inexpensive low-poly assembling/reveal animation and sounds.
6. At completion, verify clearance/connection and atomically replace the nonfunctional placeholder/temporary reveal with the correctly attached and usable final object/structure/module. Do not prematurely add blocking final colliders that trap people/cargo.

**Timing:** data-driven by blueprint. Simple/small objects take approximately **5–20 seconds**; intermediate walls, fittings and rooms can take longer; large/complex construction lasts up to **5 minutes**. These are short build timers, not intricate assembly minigames. An exact cost/timer catalog is not approved yet; do not invent a fixed 100 kg/50 kg resource requirement from UI examples.

**Power:** Ordinary automatic work uses legitimately available Core construction energy where its blueprint calls for power, without violating emergency/flight reservations. Insufficient supply pauses funded work rather than granting free progress. Core upgrades are a specific exception below.

**Progress/cancellation/refunds:** Before a fully funded placeholder begins building, cancellation returns **100% of physically supplied materials**. Once building, at fraction `p` of completed progress (`0 <= p < 1`), cancelling returns `(1 - p)` of **each accepted resource type** as actual physical fragments, and consumes `p`; e.g. **50% complete → 50% of every contributed resource type returned**. Remove the unfinished placeholder and temporary build visuals in an atomic cancellation transaction. Do not require separate stage-specific deconstruction/salvage gameplay for cancelled unfinished work. At 100% the object is completed; further removal is regular safe dismantling rather than project cancellation. Use precise quantization, predictable rounding and idempotent transactions; no resource or mass duplication. Damage-related material survival can still depend on actual physical damage; never confuse that with voluntary cancellation.

**Mass:** Delivered material remains real mass while held by the placeholder; as progress consumes a proportional quantity, that quantity belongs to the in-progress assembly ledger and is not also counted as loose cargo. Completion converts it to the installed object's mass exactly once. No resource identity is simultaneously cargo, intake and finished hull.

**Project states:** planned/waiting, building, paused (including stopped-secured or stopped-unsecured exterior work), completed, cancelled, damaged/recovery. The UI may be simpler than this internal state list. Pausing preserves project state and produces no refund. Damaged sites/parts stop work when unsafe; surviving materials/stages remain physical, with recovery or repair rather than arbitrary deletion.

## Flight safety and offline behavior

- Structural expansion, exterior hull modification, new rooms/decks and any work creating hull breaches can run **only while landed or docked**, including while the game is closed. Permission is based on *physical consequences*, not the category name.
- Interior objects and ordinary eligible module installations may build while flying or coasting, including verified offline progression, when their blueprint's safety contract allows it.
- **Takeoff must be refused** with an actionable cockpit message while external construction is running, unsecured or otherwise compromises flight-ready hull state. Extend current ramp/door takeoff validation; do not automatically cancel projects.
- The player can request **immediate Stop** on exterior building. Construction pauses, retains progress and materials, and secures/retracts temporary construction apparatus **where physically possible**. A stopped-secured external project may permit takeoff; stopped-unsecured must still block it. Cancelling applies proportional physical refund and clears the unfinished work only when the resulting hull is safe.
- Exterior construction is stopped throughout flight and **must not automatically restart after landing**. Unexpectedly airborne ships stop exterior work; do not freeze the ship to mask physics.
- Final sealed boundary/collision changes must be validated rather than pretending half-built hull sections are airtight. Current gameplay lacks actual aerodynamics, pressure and decompression; this is a flight-readiness invariant anticipating them, not a claim they ship.
- Construction proceeds during unattended play. On game close, eligible work advances using the approved real-time offline model in chronological order with real materials, available Core power and collisions/damage/state boundaries. Do not progress during an in-game pause. Do not grant work after energy runs out, ship takes off or a project becomes unsafe. Resume only after authoritative ship/world/offline event verification.

## Module installation and upgrades

- Ordinary modules (including the future **Spatial Jump Drive**) are real, installed, persistent hardware with blueprint-defined physical volume, mounting, power/safety checks, inputs and commissioning. A multi-cell module can be placed through one placeholder.
- **Placing a module-upgrade placeholder and prefeeding materials leave the current module functional.** Once all requirements are met, automatic upgrade begins and the target module is **fully disabled for its entire upgrade**, regardless of individual construction stages. Pausing, power loss or cancelling never magically restores the old module.
- At successful build completion, perform automatic commissioning to validate actual installed components, mass, connections and clearance; capabilities only become available once validation succeeds. Failed commissioning leaves module unusable with a specific recoverable fault. Finishing an upgrade never automatically fires thrusters or executes a jump.
- Module-specific safety validation governs whether each upgrade may run in flight. Hull-affecting changes require landed/docked status. Critical systems must not enter unsafe states; in-flight existing ship inertial motion is not magically stopped on loss of controls.
- For module-upgrade cancellation before completion, use the agreed construction progress-based proportional material return for the unfinished project; the target module's real state may need a separate physical repair or restoration project. This is not a free rollback.
- The future Jump Drive remains a later installation, not present in the starting Scout. It is powered by ordinary Core joules when operated, begins at N=20%, and has its own canonical design; these construction changes do not alter jump physics.
- The **Energy Core is explicitly exempt** from normal module upgrade sites, timed assembly, powered work, shutdown, and commissioning wait.

## Special instant Energy Core upgrades (approved exception)

- The Core keeps its permanent identity as the ship's irreducible recovery anchor.
- Player selects a Core upgrade and **carries physical required materials directly to the Core one at a time**. Upgrade resource acceptance must be **distinct from the existing fuel intake**, which consumes fragments for energy (even during blackout).
- When all required inputs are provided, the player may apply the upgrade **instantly**; no separate construction placeholder, worksite, build timer, stages, charged portable battery, external construction feed, Core shutdown or upgrade energy cost.
- Validate required materials and blueprint atomically; successful upgrade consumes exact accepted input material once and updates approved parameters, while failed application changes nothing. Upgrade can work with an empty Core battery. Pending contributed materials must be accounted for and retrievable before application.
- Preserve existing Core charge and valid reservations; increasing charge capacity **does not fill** the new capacity or grant joules. Keep Core identity and ship mass/power ledgers consistent.
- Core upgrade tiers, recipe quantities and exact improved effects are **not yet decided**. Don't mistake illustrative resource numbers for approvals.
- Earlier proposals for independently powered multi-stage **Core upgrades**, emergency construction-power reserves and time-consuming Core commissioning are **superseded and not requirements**. Do not change ordinary blackout throttle behavior incidentally.

## Damage, destruction and recovery

- Construction placeholders/temporary assembly can be damaged and stopped; do not delete every surviving material or structure merely because temporary site equipment was destroyed.
- Finished hull cells and physical modules have durable identity, can be damaged independently and can detach from the ship in combat; normal player construction may not deliberately disconnect remaining structure.
- Physical salvage, repair, module recovery and project continuation must respect actual damage and stable identities. Intentional placeholder cancellation always uses the **simple proportional refund** instead of a stage salvage minigame.
- A surviving owner may recover a damaged/destroyed ship from its persistent Core as the broader ship design already specifies. Never remove/replace that Core identity through instant upgrades.
- Cargo and non-abstract world objects remain retrievable based on physical access; no magic protections.

## Delivery boundaries and existing implementation

This is a design contract for future implementation, **not a claim that current Rust code has construction, habitation, a starter cargo room, advanced hull damage, full offline construction or the Jump Drive**. The existing implemented ship uses authored anchors, cockpit, Core fueling, Fabricator, Archive, ramp, cabin gravity, physical fragments and mass accounting; existing files and native tests continue to describe that implementation until changed.

The approved first reference slice is **a completely usable starter Scout**, not a beauty-only Blender render: cockpit, working Core/Fabricator/Archive, dedicated physical cargo, interior door/ramp and integrated habitation, with reliable HUD/UI/lighting/sound feedback. GLB budgets remain necessary. Cloud wgpu routes establish automated integration; #561 owns human spatial/presentation review, #557 actual platform experiments and #563 hardware budgets. M1 access is not a routine completion prerequisite.

## Structural foundation delivered (#541)

`salimon_ship::structure` now provides the portable 3 m Core-rooted graph,
room/equipment footprint metadata, safe removal, damage-component handoffs,
explicit baseline/additional mass and domain schema 1. See the
[owned architecture](../client/ship/architecture.md#portable-structural-foundation-541).
This is an independent domain boundary, not playable construction, a redesigned
starter ship, generated colliders or a whole-game save migration. Later delivery
boundaries above remain future work.

## Implementation issues (2026-10-22)

**Starter Scout:** [#538](https://github.com/mr-exception/salimon/issues/538) layout/cargo/habitation envelope → [#539](https://github.com/mr-exception/salimon/issues/539) cargo mechanics and door; [#540](https://github.com/mr-exception/salimon/issues/540) habitation fixtures.

**Modular construction:** [#541](https://github.com/mr-exception/salimon/issues/541) 3D structure → [#542](https://github.com/mr-exception/salimon/issues/542) placeholders → [#543](https://github.com/mr-exception/salimon/issues/543) resource delivery/proportional refunds → [#544](https://github.com/mr-exception/salimon/issues/544) automatic progress → [#545](https://github.com/mr-exception/salimon/issues/545) multi-cell room/deck geometry → [#546](https://github.com/mr-exception/salimon/issues/546) takeoff interlocks; [#547](https://github.com/mr-exception/salimon/issues/547) module installation/upgrades; [#549](https://github.com/mr-exception/salimon/issues/549) damage/salvage; [#550](https://github.com/mr-exception/salimon/issues/550) offline/persistence. Independent exception: [#548](https://github.com/mr-exception/salimon/issues/548) instant Core upgrades. Cloud integration gate: [#551](https://github.com/mr-exception/salimon/issues/551).

Related existing work includes asset modularization #95, Scout cloud verification #489/#502 and human presentation review #561, and universe offline timed systems #433. Do not create duplicate issues or imply these existing tickets are complete.

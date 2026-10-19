# Project Phases

> Migrated from the Salimon project documents on 2026-09-29. For current task status and dependencies, use [GitHub issues](https://github.com/mr-exception/salimon/issues).

> The roadmap is intentionally incremental. Exact phases will be finalized after the first design questions are answered.
## Phase rules
Every phase should:
- Have **one primary playable goal**.
- Be small enough to reason about and finish without bundling many unrelated systems.
- Have explicit **entry assumptions** and **completion criteria**.
- Contain a limited set of tasks linked to the GitHub issues.
- End with something that can be run, demonstrated, or tested.
- Avoid implementing future-scale complexity before it is required.
## Phase template
### Phase N — Name
**Goal:** What becomes possible for the player after this phase?
**Scope:** The smallest systems required for that goal.
**Out of scope:** Related features intentionally deferred.
**Completion criteria:** Observable conditions proving the phase is finished.
**Tasks:** Managed in GitHub issues.
**Notes / decisions:** Links to relevant specifications.
## Phase 0 baseline and evaluation

[Phase 0 — Technical Feasibility Showcase](phase-0-technical-feasibility.md)
established the custom native Rust + `winit` + `wgpu` client, compressed static
Solar System, first-person traversal, direct-speed flight, uncancellable landing/
takeoff assists and diagnostics. Its original exclusions bound that baseline;
they do not prohibit systems added afterward.

The [2026-09-15 evaluation](phase-0-evaluation.md) records **GO WITH REVISIONS**
for its evaluated revision. It does not establish a hard 60 FPS floor or certify
later revisions. Preserve its machine-specific evidence and limitations.

## Current post-baseline prototype

The client remains native-first and client-only. Local build scripts target macOS,
Windows and Debian-based Linux. PR CI verifies an Ubuntu source build;
macOS is the reference playable/performance target. Web/WASM portability remains future work. No new numbered phase or formal
phase closure is implied by this implementation summary.

Implemented beyond the navigation baseline:

- Open-space and nearby-body EVA with ship/frame transitions and airlock gates.
- Deterministic planetary iron, silicate and water-ice deposits, local streaming,
  aimed extraction and physical fragment output.
- One-object carrying, pickup/release and transfer between surface and ship.
- Fragment gravity, ejection/release motion, contact and piling through the
  portable `salimon-physics` crate and runtime environment/frame adapters.
- Blender-authored scout, deposits, fragments and mining tool with checked-in
  exports and generated scout spatial contracts.
- Native build staging and declarative gameplay/evidence automation.

Modified deposits and physical fragments survive local streaming **in memory for
the current session**. This is not disk/backend persistence. Production survival, NPCs, networking, multiplayer, economy and
orbital simulation remain deferred. `core/` remains a documentation-only backend
boundary. The [technical architecture](technical-architecture.md) and
[feature maintenance map](maintenance-map.md) define implemented owners/contracts.

## Current work and later roadmap

[GitHub issues](https://github.com/mr-exception/salimon/issues) and their explicit
blockers/acceptance criteria govern execution; historical Task N numbering is
not a current queue. Later small phases can extend physical cargo and ship
interaction, then survival/ship systems, persistence/backend authority, ship
modification, NPC interaction/economy and shared-universe capabilities. Exact
ordering and phase completion criteria require their own decisions/issues.

## Realistic flight/Core wave (#185–#197)

The original baseline's direct-speed flight and no-energy exclusions are historical
and explicitly superseded by inertial propulsion, signed throttle, event-driven
mass, Core storage/conversion/reservations, power-dependent cabin gravity,
blackout controls, swept crashes/recovery and physical instrumentation. This does
not close or renumber Phase 0. See the [accepted gameplay decisions](spaceship-building-and-survival.md#accepted-propulsion-and-core-review-2026-10-14).

Review decisions **9, 58, 63, 72, 84** constrain scope/workflow: ship gravity/orbits,
manual power management and detailed inertial cabin forces remain OUT OF SCOPE;
implementation uses small dependency-aware issues. The earlier instruction to
hold issue creation was superseded by the later explicit request to create the
GitHub queue. Update existing owned documents only, without a new central spec.
Future thruster/landing upgrades and module loads remain documentation-only.
Cloud unit/E2E/profiling work is #195/#196; Apple M1 physical signoff is isolated
in native-labeled #198–#200 and is not established by cloud software rendering.


Fabricator crafting is newly authorized by the [accepted five-round contracts](spaceship-building-and-survival.md#accepted-fabricator-contracts-2026-10-19-336). Implementation is staged: #336 contracts, #337 raw materials, #338 portable domain, then separate runtime/UI integration. Survival effects, disk persistence and playable progression remain deferred.


### Fabricator portable foundation (#336–#338)

Approved contracts, real organic biomass/medicinal raw resources and the portable
recipe/storage/FIFO/tray domain are delivered. Crafting remains unconnected to
native interactions pending #339–#347; no playable-wave completion or phase
renumbering is implied. See [fabrication architecture](../client/fabrication/architecture.md).

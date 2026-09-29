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
## Committed Phase 0
[Phase 0 — Technical Feasibility Showcase](phase-0-technical-feasibility.md)
Phase 0 is a **technical feasibility showcase**: a native macOS, fully client-side, compressed Solar System test world using the custom Rust + `wgpu` runtime. It includes first-person ship/interior traversal, direct-speed ship orientation and thruster control, interplanetary travel, seamless planetary approach, assisted landing/takeoff, and performance diagnostics. The Phase 0 ship must be at least 2× the original Task 7 linear scale, provide useful exterior visibility through cockpit windows, and use correctly directed horizontal mouse look. It intentionally excludes gameplay systems such as survival, energy/fuel, inventory, NPCs, multiplayer, combat, economy, persistence, and backend behavior.
The repository must already contain both `client/` and `core/` top-level boundaries; `core/` remains intentionally empty during this phase.
### Current Phase 0 execution priorities
The ordered [Related page](https://github.com/mr-exception/salimon/issues) database remains authoritative. Tasks 1–7 are the completed foundation. The next priorities are Task 8 mouse-look correction, Task 9 cockpit windows, and Task 10 ship scaling. The completed first-person/walkable-shell work is now Task 11; flight controls, assisted landing/takeoff, benchmarking, and final evaluation continue as Tasks 12–15.
## Later roadmap shape
After Phase 0 is validated, later phases can grow through player embodiment/ship interior interaction → resources and physical cargo → survival/ship systems → persistence/backend authority → ship modification → NPC interaction/economy → multiplayer/shared universe capabilities. Exact ordering remains subject to design decisions and Phase 0 findings.

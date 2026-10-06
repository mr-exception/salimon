# Product Requirements

> Migrated from the Salimon project documents on 2026-09-29. For current task status and dependencies, use [GitHub issues](https://github.com/mr-exception/salimon/issues).

## Product direction
- The game is **3D** and **native-first**, using a custom Rust + `winit` + `wgpu` client. Native builds target macOS, Windows and Debian-based Linux; macOS is the reference playable/performance target. Future web/WASM portability is an architectural goal, not a shipped browser client.
- The primary experience combines **space exploration** and **survival**.
- The world should follow a **realistic / believable direction** rather than arcade-only rules.
- Players can own, upgrade, customize, and eventually build or structurally modify spaceships.
- Players can collect resources from **asteroids, space, and planets**.
- Players can interact with **NPCs** in the world.
- The design may include **multiplayer elements**, including players entering each other's spaceships and traveling together.
- The project is developed through **small, reachable phases**.
- Every phase must have a clear goal and a bounded set of tasks.
## Current playable implementation

The client-only prototype extends the Phase 0 navigation showcase with surface
resource generation/mining, physical fragments, one-object carrying, ship/surface
transfer, fragment gravity/contact and airlock/EVA. Resource changes survive local
streaming in memory for the current session. Survival, crafting, NPCs, multiplayer,
disk/backend persistence and production energy management remain deferred.
See [Project Phases](project-phases.md) for current scope and
[Technical Architecture](technical-architecture.md) for implemented ownership.

## Planning rule
A phase should produce a demonstrable improvement to the playable application. Avoid phases that are only broad infrastructure projects unless that infrastructure is required to unlock the next playable goal.
## Engineering quality rule
- Development must include automated testing continuously as modules and packages are built.
- Every module/package should have focused unit tests for the logic it owns.
- Bug fixes and fragile behavior should be protected with regression tests so previously solved problems do not silently return.
- Cross-module contracts and important integrations should have contract/integration tests where appropriate.
- Test creation and test maintenance are part of the implementation task itself, not a separate optional cleanup phase.
- A task or module should not be treated as complete if its important behavior cannot be automatically verified.
## Documentation flow
1. Resolve open design questions in **Project Q&A**.
2. Update the relevant system specification.
3. Define or refine the next small phase.
4. Break that phase into tasks in **GitHub issues**.
5. Complete and validate the phase before expanding scope.

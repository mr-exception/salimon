# Phase 0 — Technical Feasibility Showcase

> Migrated from the Salimon project documents on 2026-09-29. For current task status and dependencies, use [GitHub issues](https://github.com/mr-exception/salimon/issues).

> **Purpose:** Prove that Salimon's custom native Rust runtime, renderer, large-scale scene approach, first-person ship experience, and seamless space-to-surface presentation are technically viable before adding real gameplay systems.
## Primary goal
Create a small, fully client-side playable Solar System showcase where the player can walk inside a simple spaceship, control it from first person, freely fly through a deliberately compressed Solar System, approach planets, use assisted landing, leave the ship, walk outside, and take off again.
This phase is a **technical feasibility, graphics, performance, and architecture validation**. It is not intended to implement survival, economy, multiplayer, backend simulation, or production gameplay.
## Repository shell
The repository should reflect the long-term project boundary even though Phase 0 is client-only:
```plain text
salimon/
├── core/
│   └── README.md
└── client/
    ├── runtime/
    ├── renderer/
    ├── world/
    ├── character/
    ├── ship/
    ├── platform/
    ├── assets/
    └── diagnostics/
```
- `core/` represents the future authoritative backend/runtime service and remains intentionally empty in Phase 0 except for minimal repository documentation/placeholders.
- `client/` contains the entire Phase 0 implementation.
- The client must avoid temporary coupling that would prevent future extraction of portable/shared modules or backend authority.
## Included scope
### Native macOS application
- Native macOS is the first implementation target.
- Custom Rust runtime; no full game engine.
- `wgpu`-based custom renderer.
- Keyboard and mouse only for Phase 0.
- Reference performance machine: **Apple M1 iMac** (`iMac21,1`, model `Z12X002L9GR/A`), with 8 CPU cores, 8 integrated GPU cores, and 16 GB unified memory.
- Performance target: **never below 60 FPS at a fixed internal and output resolution of 1920×1080** across Phase 0 benchmark scenarios on the reference Apple M1 iMac.
- Do not rely on upscaling, reduced internal resolution, shadows, or other expensive visual effects in Phase 0. Performance should come from simple rendering, sensible LOD/scene management, and inexpensive materials/effects.
### Starting state
- The showcase starts with the **ship landed on Earth** and the player already inside it.
- Boarding from outside is not part of the startup flow.
- The player may immediately sit in the cockpit and begin flying, or open the ship door and walk outside before taking off.
### Compressed Solar System showcase
- Include the **Sun, Mercury, Venus, Earth, Mars, and Earth's Moon**.
- Exclude Jupiter, Saturn, Uranus, Neptune, Pluto, and other dwarf planets from Phase 0.
- All celestial bodies are fully **static** in Phase 0: no orbital motion and no axial rotation.
- Distances and body sizes are intentionally compressed for a small, easily traversable showcase rather than derived from a fixed real-world scale ratio.
- Target approximately **24 seconds of travel from Earth to Mars at 100% ship speed**, using the Phase 0 maximum of **2,500 km/s**.
- Planet/Moon sizes may be reduced substantially for easy travel, but should still have clearly visible relative differences—for example Earth should read as larger than the Moon even though the ratios are not realistic.
- The compressed layout should allow nearby planetary bodies to be visually apparent at useful times—for example Mars may be visible from Earth depending on scene arrangement.
- Keep solid-body placement far enough apart that their assisted-landing activation zones never overlap. With Phase 0 landing range defined as `0.15 × body radius` above each surface, the layout must guarantee that the ship can only be inside one solid body's landing range at a time.
- The Sun is a **visual/light source only** in Phase 0 and has no solid collision body; the ship may pass through its visual volume if reached.
- Planet surfaces are simple textured spheres in Phase 0; no production terrain/elevation system is required.
- Use simple custom/generic textures rather than recognizable real-world planetary imagery. The purpose is renderer/scale validation, not accurate planetary surface reproduction.
- Do not implement atmospheric glow/haze or cloud rendering in Phase 0. Clouds are not currently planned for later phases either.
- World data and all simulation/state are entirely client-side.
- The scene is continuous: no loading screen between interplanetary space and planetary approach/surface transition.
### First-person character and ship
- The player is always presented in **first person**, including while controlling the ship.
- The Phase 0 ship has a modeled, walkable interior.
- The scout retains a **9.20 m-wide cabin deck**, a **4.00 m height**, and human-scale interior circulation. Keep side and aft observation windows and a low cockpit seat obstruction for standing views. Use warm ivory/copper/terracotta/teal materials, inexpensive emissive lamps, a central modeled Energy Core with walking routes around it, and detailed lightweight aft thrusters. The Core is the energy-storage battery and recoverable heart described in Project Vision; Phase 0 represents it visually without energy-management simulation. Exterior geometry, interior objects, collision proxies, interaction markers, landed placement, and camera assumptions must be updated together. The player is 1.80 m tall with a 1.75 m eye height. The current exterior dimensions are recorded in the authored ship manifests after export.
- The cockpit must include modeled windows with useful, unobstructed exterior visibility from both the seated cockpit viewpoint and the walkable interior. Window treatment must remain inexpensive enough for the Phase 0 performance target.
- The current scout revision removes the protruding port cargo module, shortens the nose and cockpit glazing, and combines the cockpit instruments on one centered console with three physical monitors. Keep clear walking routes on both sides into the cockpit nose and preserve the seated forward and downward exterior views. Loose fragments may be dropped on the main-cabin deck; the ship has no dedicated cargo room or storage volume.
- The player can walk inside the ship.
- The ship provides artificial gravity toward its local floor for the player while inside. Ship gravity uses the **same fixed strength** as planetary walking gravity and completely overrides planetary radial gravity while the player is inside the ship, including when the ship is landed. Phase 0 does not simulate a physically realistic gravity generator.
- The player can use/open the ship door and walk outside after landing.
- The cockpit is the ship-control position. The player physically sits in the cockpit to enter ship-control mode, then uses keyboard/mouse controls.
- A simple greybox-scale interior is sufficient: cockpit, one small room/corridor, and an exit door.
- Use a **polished greybox/material-demo** presentation rather than committing to the final art direction. Add textures/material variation, basic lighting, and subtle edge/detail outlines on selected geometry/materials. Avoid a global toon/comic outline.
- Character movement inside the ship may use simplified invisible collision geometry rather than matching every visible mesh surface exactly, prioritizing stable movement and performance.
- The ship door rotates outward and upward about its top hinge over 700 ms. Passage opens once the leaf has nearly cleared the aperture, and takeoff waits until it is fully closed.
- Airlock access (#35) extends the original landed-only rule: `E` may open/close the door while landed or while flying outside the shared inclusive 3,000,000 m nearby-body threshold. It stays locked during nearby-body flight and assisted landing/takeoff, with `Door locked while in flight` feedback. A stationary open-space exit and re-entry use a separate inertial character state. EVA inherits exit velocity (#37); within the player's inclusive 3,000,000 m surface distance to a solid body, nearby EVA adds fixed radial gravity without snapping position (#36), then adopts surface walking on contact. No pressure, oxygen, decompression, or suit-damage simulation is included.
- The exterior ship model should also avoid being bland: keep it modest in scope, but include enough form, surface detail, materials, and silhouette work to showcase what the AI-driven 3D asset workflow can produce without turning Phase 0 into a major art task.
- The ship itself must be **custom-created for Salimon**, not a ready-made ship asset.
- Use an **external AI-assisted 3D modeling workflow (Blender or equivalent) and export glTF/GLB** for the Rust client rather than hard-coding the full ship geometry in Rust.
- Free/public assets may be used for **textures/materials and small mesh components/props** such as controls, bolts, seats, screens, or similar details. They are temporary Phase 0 support assets and may be replaced by custom models in later phases.
- Prefer public supporting assets that **require no attribution** where practical. Attribution-required/permissive assets may still be used when useful, but their source/license must be documented.
- Commit editable **Blender/source 3D files** to the repository alongside exported game-ready glTF/GLB assets so AI agents can modify the ship directly in later iterations.
### Ship flight / navigation prototype
- While seated in the cockpit, **W pitches the nose up, S pitches the nose down, and A/D yaw left/right**.
- **Left/Right Arrow** roll the ship left/right around its longitudinal X axis.
- Pitch, yaw, and roll rotate **continuously while their keys are held** and stop when released. Phase 0 uses a fixed rotation rate of **5°/second** for pitch, yaw, and roll.
- Apply a very small input smoothing/ramp when steering starts and stops for visual feel. This is presentation smoothing only, not rotational inertia.
- The player's first-person view remains independent free-look while seated; looking around does not steer the ship, the view **does not auto-recenter**, and Phase 0 imposes **no camera-angle limit**.
- **Up/Down Arrow** change thruster power by **1 percentage point per key press**. The selected value persists until the player changes it.
- Thruster power is a **continuous 0–100% direct speed control**, not acceleration/inertial thrust. The selected percentage maps directly to current ship speed.
- The player may press `E` to leave cockpit-control mode while the ship is flying. The ship continues flying at its currently selected thruster/speed setting and current orientation until the player returns to the cockpit and changes control input.
- At 100% speed, the compressed Earth→Mars trip should take roughly **2 minutes**.
- Flight is intentionally simplified: **no planetary/Sun gravity, orbital mechanics, damage physics, or high-speed collision simulation** in Phase 0.
- Solid celestial bodies use simple non-damage collision boundaries. If the ship penetrates/intersects a boundary, push it back to the nearest valid position rather than simulating bounce, slide, or damage.
- The player can travel between the included celestial bodies.
### Assisted landing
- Phase 0 includes an **assisted landing mode** rather than requiring a realistic manual landing simulation.
- When the ship enters landing range of a solid body, show **Press L to land** on the cockpit monitor while the player is actively controlling the ship. Landing range is defined as **0.15 × the target body's radius above its surface**. The prompt/assist is available regardless of the ship's current speed or orientation; Phase 0 does not require the player to slow down or align manually before engaging landing assist. Assisted landing can only be triggered while the player is actively controlling the ship from the cockpit.
- Assisted landing is not limited to predefined landing sites. Once within landing range, the player can initiate landing based on the current approach location and land at the corresponding valid point on the planet/Moon surface.
- Activating landing assist immediately takes authority over the ship regardless of its prior speed/orientation, auto-aligns it to the local surface, reduces/normalizes motion as needed, descends at a controlled speed, and transitions into a stable landed state. Once assisted landing starts, it **cannot be cancelled in Phase 0** and must run to completion.
- Surface walking uses a **single fixed gravity strength** on every solid Phase 0 body rather than planet-specific gravity values. This is the same fixed strength used by ship artificial gravity. For full-sphere walking, gravity direction follows the local surface normal toward the body's center while its strength remains constant everywhere.
- The player can walk around the **entire spherical surface** of each supported solid body; Phase 0 does not simulate different gravitational strengths for Mercury, Venus, Earth, Mars, or the Moon.
- Assisted landing and assisted takeoff must work on **every solid Phase 0 body: Mercury, Venus, Earth, Mars, and the Moon**.
- Takeoff is also assisted in Phase 0 and uses the **same contextual key/action as landing** (for example `L`). While landed and controlling the cockpit, the cockpit monitor shows the takeoff action. Assisted takeoff can only be triggered while the player is actively controlling the ship from the cockpit. If the exit door is open, takeoff is blocked until the player closes it; the system does not auto-close the door and shows `Close door before takeoff` on the cockpit monitor. Assisted takeoff lifts the ship away from the surface and clears the landing zone before returning normal ship control. After an assisted landing or takeoff sequence has started, the automatic sequence continues even if the player leaves cockpit-control mode and walks around inside the ship. **Neither assisted landing nor assisted takeoff can be cancelled in Phase 0; once started, the sequence must finish.** A more complete/manual takeoff model is deferred to Phase 1.
- The purpose is to validate visual scale transition, surface approach, character exit, global spherical walking, and takeoff—not realistic landing mechanics.
### Graphics and rendering validation
The showcase should exercise the renderer enough to answer whether the custom runtime is viable:
- very-large-range coordinate/camera strategy;
- planet rendering from distant view through close approach;
- simple Sun lighting;
- **no dynamic shadows** in Phase 0;
- basic materials and planetary textures;
- no atmospheric rendering or cloud layer;
- depth/precision stability;
- scalable meshes/LOD where useful;
- first-person interior rendering;
- transitions between ship interior, exterior space, and planetary surface;
- stable camera behavior;
- GPU/CPU profiling instrumentation.
### Controls and interaction
- **E** is the general interaction key for entering/leaving cockpit control mode and opening/closing the ship door. Contextual flight/door messages such as `Press L to land`, `Press L to take off`, `Close door before takeoff`, and `Door locked while in flight` are presented through the **cockpit monitor/message system**, not as a floating screen overlay.
- Entering/leaving the cockpit seat is **instant** in Phase 0; no seat animation or transition is required.
- First-person walking uses **WASD + mouse look + Space to jump**.
- Mouse look uses direct horizontal mapping: moving the mouse left turns the view left and moving it right turns the view right, including independent cockpit free-look. The input mapping must have focused regression coverage.
- When the player crosses the ship doorway while landed, transition between ship-floor gravity and planet-radial gravity with a **0.25-second blend** rather than an instantaneous switch, to avoid a visible camera/character snap.
- Sprint and crouch are out of scope for Phase 0.
- Cockpit speed readout should automatically use practical metric units such as **m/s, km/s, and Mm/s** depending on magnitude.
- Normal flight information belongs on cockpit displays rather than a floating HUD.
### Diagnostics
Include an optional engineering overlay showing at minimum:
- FPS and frame time;
- CPU frame/update time where measurable;
- GPU frame time where supported;
- visible/rendered object count;
- draw-call count where available;
- current player/ship position;
- current ship speed and 0–100% thruster/speed percentage;
- distance to selected/nearby bodies;
- normal gameplay flight information such as current speed and speed percentage should be presented on **cockpit monitors**, not as a floating screen HUD;
- memory-related metrics where practical.
## Explicitly out of scope
Do **not** implement these in Phase 0:
- Backend functionality or networking
- Multiplayer
- Persistence/server snapshots
- NPCs
- Quests/story
- Resources/mining
- Inventory/cargo
- Crafting
- Fuel
- Energy/Core management
- Oxygen or life support
- Food/water/sleep/stamina
- Damage or durability
- Repair/maintenance
- Combat/weapons
- Ship construction/customization
- Economy/trading
- Planetary/Sun gravity and orbital mechanics
- Real astronomical distances
- Moving planetary orbits
- High-speed crash/damage physics
- Production terrain/elevation
- Production-quality planet content
- Atmospheric glow/haze
- Cloud rendering
- Dynamic shadows and expensive post-processing/effects
## Architecture constraints
- No Unity, Unreal, Godot, Bevy engine, or other full game engine.
- Rust is the primary implementation language.
- Renderer remains isolated from simulation/game-domain logic.
- Character, ship, world, renderer, and platform concerns should remain separate modules even in this small phase.
- Platform-specific macOS concerns stay behind platform/runtime boundaries where practical.
- Code should be organized so future Windows and web targets can reuse portable runtime/domain modules.
- The `core/` shell must exist from the beginning, but Phase 0 must not spend effort implementing backend behavior.
- Each Phase 0 module/crate/package must gain focused unit tests as its behavior is implemented.
- Bug fixes and precision/behavior edge cases must add regression tests whenever practical.
- Cross-module boundaries should use contract or integration tests where useful, especially for runtime/renderer/world/character/ship interactions.
- Tests are part of the task definition of done and should run automatically in the normal development/CI workflow.
## Completion criteria
Phase 0 is complete when all of the following are demonstrably true:
1. The native macOS application launches from the repository with a documented development workflow.
2. The repository has clear `client/` and intentionally empty `core/` boundaries.
3. The scene contains the Sun, Mercury, Venus, Earth, Mars, and Earth's Moon in a compressed, static layout.
4. The player can walk in first person inside a ship scaled to at least 2× the Task 7 baseline, use correctly directed mouse look, and see the exterior through usable cockpit windows.
5. The player can sit in the cockpit and freely fly the ship using direction controls and direct-speed thruster control.
6. The player can continuously travel between multiple planets without a loading screen.
7. Mercury, Venus, Earth, Mars, and the Moon each support prompted assisted landing, opening/leaving the ship, full-sphere first-person surface walking with simple radial gravity, re-entry, and assisted takeoff.
8. Planet rendering remains visually stable from distant space through close surface approach without unacceptable precision artifacts.
9. The application exposes diagnostics sufficient to identify CPU/GPU/frame bottlenecks.
10. Every defined Phase 0 benchmark scenario sustains **at least 60 FPS at fixed 1920×1080 internal/output resolution on the reference Apple M1 iMac**, without upscaling or dynamic-resolution fallback.
11. A Phase 0 evaluation records measured performance, visual/precision limitations, architectural problems, and a **go / revise / stop** recommendation.
12. Phase 0 modules/packages have automated unit/regression coverage for their important owned behavior, and the project test suite passes as part of validation.
## What Phase 0 is meant to answer
- Can custom Rust + `wgpu` render this style of large continuous scene efficiently?
- Can one coordinate/rendering architecture handle planetary-scale approach and first-person interior/surface rendering?
- Can the reference Apple M1 iMac sustain the required presentation quality without dropping below 60 FPS at 1920×1080?
- Can first-person walking, cockpit control, free space flight, assisted landing, surface exit, and takeoff work cleanly without a full game engine?
- Are the project/module boundaries practical for autonomous AI implementation?
- Which renderer, LOD, precision, and scene-management techniques must be established before Phase 1?
## Deliberate simplifications
- Celestial bodies do not orbit in Phase 0.
- Spaceflight does not simulate gravity/orbital mechanics.
- Planets use textured spherical surfaces rather than real terrain.
- Landing is assisted.
- No survival or resource systems exist.
- Content can be simple/greyboxed where visual fidelity is not required for the technical test.
## Open decisions
No blocking Phase 0 implementation decisions remain. Any future ambiguity discovered during implementation should be added to [Project Q&A](project-qa.md) before changing intended behavior.
## Phase 0 coordinate/camera implementation decision
Phase 0 validates the project's large-range rendering approach using the canonical coordinate rules documented in Technical Architecture & AI Maintenance.
- Use one right-handed coordinate space with `+Y` up and meters as the canonical unit for the Phase 0 portable world representation.
- Keep authoritative world/camera positions CPU-side in `f64`; produce GPU positions by subtracting the camera in `f64` and converting only the camera-relative result to `f32`.
- Use `Depth32Float` infinite-far reverse-Z with explicit configurable near plane.
- Coordinate/camera prototype values such as a `120 Mm` to `12 m` approach range, `60°` vertical FOV, `0.05 m` near plane, large anchor coordinates, proxy geometry, colors, and transition timings are validation defaults only. They must not be treated as final Solar System layout, celestial content, ship/camera behavior, or tuning.
- Phase 0 may use CPU camera rebasing for this small showcase. Future large meshes and universe-scale content must move toward body-local/patch-local geometry and the project's spatial hierarchy.
## Task 7 implementation note — custom Phase 0 ship asset
Task 7 was completed on 2026-09-10 in commit `042f7ae`. The repository now contains the custom original **Salimon Phase 0 Scout** under `client/assets/ship/`: deterministic editable source, Blender-importable glTF, a self-contained GLB, an original procedural texture, licensing documentation, collision proxies, and cockpit/door/player interaction markers.
The asset is a polished greybox/material demo with a designed exterior silhouette and a 3.1 m-wide walkable cockpit/cabin/corridor. It uses 620 triangles, 41 primitives, nine reusable materials, and a 77,032-byte GLB with no third-party asset dependencies. Runtime character/ship integration was completed in the task now ordered as Task 11, and measured M1 1920×1080 evidence is now Task 14 scope.
## Current Phase 0 task plan
The ordered [Related page](https://github.com/mr-exception/salimon/issues) database is authoritative for status and execution order. After Tasks 1–7 established the repository, runtime, diagnostics, coordinate strategy, Solar System scene, renderer, and first ship asset, hands-on playtesting inserted these immediate priorities:
1. **Task 8 —** [Fix inverted horizontal mouse look](https://github.com/mr-exception/salimon/issues/67).
2. **Task 9 —** [Add cockpit windows with exterior visibility](https://github.com/mr-exception/salimon/issues/68).
3. **Task 10 —** [Resize spaceship for player-scale proportions](https://github.com/mr-exception/salimon/issues/69).
The previously completed first-person character and walkable ship shell is now Task 11. Direct-speed flight controls, assisted landing/takeoff, reference Apple M1 iMac benchmarking, and the go/revise/stop evaluation follow as Tasks 12–15.
## Phase 0 implementation summary — 2026-09-14
> **Implementation status:** Core Phase 0 build work is complete. Thirteen implementation tasks are finished. Formal phase closure is still pending the interactive M1 benchmark and final go/revise/stop evaluation.
- Established the Phase 0 repository/module boundaries and native macOS Rust runtime with an isolated `wgpu` renderer.
- Added diagnostics/profiling, large-scale coordinate and camera handling, the compressed static Solar System, and scalable planet rendering.
- Built a custom editable/exported Salimon spaceship, refined cockpit visibility and player-scale interior proportions, and added first-person walking and interaction inside/outside the landed ship.
- Implemented direct-speed cockpit flight controls, independent free-look, assisted landing/takeoff, surface transitions, full-sphere walking on supported solid bodies, and focused regression fixes/tests.
- Automated quality gates currently pass formatting, Clippy with warnings denied, all 119 tests, and the optimized release build.
### Remaining closure work
1. Run the seven-scenario interactive benchmark on the reference M1 iMac at fixed 1920×1080 and confirm the 60 FPS hard floor.
2. Record measured limitations/bottlenecks and complete the final go/revise/stop evaluation for the Rust/`wgpu` architecture.
Phase 0 should therefore be treated as **implementation-complete, but not formally closed** until these two closure items are finished.

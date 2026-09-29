# Game Systems Overview

> Migrated from the Salimon project documents on 2026-09-29. For current task status and dependencies, use [GitHub issues](https://github.com/mr-exception/salimon/issues).

## Purpose
This page is the high-level index of Salimon's major gameplay and simulation systems. Detailed behavior belongs in dedicated specifications as each area is refined.
## Confirmed system areas
1. **Exploration & navigation** — realistic-distance travel through a universe-scale world, seamless space-to-surface movement, long-term travel toward Absenat.
2. **Player character** — first-person walking, EVA, planetary exploration, physical interaction, health and survival state.
3. **Spaceship flight** — owner-controlled navigation, propulsion upgrades, physically located ship systems, large speed progression.
4. **Ship construction** — modular rooms/hull expansion while landed/docked plus interior module placement during flight.
5. **Ship simulation** — power/Core, propulsion, oxygen, pressure, thermal state, hull, sensors, weapons, damage, maintenance and environmental protection.
6. **Survival** — oxygen, food, water, temperature, sleep, health, radiation, pressure and stamina; permadeath resets the player's run.
7. **Physical inventory & cargo** — items/resources are physical world objects rather than abstract slots wherever possible.
8. **Mining & gathering** — asteroid fragmentation, planetary resource destruction/extraction, physical collection and transport.
9. **Processing & crafting** — refining raw resources, repair kits, survival supplies, construction materials, ammunition, equipment and modules.
10. **Research & progression** — technologies that improve propulsion, modules, weapons, tools and environmental survivability, allowing farther travel.
11. **World simulation** — real astronomical data where practical, realistic orbital motion, fictional fill content, sectors/snapshots and deterministic catch-up.
12. **Persistence** — persistent world changes including character location, ship, inventory, NPC state, economy, mined resources and partially modified objects.
13. **NPC simulation** — autonomous AI-backed NPCs with shared global memory, relationships, travel, factions, quests and free-form dialogue.
14. **Economy & factions** — local supply/demand, local pricing, trade, resource regeneration and faction control/influence.
15. **Multiplayer** — persistent shared world, natural encounters, ship boarding, cooperative operation and persistent physical player location.
16. **Combat** — ship combat and on-foot PvE/PvP with backend-authoritative outcomes.
17. **Networking & authority** — local state broadcasting, server validation, protected canonical world state and anti-cheat-sensitive actions.
18. **Web performance & streaming** — desktop WebGPU first, sector streaming, workers, WASM domain modules, LOD/asset streaming and IndexedDB/local caching where useful.
19. **Audio, UI & feedback** — first-person HUD, compass/home-ship marker, ship instrumentation, environmental feedback and physical interaction affordances.
## Design principle
Game systems should be independently understandable and versioned domains. Their public contracts, invariants, persistence rules and authority boundaries must be explicit so autonomous AI agents can safely change one area without reasoning over the entire codebase.
## Technical architecture
Implementation strategy is documented in [Technical Architecture & AI Maintenance](technical-architecture.md).
Key direction: **TypeScript browser host + WebGPU renderer + Web Workers + WASM gameplay/simulation domains + authoritative backend**, with typed contracts and event-driven communication between major systems.

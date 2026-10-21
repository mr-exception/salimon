# Project Vision

> Migrated from the Salimon project documents on 2026-09-29. For current task status and dependencies, use [GitHub issues](https://github.com/mr-exception/salimon/issues).

> **Salimon** is a realistic, native-first 3D space exploration and survival game set in a persistent shared universe. The player lives physically inside an evolving spaceship, travels across real and fictional space, gathers and processes resources, survives harsh environments, meets autonomous AI-driven NPCs and other players, and works toward the distant mystery of **Absenat**.
## Current delivery status

The shared universe and survival pillars below describe the long-term vision.
Today the playable application is a native, client-only Rust + `winit` + `wgpu`
prototype with navigation, walking/EVA, mining, carrying and fragment physics.
Native builds target macOS, Windows and Debian-based Linux; macOS remains the
reference playable/performance target. Web/WASM portability, backend persistence
and networking are future capabilities. See [Project Phases](project-phases.md).

## Approved future opening and narrative direction (2026-10-21)

The updated narrative begins with the traveler **waking outside a landed Scout on an unknown alien world**, with lost autobiographical memories but retained ship-operating skills and the knowledge that **Earth is home**. Earth's location/Solar System route is *not* known to the traveler and must be discovered and charted; seeking Earth and seeking Absenat are two independent, non-mandatory goals. Intact Earth-departure, Mars and Solar System exit logs precede recovered distress and warning leads. Whether the traveler actually entered an alleged wormhole is intentionally unknown. See [full narrative](narrative-and-lost-journey.md) and [Archive/navigation](archive-recovery-and-navigation.md).

The future Absenat gate opens onto the cosmic boundary between successive universes and a monumental endless-ring Hall with player inscriptions. The Hall is future **backend-only shared endgame** and has an absolute no-spoilers-before-first-completion rule; see [Absenat contract](absenat-and-hall.md). This paragraph is design, **not current implementation**. Historical prototype requirements are preserved only in explicitly historical Phase 0/evaluation documents, not as active future gameplay requirements.

## Narrative premise
An unknown entity teleports an object to Earth every 18 hours. Humanity calls it **the Core**: a battery capable of storing extremely large amounts of electrical energy for effectively unlimited time, solving Earth's energy crisis.
Whenever a Core appears, the entity leaves the same message:
> **Reach the Absenat, where everything started.**
Absenat is an extremely distant destination and the long-term narrative goal. Reaching it may take years of real travel time without sufficient technological progression.
## Core player fantasy
The player is an explorer, survivor, ship operator, engineer, scavenger, trader, and traveler living out of one persistent spaceship that grows into a personal mobile home.
The experience should feel mysterious and adventurous rather than structured as a linear campaign. The world is primarily sandbox-driven, while quests, discoveries, hostile encounters, anomalies, faction activity, and other events continually create direction.
## Primary gameplay loop
**Travel → collect resources → process/craft → maintain and upgrade ship/equipment → complete quests and encounters → travel farther.**
Typical quests may involve transporting items, traveling to remote locations, fighting NPCs, following clues, finding treasure, trading, or responding to dynamic events.
## Core pillars
- **Real universe direction:** known stars, planets, moons, and large astronomical structures should be represented at believable scale and motion; fictional systems and points of interest may fill gameplay gaps.
- **Physical survival:** oxygen, food, water, temperature, sleep, health, radiation, pressure, and stamina matter. Death is permanent for that run: the player starts fresh and loses the previous character's possessions and ship.
- **Spaceship as home:** every player owns one ship. It is the player's primary base, vehicle, progression platform, and cozy personal space.
- **Physical interaction:** first-person movement, walkable interiors, switches, doors, machinery, tools, and physical cargo. Containers are physical spaces, not abstract item lists.
- **Meaningful construction:** the ship grows through modular rooms and functional modules whose mass, location, durability, power needs, heat, structural implications, and performance matter.
- **Resource economy:** asteroid and planetary materials are physically gathered, transported, refined, crafted, traded, and used for survival, repair, construction, upgrades, ammunition, and research.
- **Living AI world:** NPCs are autonomous actors backed by AI models, with globally shared memory and world state across players. NPC actions, relationships, quests, alliances, hostility, factions, trading, and movement affect one common world.
- **Persistent shared multiplayer:** players encounter one another naturally in the universe, can walk into each other's ships, cooperate or fight, and remain where they physically are even after disconnect/reconnect.
- **Mystery and exploration:** Salimon should continually suggest that the universe contains unknown history, unexplained entities, anomalies, rare discoveries, and paths toward Absenat.
- **Native-first delivery:** macOS is the reference playable/performance target; native build scripts also target Windows and Debian-based Linux. Portable Rust domains preserve future web/WASM and server reuse without requiring a browser host or backend today.
## Distinctive identity
Salimon should differentiate itself through the combination of:
- a physically believable, extremely large shared universe;
- deep survival and ship-maintenance systems;
- a fully walkable and expandable spaceship as the player's home;
- physical rather than abstract cargo and resource handling;
- autonomous AI-backed NPCs with globally shared memory and consequences;
- persistent multiplayer where everyone inhabits the same evolving world;
- a custom low-level runtime with no full game engine, optimized for native desktop delivery while preserving future web/WASM portability.
## Development principle
The project is delivered through **small, reachable phases**. Each phase must produce a player-visible result, remain narrow enough for autonomous AI implementation and validation, and preserve the long-term architecture rather than creating disposable prototypes.
## Approved story and progression decisions
- The traveler awakens **outside the landed Scout on the unknown alien homeworld**, with autobiographical memory missing but practical ship knowledge retained.
- The traveler remembers Earth as home, **not** its location; the Solar System must be rediscovered and a route charted.
- **Returning to Earth and investigating Absenat are independent, optional long-term goals**. Neither is a compulsory first quest.
- The intact Archive includes Earth departure, Mars passage, ordinary voyage communications and Solar System territory exit. Damaged records can reveal the distress/accident and later hostile warning locations.
- The warning describes the alleged wormhole as a trap. **Whether the traveler entered that wormhole remains unknown.**
- **Absenat contains the endgame gate.** Passing it completes the active journey and eventually reveals the spoiler-protected Hall of Passage, as defined in [Absenat and Hall](absenat-and-hall.md).
- The **Core** is the critical heart of the spaceship. If its durability reaches failure state it shuts down rather than being physically erased, and the ship becomes unusable until repaired.
- Additional recovered transmissions, identities, and later narrative events are intentionally undecided; see [lost journey canon](narrative-and-lost-journey.md).

## Time and persistence rules
- Game time follows real-world time for **all players and ships on a single multiplayer universe clock**. Velocity does not change player/world time; **relativistic time dilation is not a gameplay mechanic**. The [Spatial Jump Drive](jump-drive-and-interstellar-travel.md) changes position rather than skipping calendar time.
- Offline time contributes to sleep recovery; as a baseline, roughly 8 hours offline should restore enough sleep/energy for about 16 hours of play.
- Sleep deprivation increases stamina drain and makes other survival pressures harsher.
- A ship continues moving while the player is offline according to its current position, velocity, gravity, and world events. Optional logged-in hours during long real-time voyages may be spent on maintenance, Core resources, Fabricator chores, Archive reconstruction, scientific observation and planning; do not require a compulsory daily login.
- Thrusters and other active ship modules turn off while the player is offline; there is no automatic offline Jump Drive activation. This does not stop existing approved offline Fabricator/Core conversion timers from progressing under their own explicit rules.
- Long-distance space is still astronomically large: continuously powered **sub-light** propulsion, inertial offline coasting, braking and a reusable **Core-powered Spatial Jump Drive** coexist. The Jump Drive provides an approved **fictional FTL displacement exception** (not a generic unrestricted destination search): physical module, charged energy cost, explicit engine-off interlock, quantum-themed loading, retained velocity, and one shared clock. It is separate from the unverified narrative wormhole; no conventional time dilation. The goal of exploring far across the galaxy over roughly 6–12 months of dedicated play is a **balance target, not already validated** under the existing 60 GJ Core and N=20% formula.
## Death and continuity
- Player death resets the run completely. No character, technology, inventory, or other progression carries over.
- The dead player's ship and physical belongings remain in the shared universe as a wreck/loot source for others.
- Even a catastrophically damaged ship is conceptually recoverable from its Core if the owner is still alive and can reach it.

# Project Vision

> Migrated from the Salimon project documents on 2026-09-29. For current task status and dependencies, use [GitHub issues](https://github.com/mr-exception/salimon/issues).

> **Salimon** is a realistic, native-first 3D space exploration and survival game set in a persistent shared universe. The player lives physically inside an evolving spaceship, travels across real and fictional space, gathers and processes resources, survives harsh environments, meets autonomous AI-driven NPCs and other players, and works toward the distant mystery of **Absenat**.
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
- **macOS-native first:** the first playable client targets macOS, with the architecture remaining platform-independent so Windows and web clients can reuse the same domain modules and backend.
## Distinctive identity
Salimon should differentiate itself through the combination of:
- a physically believable, extremely large shared universe;
- deep survival and ship-maintenance systems;
- a fully walkable and expandable spaceship as the player's home;
- physical rather than abstract cargo and resource handling;
- autonomous AI-backed NPCs with globally shared memory and consequences;
- persistent multiplayer where everyone inhabits the same evolving world;
- a custom low-level runtime with no full game engine, initially optimized for native macOS while preserving future Windows and web portability.
## Development principle
The project is delivered through **small, reachable phases**. Each phase must produce a player-visible result, remain narrow enough for autonomous AI implementation and validation, and preserve the long-term architecture rather than creating disposable prototypes.
## Confirmed story and progression decisions
- Every new run begins on **Earth**.
- The player boards their spaceship and begins the story from there.
- The first planned main quest sends the player toward **Mars** to investigate an anomaly.
- The detailed main-quest chain will be authored later.
- **Absenat is the end-game destination.** Reaching it ends the game; its final physical form may be a station, planet, anomaly, or another destination decided later.
- The **Core** is the critical heart of the spaceship. If its durability reaches failure state it shuts down rather than being physically erased, and the ship becomes unusable until the Core is repaired.
## Time and persistence rules
- Game time follows real-world time.
- Offline time contributes to sleep recovery; as a baseline, roughly 8 hours offline should restore enough sleep/energy for about 16 hours of play.
- Sleep deprivation increases stamina drain and makes other survival pressures harsher.
- A ship continues moving while the player is offline according to its current position, velocity, gravity, and world events.
- Thrusters and other active ship modules turn off while the player is offline.
- Long-distance travel should remain physically grounded. FTL, teleportation, or wormholes may only exist if represented through a physically motivated in-world mechanism rather than ordinary fast travel.
## Death and continuity
- Player death resets the run completely. No character, technology, inventory, or other progression carries over.
- The dead player's ship and physical belongings remain in the shared universe as a wreck/loot source for others.
- Even a catastrophically damaged ship is conceptually recoverable from its Core if the owner is still alive and can reach it.

# World & Realism

> Migrated from the Salimon project documents on 2026-09-29. For current task status and dependencies, use [GitHub issues](https://github.com/mr-exception/salimon/issues).

## Direction
Salimon's world is intended to model the **known universe**, not only one galaxy. Real astronomical bodies should be represented as faithfully as practical, while fictional planets, systems, anomalies, wrecks, stations, and other points of interest may be added to make the universe playable and interesting.
## Scale and astronomical model
- Known stars, planets, moons, and major systems should use real-world positions, scale relationships, and orbital behavior where data exists.
- Planets and moons move in realistic orbits rather than remaining static.
- Space-to-surface travel should be seamless; loading transitions should not be part of the intended experience.
- Distances should remain meaningful and physically large.
- Ship speed is primarily determined by propulsion upgrades; intended capabilities can range roughly from `10 km/s` to `1 Mm/s` and potentially evolve further through progression.
- Absenat is deliberately extremely far away and serves as the long-term exploration target.
## World hierarchy
The simulation should be spatially partitioned so only relevant regions are active at full fidelity.
Suggested hierarchy:
**Universe → galaxy/large region → stellar system → local sector → planet/station/asteroid field/ship → interior/local interaction space.**
This hierarchy is required for browser performance, networking, persistence, and backend simulation.
## Locations
Initial supported location categories:
- Planets
- Moons
- Stations
- Asteroid belts
- Wrecks
- Derelicts
- Anomalies
Later phases may add colonies, caves, orbital megastructures, and other location types.
## Planets
- Planets are fully explorable spheres rather than isolated landing zones.
- Different biomes can exist on a planet.
- Biomes may contain different resources, NPC activity, hazards, and environmental conditions.
- Players may encounter NPCs and other players naturally on planetary surfaces.
## Environmental hazards
Planets, moons, and biomes can expose ships and players to hazards such as:
- Temperature extremes
- Acidic or toxic atmospheres
- Radiation
- Pressure differences
- Gravity differences
- Storms and other environmental events
- Darkness and visibility conditions
Ship equipment and player equipment must determine whether a location can be entered safely. Hazards can damage hulls and modules when appropriate protection is missing.
## Persistent simulation
The universe continues evolving even when players are offline or no player is nearby.
A sector/snapshot model should be used:
1. The backend stores persistent snapshots for spatial sectors with timestamps.
2. The backend periodically advances or refreshes authoritative state.
3. A client requests only the sectors relevant to the player.
4. If a snapshot is old, deterministic simulation fast-forwards from its timestamp to the current time before real-time simulation begins.
5. The backend remains authoritative for state that players must not control, including large astronomical bodies and other protected world state.
Not every object must be simulated at full tick frequency globally. Analytical motion, scheduled events, deterministic catch-up, lower-frequency simulation, and aggregation should be preferred for distant/offline sectors.
## Resource persistence
World resource extraction must persist. Examples include:
- mined-out deposits;
- partially mined asteroids;
- removed physical resources;
- wreck state;
- player-caused damage;
- economy-affecting resource changes.
The backend may regenerate renewable resources periodically, but replenishment must account for local and global economic balance rather than blindly resetting locations.
## Authority
The world is a persistent shared simulation.
- Backend-authoritative: astronomical motion, protected large objects, economy-critical global state, NPC canonical state, regeneration schedules, and anti-cheat-sensitive outcomes.
- Client simulation: local presentation, prediction, deterministic catch-up, and approved interactions.
- Player actions are submitted to the backend and become canonical only after validation.
## Confirmed world-model decisions
- Fictional planets are generated authoritatively on the backend and are identical for all players.
- Real planets should use real geographic/terrain data where practical; Earth and Mars should aim for recognizable real-world terrain rather than purely fictional surfaces.
- Players cannot build permanent bases or structures on planets. Permanent player construction is limited to the spaceship.
- The universe uses **one global shared world** for all players rather than independent shards.
- In principle, players occupying the same location should all be part of the same world presence; scalability techniques must preserve that shared-world model rather than silently creating separate copies.
## Communication realism
Player communication should be tied to physical communication systems such as radio equipment rather than unrestricted global chat. Communication can account for signal propagation delay, including radio-wave travel time over large distances.

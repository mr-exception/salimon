# Multiplayer & NPC Interaction

> Migrated from the Salimon project documents on 2026-09-29. For current task status and dependencies, use [GitHub issues](https://github.com/mr-exception/salimon/issues).

## Multiplayer direction
Salimon uses a **persistent shared world**. Multiplayer is not a separate session mode; players physically inhabit the same evolving universe and encounter one another by traveling to the same places.
## Player presence
- Players meet naturally in the world rather than through lobby matchmaking.
- A player can physically enter another player's ship.
- Multiple players can travel together aboard one ship.
- A player remains at their actual world position across reconnects. Leaving one's own ship does not create a teleport-back shortcut.
- The player's own spaceship should always remain visible as a home marker on the compass/navigation UI when appropriate.
## Shared ship rules
- Ship ownership is permanent and each player owns one ship.
- Only the owner may operate the navigation/cockpit navigation controls.
- Visiting players may walk inside and interact with other modules, including weapons, repair, cargo, and machinery unless a later system-specific rule restricts an interaction.
- There is no general invitation/role system required for basic access; physical access to the ship is the main gate.
- If the owner disconnects, passengers stay where they are inside the ship and cannot navigate it until the owner returns.
## PvP and conflict
PvP is a natural part of the world.
- Players may damage one another.
- Players may damage other players' ships.
- Players may fight or rob NPCs.
- Ship and on-foot combat coexist with survival, travel, faction, and economy systems.
The backend must validate combat outcomes and other anti-cheat-sensitive changes.
## NPC direction
NPCs are autonomous actors rather than static quest menus. Each NPC is backed by AI-driven decision-making and exists as one canonical entity in the shared world.
## NPC capabilities
NPCs may:
- travel through the same simulated universe as players;
- remain at locations;
- trade;
- give or complete quests;
- form alliances or hostility;
- form groups/unions/factions;
- fight, rob, cooperate, negotiate, or flee;
- develop relationships and reputation with players and other NPCs;
- influence territories, stations, resources, and markets.
NPCs are not intended to become permanent crew aboard the player's ship.
## Dialogue and memory
Dialogue is free-form rather than traditional fixed multiple-choice RPG dialogue.
- The player writes what they want to say.
- The NPC responds through an AI model using the NPC's personality, state, knowledge, memories, and world context.
- Important dialogue and relationship memories are globally shared across all players.
- A quest belongs to the shared world: if another player resolves an NPC's unique quest, it should not still appear as unresolved for everyone else.
## Factions and economy
NPC factions may control or influence:
- territories;
- stations;
- resource access;
- local supply chains;
- local prices and markets.
Markets should support location-dependent supply and demand rather than one universal price table.
## World synchronization
The backend broadcasts changes relevant to each player's local area and receives player actions that may affect the world. Some state is strictly backend-authoritative, including protected astronomical motion, canonical NPC state, economy-critical changes, and other global simulation rules.
## Combat scope
Combat includes both:
- **Ship combat:** railguns, lasers, cannons, beams, and other installed ship weapon modules.
- **On-foot combat:** melee weapons, ranged weapons, throwable weapons, grenades, grenade launchers, and future equipment.
Threats may include pirates, hostile factions, creatures, environmental hazards, collisions, debris, mechanical failures, and other dynamic events.
## Confirmed NPC autonomy
- NPCs make high-level decisions periodically and also react when relevant events or direct interactions occur.
- NPCs can die permanently, including important characters; the story and faction state may change as a result.
- Some immortal NPCs or persistent world clues may be authored where required to keep critical narrative paths recoverable.
- NPCs can dynamically create quests from their own goals, problems, relationships, and world state rather than relying only on authored quest templates.
- NPCs have player-like freedom to found factions, start wars, capture stations, change markets, cooperate, betray, attack, or destroy other actors.
## Confirmed economy model
- The universe can have multiple currencies. A currency is represented by a physical trade item, such as coins, rather than an abstract global account balance.
- Players can craft and sell almost everything, but some unique world items are non-craftable and remain scarce.
- Supply chains are physical: goods must actually be transported between locations, and distant markets can run out of resources.
## PvP, crime and combat consequences
- The **Solar System is the primary restricted/safe-law region**. Outside it, PvP is generally unrestricted.
- Murder, piracy, theft, and attacks on neutral actors can produce reputation loss, bounties, faction hostility, NPC security/police response, and player-driven bounty consequences.
- Ship combat damage is physically localized to hull sections and modules, enabling decompression, disabled systems, and internal damage.
## Global population model
- Salimon targets one global MMORPG-style universe rather than separate gameplay shards.
- Players sharing the same location should, in principle, be able to see one another. Scalability solutions should preserve shared canonical presence instead of creating independent copies of the world.

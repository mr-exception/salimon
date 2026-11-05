# Multiplayer & NPC Interaction

## Current implementation and 2D presence boundary

**No multiplayer server or NPC world exists in the current browser.** The local cockpit owner/chair reservation and Web Lock are **not** authoritative online admission, player presence, anti-cheat or reconnect. This document is future multiplayer design; implement authoritative actor/ship state separately before marking any session-crossing scenario complete.

When implemented, the world appearance remains overhead 2D: a pilot using the 3D radar stays seated and visible to other players in the 2D Scout; disconnect removes that avatar and frees interactions while preserving a planet-fixed or ship-local safe anchor. The current approved damage model is **whole-ship**, including offline death only if the destroyed vessel contained the disconnected player. Never restore historical partial-room wreck behavior as a hidden edge case. See [cockpit/reconnect decisions](cockpit-navigation-and-reconnect.md).


> **Corpus migration status (#791): Approved future 2D design.** The current [React overhead 2D world](current-client-status.md) uses a separate **cockpit-only 3D radar**. Any older Rust, wgpu, 3D assets, first-person view or issue-delivery statements below concern the [archived native tree](https://github.com/mr-exception/salimon/tree/8f112bb6e6be9f813174efb47d706a8601930d04) rather than shipped browser features. Reuse conceptual design only after explicit browser implementation and checks; previous native code, commands and completion notes do not certify any current 2D feature. See the [migration register](2d-migration-register.md) for readiness and ownership.


> **Updated player presence (2026-11-04):** The approved [cockpit, disconnect and reconnect contract](cockpit-navigation-and-reconnect.md) supersedes any implication below that an **offline avatar stays visible** or a disconnected player retains an occupied ship chair. Individual room/module damage is deferred; ship-wide damage/destruction governs the new 2D gameplay scope. No multiplayer backend is implemented in the current navigation-only client.

> Migrated from the Salimon project documents on 2026-09-29. For current task status and dependencies, use [GitHub issues](https://github.com/mr-exception/salimon/issues).

## Multiplayer direction
Salimon uses a **persistent shared world**. Multiplayer is not a separate session mode; players physically inhabit the same evolving universe and encounter one another by traveling to the same places.
## Player presence
- Players meet naturally in the world rather than through lobby matchmaking.
- A player can physically enter another player's ship; multiple players can travel together aboard one ship.
- **On disconnect the avatar is removed entirely** from the world, and active interactions (including cockpit seating) are aborted and released. The user does not leave behind a simulated offline body.
- **On reconnect, restore the same logical location:** an exact planet-fixed surface point on the same planet, or an exact ship-local point on the same moving ship. If the player disconnected during any interaction, restore them **standing beside that object/module**, with the interaction cancelled and not automatically resumed.
- A disconnected player who was aboard a ship **dies if the entire ship is destroyed while offline**. On reconnect, show **"You died while disconnected"** and begin a fresh run; do not spawn at the wreck. Destruction of a distant ship does not kill a player logged off on a planet.
- Leaving one's own ship does not create a teleport-back shortcut. The player's own spaceship should remain visible as a home marker where appropriate.
- Detailed rules: [cockpit and reconnect contract](cockpit-navigation-and-reconnect.md).

## Shared ship rules
- Ship ownership is permanent and each player owns one ship.
- Only the owner may operate navigation/cockpit controls. The powered cockpit desk opens the 3D radar from default 2D play; the owner's avatar stays **seated** in the cockpit and visible to other connected players.
- One active pilot-chair occupant at a time; normal exit returns to standing 2D, power loss returns to seated 2D, and power restoration does not reopen radar.
- Visiting players may walk inside and interact with other modules, including weapons, repair, cargo and machinery unless a later system-specific rule restricts an interaction.
- There is no general invitation/role system required for basic access; physical access to the ship is the main gate.
- When the owner disconnects, remove their avatar and release cockpit authority. Passengers who remain connected stay aboard and cannot pilot the owner's ship. Ship motion and independent machinery continue under their existing simulation/offline rules.
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

**Planet consistency:** the installed offline pack and future authoritative backend resolve each celestial body to the **same versioned canonical multi-seed definition** (or verified hash/ID), so two players cannot receive different base canyons, resources or fixed sites at the same coordinates. Mutable depletion/world events are synchronized as server-authoritative deltas. For approved migrations of previously published fictional planets, the server coordinates a **single active canonical revision**, gates affected interactions while migrating persistent state and requires validated preservation/remapping of character/ship/object positions, terrain support, resources and discoveries; players must not simultaneously collide with different versions of the same planetary surface. Published real observed solid bodies retain their original physical surface from first authoritative real-scale publication; the current compressed six-body fixtures are exempt. The server rejects incompatible generator/catalog versions rather than quietly sending a different world. See [canonical planetary generation](canonical-planet-generation.md). This is a future architecture contract; no multiplayer backend exists in the current client.
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
- **Current new 2D scope:** ship damage/destruction applies to the **ship as a whole**; individual room/module damage and detachment are postponed. Historical detailed localized-damage concepts remain a future design topic, not an active rule. See [cockpit/reconnect scope](cockpit-navigation-and-reconnect.md).
## Global population model
- Salimon targets one global MMORPG-style universe rather than separate gameplay shards.
- Players sharing the same location should, in principle, be able to see one another. Scalability solutions should preserve shared canonical presence instead of creating independent copies of the world.

## Future multiplayer travel and shared-clock Jump Drive

The [approved future travel contract](jump-drive-and-interstellar-travel.md) specifies exactly **one authoritative global real-time clock** for every player, ship and offline journey; velocity does **not** cause relativistic proper-time divergence, alternate player timelines, or a world-time skip. Real-distance coasting and new Core-joule-to-impulse thrusters coexist with a **fictional Spatial Jump Drive** that changes the ship's position but retains its pre-jump inertial velocity and orientation. This module is not the unverified lore wormhole. Ordinary continuous propulsion above c remains a separate unresolved decision.

The owner-operated future Jump Drive must reject activation when **engines are switched on** (including zero throttle), validate Core charge and destination safety, and atomically move the **whole occupied ship and onboard cargo** into a shared quantum-themed loading stage before committing exactly once to a canonical target. All passengers share the same transaction and current universe time; disconnected players cannot trigger automatic jumps or end up in duplicate/alternate versions of the ship. Server authority must serialize jump command identity, energy debit/refund, destination+epoch, participant state and reconnection recovery; clients may predict visuals but never authorize their own persistent relocation. The panel uses six light-distance inputs and initial N=20% virtual speed-gap conversion. Preserve player-known-chart secrecy for unknown celestial targets. None of this implies that a multiplayer backend has already shipped.

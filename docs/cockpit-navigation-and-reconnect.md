# Cockpit navigation, multiplayer presence and reconnect

**Status:** Approved gameplay design, 2026-11-04. #766 implements the local
2D/cockpit/radar transition, single-chair reservation, owner-control gate and
explicit development power fixture in `client/`. The default entry is the 2D
landing site; ship simulation survives radar presentation changes. This is
single-player local state, with no multiplayer backend, shared persistence,
authoritative reconnect or Core power model. #802 binds world and cockpit to one
grounded Scout on sample Vesper. Until 2D relocation is implemented, takeoff,
steering, propulsion, alignment and restart-to-flight are unavailable. Radar time,
scanner and camera remain active. Existing site-bound local saves restore the
same location/lifecycle; they contain no independent free-flight state.
See [client guide](../client/README.md#cockpit-gameplay-766). The remaining
multiplayer/reconnect/death rules below are future contracts.

Companion contracts: [2D art direction](top-down-art-direction.md), [navigation-only radar prototype](radar-navigation-prototype.md), [multiplayer](multiplayer-and-npc-interaction.md), [spaceship and survival](spaceship-building-and-survival.md), and [legacy save contract](save-contract.md).

## Player view, chair and navigation authority

- The **default actual-world view is strict overhead illustrated 2D**. This includes walking inside the Scout, planetary traversal, mining, storage and module interactions.
- The **3D navigation radar is an in-world cockpit interface**, not the world representation or an always-available global menu. A player must approach and interact with the cockpit desk to enter it.
- When an authorized player activates a **powered, unoccupied** cockpit, the player occupies the pilot chair in the 2D world and switches **locally** to the 3D navigation view. Other connected players aboard see that player **seated at the cockpit**, not a vanished character or an avatar inside the radar.
- Preserve existing **owner-only ship-navigation control** and **one occupant per pilot chair**. Other players may enter the cockpit area and operate other allowed ship systems; observing an occupied chair never grants navigation authority. Multiplayer authority must confirm chair/control acquisition atomically before allowing control.
- Player world position, cockpit occupancy, local view mode and ship navigation/control ownership are distinct pieces of state. The pilot's radar camera is local presentation; the world communicates seating and ship-control status rather than streaming the pilot's 3D interface to spectators.
- Entering/leaving the view does **not** reset ship orientation, velocity, throttle or ship time. Ship physics and automatic modules evolve independently of whether the pilot is looking at 3D, walking in 2D, or disconnected, subject to the existing power and offline-flight rules.

## Transition state table

| Event | Local view after transition | Physical character in 2D world | Pilot chair/control |
| --- | --- | --- | --- |
| Normal play / walking | 2D | Standing/moving | No cockpit occupancy |
| Owner interacts with powered available cockpit desk | 3D navigation | Seated in cockpit chair | Occupied; authorized controls |
| Pilot exits navigation voluntarily | 2D | **Automatically stands** beside chair | Occupancy and active controls released |
| Cockpit power fails during navigation | **Automatically 2D** | **Remains seated** | Chair stays occupied; powered controls unavailable |
| Player stands up from unpowered chair | 2D | Stands (or follows independently approved zero-G movement rules) | Chair released |
| Power returns while player remains seated | 2D | Remains seated | Navigation **does not reopen automatically** |
| Seated authorized player explicitly interacts with powered desk again | 3D | Remains seated | Active navigation resumes |
| Active pilot disconnects | No local view; character removed from shared world | No avatar remains | Interaction aborted; occupancy/control released |
| Pilot reconnects following an interrupted cockpit interaction | 2D | **Standing next to cockpit** | No active navigation / chair occupancy |

Voluntary exit is **one action** that returns to ordinary movement; it should not require a separate stand-up command. After a forced power-failure return, the player may remain seated or stand, including without powered instruments. Seating visuals/animation should suit the strict top-down art direction, but timing and input bindings are deferred.

If power recovers after blackout, the user **chooses** when to open navigation again. Existing saved ship throttle/propulsion rules remain separate from cockpit screen presentation.

## Multiplayer synchronization and disconnect

- A disconnected player is **removed from the world**: no visible avatar, occupied chair, blocking character collider, or ongoing player interaction remains. The player is not simulated as an offline seated NPC.
- On disconnect, **abort/cancel the in-progress player interaction and release its locks/seat/control**: cockpit navigation, storage and Fabricator interfaces, Core access, mining actions, construction interactions or other actively used modules. Do not automatically resume any of these upon reconnect.
- Previously completed resource transfers or gathered litres remain committed. Already queued, self-running Fabricator jobs and automatic Core conversion are independent machine processes: closing the player's interface **does not undo or cancel them**. No halfway/duplicate resource transactions are permitted.
- Persist the last safe **non-interaction** position in its appropriate coordinate frame. If disconnect occurs **during an interaction**, persist a spawn point **beside the interacted object/module**, not on the chair or inside its interaction spot. This applies to cockpit, Fabricator, Core, storage, mining and future modules.
- Clear held movement/steering commands and **active pilot authority** on disconnect; never give another connected player owner-only navigation permission merely because the owner is absent. Ship motion and offline powered-module behavior follow existing authoritative ship/offline contracts; client view changes do not authorize changes to ship physics.
- On reconnect, restore the player in **2D normal gameplay**, with no menu/mining/navigation interaction resumed and no occupied pilot chair. Multiplayer presence and collision return only after a valid spawn has been resolved.

## Precise reconnect location

| Last state | Saved location / reconnect |
| --- | --- |
| On a planet, not using an object | **Same exact surface point** on the **same planet**. Persist canonical planet/body ID and a **planet-fixed surface-relative** anchor, not an obsolete inertial world XYZ that would drift as the planet moves/rotates. |
| On a planet, actively interacting | Cancel interaction and return **beside that resource/object**, in a valid reachable surface location. |
| Inside a live ship, not interacting | **Same ship-relative interior point**, on the **same ship**, even if the ship has moved through space while the player was away. |
| Inside a live ship, operating a module or chair | Cancel/release interaction and reconnect **standing next to that object**, in ship-local space. |
| Ship containing offline player is damaged but still exists | Reconnect on the ship as above; ship-wide nonfatal damage does not itself kill the offline player. |
| Ship containing offline player is destroyed | **Offline death**; do not spawn at the wreck or transport the player to a saved planet. |

An invalid or obstructed stored point must never create an overlapping/unsafe avatar. Exact fallback placement for changed planetary surfaces, moved/removed furniture or blocked approach points is a **future design task**, not permission to relocate the player to an unrelated planet or ship.

## Offline death and restart

- During disconnect, the server/persistent world tracks the player's **last anchored location and current alive/dead outcome**, even though the avatar is absent.
- **If the player was aboard a ship when disconnected and that ship is wholly destroyed while they remain offline, that character is dead.** Record a durable terminal/death result, not a merely missing ship reference.
- On next reconnect, **do not spawn** that character. First display the clear notification: **“You died while disconnected.”** Explain that the ship they were aboard was destroyed, then begin a **fresh run from Salimon's normal starting state**, applying existing permadeath rules.
- A player whose last saved location is on a planet **is not killed merely because their ship elsewhere is destroyed**. Nonlethal ship-wide damage to the ship containing an offline player also does not trigger this death rule.
- Preserve the destroyed ship's world/wreck/salvage consequences under their own existing game systems; do not turn the wreck into an automatic respawn point. The same destruction event must not apply death or reset repeatedly.

## Current ship-damage scope (simplification)

**For the present/forthcoming 2D multiplayer gameplay scope, damage and destruction are ship-wide, not per-room or per-module.** The ship can take nonfatal damage as one vessel; if destruction occurs, it affects the **entire ship**. There is **no presently active partial room loss, detachable compartment survival, or offline occupant relocation between intact rooms**.

Past native implementation documents describing localized cell, module, door or room damage are historical or **deferred future mechanics**. Preserve them for a later design review rather than applying them to the present design. The question of what happens to an offline player if only their room is destroyed is **explicitly deferred**, since partial-room destruction is not in scope.

## Invariants, implementation and deferred discussion

- One cockpit seat has at most one active occupant; one ship has at most one authorized active pilot; ship owner-only navigation remains protected.
- View switching is a client presentation state change; the authoritative player stays physically inside the ship when seated, and ship motion persists independently.
- Disconnect must atomically release occupancy/interaction locks and save a valid planet-fixed or ship-local re-entry anchor. It must not leave an offline physical player in the world.
- Reconnect checks terminal offline death **before** spawning, resuming commands or presenting controls.
- Implementation is authorized by active issues. #766 implements local 2D gameplay, cockpit seating and radar transitions; live multiplayer, server persistence and reconnect remain deferred.
- **Still to design later:** exact cockpit input key/animation timing, valid-adjacent spawn fallback when a saved spot changes, server timing/race handling, full power/zero-G seating safety, multiplayer interaction with an occupied pilot, reconnect if a ship is transferred/removed without destruction, and eventual localized hull/room/module damage.

## Review examples / acceptance scenarios (future)

1. Two connected players aboard the Scout: owner enters 3D navigation, observer still sees owner's character seated in the 2D cockpit; another occupant cannot claim the same chair.
2. Normal navigation exit returns the owner to 2D **standing**; power loss returns them to 2D **seated**; restored power does not force 3D open.
3. Disconnect during cockpit navigation removes the avatar and releases the chair/control; reconnect places them standing beside the desk in 2D.
4. Disconnect on a rotating/moving planet returns to identical planet-fixed surface coordinates, not inertial-space coordinates.
5. Disconnect in a coasting/traveling ship returns to the same ship-local point despite movement.
6. Disconnect during active mining or inventory transfer cancels the active action, keeps already committed output, and returns beside the object without duplicating resources.
7. Offline ship-wide nonfatal damage permits normal reconnect aboard; **whole-ship destruction** produces a one-time offline-death notice and fresh start instead of spawning in wreckage.
8. Disconnect on a planet survives destruction of a separate ship; no unrelated death.

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

**Future reconnect precedence (2026-11-06):** [Stasis Pod, logout, transfer and death](stasis-pod-and-transfer.md) supersedes exact-last-location reconnect and blanket offline-passenger death below. The browser's existing site-bound local save remains unaffected until a backend feature is implemented.

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
| Pilot reconnects after an interrupted cockpit interaction | 2D at **assigned Stasis Pod** | Standing at the pod; no resumed interaction | No active navigation / chair occupancy |

Voluntary exit is **one action** that returns to ordinary movement; it should not require a separate stand-up command. After a forced power-failure return, the player may remain seated or stand, including without powered instruments. Seating visuals/animation should suit the strict top-down art direction, but timing and input bindings are deferred.

If power recovers after blackout, the user **chooses** when to open navigation again. Existing saved ship throttle/propulsion rules remain separate from cockpit screen presentation.

## Multiplayer synchronization and disconnect

- A disconnected player has **no visible avatar, collider, chair occupancy or active module interaction**. Disconnect clears held steering, mining/work interfaces and cockpit control; independently committed jobs and ship motion continue.
- Safe logout is a **server-acknowledged interaction with the character's own assigned functional Stasis Pod**, not ordinary disconnect at a convenient location. It preserves carried inventory, gear and worn suit; next session starts at the pod if it survives.
- Unsafe disconnect/logout elsewhere removes carried ordinary inventory, gear, equipped suit and its modules into a **recoverable loot cache at the final authoritative departure location** (except future explicitly protected primary items). The next session starts at the active assigned Stasis Pod if still valid. This is a new explicit gameplay rule; exact logout/grace timing is **TBD**.
- Connection failure and session timeout cannot accidentally claim an uncommitted safe exit. Complete durable resource transactions survive both paths; no duplicate inventory drop or resumed interactions. The location/frame at departure remains important for loot and crash recovery, **not the normal reconnect spawn target**.
- A living character may transfer to a friend's available Stasis Pod through a **single-use, one-hour code**, friend entry and **online player approval**. Display loading, drop all ordinary possessions (including suit) at departure, atomically release the old pod and assign the new one, then spawn at the new ship-local pod. Original ship ownership and ship cargo remain unchanged. See [Stasis Pod contract](stasis-pod-and-transfer.md).
- A player can occupy any compatible visited ship and use permitted modules, but neither pod assignment nor being a guest grants owner-only cockpit/navigation authority.

## Stasis Pod reconnect location

| Condition | Next-login rule |
| --- | --- |
| Assigned Stasis Pod intact and functional | Spawn at that pod in overhead 2D, even if last adventure ended on another planet or ship; no interaction resumed. |
| Assigned pod lost while character **offline** | Character permanently dead; next login processes terminal notice and begins a **fresh run** outside a new landed Scout, with a new assigned pod. |
| Assigned pod lost while character **online** | Remains alive in present session; can bind another valid pod before ending session. Otherwise next login begins a fresh run. |
| Whole ship destroyed, but character's **bound pod is on a different ship** | No automatic pod-loss death from that ship alone. Separate truly lethal physical/world events still apply. |
| Pod temporarily disabled or unpowered but not destroyed | **TBD:** safe spawn/repair, without inventing an automatic rescue mode or death from mere power loss. |

Persist planet-fixed/ship-local frame anchors for ordinary physics, boarding/disembarking, loot placement, causal events and crash reconciliation. Their precision still matters, but they **do not override the assigned pod on normal login**. If the pod's local spawn is blocked/unsafe, fail closed pending an explicit spawn/safety rule; do not teleport elsewhere without approval.

## Offline death and restart

- Destruction/dismantling of an **offline character's active assigned pod** is terminal and durable. There is **no rescue-only mode, no rescue deadline, no replacement after death**, and no offline friend's code that reverses death.
- Pod loss while the character is online invalidates their binding but is not immediate character death; if they log out without a replacement, their next login starts a new run.
- New-run initialization creates a **new character/run, Scout, starting inventory and starting Stasis Pod**, while the old character/ship identity and world history are preserved. An intact old owner's ship remains in the universe as salvage/loot and its old owner-only cockpit rights are never inherited.
- A friend wishing to play together after death must construct or supply another **available pod** for the new character and use a **new one-hour transfer code with approval**. Transfer drops ordinary gear and suit as recoverable loot; it cannot revive a dead character.
- Whether uniquely classified primary items survive **permadeath**, safe-logout UX details, and disconnect grace are **TBD**. Primary items are protected from unsafe logout and transfer only when explicitly cataloged.

## Current ship-damage scope (simplification)

**For the present/forthcoming 2D multiplayer gameplay scope, damage and destruction are ship-wide, not per-room or per-module.** The ship can take nonfatal damage as one vessel; if destruction occurs, it affects the **entire ship**. There is **no presently active partial room loss, detachable compartment survival, or offline occupant relocation between intact rooms**.

Past native implementation documents describing localized cell, module, door or room damage are historical or **deferred future mechanics**. Preserve them for a later design review rather than applying them to the present design. The question of what happens to an offline player if only their room is destroyed is **explicitly deferred**, since partial-room destruction is not in scope.

## Invariants, implementation and deferred discussion

- One cockpit seat has at most one active occupant; only living owner may navigate. Pod assignment never grants navigation authority.
- One assigned pod per character/run and one living character per pod. The **Stasis Pod** is a separate buildable module; the decorative/functional sleeping bunk does not automatically act as a spawn pod.
- Safe logout, unsafe-loot settlement, pod reassignment, pod destruction/death and new-run creation require idempotent durable operations; old physical checkpoints cannot revert them. Backend still targets **5 s** verified checkpoints with strict suspension at **30 s** verified age.
- Continue using precise reference frames for movement, departure loot and server recovery; normal reconnect uses assigned pod. Current browser has neither server pod admission nor multiplayer implementation.
- **TBD:** pod power/disablement and suitless arrival environment; disconnect/grace and combat logout; valid pod spawn clearance; protected primary item eligibility/permadeath; structural removal and Core-root terminal disposal; code UI/security and pending-approval timeout; simultaneous pod-loss/transfer races; exact online-unbound session exit handling.

## Review examples / acceptance scenarios (future)

1. Owner enters 3D radar while guest sees a seated 2D character; disconnect releases cockpit without resuming it on login.
2. Safe pod interaction preserves inventory, suit and gear; reconnect spawns at that pod.
3. Disconnect on a planet without safe pod logout creates one loot cache with ordinary possessions; reconnect spawns at **assigned pod**, not at planet position.
4. Friend enters valid hour-long code at vacant pod; online user approves; loading/commit drops ordinary gear/suit, changes binding and frame exactly once; old ship remains their property.
5. Offline character's assigned pod is dismantled or destroyed; next login starts a new run, no resurrection or rescue-only session.
6. Online character's pod is lost; current play continues, and reassignment before logout avoids next-login run loss.
7. Destruction of a ship that contains an offline visitor **without their active pod** does not independently trigger pod-loss permadeath; lethal damage mechanics remain separate.
8. Same-account new character cannot use old run's pod/code or pilot an intact ship owned by the deceased character.

# Spaceship, Building & Survival

> Migrated from the Salimon project documents on 2026-09-29. For current task status and dependencies, use [GitHub issues](https://github.com/mr-exception/salimon/issues).

## Direction
The spaceship is the player's **primary home, vehicle, survival system, workshop, and progression platform**. Each player owns exactly one ship and develops it over a long period rather than collecting fleets.
## Interior and operation
- The player walks inside the ship in first person.
- Ship systems are physically located and manually operated.
- The cockpit/navigation module controls navigation and can only be operated by the ship owner.
- Other systems such as railguns, laser weapons, fabrication, charging, processing, maintenance, and life-support functions should exist at distinct physical locations.
- Other players aboard the ship may operate all modules except owner-only navigation.
- The initial ship must be large enough to contain a **dedicated walkable cargo room** in addition to the cockpit, access/airlock space, and other required interior circulation. The cargo room is a real physical room for loose objects and future containers, not an abstract inventory screen.
### Initial cargo-room layout
The initial scout has a dedicated port module with a 29.7 m² clear cargo floor
(local X −3.2…2.2 m, Z 5.1…10.6 m), level with the cabin at Y 0.2473118 m.
A 1.8 m passage at X 0.4…2.2 m leads from the cabin port aisle into the room.
The cockpit and aft airlock anchors are preserved. The physical space supports
future loose-fragment storage and containers; counting, containment behavior and
planet-to-ship fragment transfer are implemented by their separate dependent
tasks. The room remains in the ship's reference frame during flight.

## Construction
- Ship expansion uses **modular rooms/hull sections**.
- Adding new structural rooms requires the ship to be landed or docked.
- Interior modules and objects can be built while the ship is in flight where physically reasonable.
- Construction changes both the exterior silhouette and the walkable interior.
- Every gameplay module must have a functional purpose and a physical location.
- Mass, balance, structural integrity, power requirements, heat, and relevant aerodynamic/flight effects should influence performance.
## Core ship systems
Planned simulation areas include:
- Energy Core / power generation and storage
- Propulsion and thrusters
- Oxygen generation/filtering and life support
- Internal atmosphere and pressure
- Hull and structural integrity
- Navigation
- Sensors and communication
- Cargo and physical storage
- Resource processing and fabrication
- Weapons
- Thermal behavior
- Environmental protection
- Damage and maintenance
The Energy Core is central to the ship's power and propulsion systems.
## Durability and maintenance
Ship hull sections, engines, and other modules have durability. Durability can fall due to:
- normal use;
- elapsed time;
- combat damage;
- environmental hazards;
- collisions or failures.
Players maintain modules using crafted **repair kits**. A repair kit restores a defined amount of durability; more advanced modules may require more kits or more expensive maintenance.
## Survival systems
Player survival needs include:
- Oxygen
- Food
- Water
- Temperature
- Sleep
- Health/injury
- Radiation
- Pressure
- Stamina
The ship itself must provide or support the resources and equipment needed to manage these needs.
## Death
Death is intentionally severe. If the player dies or suffers unrecoverable total loss, the run ends and the player starts the game fresh. This makes preparation, maintenance, navigation, and cooperation meaningful.
## Physical inventory and cargo
Inventory should avoid abstract chest/list behavior wherever possible.
- Resources exist as physical objects/fragments.
- Players can drop resources directly on the ship floor and create visible piles.
- The initial spaceship includes a dedicated physical cargo room sized for carrying and organizing multiple loose resource fragments over repeated one-object-at-a-time trips.
- Physical containers organize objects spatially; the player can see and retrieve the actual contents.
- Cargo mass and placement may affect the ship where simulation depth permits.
## Space EVA and airlock
- When the ship is in open space and is **not landed**, the player may open the airlock and leave the ship for EVA/free-flight.
- On exit, the player starts from the airlock's world-space position and **inherits the ship's current linear velocity**. EVA movement then adds player-relative motion on top of that inherited velocity instead of resetting the player to a stationary world frame.
- While in open-space EVA, the player can freely translate and orient around the ship. The motion model must remain consistent with the ship's current world-space motion so the player and ship continue travelling through space together unless the player deliberately changes relative velocity.
- This open-space EVA condition remains active until the player comes within **3,000,000 m** of a planet or other solid celestial body. Inside that distance, movement must transition to the nearby-body interaction/gravity regime rather than continuing indefinitely as deep-space free-flight.
- The 3,000,000 m threshold is the same nearby-body distance used elsewhere in the game and should be treated as one shared gameplay constant rather than duplicated values.
- Opening the airlock in space is intentionally allowed. Future pressure, oxygen, suit-safety, decompression, and death consequences may add risk, but they must not invalidate the core ability to perform EVA.
## Resource gathering
### Asteroids
A mining laser/tool detaches fragments from asteroids or breaks sufficiently small asteroids into multiple pieces. Fragments are physically collected into the ship, then processed by ship modules.
### Planetary resources
A handheld or equipment-based mining laser breaks resource objects into carriable physical fragments. For example, harvesting a tree can produce wood fragments and fruit objects.
### Processing
Raw materials can require processing before construction or crafting. Example: iron-bearing fragments can be refined/processed into steel for ship upgrades.
## Resource uses
Resources may be used for:
- food and survival supplies;
- repairs and maintenance;
- energy/fuel-related systems;
- trade;
- crafting;
- ship construction;
- upgrades;
- ammunition and weapons;
- research and technology progression.
## Technology progression
Research and technology unlocks should enable:
- faster/better thrusters;
- improved ship modules;
- better environmental protection;
- stronger weapons;
- improved tools and resource processing;
- the ability to safely travel farther from Earth toward Absenat.
## Confirmed ship destruction, salvage and access rules
- Other players may damage, destroy, salvage, and loot a ship whether or not its owner is alive.
- If the owner dies, the ship becomes orphaned and remains in the world as salvage/loot.
- If the ship is destroyed while the owner survives, the owner can still return and rebuild/repair from the **Core**, which is the irreducible recovery point of the ship.
- There are no lockable protected containers or general anti-loot protections planned; physical access means items can be taken.
## Confirmed personal inventory rules
The game should avoid conventional abstract inventory.
- **Core carrying rule:** the player can physically carry exactly **one carriable world object at a time**. This is a permanent rule for all current and future carriable world objects, including mined resource fragments; it is not a temporary limit that increases with progression.
- The player has no general backpack or pocket inventory for resources or other ordinary world objects.
- Equipped gear and dedicated equipment slots are separate from the one-object carrying mechanic and do not consume that carrying capacity.
- A player can equip one primary gun/tool and carry one extra in a toolbelt; these equipped/toolbelt items are separate from the one-object carrying mechanic.
- Future suit/outfit gear may provide dedicated consumable slots, such as **4 food-ration slots** or **3 medkit slots**. Exact slot design is deferred, but these slots must remain separate from physical world-object carrying.
- Future transport aids such as electrical carts may help move multiple physical items, without increasing the player's personal one-object carrying capacity.
- Every physical item has mass and volume and these properties affect carrying, storage, and ship mass.
- Loose objects inside the spaceship do not need to react to ship acceleration because the ship provides internal gravity; acceleration effects are intentionally abstracted away for interior object stability.
## Confirmed construction and damage model
- Structural expansion uses a **grid-based modular system with snap points along the Z axis**.
- Rooms and modules can be individually detached or destroyed during combat and can physically separate from the ship.
- Structural integrity is primarily a numerical design constraint rather than a full bending/breaking simulation.
- Weapon damage should be localized to specific hull sections/modules and can cause decompression, disabled systems, and internal damage.

### Nearby-body EVA implementation (#36)

Nearby means an inclusive **surface distance** of 3,000,000 m measured from the
player, using the world-owned constant also re-exported by ship telemetry.
Only solid bodies apply EVA gravity. The nearest eligible surface wins;
exact ties follow catalog order. Entry preserves world position and inherited
velocity and adds the shared fixed 9.81 m/s² radial gravity. Airborne assisted
3D controls remain available, and the camera uses radial up. On body contact,
existing eye-height surface walking takes over. Exiting influence disables radial
gravity and preserves accumulated drift. Ship interior gravity overrides this
mode after re-entry. This does not add orbital physics to ship flight.

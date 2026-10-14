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
- The initial scout has a walkable cabin, cockpit, and aft airlock. Its separate port cargo room and protruding cargo hull module are removed to keep the exterior silhouette coherent.
- The cockpit has a shorter nose and shorter canopy glazing. One center console carries one wide multifunction monitor, with clear walking routes on both sides from the cabin past the console into the cockpit nose. Seated forward and downward visibility remains usable.
- Loose physical fragments can be dropped on the main-cabin deck and travel with the ship. The initial scout has no dedicated cargo room or designated storage volume; its cabin bench and storage cabinet are removed. Future dedicated physical storage is an open design question in [Project Q&A](project-qa.md).

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
- The initial scout can carry loose fragments dropped on the main-cabin deck. It has no dedicated cargo room or cargo-counting volume.
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

### Physical planet/ship transfer (#47)

E picks up an aimed fragment on the surface or while walking inside; E releases
the carried fragment from hand height. Gravity and contact let fragments settle
on the ground or pile against one another. Doorway
movement keeps the same carried world entity relative to the player's eye/up
basis. Releasing inside attaches that physical entity to a ship-local support
anchor: it moves with the ship while simulated against the cabin deck. Pickup detaches
it; releasing outside lets it fall onto radial ground.
Identity, material, mass, and source stay unchanged through repeated trips.
Current greybox cubes retain world-axis orientation with conservative clearance
for rotated decks. This adds no abstract inventory, container system, cargo
containers or backend persistence. Physical fragments now contribute to cached ship mass; see below.

## Implemented starter flight (#201)

The 100,000 kg dry hull has 4 MN forward and 2 MN reverse force capacity.
World-space momentum persists independently of orientation; zero throttle
coasts. Acceleration is force divided by cached total mass. Steering remains
5°/s with its existing presentation ramp. There is no drag, speed cap or
planetary gravity in this flight wave. Core energy now gates free-flight force and cockpit/artificial-gravity power.

Up/Down select signed −100..+100% throttle, one percentage point on press and
one more per 100 ms held. Selection persists when leaving the cockpit. Only
a powered cockpit in free flight can change it; assists reject input. Landing
completion and contact reset selection to zero.

## Implemented contact response (#202)

Free flight sweeps solid bodies/terrain to prevent high-speed tunneling. Contact
zeros throttle and produces bounded, progressively damped normal rebounds, then
rests in `GroundedAfterCrash`. This is separate from a proper assisted landing;
L requests powered recovery from a settled crash through the same cockpit action.
Recovery reserves 12 seconds of power for lift, safe relocation and standard
landing; without sufficient energy the ship stays grounded. It does not count
as Landed until completion. No damage is applied. The current conservative
collider remains; exact hull/contact physics is future work.

## Implemented physical ship mass (#187)

Total mass includes the 100,000 kg dry hull and each positive-mass physical object
whose center is in the authored cabin/nose, including carried and loose cargo.
Installed equipment uses separate stable ledger identities when configured;
current authored fixtures have no separate equipment weight definition. Pickup,
release, growth, boundary crossings and despawn/consumption update the cache once
per identity. Mass changes affect force/mass acceleration and preserve velocity.
The lowered ramp outside the cabin is outside onboard containment. Session
streaming never removes physical cargo. This does not add abstract inventories.

## Implemented Energy Core (#188)

The configurable starter battery stores 60 GJ, initially 45 GJ (75%). Maximum
forward/reverse draw is 100 MW/50 MW scaled by signed throttle magnitude.
Artificial gravity and cockpit instruments independently draw configurable idle
watts (starter tuning: 100 kW and 10 kW). A full-capacity ten-minute forward-burn
estimate excludes idle loads; endurance is not a fixed gameplay invariant.

Zero energy stops thrust, cockpit controls/display and artificial cabin gravity,
preserving throttle selection and velocity. Inside gravity never falls back to
planetary gravity. Charge restoration resumes selected throttle automatically.
Core transactions can charge and discharge together; requested loads receive
incoming energy before surplus storage is capped. Fractional joules are retained
internally. Physical fragment intake/conversion and powered recovery are implemented.
Assist activation reserves forward watts × duration atomically (800 MJ landing,
600 MJ takeoff at starter tuning). Reserved energy is isolated from ordinary
consumers; bus blackout cannot cancel the path. Unused budget is refunded on
completion, clipped to capacity. Insufficient energy rejects activation.

`advance_with_core_input(delta, incoming_joules)` settles converted energy at a
step boundary with that step's normal idle/propulsion consumers before storage
clipping. It preserves exact powered-thrust duration. Conversion timing/rate is
owned by E03; no continuous charging source is assumed by this task.

## Physical Core fueling (#189)

Carry one fragment to the Core, aim at it within 3 m and press E. Mechanical
intake works without power, immediately consumes the fragment and removes its
cargo mass. One fragment processes at a time, automatically, wherever the player
is. Conversion defaults to 200 MJ/s with material-specific configurable yields
(iron 1 GJ/kg, rock 0.5 GJ/kg, ice 2 GJ/kg). Discharge continues during conversion;
excess energy above capacity is wasted and processing never pauses at full charge.

## Interior blackout movement (#191)

Without artificial gravity, indoor WASD and Space/Shift translate in three
dimensions. Releasing controls retains drift until contact. Power return restores
walking/deck gravity from the current position. Loose cargo becomes weightless,
retains velocity, contacts other fragments and stays bounded by cabin geometry.
Nearby planets do not pull crew/cargo while inside. Carrying and mechanical Core
intake remain available. Detailed acceleration-induced cabin forces are deferred.

## Cockpit blackout authority (#192)

An unpowered cockpit has no mechanical flight-control backup. Selected signed
throttle and heading survive outage; the ship coasts until conversion restores
power, then selected nonzero thrust automatically returns even without a seated
pilot. Steering stops at exhaustion and needs fresh powered input after outage.
Assisted sequences retain their reserved energy and cannot be overridden.

## Accepted propulsion and Core review (2026-10-14)

These numbered references reconcile the 15-round, 84-decision review with the
owning gameplay rules above. They supersede the original direct-speed/no-energy
Phase 0 baseline; historical evidence remains evidence of its original revision.
The [ship architecture](../client/ship/architecture.md) owns numerical contracts,
and [technical architecture](technical-architecture.md#flight-update-and-validation-decisions-197)
owns timing, validation and implementation boundaries.

- **1–8:** Velocity persists independently of orientation, with no speed cap.
  Ship-local forward/reverse force changes world velocity through total mass;
  turning alone does not redirect travel. Signed −100..+100% throttle persists
  after release/seat exit. Reverse is deliberately weaker.
- **9–13:** This wave has no planetary ship gravity or orbital simulation.
  Braking is manual reverse thrust; landing requires safe total speed; takeoff
  retains terminal velocity. Mass affects translation only, not steering inertia.
  Existing nearby-body EVA gravity remains a distinct character feature.
- **14–18:** Count dry hull, configured equipment and all physical cargo.
  Reverse force is 50% of forward; arrows change 1% and repeat while held.
  Propulsion consumes Core energy; no reaction-mass fuel tank is introduced.
- **19–24:** Every velocity component counts toward the starter 20 m/s landing
  limit. Future upgrades may increase it. Show vector/relative speed and braking
  distance/time. Empty energy prevents thrust/new assistance; watt draw scales
  with absolute throttle and forward/reverse capacity. Fuel with physical
  fragments using resource-specific energy ratios.
- **25–30:** Interact with the Core while carrying one fragment. Its yield is
  J/kg × material mass, converted at a configurable rate; over-capacity energy
  is discarded. Reserve assistance energy up front. Future upgrades may improve
  consumption; no progression/modifier system is implemented here.
- **31–36:** Loose, held and future stored onboard objects count once. Mass
  changes immediately affect acceleration without changing velocity or adding
  recoil. Keep existing physical cargo contacts and permanent one-object carry.
  Intake accepts one process at a time and keeps processing at full charge,
  wasting excess. EVA inherits velocity once at exit; later ship acceleration
  does not tow the player.
- **37–41, 43–48:** Starter hull is 100 tonnes; 4 MN forward gives 40 m/s² dry,
  with 2 MN reverse. Watt demand and resource J/kg are configuration, not physics
  constants. Forward draw is 100 MW and reverse 50 MW; battery capacity is 60 GJ,
  initially 75% (45 GJ). Ten minutes at full charge/max forward is a rough tuning
  estimate excluding idle, not guaranteed endurance. Thrusters, assistance,
  artificial gravity and cockpit consume power. Accepted fragments become energy
  and cease to exist physically. Fixed stepping (42) belongs to technical architecture.
- **49–54:** Accepted fragments disappear immediately; conversion transfers
  joules gradually and continues without a nearby/seated player. Gravity and
  cockpit have independent configurable idle loads. Depletion switches off
  powered systems while preserving heading, signed throttle and coasting.
  Restored power automatically resumes the selected nonzero throttle.
- **55–60:** Without cabin gravity the player floats using EVA-like controls,
  retaining momentum; loose fragments retain velocity and collide. Cockpit
  controls wait for power; throttle persists. Manual power management is deferred
  (58). Charging/discharging operate together on net available energy. A physical
  Core-local status display includes remaining-runtime information.
- **61–66:** The Core display blanks at depletion and restarts with power.
  Interior movement retains momentum; detailed acceleration-induced cabin
  forces are deferred. Contacts prevent penetration; ship contact response is
  refined by decisions 74/79 below, with no damage. There are no powerless
  alternative flight controls. Show velocity magnitude/vector and heading versus
  travel angle on the cockpit console.
- **67–71:** Landing/contact reset throttle to zero. Landing permits any heading
  at total relative speed ≤20 m/s. Normal engine thrust is disabled during
  assistance and stays zero afterward until changed. Braking estimates use the
  full vector, orientation, reverse force and cached total mass; misaligned
  reverse thrust cannot promise a stop. Sweep travel continuously against bodies
  and terrain to prevent high-speed tunneling.
- **73–77:** Endurance is an estimate, not a fixed core mechanic. A crash bounces
  and settles without damage; the pilot requests recovery to standardize the
  landed pose for takeoff. Proper landing finishes stationary, throttle zero.
  Float to the Core and insert the carried fragment through normal interaction.
  Cache mass from entry/exit/consumption events and reconcile explicitly for
  validation, rather than scanning every flight tick.
- **79–83 (final collision/recovery clarification):** Rebounds lose velocity until
  resting in GroundedAfterCrash, distinct from Landed. L requests powered lift,
  alignment and proper touchdown; the 20 m/s limit still applies. Insufficient
  energy leaves the ship grounded until refueled. Deduct the entire assistance
  budget atomically on activation, isolate it from other consumers, and refund
  unused joules if applicable. Mechanical intake works without electricity and
  can bootstrap the Core. These final clarifications supersede earlier shorthand
  about stopping immediately or calling crash contact a stable Landed state.

## Future upgrades and systems — OUT OF SCOPE for this wave

Future thruster upgrades may change force/efficiency; landing upgrades may raise
safe speed/range or reduce reserved cost. Future physical modules may add energy
loads. Full cabin acceleration/inertial forces, gravity/orbits, manual per-system
power management, damage, fuel reaction mass and survival consequences are not
implemented by this wave. Existing configuration seams do not constitute an
upgrade/progression system. Owner-only multiplayer navigation is a future rule;
current client-only authority means the powered occupied cockpit.

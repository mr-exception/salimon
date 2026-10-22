# Spatial Jump Drive, sub-light travel and shared-time propulsion

Status: **Jump Drive/shared-time travel remains approved future design, not implemented**.
The ordinary funded-impulse engine law is implemented by #493 in this revision;
it does not deliver the Jump Drive, Fabricator module or universe clock. This
contract supersedes former blanket no-warp/FTL and future velocity-dependent
relativity assumptions for the Jump Drive. Current compressed-world/offline
simulation otherwise retains its existing owners.

## Experience, journey duration and shared time

- Keep the Milky Way at realistic 1:1 distances and advancing astronomical positions. One authoritative universal game time, progressing 1:1 with real elapsed time, applies to all multiplayer participants and offline ships. There is **no velocity-dependent time dilation, player-specific time scale, leap forward in time, or causality-changing time travel**, including during a Jump Drive transition. This is a deliberate fictional physical model, not a claim that real relativity permits faster-than-light travel.
- The Scout is the player's lived-in home on journeys lasting real days, weeks, months or, exceptionally, a year. Players may log in for roughly 1–3 hours when they choose to inspect/repair machinery, craft, organize cargo, read and recover Archive records, study observations and plan encounters or arrival. These are meaningful optional activities, **not a mandatory daily attendance schedule or arbitrary failure timer**. Preparation for long voyages, Core supply, maintenance and safe braking matter.
- Normal propulsion drives takeoff, orbital maneuvers, braking, landing, short and long continuous travel and real-time **offline coasting**. An engine-off ship retains inertial velocity, orientation and encounters gravity/drag/collisions during deterministic offline propagation; closing the game stops active engines, as already approved. **No automatic offline Jump Drive activation.** Do not silently add unattended burns, autopilot or automatic jump scheduling.
- Engine upgrades may improve thrust and energy-to-impulse conversion without an arbitrary conventional engine speed-cap per module. Whether **ordinary continuous thrusters themselves** may deliberately reach/exceed light speed is **not finally decided by this contract**; the approved extraordinary-distance mechanism is the Jump Drive. Avoid presenting existing Newtonian numerical superluminal limitations as a validated physical propulsion feature. Do not simulate relativistic proper-time differences.
- The intended progression is exploration across the galaxy during an approximately **six-month to one-year full playthrough** for a dedicated progressing player, without making routine travel require constant grinding. This is a design target needing quantitative end-to-end balance, **not a guarantee** under the current Core and energy figures.

## Core-powered thruster impulse — implemented engine law (#493)

The Core directly converts funded electrical joules into impulse. **A joule is energy, not a newton of force.** The agreed baseline is an explicitly fictional conversion coefficient:

- kappa = **1 newton-second per joule (1 N·s/J)**; therefore 1 kJ of actually funded propulsion energy gives 1,000 N·s impulse.
- Funded energy E (J) yields impulse J = kappa * E (N·s); the change of velocity along the selected thrust axis is delta-v = J / current_ship_mass_kg (m/s). For a continuous powered interval dt with power P (W): force F = kappa * P (N), acceleration a = F / m, and consumed energy E = P * dt. Only **actually funded propulsion energy** may generate impulse.
- Keep separate forward and reverse throttle/power budgets, ship-relative thrust direction, world-frame persistent velocity, manual braking, zero-throttle coasting, ship-mass ledger, exact energy-boundary splitting, and no independently billed kinetic-energy delta. Future upgrades can change kappa or permitted power without inventing reaction-mass tanks.
- Test fixture: at m=100,000 kg and kappa=1, +10 km/s requires 1 GJ propulsion
  energy regardless of existing speed. Default 100/50 MW now produces 100/50 MN
  and 1000/500 m/s², 25× historical force tuning. Separate service draw still
  consumes Core energy. Captured assists remain conservative kinematic paths,
  ending at rest/throttle zero; landing remains <=20 m/s.
- **Implementation status:** #493 implements exact funded impulse, explicit B
  engine switch and schema-5 migration. Core remains 60 GJ/45 GJ initially.
  Native handling/control/instrument approval remains #498; Jump Drive is future.


## Reusable Spatial Jump Drive

> **Construction integration (approved 2026-10-22):** The Jump Drive is **not installed in the starter Scout**. It is a later physical module installed through an in-world blueprint placeholder: place it within sufficiently large ship structure, feed real material/components one carried object at a time, then begin short automatic construction when fully funded. Ordinary upgrades disable the module for their entire automatic build/commissioning period, and module-specific flight-safety rules apply. This **does not** change the drive's canonical N=20% initial parameter, operation, Core joule cost or impulse mathematics. See [Scout and modular construction](scout-and-modular-construction.md) and #547. The separate instant-material/no-timer rule applies **only to Energy Core upgrades**, not Jump Drive upgrades.

The owner-approved module is named **Spatial Jump Drive** (short form: **Jump Drive**). The abandoned consumable 'wrap/warp cell' concept is **not** part of this design.

- The player builds and installs a **physical, persistent, upgradeable** ship module. Its components and later upgrades can be fabricated with the existing Fabricator design, without disposable jump cells or a new power currency. Core joules are the per-jump cost.
- Upgrade progression reduces the virtual velocity-gap parameter **N**, starting at **N = 20%** for the initial module. Upgrade recipes, resource rarity, magnitudes, number of upgrades, timing and additional Core/storage progression are **not finalized**. There is no separate hardcoded jump-distance cap; charge and physical validity bound the distance. Long-term economy must make repeated travel across many systems sustainable.
- The operator inputs a **distance** in light-based units, not elapsed travel time, then activates a displacement **along the ship's current forward axis**, evaluated in the current authoritative inertial coordinate frame. A jump alters the ship's **position only**: orientation, inertial velocity vector and its magnitude are preserved. Relative approach speed to a moving destination may therefore be dangerous; the module never supplies braking for free.
- All onboard occupants, physical cargo and installed machinery accompany the same ship atomically, preserving stable identities, states, containment and mass. Objects/players outside the ship do not silently accompany it.
- The jump uses the same single shared universe clock. There is **no fictional years-long passage of time** for a distance entered as light-years. Real time naturally continues while assets load.

### Approved energy formula

Define:
- c = **299,792,458 m/s** (the defined light-second conversion; not a gameplay speed cap for the jump itself).
- m = **current ship total mass in kilograms**, including physical carried/stored items under the mass contract.
- v = **pre-jump scalar ship speed in m/s** in the agreed inertial frame (not a target-relative closing speed).
- N = **20** for the initial Jump Drive, interpreted as a percent; future upgrades lower it.
- D = chosen jump **distance in light-seconds**, potentially fractional.
- kappa = **1 N·s/J** in the approved future impulse-conversion baseline, with an explicitly versioned upgrade if changed.

For the virtual speed change used *only to price a jump*:

~~~text
delta_v_virtual_mps = (N / 100) * (c - v)
virtual_target_mps = v + delta_v_virtual_mps
energy_per_light_second_J = m * delta_v_virtual_mps / kappa
required_energy_J = D * energy_per_light_second_J
~~~

Calculate this **independently for each light-second using the same pre-jump v**. Virtual delta-v does **not** accumulate into real ship velocity. Do **not** use relativistic/ordinary kinetic-energy differences, a fixed kJ per light-second price, a 90%-decay rule, or a base activation energy: those were explored and **superseded** by this formula. Reject invalid mass, distance, conversion coefficients and nonfinite arithmetic; zero-distance input cannot activate a jump.

The current design defines positive energy for **0 <= v < c** only. At **v >= c**, the formula would give zero or negative energy, which cannot become a free jump. **Block activation with an explicit explanatory error until the owner approves a separate at/above-light-speed rule**; never clamp silently or mint energy. The approach-to-c edge (cost tending to zero) also needs balance and exploit review before gameplay implementation.

**Cost fixtures with N=20 and kappa=1:**

| Ship mass | Current speed | Distance | Required energy |
| --- | --- | --- | --- |
| 100 tonnes | 0 km/s | 1 light-second | 5,995,849,160,000 J (5.996 TJ) |
| 100 tonnes | 100,000 km/s | 1 light-second | 3,995,849,160,000 J (3.996 TJ) |
| 100 tonnes | 0 km/s | 1 light-year (31,557,600 light-seconds) | ~1.892e20 J |

The current **60 GJ** starter Core cannot fund even **one whole light-second** at either example speed; the stationary example requires about **100 full Core charges**. This conflicts with effortless frequent jumping and the desired six-to-twelve-month galaxy progression. **Do not secretly reduce N, rewrite the formula or claim it is currently balanced.** Prototype balance should explicitly consider future Core capacities, progression economics and/or a subsequently owner-approved impulse/jump efficiency model.

### Physical control panel, distance input and feedback

- Place a physical, readable Jump Drive control panel on the ship. Navigation is owner-authorized under the future multiplayer contract; other players aboard may access permitted ordinary modules but may not command the owner's jump.
- Provide **six separate, clearly labeled, nonnegative numeric inputs**: **years, months, days, hours, minutes, seconds**. These express equivalent **light travel distance**, *never elapsed world time*. Display resulting light-seconds, optionally light-years, required Core joules in readable units, current Core charge, energy deficit/sufficiency and visible errors **as the fields change**, before activation.
- Define **1 light-year = 365.25 light-days = 31,557,600 light-seconds** and **1 light-month = (365.25/12) light-days** (Julian-year convention), not calendar-variable months. Hours/minutes/seconds convert normally; avoid calendar/timezone ambiguity. Validate overflows and fractional input rules consistently before any consumption.
- Show direction, anticipated target and approach/arrival warning when calculable, without giving the player a forbidden full-galaxy search, undiscovered Earth location, or unrevealed Absenat marker. The canonical catalog may be used **internally** for collision checks, not as player knowledge.

### Activation, interlocks and quantum loading

1. Require an explicitly **powered-off propulsion engine state**, not merely selected throttle = 0. If the engines are on, reject activation and show **'Jump unavailable — shut down propulsion engines'**. Also reject incompatible active takeoff, landing, recovery or propulsion assists. Current engine code lacks this separate engine-off state; implementation must introduce it without breaking the established throttle-preserve/power-restoration behavior.
2. Check valid distance/math, operator authority, available unreserved Core charge, ship condition, current mass and an authoritative candidate target = current position + forward_unit_vector * (c * D). Verify the physical destination against canonical moving-body/terrain collision and streaming validity at the authoritative current epoch. A destination inside a solid body or otherwise provably unsafe must refuse activation. **Near-planet departure policy and exact arrival clearance are not yet approved**; do not silently choose deep-space-only or ground-to-space access.
3. On successful validation, reserve/debit **exactly once** the quoted Core energy. Execute a crash-safe, all-passenger atomic jump transaction; no duplicate Core debit, no duplicated ship, no half-applied cargo state. Failure/cancellation before commit must leave the origin and Core charge consistent under a defined refund/rollback protocol.
4. During destination loading, **put the ship and its occupants in a quantum-themed transition environment** rather than displaying unrelated gameplay or an ordinary world-teleport loading spinner. This is a loading/staging presentation **not a separate real universe, proof of wormhole lore, or alternate multiplayer timeline**. Preserve onboard state and shared real-time clock; expose progress/status truthfully, avoid entering the target until canonical region state is ready and verified.
5. On completion, publish the new world position once, preserving exactly the pre-jump **world-frame velocity vector and orientation**, then resume normal simulation, including local gravity/contact risks. If the client closes mid-transition, reconnect deterministically to the unique authoritative committed result or verified origin; do not expose a half-complete jump.

### Interaction with existing systems and scope

- **Fabricator:** reuse versioned ingredient/recipe/unlock/queue/Core-funding, physical output tray and 20 kg *per-material* raw-bin limits; large module components may need an assembly path rather than illegally fitting an entire module into a consumable slot. A Jump Drive is **not** food/medicine inventory, is not held in a general backpack and cannot bypass the one-carried-world-object rule. Material inputs/blueprints have not been priced.
- **Core:** jump draw is from the ordinary existing Core energy authority, respecting higher-priority reservations (including emergency landing), charge/capacity, outages, and fractional joules. It is not a second energy pool. Jumping does not auto-recharge the Core.
- **Offline:** current engines immediately stop on closing the game. Ship continues to coast in real time; no automatic offline jump or magical travel-time acceleration. Existing real-world time and deterministic catch-up remain.
- **Navigation/scanner:** retain one automatic scanner pulse every five seconds, player-known chart discovery, moving-body ephemerides and uncertainty. A vector displacement is not a free chart/reveal or autopilot. Jump mechanics do **not** validate the narrative's alleged wormhole or expose the Hall.
- **Persistence/networking:** the future shared server is the single authority for time, jump validation, ship state and Core spending; offline-only implementation must use the same deterministic and atomic protocol shape without implying multiplayer already ships. Save pending/committed transaction identity, input distance, origin/target frame+epoch, Core reservation, passengers/cargo, and retry/recovery state in a future versioned schema. Do not change today's schema 3 through documentation alone.
- **Performance:** use time-addressable and indexed galaxy catalogs; no full-universe tick/load. Confirm large-distance precision, frames, numerical overflow, extreme D and boundary cases. The quantum scene may mask loading but must not replace deterministic checks.

## Implementation and test boundaries

- **Issue #493:** implements the direct joule-to-impulse engine change and reconciled flight tests/contracts in this revision; native review remains #498. It does **not** implement Jump Drive.
- A **separate future implementation plan/issues** must cover physical Jump Drive and Fabricator components/upgrades, Core transactions, ship interlock, six-field panel/validation, destination safety, quantum staging and native verification, save/restart, performance and multiplayer authority. This document and linked changes are **docs only**, not claims of shipping code.
- Required model fixtures: 100 t, 0 km/s, N20%, D1 gives 5,995,849,160,000 J; 100 t, 100,000 km/s, N20%, D1 gives 3,995,849,160,000 J; double mass doubles cost; halve N halves cost; 60 light-seconds costs sixty times the 1-light-second quote; fractional distances scale linearly. Fuel equality must hold across batching and Core restoration. Test engines-on rejection (including throttle 0), no-charge rejection, unknown chart IDs, changing nearby bodies, save/close/reopen during loading, concurrent passengers, duplicate commands and preserved velocity.
- **Remaining owner decisions** requiring explicit follow-up: safe departure zones/arrival clearance; handling when v>=c or approaching c; late-game realistic energy/capacity/efficiency economics for galaxy-scale progression; whether and how upgrades reduce N; how future upgrades fit physical machinery and Fabricator constraints; optional destination selection beyond a forward-vector jump; damage/interruption and multiplayer jump consensus; whether ordinary continuous thrust may deliberately exceed c. No implementation may invent these as if approved.

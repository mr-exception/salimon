# Resource inventory, gathering and ship processing — strict top-down 2D

**Status: approved gameplay direction (2026-11-04); player gathering implemented in #757.** The continuous Scout/planet browser world supports whole-litre gathering, exact integer grams/L, independent 10 kg / 10 L limits, and versioned local inventory/depletion/location saves. Ship storage transfers and continuous Core processing remain future work. Do not treat historical native physical-fragment behavior as current 2D gameplay.

## Resource representation and player gathering

- Raw resources are **stacked in whole litres**, not carried as physical fragments. Resource quantities and mining/source depletion are whole-litre increments; no fractional litre can enter an inventory.
- Each resource type specifies **mass per litre**; fractional kilograms are allowed (e.g. 0.5 kg/L). Use **integer grams per litre** and integer litre quantities internally to avoid floating-point inventory drift. Display kg and L independently.
- Starting player inventory: **10 kg maximum weight AND 10 L maximum volume**, enforced simultaneously. Future inventory capacity upgrades are possible; their costs/tier values are not defined.
- The player leaves the Scout, approaches a rock/plant/other harvestable source, equips the mining tool and mines **1 L per second** while the interaction is valid. Suitable gathering interactions for other resource categories need the same capacity and depletion rules.
- Credit only the litres fitting BOTH remaining limits and remaining source quantity, debiting the corresponding persistent source in the same authoritative operation. Uncollected material remains at the source; no fragments, growing spheres or physical fragment piles are created.
- The starting sample reuses each source's existing interaction reach (2.3 m mineral; 1.9 m biological). Equip the gathering tool, then **E / Gather** starts or stops continuous collection. Movement, Escape, tool stow, loss of reach, page blur/hide or cockpit entry interrupt it. Only completed 60-tick seconds commit; unfinished progress is discarded, with no offline catch-up or automatic restart after reload.
- Authored sample catalog `landing-resources:v1`: banded mineral **2500 g/L**, initially **24 L**; plate biomass **500 g/L**, initially **30 L**. These are explicit sample values, not upgrade balance or Core conversion values. Depleted sources keep their IDs/colliders and are drawn dimmed. Tool upgrades and animation polish remain future work.

## Initial ship-side resource capacities

| Destination | Starting weight capacity | Starting volume capacity | Operation |
| --- | --- | --- | --- |
| Player inventory | 10 kg | 10 L | Collect and transport |
| One built-in Scout storage container | 50 kg | 50 L | Store and retrieve raw resources |
| Fabricator input buffer | 50 kg | 50 L | Supply recipes; withdraw unused inputs |
| Energy Core raw-material buffer | 50 kg | 50 L | Feed energy conversion; withdraw unconsumed inputs |

- Capacities are **independent per inventory/buffer**, with both weight and volume checked. Each may hold mixed resource types up to its combined limits.
- The player can deposit and withdraw **unused** material from the general container, Fabricator and Core. Material committed to a queued Fabricator job or already converted by the Core is not withdrawable.
- Storage-container construction and upgrades, Fabricator-buffer upgrades and Core-buffer upgrades are **future design discussions**; no recipes, tiers or costs are set here.
- Transfers must conserve exact whole litres and associated mass without duplication. The transfer UI, whether module-to-module transfers are allowed, partial-transfer UX and how a partly converted active Core batch is treated on withdrawal remain to be finalized.
- Player and ship resource mass must be counted once in applicable physics. General storage is distinct from physical furnishings and separately equipped tools/consumables.

## Fabricator

- The Fabricator has a 50 kg / 50 L reserved resource-input buffer at game start; unused materials can be retrieved.
- Preserve the earlier crafting contract: jobs reserve/consume their ingredients on queue, operate one at a time, pause without power, and deliver finished consumables/items to the output tray. Resources already committed to a job cannot be recovered by withdrawing them from the input buffer.
- Replacing legacy physical-fragment intake with inventory transfers is future implementation work. Fabricator capacity-upgrade behavior is deferred.

## Energy Core — approved processing model

- The Core starts with a **separate 50 kg / 50 L raw-material buffer**, independent of the Core's stored energy capacity (joules). Unconsumed raw resources can be withdrawn.
- Material is selected **automatically by highest energy yield first**. The precise comparison metric when densities differ (yield per kg vs yield per whole litre) and deterministic tie-breaking still require an explicit design decision; do not silently claim either ranking is approved.
- The Core processes **whole-litre input batches with continuous energy output over time**, not instantaneous full-batch energy grants. The energy reserve can be charged while a batch is being processed.
- **The Core has a fixed maximum energy-generation POWER, measured in watts (J/s).** Actual conversion is bounded by this power rate, available convertible energy, and free energy-storage capacity. A more energy-rich litre can take longer to finish at the same maximum power.
- Conversion **automatically pauses** when the energy reserve is full, preserving remaining unconverted energy/material, and **automatically resumes** as energy capacity becomes available. It must not discard fuel or generated energy simply because the reserve is full.
- **The maximum generation power is upgradeable in the future.** This is distinct from upgrading the Core's raw-material buffer capacity or energy-storage capacity.
- **Design deliberately deferred:** The initial maximum generation power (in MW), the upgrade mechanism, upgrade tiers, maximum upgradeable rate, required resources/costs, prerequisites, and exact power/progress behavior must be **discussed and defined in a later design session**. This document does not authorize adopting old native conversion values or any example number as the new starting rate.
- Also deferred: how a partially converted 1 L batch is reserved/represented, whether it occupies original capacity throughout processing, what can be withdrawn during processing, save/resume and interruption details, and exact priority/tie-breaking semantics. Preserve material-energy conservation regardless.

## Persistence, authority and current implementation gap

- Source depletion and inventory credit must be authoritative and atomic so interruption, repeated input, saving/reloading or future multiplayer cannot create duplicate resources.
- `resourceInventory.ts` now uses whole-litre quantities, `gramsPerLitre` and `capacityGrams`. `resourceSession.ts` owns the sample catalog, gathering ticks and one atomic save envelope: schema/catalog/site version, inventory quantities and capacities, source remainders, revision and split frame-relative player location.
- `TopDownView.tsx` holds an exclusive browser Web Lock while the expedition is open. A second tab cannot gather/save. Each transaction checks the previous stored bytes and replaces the entire localStorage record before publishing its inventory credit. Failed/stale writes stop gathering without publishing the candidate. Unsupported/corrupt/nonconserving saves are preserved and block saving; they are never silently replaced by fresh resources.
- Completed litres save synchronously. Location checkpoints occur once per simulated second, on movement stop and page hide/blur; a hard interruption may lose only movement since the last checkpoint, never half of a resource transfer. Reload restores standing in the same frame; cockpit saves restore safely beside its chair. Tool equip state and partial progress do not resume. Future upgraded capacities are validated and retained; no upgrade UI/cost is introduced.
- This is single-browser, same-origin local persistence, requiring HTTPS or localhost for the lock. It is not a server, multiplayer authority, cloud sync or protection against manually edited/deleted browser data. Core batch/energy persistence is still unimplemented.
- Resource payload is derived from inventory grams. It contributes once to player payload and, while ship-local, once as nested payload in the existing ship simulation's total mass. Walking has no separate mass-based physics. Payload is recomputed from dry hull mass, never accumulated per frame.
- Older native Core fueling, physical-fragment construction refunds and storage/harvesting implementation notes remain historical only; they are not proof of 2D migration.

## Open design questions for later discussion

1. Initial fixed generation power (MW) and its **upgradeable rate progression**, separate from energy-storage upgrades.
2. Core priority yield metric (per litre versus per kilogram), deterministic tie-breaks, and currently processing batch behavior.
3. Core active-batch reservation, withdrawal, interruption and persistence.
4. Gathering tool upgrades/animation and broader resource catalogs/balance beyond the authored starting sample.
5. Storage/transfer UI, direct module transfer policy, and capacity upgrade tiers for inventory/storage/Fabricator/Core.

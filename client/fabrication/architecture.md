# Fabrication architecture

`salimon-fabrication` is an independent portable domain depending only on
`salimon-world` raw resource contracts and std. It does not call the ship,
runtime, renderer or world session. Canonical behavior is the
[accepted Fabricator specification](../../docs/spaceship-building-and-survival.md#accepted-fabricator-contracts-2026-10-19-336).

| Source | Ownership |
| --- | --- |
| `mass.rs` | Positive integer milligrams, one-time kg quantization |
| `recipes.rs` | Stable IDs, validated immutable recipe book, proposed starter tuning and unlock/category schema |
| `machine.rs` | Typed commands/results, bounded storage, atomic committed costs, stable jobs, FIFO stepping, single tray, observations and mass contribution |
| `lib.rs` | Reexports and typed errors |
| `tests/fabricator.rs` | Public transactions, edge arithmetic, power/tray/time boundaries and deliberate mass transitions |

`RecipeBook::new` accepts data records and validates the complete set before
publishing it. Callers may build records from a later data adapter; there is no
file format/parser in this crate. The two basic starter recipes are unlocked; the three advanced food recipes are locked. Advanced
requirements can be represented but cannot be acquired here. Jobs capture recipe
values so balances/progress never depend on later tuning. There are no setters
that mutate a validated recipe book inside a running machine.

`Fabricator::execute` dispatches the same public insert/enqueue/cancel/collect
methods. Bins use world catalog identities in stable catalog order. All balances
and recipe costs are integer milligrams; world positive finite kg round once to
the nearest mg (half upward). Sub-half-mg and overflowing values are rejected,
preserving the original fragment. Accepted insertions consume the entire original
fragment; a full bin rejects input without consuming it (#339). `Insertion` reports accepted
and discarded mg; runtime must remove the source exactly once. There is no raw
withdrawal operation. This crate cannot itself despawn physical entities.

`enqueue` checks positive quantity, technical job capacity, recipe/unlock, job-ID
space, multiplication and every balance before mutation. It then debits the full
batch and captures the job. No reserved material remains in raw storage and no
cancellation can credit it back. Capacity defaults to 64 queued jobs (including
active head), configurable at construction; quantity is not counted as extra jobs.
This is an implementation bound, not an approved gameplay upgrade/capacity.

`power_demand` exposes current watts and duration until output. `step(delta,
powered)` advances only the funded head and returns actual worked time/joules and
unused elapsed time. Runtime must split at Core exhaustion and settle every
returned worked interval; `powered` is a caller funding assertion, not an energy
storage implementation. Partial funding uses a funded substep plus an unpowered
substep. The machine cannot debit Core itself. Power demand may be nonzero in a
NoPower snapshot; it is requested, not actual, consumption. Idle and occupied tray
request zero watts. A step stops on its first output; blocked/unpowered/unused
time is never banked. Duration arithmetic is exact and bounded without a dt loop.

A tray item has stable `(JobId, unit)` identity. Collection removes it once; the
runtime must first validate the receiving slot. A final output may outlive its
completed job; cancel only applies to pending work and never deletes tray output.
The separate integer storage/tray mass components avoid an overflowing combined
u64 and exclude jobs/committed inputs/collected inventory deliberately. Runtime
maps these to cached ship ledger entries alongside world fragment removal.

All state remains in-memory; `FabricationSaveState` provides versioned save
source state through the runtime envelope. Ordinary snapshots are observations.
Filesystem persistence
must version material/recipe/job IDs, captured job parameters and state and
validate restoration atomically, as documented in the canonical specification.
Runtime/physical/power/UI adapters are separately tracked by #339–#347; native
reference signoff is #348. This crate alone does not make crafting playable.

`inventory.rs` provides the typed receiving slots required by deliberate tray
collection (#344): three food and two medical, one item each. Runtime validates
first-vacant placement before removing the tray under exclusive session ownership;
full/unsupported/duplicate placement preserves both domains. These slots are
independent of equipment, world fragments and ship mass. Consumption input is #345.

## Consumable inventory (#345)

`ConsumableInventory` owns exactly `FOOD_SLOT_COUNT = 3` food and
`MEDICAL_SLOT_COUNT = 2` medical slots. Placement derives category from the typed
output and takes the first vacant slot; equipment/component outputs are rejected.
`consume_slot(ConsumableSlot)` removes precisely the selected one-item slot
without shifting survivors or stacking items. The five enum variants preserve
food/medical category boundaries and stable HUD order; empty selection is a no-op. Empty input is
observational. There is no health/hunger/full-vitals check or healing effect.
Read-only slot arrays expose stable item/recipe/job identities for later HUD and
versioned save adapters; serialization is implemented, while disk IO remains #366.

## Whole-session saves (#365)

`FabricationSaveState` validates recipe definitions, ordered complete bins, queue
IDs/captured recipes/committed quantities/progress, allocator high-water marks,
tray and exact inventory slots together. Cross-owner item identities cannot be
both collected and on the tray. Runtime's `SessionState` is the concrete payload
inside `SaveGame`; `save_bytes` and transactional `restore_bytes` integrate all
physical state, equipment and exact elapsed/backlog clocks. Stored mass and
recipe watts are reconstructed without charging or refunding. UI/input/focus,
feedback and render caches reset; interactions are recomputed from restored
world state. The native host's automation policy stays host-owned. No disk IO,
autosave scheduling or offline simulation is implied; #366 owns durable storage
and shell lifecycle integration. See [canonical save contract](../../docs/save-contract.md).

## Exact consumption selection (#370)

The `ConsumableSlot` facade matches the integrated #377 wheel: five named
slots, `ALL`/`index`, observational `item` and exact `consume_slot`, without
compaction. New runtime consumers must identify `(slot, JobId, unit)` and reject
stale selection before removal. The category-first compatibility helper is removed;
no native M/N dispatch exists and no new first-available UI should use it.

Runtime's #370 adapter validates item/category/effects and prepares survival before
removing the one selected identity; fabrication itself still owns no health,
hunger, effect timer or full-vitals gate. Inventory/tray/raw mass semantics and
save representation are unchanged. See the
[recovery contract](../survival/architecture.md#consumable-recovery-definitions-370).

## New-run supplies (#379)

`Fabricator::starter_with_supplies` constructs the machine and existing typed
inventory together, using starter recipe definitions, exact bounded integer bins
and the shared job identity allocator. It is a constructor, never a replenishment
method. It performs no crafting, simulation or energy settlement. Runtime owns
when a new run is created and reconciles the ordinary reserve mass. See the
[canonical supply/save contract](../../docs/save-contract.md#starting-supplies-379).

## Advanced food definitions (#378)

The starter book contains five recipes: basic food ration and medkit unlocked,
plus locked nutrient gel, complete meal and energy drink. There is no unlock
acquisition operation. Existing FIFO, physical ingredient intake, Core funding,
tray, mass and three-food/two-medical inventory rules apply unchanged. Effect
definitions live in [survival](../survival/architecture.md#four-food-catalog-378).

Explicit provisional production tuning (subject to #384 balance):

| Recipe | Organic biomass | Water ice | Work | Power | Output mass |
| --- | --- | --- | --- | --- | --- |
| `nutrient-gel` | 0.20 kg | 0.15 kg | 10 s | 10 kW | 0.20 kg |
| `complete-meal` | 0.70 kg | 0.20 kg | 20 s | 15 kW | 0.50 kg |
| `energy-drink` | 0.10 kg | 0.25 kg | 10 s | 10 kW | 0.25 kg |

These recipes are data records, not a playable progression system. Tests explicitly
provide an unlocked fixture book to verify production/consumption/save integration;
the production starter book rejects all three advanced enqueue requests atomically.

## Future physical Jump Drive component fabrication (approved design, NOT SHIPPED)

See [Spatial Jump Drive and shared-time travel](../../docs/jump-drive-and-interstellar-travel.md). The reusable ship-mounted Jump Drive and its upgrades will use **physical Fabricator-built components**, **not** consumable 'wrap cells', food/medical slots, or a separate fuel currency. Actual jump costs are paid by the existing authoritative Core. The first module uses virtual speed-gap parameter **N=20%** per light-second; future upgrades may reduce N, but component recipes, unlock timing and raw material costs are **not yet approved**.

Reuse world material IDs and genuine mineral supply, the current **20 kg per-material bin**, strict FIFO job, Core-funded job work, single physical output tray, and one-carriable-world-object rule. A large assembly needs an explicit **future physical assembly/interface design** rather than placing an entire full-sized Jump Drive into the present consumable slots or silently expanding Fabricator capacity. No source code, recipe book, save schema or gameplay inventory is changed by this note.

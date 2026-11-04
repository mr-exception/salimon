# Resource inventory and gathering — strict top-down 2D

**Status: owner-approved design (2026-11-04).** Domain logic and tests are implemented in the browser client, but the navigation-only prototype has no 2D surface gameplay, mining controls, inventory UI or machine integration yet. Previous native fragment-carry work is historical.

## Resource storage

- Starting player inventory: **10 kg of weight and 10 L of volume**, enforced as independent caps. An item must fit in BOTH free capacities.
- Minimum granularity: **1 kg and 1 litre**. Each resource type declares positive integer kilograms-per-unit and litres-per-unit, and quantities are integers. A resource unit can weigh more than 1 kg or occupy more than 1 L. No fractional mass, volume or stored resource units.
- Stacks are stored by resource ID; total usage is derived from quantities and the catalog, never editable totals. Reject invalid values, unknown resource keys, and arithmetic overflow.
- Capacity upgrades are possible in the future, potentially increasing mass and volume independently. Starting capacities are fixed; upgrade unlocks, costs and tiers are not yet decided.
- Raw resource stacks do not occupy tool slots or the existing separately designed food/medkit slots. Physical non-resource props and installed furniture are distinct systems pending further 2D decisions.

## Direct extraction

- In strict top-down planetary/surface play, the player interacts with a valid mineral deposit, asteroid source, or harvestable biological source using a suitable tool. No mined sphere, fragment carrying, fragment drop physics or loose raw-fragment pile.
- Each successful extraction atomically credits whole resource units to inventory and debits the same quantity from the persistent source. Collect at most the requested units, source remaining units, and the free weight and volume capacities.
- If a request only partially fits, collect the amount that fits and leave the rest at the source. If nothing fits, do not change either state and report the limiting capacity to the player.
- Keep stable source IDs and depletion journals. A future authoritative/multiplayer implementation must use revision checks to prevent stale or repeated extraction from duplicating resources.
- Mining/harvesting yield adapters must deterministically convert old continuous mass rates to whole units and persist any fractional progress as non-collectible source remainder until a full unit accrues; never mint units through repeated rounding.

## Transfer, machine intake and persistence

- Material consumption is an exact whole-unit debit; insufficient balance rejects unchanged. New Core, Fabricator and construction interactions must consume from resource inventory rather than inserting physically carried fragments.
- Ship storage is a separate, **undecided** capacity/transfer system. Do not assume the player's 10 kg / 10 L inventory also governs the ship. Furnishings and special non-resource world items remain separate.
- Player resource mass affects relevant physical mass only once. A transfer to ship storage must not double-count the same mass. The current navigation-only runtime does NOT consume this inventory or change ship mass.
- Future saves will need catalog version, integer quantities, upgraded caps and consistent source-depletion transaction state. Older native physical fragment saves need explicit migration decisions, not silent reinterpretation.
- Legacy physical construction refunds, physical mounts, Fabricator raw input and Core fragment fueling are NOT integrated into this new domain yet.

## Implemented starting point

- client/src/resourceInventory.ts: pure immutable starting inventory, derived usage, direct capacity-limited source extraction, exact removal and independent-capacity increases.
- client/src/resourceInventory.test.ts: weight, volume, mixed resources, depletion, conservation, exact consumption, upgrade and invalid-input checks.
- Integration into the future top-down 2D world, UI, machine adapters, storage, saves, and concurrent authority remains future work.

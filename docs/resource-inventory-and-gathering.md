# Unified weight-based inventory, gathering and ship processing — top-down 2D

**Status (2026-11-05):** owner-approved weight-only revision superseding the prior dual 10 kg / 10 L design. The browser's current 2D mining and local saves use grams and one weight limit; generic count-item registration, atomic add/remove and the unified field-inventory UI are implemented, but gear item definitions, equipment UI and Core processing are **not** playable. Scout Resource Container
transfers are playable through #785. The older native physical-fragment system is historical, not current 2D gameplay. See [spacesuits, environmental hazards and gear storage](spacesuit-and-hazards.md).

## Universal inventory quantities and limits

- **No volume restriction** applies to player inventories, mining, harvesting, storage containers, Fabricator input or Energy Core fuel. Weight is the only physical capacity measure; use **integer grams** in durable state and calculations. Display kilograms with fractional precision when needed.
- Each object, resource and count-based item has mass. Total carried inventory mass is the sum of **all** raw-resource quantities and count-item stacks; all count against the same maximum weight. Do not round fractional kilograms to whole kilograms.
- The player's **default inventory has 12 shared slots and a maximum payload of 10 kg**. Slots are an organizational/count limit, not an additional physical dimension like volume. Gathering or adding an item must fit both the slot count and the one weight cap.
- Raw resources are recorded as **integer grams** and stacked by resource ID (one inventory slot per resource type, until the weight cap). There is no litre quantity, density-based quantity conversion, carried raw sphere, or separate fragment physics.
- Count-based item definitions specify **weight per item (integer grams)** and **maximum count per stack**. An item type with stack size 1 (e.g., each suit protection module) uses one slot for every item even if multiple copies are identical. Other items may stack up to their authored type limit. Invalid zero-mass, fractional-count or overflowing quantities are rejected.
- Item definitions and exact equipment weights, inventory expansion equipment and upgrade costs remain **TBD**. A future backpack may raise personal slots from 12 to 16 and add two suit module slots; another proposed module may grant **+20 kg** resource carrying capacity (the former +20 L part is removed). These are examples, not shipped upgrades. Equipped gear mass and its effect on personal carrying capacity need a separate rule; never silently double-count mass in the ship.
- All inventories/containers conserve item counts and gram mass exactly through transfers, crafting, persistence and multiplayer-authoritative transactions.

## Player mining and harvesting

- Source quantities/depletion and player resource credit use integer **grams**. For the current authored sample, equipping the gathering tool and using **E / Gather** collects up to **1,000 g per completed work interval** (1 kg/s baseline; approved hunger/fatigue multipliers apply), limited by remaining source mass, the player's free weight and the appropriate shared inventory slot.
- If less than 1 kg is available or fits, credit only the exact remaining grams at the next completed interval; never overfill or delete the source remainder. One atomic transaction debits exactly the mass credited. No mined fragments are created.
- Starting sample catalog `landing-resources:v2`: mineral source **60,000 g** and biological source **15,000 g**. IDs and the authored location remain unchanged. These mass totals match the prior illustrative 24 L at 2500 g/L and 30 L at 500 g/L, for safe save migration, but **density/volume no longer governs gameplay inventory**.
- Existing interaction reaches are 2.3 m mineral and 1.9 m biological. Movement, Escape, stowing the tool, leaving reach, losing focus/visibility and cockpit entry interrupt gathering. Partial work is checkpointed per source and retained across interruptions/reload; gathering itself requires explicit restart. See [approved tuning](browser-survival.md#approved-work-and-action-tuning-799). Completed mass and work debit share the atomic source/inventory transaction.
- A full-weight or full-slot message explains why collection stopped. Depleted sources preserve their stable IDs and collider identities and remain visibly depleted. Gathering tool upgrades and additional source types are **TBD**.

## Ship storage and equipment storage

| Destination/module | Weight capacity | Slots | Permitted contents |
| --- | --- | --- | --- |
| Player inventory | **10 kg**, starting | **12**, starting | Raw resources and count-based items |
| **Resource Container** (formerly general storage) | **50 kg**, starting | TBD if needed | **Raw resources only** |
| Fabricator raw-input buffer | **50 kg**, starting | TBD if needed | Raw resources for recipes; unused inputs withdrawable |
| Energy Core raw-fuel buffer | **50 kg**, starting | TBD if needed | Raw resources for conversion; unconsumed fuel withdrawable |
| **Spacesuit Drawer**, tier 1 / 2 / 3 | Weight cap TBD | **4 / 8 / 12 suit slots** | Complete, unequipped spacesuits |
| **Protection Module Storage**, tier 1 / 2 / 3 | Weight cap TBD | **12 / 16 / 24 module slots** | Uninstalled protection modules |

- **Storage modules do not have any volume capacities.** Weight remains the only physical capacity. Specific drawer/module-storage total weight limits are not yet approved. Their explicit suit/module slot counts remain enforced.
- One stored suit consumes one drawer slot, including any modules installed on it. A protection module occupies a separate module-storage or personal-inventory slot only when uninstalled; an installed module is part of its suit.
- The player starts wearing one basic suit, without suit tiers. Crafting additional suits/modules and upgrading them requires a future **Gear Crafting Bench** built inside the ship; crafting recipes, replacement/equip location, access and expansion are TBD.
- The player may travel outside without any suit, at environmental risk. No automatic suit/module switching based on hazards is approved.
- Players may withdraw **unconsumed** materials from the Resource Container, Fabricator or Core; resources already reserved/used by a fabrication job or converted by the Core are not recoverable. The Resource Container transfer UI is implemented in #785. Machine/module-to-module
transfers and resource container upgrade rules remain TBD.

## Fabricator and Energy Core material handling

- Fabricator jobs consume/reserve ingredients as **integer grams of raw material** with exact mass conservation. The previous 50 kg / 50 L input limit is now **50 kg only**. Preserve the existing queue, power pauses and output tray contract; integration of the new shared count-item inventory remains future work.
- The Energy Core has an independent **50 kg raw-material buffer**, separate from its energy reserve in joules and its maximum generation power in watts. Material conversion is **continuous over time**, pauses when the energy reserve is full, and resumes automatically when space for energy becomes available. Unconverted input is not lost.
- Core material yield and batch bookkeeping must be defined using **mass**, not whole litres. Its highest-yield selection, deterministic tie-breaks, initial/upgrade maximum generation power, batch reservation/withdrawal and save behavior remain unresolved; do not invent numbers or claim this is implemented.
- Each inventory and processing buffer must contribute its contained mass exactly once to ship simulation. Resource weight currently contributes to player payload, and to ship mass when the player is aboard. Registered carried count-item mass contributes through the same payload path; equipped-suit attribution remains pending.

## Save format, authority and migration

- The browser's pure `resourceInventory.ts` owns gram-based resource quantities, capacity grams/slots, and count-item stack/mass validation. `resourceSession.ts` owns initial 1 kg/s mining, stable sources, depletion and one atomic save envelope. The current playable HUD shows kg and slots, no litres/volume.
- The save envelope is **version 6** (`landing-resources:v2`), storing trusted item definitions, grams, slots, count-item map, source remaining grams, identified raw stores, player location, [survival state and launch watermark](browser-survival.md), and one revision. The existing browser storage key is deliberately retained. Previously valid v2 gram saves acquire empty declared stores without changing mass, capacities, location or revision; previously valid v1 whole-litre saves migrate **losslessly** using the old pinned sample densities for one-time conversion; their mined totals and source depletion remain conserved. Unsupported or corrupted saves remain preserved and block writes rather than silently resetting the expedition.
- The active tab holds an exclusive browser Web Lock; commits use compare-and-replace localStorage before publishing state. Movement checkpoints occur at most once per simulated second and on stop/page-hide. This is browser-local single-writer persistence, **not** future backend authority, anti-cheat security or cloud sync.

## Future Stasis Pod gear-loss and recoverable-loot boundary

The approved [Stasis Pod and Transfer contract](stasis-pod-and-transfer.md) affects future server-owned inventories; it is **not implemented** in browser-local `ResourceSession`. Each player character and each ship/module storage compartment has its own **separate inventory document**. Ordinary carried raw materials (integer grams), count-item stacks, equipped gear, worn suit and its installed modules are all subject to the same exact-identity loss policy; preserve mass and stack limits.

- **Safe pod logout:** no equipment loss; inventory and suit stay with the living character.
- **Unsafe exit:** debit ordinary carried contents and worn equipment **exactly once** into a persistent world loot cache at the validated departure frame. **Transfer** through an approved one-hour code does the same before binding/spawning at a friend's pod. No implicit equipment transport to that friend's ship. Ship/module cargo stored separately is unaffected.
- Explicitly classified unique/unobtainable **primary items** are reserved exceptions to unsafe/transfer loss only; there may be **no such authored items**. Their treatment on **actual permadeath** is TBD.
- The loot cache is an owned persistent world record, not a second free player inventory. Preserve individual suit/module identities, exact resource grams and revision-checked operation receipts. If departure frame or ship-to-wreck reconciliation is unresolved, **fail closed**; do not create or delete mass speculatively.
- **TBD:** loot recovery/decay/theft rights, unprotected gear stored at a remote ship, exact equipped-suit mass attribution, and frame conversion when a coasting ship is destroyed after departure.

## Outstanding decisions

- Suit base/expansion slot usage, worn suit payload, swapping locations, actual item weights and default inventory contents beyond the basic suit.
- Slots versus mass limits for future ship containers where only a weight capacity is currently approved, and weight caps for drawer/module storage.
- Gear bench, suit/module recipes, authored count-item content, machine transfers and upgrades.
- Core yield per gram, batch accounting, priority ranking, watts and continuous processing persistence.
- Final gathering rates, tools, full catalog/economy balance and high-capacity item stack constraints.

## Atomic multi-inventory foundation (#783)

`resourceLedger.ts` owns the validated raw-resource graph: player inventory,
fixed source IDs/materials and flat identified ship-side stores. A trusted
`ResourceStoreCatalog` declares each store's stable ID, ship ID and gram capacity;
saved metadata must match it exactly. The Scout enables `scout:storage`, a ship-local 50,000 g raw-only store. No
container slot count is introduced; #785 exposes the deposit/withdraw interface.

`transferResource` accepts an expected revision, typed source/destination,
resource ID and exact positive integer grams. Source → player/store, player ↔
store and store → store transactions either return a completely validated next
ledger at revision + 1 or throw without mutating inputs. Destination weight and
player shared-slot constraints are checked before constructing a debit. Unknown
IDs, material changes, negative/fractional/unsafe quantities, overflow, self
transfers, source refills and stale revisions are rejected. Every resource's
initial fixed-source mass equals its remaining source mass plus player/store
mass. The inventory's exact `addResource`/`removeResource` helpers never partially
succeed; only a complete ledger transaction may publish through the session.

`ResourceSession.transfer` uses the same production transaction as completed
mining intervals. It serializes the entire candidate envelope in one localStorage
replacement, under the existing lifetime Web Lock and raw-byte stale-write check.
Only successful persistence publishes source/inventory/stores/location at one
revision. Domain rejection leaves the session usable; storage failure cancels
mining and blocks further writes. The original storage key and catalog identity
are unchanged. Migration is in-memory until a successful normal commit; failed
migration writes preserve the original v1/v2 bytes. Unknown/corrupt/unsupported
save versions or store IDs/metadata block writes instead of resetting material.
Composition supplies the same trusted store declarations on reload. Schema v3
saves missing the newly introduced Scout container acquire it empty; schema v4
requires it, so a damaged current save cannot silently reset cargo. Existing
v1/v2 migration remains lossless and preserves original bytes until commit.

`resourceLedgerMass` exposes player grams, selected ship store grams and total
ship payload. `ResourceSession.payload` is used by `TopDownView`: stores count
once for their owning ship; player inventory counts once only while aboard.
Stores contain raw quantities, never nested player inventories or other stores.
Mass is recomputed from dry ship mass each frame, never accumulated per tick.
Gear mass beyond the existing item field and machine consumption/recipes are
still outside this foundation. Browser-local single-writer guarantees do not
establish multiplayer or backend authority.

## Scout Resource Container domain (#784)

`scoutResourceContainer.ts` binds the existing `scout:storage` anchor to the
50,000 g store on `ship:scout`. `ResourceSession.deposit` and `withdraw` require
raw resource ID, exact positive integer grams and expected revision. Requests
are all-or-nothing; a partial amount must be requested explicitly. No count
items, unknown raw IDs, volume cap or organizational slot cap are accepted.
Default sessions persist the identified container in the same atomic envelope
as player inventory/source depletion/location. Its mass counts once even while
the player is outside; onboard player mass remains separate. UI is described below.

## Playable container interface (#785)

Use the existing `scout:storage` anchor through shared nearby selection/reach.
The interface controller records an exact positive-gram draft and expected
revision; only confirmation calls `ResourceSession.deposit`/`withdraw`. It
consumes the confirmation before committing, so repeated input cannot replay it.
The modal shows exact kg derived from grams, remaining capacity and player slots;
no display rounding enters commands. Typed fields retain keys and the existing
gathering-tool context never changes a selected container into a mining target.

Escape/Close/focus loss/movement or reach/scene/cockpit changes discard only the
active draft. Successful transactions persist immediately and autonomous ship
state is retained. Invalid amount/resource, balance, weight/slots, revision and
save errors remain readable and non-destructive. Actual count-item content and
machine processing/transfers are still future work.

### Industrial equipment presentation (#829)

The production container uses the shared `EquipmentDialog` shell and scoped
Expedition Interface tokens. Paired carried/container capacity instruments show
kg, exact g and free capacity; the raw-resource table and whole-gram form remain
machine-specific. Pending review explicitly says nothing moved; committed feedback
appears only after the session publishes a saved transfer. Rejected/save-blocked
states explain failure in the live feedback region. Keyboard focus wraps within
the native modal, buttons include disabled reasons, and small desktop windows
scroll within the panel. Reduced motion removes the entry fade. Domain/save,
source depletion, interruption and all-or-nothing rules above are unchanged.
See [verification and original captures](../reports/issue-829/README.md).

## Generic count-item integration (#786)

`registerCountItems` validates every definition, even unused types, and freezes a
copy of authored positive integer grams/item and stack size (plus optional name).
The production `ITEM_CATALOG` is empty. No suit, tool, ration or module weights,
items, equipped slots or effects are introduced. A future producer/consumer calls
`ResourceSession.changeItem(action, id, count, expectedRevision)`; valid add/remove
publishes only after the complete ledger save succeeds. Domain rejection leaves
state/revision/bytes unchanged; failed or stale persistence blocks further writes.
Removing the last copy deletes its entry and frees its exact mass and slots.

The field HUD reports raw kg, item counts, item total kg and slots per type,
combined kg/slot limits and an explicit empty item state. Stack size 1 consumes
one slot per copy; other types consume `ceil(count / stackSize)`. Gathering and
storage withdrawal use that same combined capacity; failed credit never depletes
a source. Player payload includes item mass once while aboard.

Schema v6 extends the v5 item envelope with survival state and retains the storage key and raw catalog identity. Saved item definitions
must match the trusted catalog exactly; malformed/unknown/changed definitions or
counts preserve bytes and block writes. Valid v1–v4 saves migrate in memory with
no items, preserving raw mass, stores, location and revision. Historical production
catalogs were empty: nonempty legacy item maps are invalid, never assigned guessed
weights. Future authored catalog changes require an explicit migration rather
than silently reinterpreting saved counts.

`tests/items.html` is an explicit non-shipping entry point with synthetic test-only
stack=1 and stack=4 definitions. It injects the production session adapter into
the real `TopDownView` under the normal Web Lock. Its controls and weights are
absent from the distributed build; ordinary query parameters cannot enable them.

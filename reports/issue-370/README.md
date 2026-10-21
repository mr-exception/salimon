# Issue #370 — exact-item survival recovery

Implemented on `main` `5874fad`, after PR #396 merged #369. The dependency
implementations #346/#347/#368/#369 are present; no blocked draft was merged.

## Delivered

- Serializable validated recipe/item recovery catalog in the survival owner,
  independent of Fabricator recipes/storage. Integer milli-point instant/timed
  hunger/health/treatment definitions reuse the existing deterministic pipeline.
- Starter ration +40 hunger; explicit provisional medkit tuning +2.5 ordinary
  health each active second for 10 seconds (+25 total), one stack with refresh.
  Full meters allow waste; treatment-only wounds still block ordinary healing.
  Advanced food buffs/unlocks remain #378 and balance measurement remains #384.
- Exact named slot API matching the #377 draft, with no duplicate inventory and
  no category-first fallback. The runtime transaction captures slot/job/unit,
  checks view/crafting/terminal and emergency-consumption authority, validates
  recipe/category/effects on a clone, then commits one item plus effects together.
- Unknown/conflicting/invalid definitions, stale replacement clicks and denied
  use preserve inventory and survival. Exhaustion alone cannot prevent use.
  Consuming changes no ship mass, raw reserves, tray, queue or equipment.
- Shared portable `consume` request/inspection for eventual native wheel use;
  the existing collected-item scenario now uses that transaction instead of
  directly removing a first-occupied slot. M/N remain absent.
- Timed food added to the common effect pipeline. Damage still wins a shared
  lethal timestamp, with food/healing only afterward if alive. Serialized rules,
  partial ticks, durations and stacks continue exactly; offline fatigue recovery
  never advances these benefits.

[Canonical recovery contract](../../client/survival/architecture.md#consumable-recovery-definitions-370)
contains numeric units, tuning, bounds, stacking and failure semantics. Catalog
configuration is immutable once installed. Changing a persisted effect's numeric
rule under the same ID rejects rather than silently rewriting its history; later
save migration/versioned effect IDs belong to #371.

## Scope and integration

This completes #370's recovery definition/transaction layer without depending on
#377's unmerged UI. That wheel must capture its selected item's job/unit, call
`consume_selection`, and close only after success; it must not take inventory
first. The same slot facade is deliberately reused from the draft to reduce merge
conflicts. #377 owns native Q input/hover feedback, not a new M/N dispatcher here.

As established by #367–#369, #371 owns activating production vital clocks,
whole-game schema migration, atomic publication of survival plus consumed
inventory, and launch recovery. This PR does not start an unsaved clock or claim
full-game disk save integration. Domain recovery and inventory serialization are
tested together as its composition boundary. A later #371 checkpoint must not
publish consumption and survival as independent saves. Default playable vitals
remain full and timed effects await that admitted clock integration.

Native OS/GPU/M1 checks are unrun. Existing #376 owns full survival reference
signoff once the integration chain lands; this report does not mark it passed.
No graphics/assets, recipe costs, production time/watts/mass or unlocks changed.

## Validation

Linux x86_64; Rust stable 1.99.1. See [validation.txt](validation.txt) for final
workspace/build/lint/format/tooling outcomes and focused test names.

Domain cases cover clamping, exact bounded gradual healing, timed hunger and
wound treatment, partitioned fractional save continuation, offline freeze,
registry conflicts/duplicates, invalid intervals, repeated-kit refresh/stack cap,
and food arriving at the exact lethal starvation timestamp. Runtime cases cover
exact slot selection at fatigue zero, full-meter waste, atomic failures, stale
replacement identity, crafting/tour/terminal guards, real crafted/tray-collected
medical output through protocol, consumed inventory + active timer restoration,
and unchanged ship mass. Existing M/N retirement/input tests remain in force.

# Spacesuits, gear storage and environmental hazards

**Status (2026-11-05): owner-approved gameplay design, not yet shipped.** Extends [unified weight-based inventory and gathering](resource-inventory-and-gathering.md). The [browser hazard domain](browser-hazards.md) now implements temperature, radiation/poison exposure, exact module budgets and durable session integration (#788). Normal gameplay has no hazard sources or modules; protective suits, gear bench and equipment crafting remain unimplemented.

## Suit ownership, configuration and crafting

- The player **starts with one basic suit equipped**, may own multiple suits, can intentionally remove a suit and leave the ship unprotected, and manually chooses the suitable suit for each expedition. Suits do **not** have tiers.
- Each base suit has **two protection-module slots**. Protection modules are installed into a particular suit and can be upgraded using materials; suit modules, not suit tiers, define environmental specialization. Temperature, radiation, poisonous/acidic atmosphere, poison puddles, fire and ice protection are example families. A module affects only its supported hazard categories; cross-category coverage must be explicitly defined.
- Suits may also have passive movement-speed modifiers or physical-damage protection; exact values, compatibility and tier progression for these effects are **TBD**.
- A **Gear Crafting Bench**, built in the spaceship, will craft additional suits and modules and support future upgrades. Until built, the initial basic suit remains usable; exact bench recipes, research gates, swapping locations and installation rules are **TBD**.
- Potential later expansion examples (not approved as working mechanics): a backpack granting personal inventory **12 → 16 slots** plus **two additional suit module slots**; a resource pack granting **+20 kg** weight capacity. Whether these occupy the two base protection slots, separate attachment slots or other positions is **TBD**. No volume stat applies.

## Storage

- **Spacesuit Drawer** tier 1/2/3 stores **4 / 8 / 12 suits**, one complete unequipped suit per slot, modules included.
- **Protection Module Storage** tier 1/2/3 stores **12 / 16 / 24 uninstalled modules**, one per slot when the item stack size is 1.
- Uninstalled modules can instead occupy shared player inventory slots and their mass contributes to the player's **10 kg** default weight cap. Complete suits/gear are count-based items when carried; exact item masses are TBD.
- The former resource storage module is renamed **Resource Container**, holds only raw resources and starts at **50 kg**. Ship-side storage has **no volume constraints**; drawer and module-storage total weight caps are TBD.

## Universal player health and hazard intensity

- Base player maximum health is fixed at **200 HP**; no maximum-health upgrade is approved yet.
- Hazard intensity and module protection use **HP per second**, including fractional values. Multiple simultaneous sources of the *same type* add their intensity before protection is subtracted **once**. Different hazard types accumulate and recover independently, but all actual damage affects the same 200 HP pool.
- Effective intensity = `max(0, sum(hazard sources HP/s) - effective module protection HP/s)`. This is the rate used for direct hazard damage or the hazard's exposure buildup, depending on hazard behavior.
- **Exposure hazards:** radiation and poisonous atmospheres/puddles build a separate 0–100 exposure meter before dealing health damage; outside exposure there is no direct exposure-hazard damage. The buildup rate is the effective intensity in points/s. At **100 points**, that hazard's HP-damage effect activates while exposed; exiting the hazard stops exposure-only damage immediately.
- **Direct + buildup hazards:** fire and the selected acidic-burn example deal effective HP damage immediately while exposed and also build a 0–100 meter. Full fire buildup causes **approximately five seconds of lingering burn damage** after leaving the fire; precise tick/refresh/cancellation rules remain TBD. Example: 10 HP/s acid plus 2 HP/s extra when full, against 9 HP/s protection, yields 1 HP/s initially and 3 HP/s at full buildup.
- **Direct-only hazards:** temperature differences produce HP damage while exposed without a buildup meter or lingering damage. Reference ideal temperature is **24°C**; raw damage is `abs(temperatureC - 24) / 10` HP/s. A temperature module with 2 HP/s protection makes **4°C–44°C** safe inclusive under this formula.
- All exposure meters recover **independently** after leaving the corresponding hazard: **5 points/s + active corresponding module protection HP/s**. Recovery requires no module durability. Reentering before reaching zero retains the residual exposure. Recovery modifiers and any treatment equipment are future decisions.

## Module protection and destruction

- Every protection module has a universal **1,000 HP absorption budget**, separate from its protection strength in HP/s.
- It loses durability **only** when it actually blocks incoming hazard intensity, including preventing buildup on exposure hazards. Consumption per second is `min(incoming intensity HP/s, available protection HP/s)`, subject to the budget left. Do **not** charge twice for preventing both exposure and health damage from the same intensity.
- No passive durability loss from time equipped or distance traveled. Faster buildup recovery does **not** spend durability.
- When the remaining budget reaches zero, the module is **destroyed permanently** and provides no protection or recovery bonus. It needs replacement, not repair. Precise mid-tick destruction and installed-item removal presentation are implementation details.
- Protective strength upgrades can change HP/s absorption but the universal 1,000 HP capacity stays the baseline unless a separate owner decision changes it.

## Not yet specified

- Exact damage and buildup rules for every hazard category (especially pressure, oxygen, gravity and chemical status).
- Hazard intensity sources, authoritative multiplayer sampling and combined exposure behavior at transitions.
- Worn suit physical mass, base suit stats and equipment-vs-inventory mass attribution.
- Suit switching/accessibility, module compatibility, bench recipes, storage weight caps, post-destruction notifications and medical treatment.

# Spacesuits, gear storage and environmental hazards

**Approved contract (#913), 2026-11-26; not an implementation claim.**
The existing [hazard domain](browser-hazards.md) and
[Core/Fabricator](browser-core-fabricator-vertical-slice.md) are foundations.
Delivery is split across #914–#922. Persistent equipment (#914), ship-only transactions (#915) and raw environmental queries (#916) now have separate source owners; live hazards, bench and equipment/scanner UI remain later work. See [current browser status](current-client-status.md).

## Equipment and custody

A tierless basic suit weighs **3,000 g**, has **1,000 HP repairable durability**
and **two module slots**, with no intrinsic protection or movement penalty.
A fresh character starts with this suit equipped and one temperature T1 installed;
the gathering tool is unchanged. Removing the suit is allowed. Zero-HP suits
remain equipped but inactive until deliberately removed/replaced; their modules
remain intact. No new source of suit damage is approved in this slice.

Temperature, radiation and poison modules have tiers T1/T2/T3: **2/5/10 HP/s**
protection, **500/1,000/1,500 g** mass, and **1,000 HP** nonrepairable absorption
budget each. Higher tiers are crafted independently, not upgrades consuming an
older instance. Same-type modules add; different types act independently.
Durability uses exact rational HP, never a rounded display value.

Every suit/module has one persistent unique identity, trusted definition and
single ownership location. A module belongs to inventory, module storage, or one
specific slot on one suit. A suit belongs to inventory, drawer, or the equipped
location. Stored/carried suits retain installed module identities and durability.
Destroyed modules become unavailable permanently; preserve tombstone identities
so reload cannot resurrect/reuse them. Malformed definitions, duplicate IDs,
aliased ownership and out-of-range durability reject the whole save.

Equip/unequip, suit switching, installation and transfers are atomic and allowed
only **inside Scout**, with current ship-frame/reach admission checked at commit.
No implicit swap, discard or field equipment change. Full destinations reject
without changing either endpoint; failed/stale writes publish no changed state.

Unequipped carried suits and loose modules each use one shared inventory slot.
A carried suit's mass includes its installed modules. Equipped gear consumes no
personal carry capacity; all gear aboard, including equipped gear, counts exactly
once in ship payload. Gear outside Scout does not count aboard. No volume stat.
The preinstalled tier-1 **Spacesuit Drawer** stores **4** suits (tiers 2/3: **8/12**);
preinstalled **Protection Module Storage** stores **12** loose modules (tiers 2/3:
**16/24**). No additional storage mass cap. Actual storage upgrades are deferred.
The raw-only Resource Container remains separate.

Legacy saves gain the starter set exactly once when migrated to the equipment
schema. Ordinary reload never grants gear. Existing progress, ledger, depletion,
machine state and earlier migrations survive. Explicit terminal new-character
reset retains the existing expedition world/ledger policy but replaces all old
character equipment with one fresh starter set; advance a persisted equipment
generation so old IDs cannot reappear. This supersedes #788's retain-module-budget
reset for character equipment. It grants no raw materials or other items.

## Environmental publication and safe boundary

#916 owns a versioned, immutable supplemental environmental publication, keyed
by canonical packaged world seed/body identity and body-fixed region coordinates.
It must not change existing terrain/resource digests or infer atmosphere from names.
Use canonical sorted-key UTF-8 FNV-1a-64 hashing (the existing definition digest),
domain-separated by environmental version and field. Regional temperature is a
bounded deterministic integer in **−180..180°C**; body-wide radiation and toxicity
baselines are bounded deterministic integers in **0..20 HP/s**. Scanner summaries
state the supported range separately from a local sample. Queries use bounded
work with no mutable random generator, elapsed time or cache residency inputs.
Missing legacy publication binds explicitly to environmental v1; unknown future
versions reject rather than silently resample. Catalog revisions retain their
saved canonical body reference. Exact hash keys/bucketing belong to the versioned
implementation and its tests; algorithm changes require a new publication.

Vesper's authored **starting region** is safe: on the positive-Y hemisphere,
inside the inclusive **±128 m east/north square** in the canonical starting tangent
chart centered on radial +Y. Boundary is physical, independent of landing position,
walking chart or region cache. It is **24°C, 0 radiation, 0 toxicity**; beyond it
use generated values. Scout interior is always the same safe sample, regardless
of door or Core power. No weather, day/night, pressure or oxygen simulation.

## Exposure, absorption and health

Player maximum health remains **200 HP**. Temperature directly damages at
`abs(T − 24) / 10 HP/s`, without buildup. T1 makes 4..44°C safe inclusive.
Radiation and poison independently accumulate **0..100** exposure points at
`max(0, incoming − active protection)` points/s, then damage at that same HP/s
only during the exposed time at 100. Same-type source rates sum before protection;
different types share only the health pool. Leaving a hazard stops its damage
immediately and recovers its meter at **5 + active protection** points/s, without
budget cost. Reentry retains residual exposure.

Actual blocked intensity consumes durability once, including prevented buildup.
For active same-type modules with strengths p_i, let B = min(incoming, sum(p_i)).
Each spends **B × p_i / sum(p_i)** HP/s. Split exactly at the earliest depletion,
remove the exhausted module from its slot atomically, then recompute for remaining
time/modules. Stable-ID sequential allocation in #788 is **superseded by this
proportional policy** for #917; source/ID ordering cannot favor a module. No passive
wear, no recovery wear, no double charge for exposure and damage. Zero suit HP
disables intact installed modules. Passive health regeneration is suppressed
while actual environmental HP damage occurs. Use ResourceSession's sole admitted
60 Hz clock: pause/hidden/offline admit no hazard or machine progress. Health,
exposure, equipment budgets/removal and events commit or roll back together.

## Gear Crafting Bench (#918) and repair (#919)

Preinstall one reachable bench: separate **50 kg raw input**, one job, one output
item tray, **2 kW active draw**, explicit claim, no cancellation/offline progress.
Reuse exact reservations/revision checks and Core accounting. On each sole world
tick, convert Core fuel first, then pay **Fabricator before Gear Crafting Bench**;
only a fully paid tick progresses its job. Insufficient energy pauses without
partial progress or double spending. A full tray blocks starting another job;
full carry leaves output untouched. Unreserved input alone can be withdrawn.

| Recipe | Mineral g | Biomass g | Seconds | Output g |
| --- | ---: | ---: | ---: | ---: |
| Basic suit | 2000 | 1000 | 60 | 3000 |
| Temperature T1 | 400 | 100 | 15 | 500 |
| Radiation T1 | 500 | 0 | 15 | 500 |
| Poison T1 | 200 | 300 | 15 | 500 |
| Any T2 module | 1500 | 500 | 30 | 1000 |
| Any T3 module | 3000 | 1000 | 60 | 1500 |

Recipe input minus output is declared consumed material with no byproduct, never
extra payload. Claim creates a unique fresh full-durability instance once.

Repair reserves an **unequipped** suit identity (including zero HP), attached
modules unchanged, and requested missing HP, in the same single-job bench.
The suit leaves its source into exclusive bench custody; it cannot be equipped,
transferred or reserved twice. No full-health suit or module repair. Choose exact
positive rational HP up to missing durability; charge **ceil(requested HP) g
mineral**, restore only requested HP, no free fractional repair. Progress is
**20 HP/s**, **2 kW**; at most **1,000 g / 50 seconds**, with a final partial
tick charge rounded up to the Core’s exact 1/60 J unit and no unpaid restoration.
The material ceiling and energy ceiling are independent; HP restoration never
rounds up. The suit must be carried to the bench first; drawer access retains
its own proximity requirement. Bench custody locks attached module changes too.
Terminal new-character reset preserves bench-held suit/module identities as
machine contents across the new character generation. Completion moves the same identity into
the output tray, then explicit capacity-checked claim returns it to inventory.
No cancellation; power loss pauses, reload retains custody, failed writes roll
back all endpoints. Repair above zero reactivates a destroyed suit.

## Presentation and exclusions

#920 adds shared Expedition Interface equipment/storage UI with context actions,
auto-approach, accessible disabled reasons, confirmation and focus/Escape rules.
Show actual committed suit HP, two slots, module type/tier/protection/HP, capacities,
outside temperature, actual damage HP/s and independent exposure meters. Warn
once on downward **20% / 5%** module-budget crossings, with a prominent temporary
destruction notice; clear resolved warnings, avoid color-only signals and spam.

#921 extends only scanner-authorized contacts: body range/baselines, local values
only for an actually supported selected landing site, and current functioning
gear assessment. Unknown/unscanned information stays unknown; lost scan clears
stale assessment. Missing/insufficient gear warns but never blocks landing.

Future fire/acid/impact, oxygen/pressure, weather, ship construction, medical,
backpacks and multiplayer are outside this slice. Historical fire/burn/acid
examples are not current implementation requirements. #922 owns the combined
actual-control expedition acceptance; controlled damaged-suit fixtures prove
repair because natural suit damage is deferred.


### Repair delivery (#919)

`gearBench.ts` persists exact requested/initial HP and completed repair receipts,
including mineral and paid-energy derivation. Optional repair fields extend the
v19 bench state compatibly; older v19 craft-only saves remain valid. Reserved
suits contribute stored ship mass exactly once through equipment ownership;
completed repairs do not add a second tray suit. Claim returns that identity to
inventory only after both capacity checks pass. Full/partial/fractional, zero-HP,
custody, rollback and terminal-reset cases are in `suitRepair.test.ts`; the built
production console flow is `REPAIR-919` in `tests/gear-bench.spec.ts`.

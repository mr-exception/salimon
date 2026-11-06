# Browser hazard domain (#788)

`client/src/hazards.ts` implements the deterministic subset of the approved
[spacesuit design](spacesuit-and-hazards.md). Normal gameplay declares **no hazard
sources or modules**. This is a simulation/save boundary for later environment
sampling, equipment and presentation, without suit items, masses or balance data.

`ResourceSession.setHazardSources` accepts explicit stable-ID temperature readings
or radiation/poison HP/s rates. The environment adapter replaces this sample at
transitions; it is not saved or inferred from the landing illustration. Positive
same-type intensities add exactly before protection is applied once. Source order
has no effect. Temperature contributes `abs(temperatureC - 24) / 10` HP/s;
2 HP/s protection covers 4–44°C inclusive. Radiation and poison independently
build 0–100 points at the effective HP/s rate and deal that effective HP/s damage
only for time at full exposure while present. A threshold crossed within a tick
uses only the remaining fraction for damage. Temperature has no exposure meter.
Zero incoming intensity is treated as outside that hazard: exposure recovers at
5 points/s plus active corresponding protection. Recovery stops at zero, spends
no budget and never causes exposure-only health damage. Reentry keeps residual
exposure. Different types use the same 200 HP pool.

Explicit `addProtection` declarations start with exactly 1,000 HP absorption.
Protection spends `min(incoming, available protection)` over the time actually
blocked, once whether preventing exposure or full-exposure damage. The domain
splits a 1/60-second tick exactly at budget exhaustion; the remainder has the
surviving protection only. Zero-budget modules stay as destroyed ID tombstones,
provide no prevention/recovery bonus, and cannot be re-added under the same ID.
Multiple declarations consume intensity in stable module-ID order. This allocation
rule is a deterministic domain detail, not a suit-slot or installation policy.
Snapshots preserve per-type exposure and each module's identity, strength, type
and remaining budget. Per-advance events report exact blocked intensity, hazard
damage and destroyed IDs; failed transactions discard their events.

`exact.ts` uses reduced BigInt rational arithmetic. A number declaration means its
JavaScript decimal spelling exactly (for example 0.1 + 0.2 source declarations sum
to 3/10); callers should declare sources separately instead of first adding binary
floats. Rates must be finite, nonnegative and no larger than MAX_SAFE_INTEGER;
temperatures are finite within ±MAX_SAFE_INTEGER. Saved rational strings have a
bounded canonical numerator/positive denominator and validate before publication.
Exposure, durability and fractional HP retain exact rational state across ticks
and saves. `survival.health` is a numeric projection for existing consumers;
optional `healthExact` is the authority whenever HP is fractional. Integer health
omits it. Starvation and regeneration update that same authority. All damage
precedes passive regeneration and lethal damage prevents revival in that tick.

The existing survival loop is the sole 60 Hz authority, including navigation.
Pause/background admit no ticks; offline launch recovery changes fatigue only.
Duplicate/older world ticks cannot replay hazard simulation. Source sampling must
split calls at a change in environment; a batch uses one sample for all its ticks.
Health, exposure and budget are committed atomically with the resource ledger
**before hazard progress is published**, including partial seconds. Failed/stale
writes keep the previous whole revision and freeze play; reload cannot grant a
module or erase committed exposure. This currently writes once per admitted
hazard advance; future sampling must retain these durability guarantees when
optimizing checkpoint frequency. With no incoming hazards or residual exposure,
the existing survival checkpoint schedule is unchanged.

Schema v7 keeps the existing storage key. V6 health/timers and all inventory,
stores, source depletion and location migrate losslessly with zero exposure and
no modules. V1–v5 retain their existing survival migration. Corrupt/unsupported
saves are retained and block play. Explicit terminal fresh-character reset clears
character exposure with health/hunger/fatigue; it retains module budgets and
destroyed IDs as well as the resource ledger, so it grants no replacement gear.

`src/hazards.test.ts` covers source permutations, fractional rates, threshold
splits, safe temperatures, independent recovery, exact 1,000 HP depletion,
mid-tick destruction, survival damage/recovery ordering, reload, failed/stale
writes and tombstones. `tests/hazards.spec.ts` runs a separately built,
non-shipping `tests/hazards.html` against production session/clock/save adapters
with explicit synthetic sources/modules and write-failure injection. Playwright
builds it in `.hazard-fixture` and serves it on 4275; ordinary `dist` includes
only the normal game. The suite also rejects fixture-query activation in the
normal built game. No visual baselines are changed or promoted.

Deferred: fire and acid timing/defaults, pressure/oxygen/gravity, real planetary
sampling, wearable suit UI and installation, crafting, storage weight caps,
multiplayer authority and module-destruction presentation.

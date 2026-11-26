## Live planetary bridge (#917)

`ResourceSession.currentEnvironment` derives the admitted tick's physical player
frame and canonical surface coordinates. Scout interior always samples 24°C and
zero radiation/toxicity, independent of the door/Core. Production modules derive
from functioning persistent equipment; exact wear updates those identities and
moves exhausted modules to destroyed tombstones in the same health/exposure save.
The legacy explicit source/module declaration API remains a controlled fixture
boundary, with no production UI caller. Same-type absorption is proportional,
split at depletion; actual environmental damage resets passive recovery.
No admitted ticks means no environmental work or offline catch-up.

Focused checks: `liveHazards.test.ts`, `hazards.test.ts`, `survival.test.ts` and
built `tests/live-hazards.spec.ts`. Original evidence belongs in ignored
`client/test-results/` and CI artifacts; the PR contains the validation record.

# Browser hazard domain (#788)

**Current contract (#913–#920):** [planetary survival and equipment](spacesuit-and-hazards.md) governs persistent equipment, proportional wear, live sampling, crafting/repair and UI. Legacy synthetic hazard fixtures remain separate from production equipment; historical module declarations are retained for those fixtures.


`client/src/hazards.ts` implements the deterministic subset of the approved
[spacesuit design](spacesuit-and-hazards.md). The production bridge is delivered in #917, using persistent equipment (#914/#915) and [environmental queries](browser-planet-surface.md#deterministic-planetary-environment-916). Equipment/storage and contextual hazard presentation are delivered by #920. Explicit declarations below describe the retained synthetic fixture API, not extra production protection.

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
Multiple active same-type modules share blocked intensity proportionally to
their strengths, recomputing after each exact depletion (#917). Stable-ID
sequential allocation is superseded in both domain and production paths.
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
omits it. Positive exact HP below the numeric range projects to MIN_VALUE so
underflow cannot end a living run. Starvation and regeneration update that same authority. All damage
precedes passive regeneration; actual production environmental damage suppresses
regeneration, and lethal damage prevents revival in that tick.

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
character exposure with health/hunger/fatigue. Production replaces character
equipment once under a new generation, retaining machine-held repair custody;
legacy explicit fixture modules retain their budgets/tombstones. World resource
accounts and machine contents survive. See the current reset contract.

`src/hazards.test.ts` covers source permutations, fractional rates, threshold
splits, safe temperatures, independent recovery, exact 1,000 HP depletion,
mid-tick destruction, survival damage/recovery ordering, reload, failed/stale
writes and tombstones. `tests/hazards.spec.ts` runs a separately built,
non-shipping `tests/hazards.html` against production session/clock/save adapters
with explicit synthetic sources/modules and write-failure injection. Playwright
builds it in `.hazard-fixture` and serves it on 4275; ordinary `dist` includes
only the normal game. The suite also rejects fixture-query activation in the
normal built game. No visual baselines are changed or promoted.

Deferred: fire and acid timing/defaults, pressure/oxygen/gravity and multiplayer
authority. Real planetary sampling, wearable equipment UI, installation, crafting,
repair and destruction presentation are delivered (#917–#920). Equipment storage
has no extra weight cap by design.

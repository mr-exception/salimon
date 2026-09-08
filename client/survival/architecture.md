# Survival authority (#367)

`salimon-survival` owns validated hunger/fatigue/health, active duration, starvation
fraction and offline recovery receipt. No system clock, platform, rendering or IO.
Typed commands clamp additions/subtractions and cannot resurrect a terminal run.
Private fixed-point state retains every admitted nanosecond; snapshots alone
convert to floating point for presentation. Default hunger drains in 1800 active
seconds; fatigue in 64800; starvation removes one point each 10 seconds after
hunger reaches zero. A lethal boundary truncates active duration and all updates.
Numeric configuration permits positive whole-second divisors of 64800 for full
meter durations (the exact arithmetic lattice), and starvation intervals (0,3600s].

Hosts admit elapsed real time while alive, including ordinary menus; render frame
count is never time. Runtime's existing UpdateClock supplies admitted elapsed and
excludes explicit OS-suspend/renderer-reconstruction lifecycle gaps.
Occlusion and zero-size presentation continue bounded gameplay ticks (#372). No implicit
recovery occurs on focus change, suspend/resume, pause or save. A future full pause
(#381) excludes authoritative elapsed explicitly. Unreported stalls retain time;
reported OS lifecycle exclusions do not become active time or offline recovery.

`recover_offline(checkpoint_utc, launch_utc)` is a launch-only transaction: only
fatigue recovers, capped at one configured full recovery (default six hours).
Rollback grants zero and retains the largest credited UTC watermark; repeat calls
in the same launch do nothing. Crash recovery uses the verified checkpoint time,
so the interval since that checkpoint is conservatively treated as closed time,
with a six-hour cap. This intentionally cannot distinguish process execution
lost in a crash from genuinely closed time. Save capture itself grants nothing.

Runtime #371 composes this owner into versioned full-game saves, admits active
durations and performs a launch-only verified fatigue-credit transaction. Old
schema-1 worlds migrate once with full new meters and zero survival active time;
no old hunger/health/effect progress is invented. See the
[canonical lifecycle/save contract](../../docs/save-contract.md#survival-lifecycle-and-launch-transaction-371).

## Damage and status pipeline (#368)

`effects.rs` owns validated data definitions; `pipeline.rs` owns execution. Source
registration admits typed `Registered(u16)` sources without new runtime geometry.
Numeric definitions are serialized data; behavior is a closed Rust enum. No
radiation zones, fire geometry, fall collision damage or armor equipment is added.

Damage order is source flat resistance, armor flat resistance, source percent,
armor percent, additive source/debuff amplification, per-source cap. Integer
multiplication divides down at each percent stage at fixed-point precision.
Basis points are bounded; amplification is source-specific. Commands and effects
use the same pipeline as starvation. Ordinary healing/regen cannot fill the wound
portion of missing health; treatment reduces that debt and heals. Regeneration
adds one point every 60 eligible seconds (hunger >=70, fatigue >=50), retaining
fractions while eligible and resetting the interval at the first ineligible
timestamp, before periodic food, later commands or checkpoint capture can restore
eligibility. Exact hunger=70/fatigue=50 remains eligible; a broken interval never
contributes credit to the next one.

Advance splits at meter/regen/starvation/effect/expiry boundaries. At a shared
timestamp: drain meters and advance timers; collect starvation and periodic damage;
sort damage by Source; apply the whole damage batch; if alive apply periodic heals
by EffectId then regeneration; expire effects last (final tick is included).
Death beats simultaneous healing. Cause is the highest configured death priority
among nonzero contributors at the lethal timestamp, ties resolved by Source Ord.
Terminal state freezes subsequent time/commands, including remaining status timers.

EffectId order is canonical, independent of definition/application order. Repeat
application increases stacks to a bounded cap and refreshes duration but preserves
tick cadence. Exposure updates are idempotent; callers aggregate overlapping zones.
Radiation can use Exposure with zero linger; a burn can use nonzero linger.
Persistent and timed effects support dispel or treatment-only removal. Repeated
exit notifications cannot extend linger. Offline recovery never advances effects.

Validate definitions and decoded state before publication, including source/ID
uniqueness, durations/intervals, stack limits, armor, wound debt and death cause.
Registry bounds are 256 sources/effects; intervals are 1ms..1day, timed/linger
limits one day. Execution costs scale with actual effect events, not render frames.
Runtime #371 owns full-game persistence, native clock admission, terminal world
freeze and restart publication; #370 composes consumable effects. #372 owns
journey distance sampling and richer reports.

## Movement and interaction restrictions (#369)

`restrictions.rs` derives an exact basis-point DTO and typed action permissions
from the current snapshot. Hunger >=50 is neutral; [20,50) is 90%; [0,20) is
75%. Fatigue (50,100] is neutral; (20,50] is 85%; (0,20] is 65%; exactly zero
is 30%. Fractional points follow these continuous intervals (50.001 fatigue is
neutral; 20.001 is 85%). Combine movement multiplicatively, then apply the
configured minimum. The DTO retains both component percentages and the final
percentage for #373's persistent penalty labels.

`RestrictionConfig` validates serializable tuning. Explicit provisional defaults
are a 10% minimum movement multiplier, 75% interaction work rate for hunger <20,
75% for fatigue <=20, combined multiplicatively, and a 250 ms base interaction
recovery. These are tuning values, not fixed lore or changes to approved meter
durations; #384 may tune them. Continuous mining receives the reduced admitted
work duration; discrete nonessential actions receive an additional recovery of
`base * (1/rate - 1)`. Normal interaction rate adds no delay. The minimum never
turns a forbidden action into an allowed one.

At fatigue zero allow voluntary slow translation, emergency food/medicine,
doors and basic menus; deny mining, piloting, Fabricator operations, carrying
and Core feeding. Terminal state permits only menus. Runtime applies the policy
at shared command/tick boundaries, interrupts attached mining, closes crafting
and revokes piloting. This revocation deliberately selects zero manual throttle,
clears all steering/held throttle and cockpit control, preserving world velocity,
heading and committed assists. Recovery needs fresh cockpit entry and input;
it never resurrects held controls or automatically restarts thrust. Ordinary
cockpit exit and power loss retain their existing throttle-preservation rules.

Character receives a transient voluntary-speed multiplier in cabin/ramp/surface/
EVA movement; gravity, jump impulse, inherited drift, collisions and elapsed time
remain physical. Existing zero-G momentum is not artificially braked. Fabricator
jobs, Core and ship physics continue at full admitted time, even when the player
cannot operate the machine. Emergency interactions bypass recovery delays.

Runtime #371 activates the survival clock and persists source state with the
whole session. #370 owns consumable benefits, #377 final native wheel dispatch,
and #373 the persistent HUD. Automation exposes exact restriction basis points.

## Consumable recovery definitions (#370)

`consumables.rs` owns serializable, validated `Catalog`/`Definition` data keyed
by the stable Fabricator recipe ID string. Fabrication owns no vital mechanics.
Each definition declares Food/Medical category and bounded benefits in integer
milli-points (1000 = one meter point). Benefits are instant or timed hunger,
ordinary-health or treatment-health recovery. Treatment uses the existing wound
pipeline; no new medical lore, disease or status acquisition is introduced.

| Starter item ID | Recovery data | Status |
| --- | --- | --- |
| `food-ration` | +40 hunger instantly, clamped to 100 | Basic-ration default from approved expansion #378, still subject to #384 balance |
| `medkit` | +2.5 ordinary health each 1 active second for 10 seconds (+25 total), stack cap 1 | Explicit provisional tuning for review/balance, not an owner-approved medical efficacy claim |

Starter medical EffectId is 10000. Reusing an ID with different rules rejects
atomically; catalogs reject duplicate item/effect IDs. Timed definitions require
positive 1ms..1day intervals, positive <=1day lifetimes, an integral number of
intervals, and the shared stack bounds. Definitions contain at most 16 benefits;
catalog/effect registries retain the 256-entry bound. Decode then `Validate`
before installing configuration or restored authoritative state.

`apply_consumable` validates and prepares all registration/benefits on a clone,
then commits once. A failed definition, conflicting ID or terminal run changes
nothing. There is no full-health/full-hunger rejection: current gains clamp at
100 and excess is wasted, while timed benefits retain their configured lifetime.
Repeated legitimate medkits refresh the existing duration without exceeding one
stack or resetting its partial tick cadence; no unbounded parallel healing is
created. A stale click must not be treated as another legitimate item use by the
host. Special treatment definitions can use `Healing::Treatment`; food
buff behavior is implemented by #378 on the same typed effect registry.

Periodic food now uses `Behavior::Feed` in the existing deterministic pipeline.
At a shared timestamp damage still runs first; only a living state receives food,
then periodic health recovery and natural regeneration, with expiry last. Food
resets the starvation interval through the same Feed command. Death cannot be
averted by food/healing scheduled for the lethal timestamp.

Active recovery rule definitions, stacks, duration and fractional tick are part
of serialized `Survival`; validate and restore without stepping. Offline recovery
only changes fatigue, never these timers or benefits. Runtime #371 composes
this owner with consumed inventory in one verified whole-game checkpoint and
activates the production clock. Saved catalog rules must agree with already
captured registry IDs; conflicting definitions reject before restoration. Native Q-wheel
selection/hover presentation belongs #377 and consumes through runtime's exact
item transaction; M/N remain absent. Four-food definitions are implemented by #378; acquisition stays deferred.


## Observational effect snapshots (#373)

`effect_snapshots()` yields immutable captured `EffectRule` references plus live
stacks and optional remaining duration in stable EffectId order. Runtime can
present behavior, source, severity and removal policy without interpreting private
serialized state. This API performs no registration, advancement or mutation.
The [runtime HUD contract](../runtime/architecture.md#survival-hud-and-status-details-373)
owns text, visual thresholds, animations and native input; none belong here.

## Four-food catalog (#378)

New runs capture the following validated, serialized recovery records. Recipe IDs
are shared with Fabrication; benefit definitions stay in the survival owner.

| Recipe | Hunger | Active modifier | Initial recipe availability |
| --- | --- | --- | --- |
| `food-ration` | +40 immediately | None | Unlocked |
| `nutrient-gel` | +20 immediately | 50% fatigue drain for 600 seconds | Locked |
| `complete-meal` | +7 each 6 seconds, ten ticks (+70/60 seconds) | 200% eligible natural regeneration for 300 seconds | Locked |
| `energy-drink` | +10 immediately | 50% fatigue penalty relief for 300 seconds | Locked |

The 200% regeneration and 50% penalty relief are explicit **provisional balance
settings**, represented by `Benefit::Modifier::basis_points` (20000/5000), not
new fixed product requirements. #384 owns balance signoff. Gel drain is 5000 bps.
All values/durations are captured catalog data and validate before publication.
Medkit data remains unchanged. Only basic food is initially unlocked; medkits
remain unlocked medical items. No research/acquisition command is introduced.

Continuous modifiers have no tick interval, require one maximum stack, and use
the strongest same-kind active value rather than compounding. Reconsumption
refreshes duration. Gel modifies only active fatigue drain. A saved 0..9999
integer remainder preserves sub-unit drain across arbitrary nanosecond partitions;
old saves without it decode to zero. Offline fatigue recovery leaves all food
rules, durations and hunger/health unchanged. Meal regeneration still needs
hunger >=70 and fatigue >=50 and cannot erase wounds; its multiplier applies to
the ordinary 60-second regeneration event, including an event at expiry.

Energy drink reduces the fatigue component of movement and interaction penalties;
hunger penalties remain separate. It neither restores fatigue nor changes its
thresholds. At zero fatigue, slow walking improves but exhausted action bans
remain. Runtime obtains this same derived policy for physical movement and HUD.
Full-save restore preserves captured recipe/effect catalogs, including older runs;
loading does not inject new recipes, unlocks, items or benefits into old saves.

Native Q-wheel selection remains #377; these definitions use its existing exact
slot/job/unit consumption transaction, with no new hotkeys or input flow.

#384 supplies [portable balance evidence](../../docs/survival-balance.md) without
changing the provisional food/modifier/restriction parameters. Explicit owner
signoff and physical route difficulty review remain #496.

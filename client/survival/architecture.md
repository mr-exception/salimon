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
excludes explicit suspend/occlusion/zero-size/renderer lifecycle gaps. No implicit
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

#371 owns native/session/save integration: validate deserialized state, apply
recovery once, durably publish the credited state and new timestamp before gameplay
exposure. If publication fails, do not expose the grant. The watermark plus updated
fatigue must be one atomic checkpoint; domain receipts alone cannot ensure
exactly-once recovery across a host crash. Old checkpoints without survival require
an explicit #371 migration. This crate is not yet ticked by the playable session;
no existing save format is silently changed. Death archive/restart is #371/#372.

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
fractions while eligible and resetting the interval when eligibility is lost.

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
Native survival persistence, consumable effects, death archives and host world
freeze remain #369–#372; this owner supplies deterministic test fixtures.

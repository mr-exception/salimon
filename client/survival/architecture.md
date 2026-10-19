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

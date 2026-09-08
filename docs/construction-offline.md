# Durable construction and closed-game work (#550)

Runtime `session/offline.rs` owns the construction timeline and launch staging;
ship `controller/offline.rs` owns closed-game Core settlement. The existing
construction `Funding`/`Work`, module commissioning, Fabricator and Recovery owners
perform their ordinary transactions. There is no second construction simulator,
inventory, battery or numerical time multiplier.

## Time, energy and eligibility

Schema 22 records a versioned UTC high-water mark, a sub-16 ms remainder, blueprint
contract version 1, the original Core power source and last saved per-project
safety/status reasons. Projects already retain complete canonical blueprint
sources, original local receipts, integer-milligram accounting, paid nanoseconds,
ppm, stop/damage/completion flags and independent cell identities. Full captured
blueprints must match their registered version; unknown/altered sources reject.
Saved status strings are bounded observations, not permission to bypass a fresh
physical safety check. Their keys must exactly match saved project identities.

Only a verified launch transaction admits real closed-game seconds. Ordinary
restore/Save Now performs no catch-up. The UTC high-water mark never moves
backwards, including when publishing during clock rollback. Reopening the credited
checkpoint at the same or earlier UTC cannot buy work again. Versions 1–21 migrate
without retroactive time credit, then participate normally after a schema-22 save.

The timeline uses the live 16 ms powered-work order: ship service and active
Fabricator, Recovery, then construction in stable project ID order and existing
module maintenance. Actual conversion input and ordinary unreserved Core joules
fund each step. Captured assist reservations remain isolated. No power creates
no work; replenishing conversion can resume eligible work. Completion/downtime,
final occupancy preflight and exact proportional cancellation remain with their
existing owners. An upgrade receives no effect until real commissioning succeeds.

Sub-tick time is retained rather than rounded into free progress. A verified
event flushes the preceding partial interval before applying the event; remaining
closed time is retained across saves. Returning live controls or a live tick first
settles that remainder, so new input cannot retroactively spend it. Quiet intervals
can be settled in bulk only when no eligible auxiliary work/future energy can
change the result; Fabricator and conversion still split at their source/output
boundaries. There is no arbitrary offline progress cap or acceleration grant.

Interior work may run in the saved flying/coasting frame if its blueprint and
current occupancy/support permit it. Exterior work needs the authoritative landed
state. An already airborne site or a verified departure latches Stop immediately;
subsequent landing never clears that latch. Stops, damage, missing material,
clearance/support faults and power starvation preserve actual progress and inputs.
Normal in-game pause still freezes work. Closing from that pause begins distinct
closed-game time; player distance/departure alone never blocks eligible work.
Terminal runs grant no powered work.

## Current world and U32 integration

The existing compressed-world host keeps its saved ship/region frame, loose cargo,
door/assist path, survival active clock, health/hunger/effects and journey distance
frozen while closed. Fatigue retains its separate established recovery policy.
Closed power settles service, conversion, Fabricator and Recovery as well as
construction; a long absence can exhaust the ordinary Core. It never applies
thrust/steering, changes an engine switch, creates an assist or fires a Jump Drive.

`TimedEvent` is the integration seam for the authoritative Universe U32 host
[#433](https://github.com/mr-exception/salimon/issues/433). After verifying region
and trajectory events, that owner can submit ordered damage and validated motion/
landing/departure boundaries. Events beyond the interval, backwards event order,
invalid physical state and bad late damage identities reject the entire candidate.
The current native host supplies no invented future events. This PR does not
implement scanner windows, ephemerides, orbital propagation, networking or the
future U32 event producer, and does not predict a safe landing from wall time.
Captured assists remain with that owner rather than being silently completed or
refunded by construction catch-up.

## Atomic resume and verification

The candidate reconstructs the original save, applies chronology and validates
its resulting source through the normal full-save reconstruction path. Only then
may `Checkpoints` publish a verified generation and expose it to gameplay. Failed
publication leaves the visible session and previous generation intact. Retry
reconstructs from the same original inputs; repeat successful launch restores the
credited state without a duplicate room, collider, receipt or resource.

Tests: `cargo test -p salimon-client --locked session::offline`, checkpoint and
Recovery suites, ship offline tests and workspace gates. They cover live-vs-closed
work/energy, arbitrary second partitions/remainders, early depletion and conversion
restart, actual competing Fabricator demand and isolated assist reservations,
damage/flight boundaries, pause/close, upgrade commissioning, malformed clocks,
legacy migration, UTC rollback and real two-generation failed-publication/retry.

The E2E-only `offline_resume` fixture serializes a checkpoint and runs the real
launch reconciliation with a declared closed-time interval. It is not a physical
wall-clock wait or a disk failure test; the separate native-store tests establish
publication behavior. Render routes are
`scenarios/evidence/construction-offline.json` and
`scenarios/evidence/construction-offline-upgrade.json`.
See [actual evidence and limits](https://github.com/mr-exception/salimon/blob/f1cd0b3e878fac487ce898547a0c8c35aa40519b/reports/issue-550/README.md). Human appearance
review remains #561, physical platform/input/OS checks #557 and hardware/Metal
budgets #563.


## Canonical universe host (#433)

[The real motion/scanner host](universe-offline-propagation.md#offline-timed-systems-and-scanner-433) now composes the same powered work with canonical frame/epoch progression and actual terrain crashes. Canonical launch flushes its final sub-tick before verification; historical engineering queues retain their existing remainder behavior. Analytic idle scanner windows never bypass runnable work/conversion, emergency reserves or physical safety.

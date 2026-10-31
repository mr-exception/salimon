# #660 — automated survival/recovery evidence and deferred trial forms

Collected 2026-10-31 on the actual M1/Metal environment identified in
[#557](../issue-557/README.md). The owner explicitly chose automated collection
and documentation for later physical trials. **No ordinary-supply physical
acquisition route or balance decision is passed.** Remaining trials and tuning
decisions transfer to [#723](https://github.com/mr-exception/salimon/issues/723).

## Automated results

The optimized runtime suite passes 564 tests, including the six existing
`survival_balance` cases and production fabrication/recovery/save/input contracts.
[Balance report](balance.json) and [availability report](availability.json)
retain actual current fixture results. They use declared setup/travel budgets
and explicit placements; these are not physical navigation/acquisition times.
The slow first-ration fixture remains **431.304 s**, over proposed 360 s comfort;
the stress first ration remains **941.304 s**, margin **-12.2947 hunger points**.
Neither proposed 360/720 s target nor recipe/modifier balance was approved.

The restored maintained collector
[`scripts/verify_recovery.py`](../../scripts/verify_recovery.py) executes the
historical #515 route against current production adapters. Its
[result](recovery/result.json) passes with nine Metal originals: held-iron intake,
paid module crafting, exact 0.75 kg tray collection/installation, identity/mass
preservation, cabin walking, Archive R start/stop, pause/funded-progress retention,
and both exact 3/5 GJ records. Completion renders at 1280×800 and 1920×1080.
It retains UNVERIFIED/IRRECOVERABLE and NO NAVIGATION FIX rather than inventing leads.

[Collector context/hash](recovery/collector-context.json),
[full states](recovery/states.json.gz), [stderr](recovery/stderr.log.gz) and
[module tray](recovery/module-tray.png) retain actual evidence. The explicit
`scout-recovery` fixture starts with 2 kg carried iron, 0.5 kg silicate and initial
Core energy. This proves the scoped transactions, not mining/travel/Core
replenishment acquisition, ordinary-supply economics, physical controls or balance.

## Later physical trials

Three deliberately **pending, invalid-until-completed** ration forms are supplied:
[normal](trials/normal.pending.json), [impaired](trials/impaired.pending.json),
[poor layout/scarce](trials/poor-layout.pending.json). Null stage durations,
operator/date/evidence and outcome are unrun fields, not zeros or failed physical
observations. Replace revision/context with the actual trial build and complete
every required observation; run:

```sh
python3 scripts/survival_route_review.py path/to/completed-trial.json --output artifacts/route-review.json
```

Record complete discovery/setup/outbound/gather/return/intake/craft/collection/Q
routes, independent elapsed total and exclusive phases, real source identities,
funding/meters, normal versus impaired/obstacle conditions and no-feedstock/failure
outcomes. No debug placement or starter-funded repeatability. The fixtures cover
nearby Earth seeds, not universal/canonical biomass availability; Vesper remains
sterile and scarcity must be retained rather than adding biomass everywhere.

Use [recovery trial checklist](trials/recovery.md) and
[the maintained procedure](../../docs/macos-qualification.md) for actual
acquisition/carry/intake/Fabricator/collection/install/Core-replenishment/Archive-R
measurements. Return originals/logs/hashes, reviewer/date and explicit scoped
accept/revise/reject decisions to Codex in #723. Approved hunger 1800 s, fatigue
64800 s, offline recovery 21600 s and construction economy decisions remain intact.

No runtime/save/recipe/physics behavior changes. Supporting ownership remains
runtime [architecture](../../client/runtime/architecture.md) /
[invariants](../../client/runtime/invariants.md) and
[survival balance contract](../../docs/survival-balance.md).

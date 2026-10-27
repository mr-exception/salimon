# Gravity-aware ship trajectory (#417)

The portable ship `trajectory` module composes the existing funded-joule Propulsion contract and ledger-derived mass with U15 gravity using symmetric velocity-Verlet steps. Position remains a split GalacticPosition and epochs remain explicit TT/TDB. The caller chooses a fixed step <=60 seconds and supplies epoch-aware complete source candidates at both endpoints. State and gravity selection commit atomically only after both source queries and all numerical checks succeed. Mass changes use a fresh Propulsion value per step. The base step never debits Core or grants engine/autopilot authority; support, drag and contact compose through the explicit #418–#420 companion APIs. No velocity cap is added. Canonical playable runtime adoption remains later work; existing compressed controller behavior is unchanged. Numerical orbit tests establish a documented fixture error budget, not complete scientific/model or hardware qualification.

This is portable private-development implementation, separate from playable runtime adoption. Private real-data development is authorized; future distribution review remains #659 and full-release scientific data acceptance remains #524/#526. No hardware, scientific model accuracy or production completeness is inferred from fixture tests.

Run `cargo test -p salimon-universe --locked`, `cargo clippy -p salimon-universe --all-targets --locked -- -D warnings`, `cargo fmt --all -- --check` and `python3 scripts/check_docs.py`.

## Completed cloud composition acceptance

`Trajectory::step_indexed` composes the actual U15 spatial-index path through
reusable epoch-aware Arc snapshots, sharing the same atomic Verlet transaction
as the complete-candidate path. The two paths produce exactly equal states in a
1000-step regression. A Sun/Earth/Moon analytic ephemeris field is compared over
6000 seconds against an independent half-second RK4 integrator: 16.804049113 m
position difference, within the declared 25 m numerical fixture budget. A moving
Earth-like body flyby increases inertial speed from 7000 to 10249.17694 m/s while
body-frame specific-energy relative drift remains 5.659e-9 (test limit 1e-5).
These add multi-body, moving-body assist and indexed-composition coverage to the
existing orbit/escape/funding/mass/fixed-step/failure tests.

The initial draft assessment incorrectly made later playable-world adoption a
U16 delivery requirement. Issue #417 requires this portable integrator and its
reference tests. Portable support/assists #418, atmosphere #419 and continuous
contact #420 are now delivered companion APIs. World streaming #422 and later
playable terrain/new-run integration retain their own scope. Current compressed controller behavior is deliberately preserved. No
human gameplay, M1 qualification or future source-release review gates this PR.


## Real-scale surface flight (#418)

See the [surface-flight contract](universe-surface-flight.md) for rotating/orbiting support,
Core-funded physically bounded landing/takeoff, production source/tests and
remaining playable-world adoption. Current compressed gameplay is unchanged.


## Real-scale atmospheric drag (#419)

See the [atmosphere contract](universe-atmosphere.md) for planet-specific density,
co-rotating gas/wind, atomic online/offline drag integration and physically paid
assist compensation. Model validity, tests and playable-adoption limits are explicit.


## Real-scale moving contact (#420)

See the [continuous-contact contract](universe-contact.md) for moving solids/terrain,
certified earliest-event queries, atomic gravity/drag interception, body-relative
consequences, funded-work receipts, cloud tests and adoption/precision limits.

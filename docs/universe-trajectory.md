# Gravity-aware ship trajectory (#417)

The portable ship `trajectory` module composes the existing funded-joule Propulsion contract and ledger-derived mass with U15 gravity using symmetric velocity-Verlet steps. Position remains a split GalacticPosition and epochs remain explicit TT/TDB. The caller chooses a fixed step <=60 seconds and supplies epoch-aware complete source candidates at both endpoints. State and gravity selection commit atomically only after both source queries and all numerical checks succeed. Mass changes use a fresh Propulsion value per step; no Core debit, engine permission, autopilot, drag, contact or assist is invented. No velocity cap is added. Runtime adoption and real-scale landing/support remain later work; existing compressed controller behavior is unchanged. Numerical orbit tests establish a documented fixture error budget, not complete scientific/model or hardware qualification.

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
reference tests; launch/support/landing #418, moving-body collision #420, world
streaming #422 and later playable terrain/new-run integration retain their own
scope. Current compressed controller behavior is deliberately preserved. No
human gameplay, M1 qualification or future source-release review gates this PR.

# Gravity-aware ship trajectory (#417)

The portable ship `trajectory` module composes the existing funded-joule Propulsion contract and ledger-derived mass with U15 gravity using symmetric velocity-Verlet steps. Position remains a split GalacticPosition and epochs remain explicit TT/TDB. The caller chooses a fixed step <=60 seconds and supplies epoch-aware complete source candidates at both endpoints. State and gravity selection commit atomically only after both source queries and all numerical checks succeed. Mass changes use a fresh Propulsion value per step; no Core debit, engine permission, autopilot, drag, contact or assist is invented. No velocity cap is added. Runtime adoption and real-scale landing/support remain later work; existing compressed controller behavior is unchanged. Numerical orbit tests establish a documented fixture error budget, not complete scientific/model or hardware qualification.

This is portable private-development implementation, separate from playable runtime adoption. Production source approval remains #659 and full-data acceptance #524/#526. No hardware, scientific model accuracy or production completeness is inferred from fixture tests.

Run `cargo test -p salimon-universe --locked`, `cargo clippy -p salimon-universe --all-targets --locked -- -D warnings`, `cargo fmt --all -- --check` and `python3 scripts/check_docs.py`.

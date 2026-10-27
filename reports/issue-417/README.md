# Issue #417: Gravity-aware ship trajectory

The portable ship `trajectory` module composes the existing funded-joule Propulsion contract and ledger-derived mass with U15 gravity using symmetric velocity-Verlet steps. Position remains a split GalacticPosition and epochs remain explicit TT/TDB. The caller chooses a fixed step <=60 seconds and supplies epoch-aware complete source candidates at both endpoints. State and gravity selection commit atomically only after both source queries and all numerical checks succeed. Mass changes use a fresh Propulsion value per step; no Core debit, engine permission, autopilot, drag, contact or assist is invented. No velocity cap is added. Runtime adoption and real-scale landing/support remain later work; existing compressed controller behavior is unchanged. Numerical orbit tests establish a documented fixture error budget, not complete scientific/model or hardware qualification.

Validation results are recorded below. Linux x86_64, Rust 1.99.1; private fixtures only. No runtime/rendered change or hardware claim. Production source and full-snapshot approval remains #659, scientific/resource coverage #524/#526; those checks are unrun. This PR is a development implementation, not full production qualification.

- `cargo test -p salimon-ship -p salimon-universe --locked -- --nocapture`: 114 ship tests passed, one existing opt-in ignored; 83 universe tests passed; zero failures.
- Circular two-body fixture (7000 km radius, 6000 seconds): position error 67.163727119 m at 2 s steps and 16.790954444 m at 1 s steps. The 1 s test budget is 25 m; refinement is within 3.8–4.2 times improvement. These are numerical fixture bounds, not scientific release tolerances.
- Moving-ephemeris endpoint composition, escape-energy positivity, exact fixed-step replay, already-funded impulse, changing ledger mass, superluminal numerical continuity and atomic provider failures pass.
- `cargo clippy -p salimon-ship -p salimon-universe --all-targets --locked -- -D warnings`: passed after correcting a numeric grouping warning in the new test.
- Formatting and docs/base-impact checks pass. Reuses the existing paid `Propulsion` type; Cargo.lock deliberately adds the direct portable universe dependency.
- Initial workspace compilation failed because ALSA pkg-config development metadata was absent. The installed actual libasound.so.2 (Debian 1.2.14) is linked through a local /tmp development symlink and pkg-config file; no audio behavior is claimed. The original setup failure is retained.

Final stack verification:

- `PKG_CONFIG_PATH=/tmp/salimon-native-deps/lib/pkgconfig cargo test --workspace --locked`: **1052 passed, zero failed, 38 existing opt-in ignored**. All 39 reported unit/doc suites completed; original logs are retained compressed.
- With the same local ALSA link metadata, `cargo build --workspace --locked` and `cargo clippy --workspace --all-targets --locked -- -D warnings` both pass.
- `python3 -m unittest discover -s scripts -p test_check_docs.py`: 5 passed.
- `python3 scripts/check_docs.py --base codex/issue-416-celestial-gravity` and `cargo fmt --all -- --check`: pass.
- Parent PRs #663–#666 each passed Ubuntu source CI and macOS/Windows fictional-publication goldens at their published heads. Those platform goldens do not establish native device/Metal performance or real-scale gameplay.
- No graphical/runtime adapter changed, so no new renderer captures are claimed. Source permission, production data completeness, independent ephemeris accuracy calibration, production spatial indexing/performance and downstream playable adoption remain explicitly incomplete as described in the parent PRs and source issues. All five PRs remain draft for those scope/acceptance limits; nothing is merged.

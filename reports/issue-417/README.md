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

## Cloud completion revision

The portable integration is now tested through U15 indexed snapshots as well as
the complete-candidate path. Added exact 1000-step indexed/direct parity, an
independent RK4 Sun/Earth/Moon reference over 6000s (16.804049113 m position error,
25 m fixture limit), and a moving-body gravity assist (specific-energy relative
drift 5.659e-9, limit 1e-5; inertial speed 7000 -> 10249.17694 m/s). Existing
circular-orbit refinement, escape, mass/funded-thrust, replay and failure checks
remain unchanged. 117 ship tests pass (one existing opt-in ignored), 88 universe
tests pass, and affected all-target Clippy passes. The initial Arc coercion
compile error is retained with the corrected results under `cloud/`.

The previous draft assessment overstated runtime adoption as missing U16 work.
The approved U16 acceptance is the portable integrator/reference suite; actual
landing/support #418, collision #420, streaming #422 and later playable world
migration remain their existing downstream tickets. Those are neither unfinished
U16 scope nor human-verification gates. Future public release remains #659;
scientific full-release data coverage remains #524/#526. No human review is
needed to establish this routine cloud delivery's automated acceptance.

## Final automated stack verification

The complete workspace run at revision `75c5b3ce` passed **1060 tests, zero
failures, 38 existing opt-in ignored**; workspace build and all-target Clippy
with warnings denied passed. The subsequent raw-producer JSON and prefetch
shutdown refinements were validated by the affected final domain suites at
`100ed644`: **117 ship tests passed (one existing ignored) and 89 universe tests
passed**, with all-target domain Clippy, formatting and documentation/base-impact
checks passing. Original complete-workspace and final-domain logs are retained
compressed under `cloud/`. Published-head CI performs fresh complete-workspace
verification before the PR is marked ready. Historical draft limitations above
are superseded by these completed cloud checks and the scoped ownership mapping.
No human/native qualification or future release approval is a routine gate.

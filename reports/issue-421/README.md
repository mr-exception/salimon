# #421 — Bounded navigation prediction and cockpit telemetry

Parent: #677 / `04327e7b`, including its CI title-expectation correction. The public ship forecast clones the production split-coordinate gravity/Verlet trajectory, accepts epoch-aware moving target/source providers and reports range, relative/radial velocity, heading, first-interval acceleration, sampled approach/entry and numerical convergence. Bounded intervals and explicit failure preserve source state. No guaranteed arrival, autopilot or authoritative Core debit is introduced.

A separate preview clones the actual playable controller, including its selected terrain/contact, power, held controls and known conversion, for at most 8.192 seconds. The runtime's disposable 500 ms observation cache reports age, adjusts countdowns to now and invalidates on commands or relevant source changes. Renderer-neutral DTOs add contact/minimum-altitude estimates, resolution difference and held-input/power validity warnings to the existing physical monitor. Invalid/free-flight-unavailable states are explicit. Existing live range, relative velocity, heading, acceleration and braking remain visible.

[Contract and source/test mapping](../../docs/universe-navigation.md). Ship/runtime/renderer owning README.ai/architecture/invariants, root usage/routing/validation, trajectory contract and approved-design status are updated in this PR. Cache/prediction state is not saved and cannot advance time or change commands.

## Verification

Established Linux x86_64 `salimon-dev` container, Rust 1.99.1 `/opt/rust`, 2 CPU / 8 GiB. Cargo uses `RUSTUP_TOOLCHAIN=/opt/rust`, `--locked -j2`.

- `cargo test --workspace --locked -j2`: **1111 PASS**, **38 existing opt-in ignored**, no failures. Final log retained.
- `cargo clippy --workspace --all-targets --locked -j2 -- -D warnings`, `cargo fmt --all -- --check`, client build: PASS.
- Focused ship navigation suite: **8 PASS** (five new forecast/live tests plus three existing telemetry tests). Analytic moving target across split cells, gravity refinement/exact authoritative propagation, sampled miss/entry, errors/nonmutation and actual 16 ms contact-time containment/power failure are covered. Existing independent RK4/orbit/contact tests pass in the full suite.
- Runtime navigation cache/restore/source-byte/countdown test: **1 PASS**. Renderer full suite: **82 PASS**, **15 existing opt-in ignored**, including new visible prediction/unchanged upload test.
- Python runner contracts: **22 PASS**. Documentation/whitespace gate: PASS.
- Production software-Vulkan `xvfb-run -a -s '-screen 0 1280x800x24' python3 scripts/salimon-test run scenarios/evidence/universe-navigation.json --binary target/debug/salimon-client --screenshot-command '["native"]'`: **19/19 PASS**. Explicit lvp ICD, `WGPU_BACKEND=vulkan`, two llvmpipe threads. Terrain warms while gameplay is paused, then real simulation resumes. Source initial Vesper pose/velocity is declared in the fixture; all later movement/power/contact/restore uses production code. Original PNGs inspected, compressed raw results, compact observations and hashes retained.

The final capture reports a 5.504–5.568 second contact estimate with 0.00004564 m coarse/half-step separation; actual integration reaches GroundedAfterCrash and the free-flight forecast becomes unavailable. This difference is an empirical numerical indicator, not a physical error certificate.

## Failed checks and corrections

- Initial inspection JSON exceeded serde macro recursion nesting. A separate observation value fixes expansion without raising limits.
- Initial cache test expected age to persist across the first real runtime mass synchronization. It now admits that production tick before recording the cache baseline; actual mass changes still invalidate immediately.
- Initial native setup omitted its Flying validation registration; fixed without changing physics. A later 800-frame action exceeded the protocol's 600 limit; split into valid actions. An unpaused preparation route timed out under concurrent CPU work; final setup pauses simulation during preparation so initial flight does not consume its approach before geometry is ready. All three failed raw native results are retained.
- The first workspace run found only the inherited #426 title assertion (477 runtime passes / 1 failure). Fixed on the parent PR, then incorporated by ancestry. Its failed CI/log remains recorded in #426; the final workspace rerun here passes.
- Capture review found that cached relative countdowns must subtract age. Fixed and verified by the final cache test, full suite and native captures. Earlier native captures are not presented as final output.

## Limits and separate ownership

The real-scale API is explicitly **coasting gravity without thrust, drag or contact**; sampled entry does not certify continuous first contact. The live preview uses today's actual local controller and is not falsely substituted with inertial ephemeris physics. Remaining runtime inertial/catalog/offline adoption is owned by #427/#432, as in the foundational #417–#420 contracts. Live predictions assume held inputs/current mass and cannot anticipate future auxiliary demand, refueling, commands or model changes. No new chart discovery or Earth route is revealed. Source/model uncertainty is not measured by step halving.

Hardware GPU/Metal/60 FPS and subjective presentation remain unverified, separately owned by #557/#561/#563. Software Vulkan and portable publication checks cannot establish those claims.

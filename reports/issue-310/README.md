# Issue #310 — coherent render-only gameplay interpolation

Implemented on 2026-10-18 against `267da32` (includes merged #309 / PR #330).

## Outcome

`session/presentation.rs` owns previous/completed ship and character observations.
The native scene uses one clamped backlog/16 ms alpha for ship position, normalized
shortest-sign orientation, door fraction, character position and loose fragments.
Interior camera positions blend in ship-local space before mapping through the
presented ship frame; exterior positions blend in absolute `f64` world space.
Live mouse look is preserved and interior direction/up follow that render frame.

Ship/fragment prompt eligibility remains authoritative; only anchors follow the
presented geometry. Resource prompts retain stable fragment identity. Carried
objects and growing previews follow the presented camera without interpolating
mouse input. The renderer's existing subtract-before-cast precision path is unchanged.

Initialization, view/lifecycle gaps, cockpit entry/exit, doorway/EVA/location and
flight-state transitions bypass incompatible history. Doorway blending bypasses
history throughout. Out-of-tick ship/character teleports are detected when sampling.
New/growing/released/transferred/carried objects bypass incompatible object history;
current ship-local attachments still map through the render ship pose. Removed
objects are not stored in presentation. Simulation sequencing and the clock's
bounded debt retention are unchanged.

Interaction, mining, physics, mass and telemetry still consume live state.
Automation disables all gameplay interpolation and inspection retains exact
current snapshots. Rendering never advances the session. This adds up to one
16 ms tick of positional/ship-orientation presentation delay, with immediate look.

Updated the runtime architecture/maintenance/invariants and canonical architecture
and maintenance map. No domain, renderer dependency, asset, or CI changes.

## Validation

Linux x86_64 container, kernel 6.18.55; rustc 1.99.1, Cargo 1.99.1.
Cargo used an existing external target cache (`CARGO_TARGET_DIR`); no dependency
or lockfile changes.

| Command/check | Result |
| --- | --- |
| `cargo test -p salimon-client --locked session::presentation` | **Passed:** six focused regressions |
| `cargo test --workspace --locked` | **Passed:** 513 tests, zero failures; 22 existing opt-in/doc tests ignored |
| `cargo test -p salimon-client --locked` after final render-snapshot adjustment | **Passed:** 163 tests, five existing opt-in tests ignored |
| `cargo clippy --workspace --all-targets --locked -- -D warnings` | **Passed** |
| `cargo build --workspace --locked` | **Passed** |
| `cargo fmt --all -- --check` | **Passed** |
| `git diff --check` | **Passed** |
| Relative file links in the five changed canonical Markdown guides | **Passed** |
| `$CARGO_TARGET_DIR/debug/salimon-client --benchmark --mode physics --count 10 --layout onboard --state active --warmup-seconds 0 --sample-seconds 1` | **Passed:** production-session executable smoke, ten IDs / 0.32 kg retained, `accounting_preserved=true`; no renderer |
| Native graphical smoke/captures | **Not run:** container has no configured graphical desktop/Xvfb; no visual/FPS claim |
| Apple M1/Metal smoothness, capture and latency signoff | **Not run:** separate native [#314](https://github.com/mr-exception/salimon/issues/314), remains pending |

Focused coverage includes synthetic 60/120/144 Hz schedules, zero/multiple-tick
frames, fractional and overloaded backlog, repeat render purity, rotating ship
and camera coherence at distant `f64` anchors, exterior motion/interior walking,
immediate look, current telemetry, exact automation, initialization/lifecycle/view
resets, teleports, cockpit/doorway/EVA location changes, new/local/carried/removed
fragments and quaternion sign equivalence. Existing fragment tests additionally
cover immediate release, size/mass changes and frame-transfer history bypass.

During development the first synthetic test incorrectly expected extrapolation
before the first tick; its startup expectation was corrected to the required
current-pose bypass. The first strict lint run flagged a test fixture's default
field reassignment; the fixture now initializes that field directly. Both checks
passed on rerun; no failed checks remain at delivery.

## Remaining verification

Portable schedules prove arithmetic/state separation, not physical display
smoothness, end-to-end input latency, native capture quality or GPU performance.
[#314](https://github.com/mr-exception/salimon/issues/314) retains all native
signoff and its existing #308/#310/#311 dependencies. This implementation can
close #310 on merge without closing or marking #314 passed.

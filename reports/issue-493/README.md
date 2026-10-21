# Issue #493 — Funded Core joules become exact thruster impulse

Implements the intentional fictional baseline `kappa = 1 N·s/J`: only the
propulsion share of the ordinary bus's funded interval produces signed impulse;
`delta-v = kappa * E_propulsion / current_mass`. Idle/cockpit/Fabricator and
isolated assist reserves cannot also produce free-flight impulse. No kinetic
energy charge or independent force capacity remains.

Default forward/reverse draw stays 100/50 MW, Core 60/45 GJ. A dry 100t ship now
accelerates at 1000/500 m/s², 25× historical force tuning. Braking/acceleration
telemetry use the same power/coefficient/mass law. Held throttle billing splits
at each real 100ms repeat boundary, instead of charging the endpoint selection
for earlier time. Conversion, depletion and assist completion preserve funding
boundaries and unpowered coasting.

B explicitly toggles engines with powered cockpit authority outside committed
assists/bounces. Off preserves selected throttle and momentum, disables propulsion
power/braking/drive visuals/audio and remains off after save/reload/refueling.
Zero throttle with the switch on is distinct. Existing idle/steering systems
remain electrical. The physical instrument shows ENG ON/OFF and B hint.

Schema 5 saves switch/coefficient. Runtime migrates v1-v4 legacy force capacities
to coefficient 1 and engine-on, retaining energy, velocity and physical source;
current-schema missing/retired/duplicate fields reject before publication.
Current restores keep typed decoding rather than round-tripping through a DOM.

Landing remains <=20m/s; swept collision and crash recovery remain authoritative.
Takeoff remains an eased six-second 50m path ending at rest/throttle zero,
peaking at 15m/s and 15m/s². Isolated takeoff/landing/recovery reservations remain
600/800/1200MJ conservative kinematic budgets. Updated all-body/cargo handoff
checks and deterministic flight/landing routes verify the stronger manual thrust;
no coefficient reduction, speed cap or silent Core upgrade is introduced.

Validation: Linux x86_64 / Rust 1.99.1. New regressions cover exact 100MJ/1s and
1GJ/10km/s examples at different initial velocities, reverse, added mass,
three-dimensional direction, coefficient tuning, partial shared loads, conversion,
throttle billing partitions, explicit switch and restore, native press/modal
admission, migration and atlas invalidation. Existing cargo/blackout/EVA/collision/
assist tests and revised scenarios remain part of workspace CI.

Reference native handling, physical B input, instrument readability and full-loop
performance are UNRUN here and tracked by
[#498](https://github.com/mr-exception/salimon/issues/498). No image baseline is
promoted. Jump Drive, N=20% travel economics, its UI/physical component and shared
universe clock are not implemented by this delivery. Source closure does not
claim native signoff or future travel completion.

- `cargo test --workspace --locked`: PASS (including 72 ship tests; one existing
  opt-in ship trace is ignored; workspace GPU/benchmark opt-ins remain ignored).
- `cargo clippy --workspace --all-targets --locked -- -D warnings`: PASS.
- `cargo build --workspace --locked`: PASS.
- `cargo fmt --all -- --check`, `git diff --check`, new/changed Markdown local
  target-path checks: PASS.

A partial-energy regression exposed a one-ULP difference at 10.001 m/s; its
explicit 1e-12 m/s tolerance covers floating addition while exact joule and axis
checks remain. Historical native reports/baselines are retained as historical
source evidence and do not certify the new acceleration or instrument text.

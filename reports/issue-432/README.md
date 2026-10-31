# Issue #432 — work in progress

Status: **canonical motion and launch implementation complete on the batch branch; shared final verification/PR pending the remaining four tasks**.
Base: `e68ccb8` (`origin/main`, 2026-10-31); branch
`codex/offline-universe-five-tasks`. This is the first part of the requested
single-PR batch for #432, #433, #434, #441 and #435.

The local `salimon-ship::offline` candidate adds bounded resumable gravity-only
propagation over the production gravity, drag and continuous-contact operators.
It stages trajectory and timed-consumer changes together, records progress and
contact uncertainty, follows verified support after impact and rejects incomplete
results or invalid provider/consumer work. Ordinary canonical flight and closed coasting now compose the same production gravity/contact path; schema 30 binds the new motion identity and epoch. The [motion, frame and migration contract](../../docs/universe-offline-propagation.md)
record exact boundaries and remaining work.

## Initial core verification at `cf8879b`

Environment: existing Linux cloud executor, Rust/Cargo 1.99.1 toolchain from
`/workspace/.salimon-tools/env.sh`, locked Cargo dependencies, debug profile.

- `cargo test -p salimon-ship --locked offline::`: **5 passed**, zero failures;
  four new propagation regressions and the existing closed-power regression.
  Orbital fixture budget: <30 m analytic and half-step position difference after
  one approximately 5,829-second orbit at radius 7,000 km. This is a fixture
  accuracy measurement, not a certified global error bound.
- Core-checkpoint `cargo test -p salimon-ship --locked`: **156 unit tests and 1 integration test passed**, zero failures, 1 existing opt-in test ignored. Includes moving-body impact/support velocity and final-nanosecond progress rounding.
- `cargo clippy -p salimon-ship --all-targets --locked -- -D warnings`: **passed**.
- `cargo fmt --all -- --check`: **passed**.
- `python scripts/check_docs.py --base e68ccb8`: **passed**, zero findings.
- `git diff --check`: **passed**.
- Iteration failure retained: the new moving-body fixture initially asserted an exactly representable 50 m/s derivative; cubic differentiation produced `49.99999999999999`. The assertion now allows 1e-12 m/s floating-point roundoff. No production tolerance or physics contract was changed. The final owner-crate run passes.

[Checked source hashes](source-hashes.json) identify the integrated source inputs after the focused motion checks and affected-crate Clippy pass. The earlier core-only counts above belong to `cf8879b`.
- Broad workspace, native GPU, scanner/timed-system, checkpoint/restart, final
  audit and integrated benchmark acceptance: **not run/not implemented yet**.

## Authorized motion model and integration

The owner selected “Use that motion model and continue the five-task PR” on 2026-10-31. The [issue decision](https://github.com/mr-exception/salimon/issues/432) and Project Q&A record that authorization. `starting_motion.rs` publishes the separate geographic-bound authored circular model, fixed reference epoch/phase/right-handed axes and zero axial spin. No terrain/resource definition or saved body-relative pose is regenerated. The shared local game-clock mapping is explicit; imported-source scientific validity is unchanged.

`controller/celestial.rs` stages power, motion and epoch; both hosts use symmetric gravity kicks around the real paid-impulse/terrain path in the translating frame. Exterior validity, checked calendar bounds, collision budget rejection, vacuum and the measured one-second/16-ms step regimes are explicit. The existing canonical terrain gradient certificate now includes its actual folded-wave/cap taper rather than empty noise layers. Closing snapshots and launch disable engines without removing momentum or re-enabling assistance.

Runtime schema-30 sources require the exact motion hash/version/epoch. Schema-29 migration retains the complete physical source, body-frame velocity and world/resource identity. Current missing/unknown motion fails without mutation. A motion error blocks saving and unpausing the candidate.

## Integrated focused verification

- `cargo test -p salimon-client --locked session::offline::motion_tests`: **4 passed**, covering actual migration, no replay, closed gravity/engine shutdown, a year-long grounded orbit update and a real terrain crash.
- `cargo test -p salimon-ship --locked celestial`: **3 passed**, covering active/closed production equivalence, atomic invalid-clock failure and a one-hour coast against a 250-ms reference (required <25 m and <0.05 m/s).
- `cargo test -p salimon-universe --locked starting_motion`: **1 passed**, fixed publication/clock partitions and invalid-calendar/hash rejection.
- `cargo test -p salimon-client --locked session::canonical`: **8 passed**, existing movement, mining, resource conservation, new-run, takeoff/landing and save regressions.
- `cargo clippy -p salimon-universe -p salimon-ship -p salimon-persistence -p salimon-client --all-targets --locked -- -D warnings`: **passed**.
- `python scripts/check_docs.py --base e68ccb8`: **passed**, zero findings after all affected owning guides were updated.
- Initial runtime build failed because this cached executor lacked `alsa.pc`; installed the Debian ALSA development/runtime packages into the existing non-root sysroot, retaining TLS/proxy settings. The subsequent runtime checks compile and pass.
- Initial migration fixture used a replacement ship without synchronizing its known cargo ledger; fixed the fixture to synchronize source mass before its snapshot. The production migration preserves the original physical source.

Native graphical evidence and complete shared workspace/script gates run once after the batch's remaining source changes. Asynchronous progress/checkpoint publication is #434; scanner contact/power chronology is #433; full documentation audit #441 and integrated evidence #435 follow. The source is not closed before the single combined PR is ready and merged.

## Limits

The live field is the complete published two-body Vesper/Nacre region. The generic portable selector/trajectory/contact APIs retain declared multi-body/atmosphere providers. The authored Vesper publication is vacuum; missing atmospheres are never guessed. The stellar interior is outside this model and rejects unverifiable coasting instead of teleporting or granting a safe landing. General observed-source coverage and full-data budgets remain #524 and their own source validity contracts.

Routine cloud work does not establish physical platform/input (#557), subjective presentation (#561), hardware/Metal performance (#563), source redistribution review (#659), balance (#660) or first-hour experience approval (#661).

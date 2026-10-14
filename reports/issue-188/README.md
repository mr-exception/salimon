# Issue #188 — Authoritative Energy Core and power

Second PR in the #187 → #188 stack. Core storage is now authoritative fractional
f64 joules with validated configuration, bounded snapshot values and an explicit
power flag. Starter tuning is 60 GJ capacity/45 GJ stored, 100 MW maximum forward
and 50 MW maximum reverse draw scaled by signed throttle magnitude. Independent
gravity/cockpit idle consumers default to 100 kW/10 kW and remain configurable.
Ten minutes is a full-capacity tuning estimate, not a fixed flight duration.

Free flight integrates force only through the exact available-energy interval,
then coasts. Exhaustion preserves selected throttle and velocity, clears held
controls, rejects powered cockpit commands, blanks the cached cockpit display
and disables character/fragment artificial gravity without planetary fallback
inside. Charging automatically resumes selected throttle on advancement.

`exchange_core_energy_joules` supports atomic external source/load transactions.
`advance_with_core_input` supplies converted joules at a step boundary and settles
normal consumers with them before capacity clipping. Neither assumes a permanent
continuous charging source; E03 owns conversion timing and physical intake.
Invalid inputs leave previous state intact. Zero crossings, fractional joules,
partitioning, incoming/load precedence and extreme finite values have regression
coverage alongside cockpit/gravity/display integration.

## Validation

Linux x86_64; Rust 1.99.0.

- PASS: `cargo test --workspace --locked` (426 tests; seven pre-existing ignored profiling/oracle tests).
- PASS: `cargo test -p salimon-ship --locked` (45 tests).
- PASS: `cargo clippy --workspace --all-targets --locked -- -D warnings`.
- PASS: `cargo build --workspace --locked`.
- PASS: `cargo fmt --all -- --check` and `git diff --check`.
- Native hardware/GPU smoke: not run here; CI graphical checks are separate evidence.

Initial lint identified constant array-chunk idioms in the new screen path;
those were fixed and the final lint run passed. The final frozen-source workspace
run passed after the display fix. This combined stack also validates unchanged
owners beyond #187's affected-area checks.

## Limits

Native Apple M1 hardware smoke/regression signoff is explicitly excluded. No
reference hardware performance or live GPU visual claim is made. Character and
fragment power flags are sampled at runtime's existing 16 ms gameplay boundaries;
ship thrust uses the exact intra-step depletion interval. Full cabin inertial
physics remains deferred. Current captured assists remain uncancellable; assist
activation budgets/refunds and powered recovery are their separate issues.
Physical conversion/intake is not introduced here. Existing fixed gravity
strength remains 9.81 m/s² when powered; planetary/EVA gravity is unchanged.

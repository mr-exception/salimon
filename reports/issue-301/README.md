# #301 — Robust event-driven cargo mass accounting

The reported unequal-mass reverse-removal sequence now returns to the exact
100,000 kg dry baseline without panicking. Cargo is accumulated separately from
the hull with an error-free partial-sum expansion; low bits survive replacement,
large/small mass mixtures and removal. Empty-ledger reset is explicit. Invalid
mass and overflow remain transactional errors; no arbitrary tolerance or per-tick
object rescan was added. The ship architecture owns this numerical contract.

## Validation

Environment: local macOS, Rust 1.89.0. No native graphical acceptance claimed.

- Regression proof: `cargo test -p salimon-ship --locked exact_reported_removal_sequence`
  with the pre-fix implementation fails on final removal with `InvalidFlightMotion`.
  The same regression passes with the fix.
- `cargo test --workspace --locked`: passed (594 tests; 16 existing opt-in ignores).
- `cargo clippy --workspace --all-targets --locked -- -D warnings`: passed.
- `cargo build --workspace --locked`: passed.
- `cargo fmt --all -- --check` and `git diff --check`: passed.
- New unit coverage includes the exact three masses, 1,000 large/small replacement
  cycles, shuffled removals, tiny residuals, overflow and invalid transactional updates.
- Runtime regression constructs the exact unequal sphere masses through the
  production spherical lifecycle (explicit test extraction rate), exercises 100
  onboard/offboard transfer cycles and reverse-order mechanical Core consumption,
  and checks exact empty mass, identity removal and reconciliation.

## Limits and follow-up

Native motion, performance and Metal gameplay checks were not run for this
portable accounting change. They remain tracked by
[#313](https://github.com/mr-exception/salimon/issues/313); its signoff is pending.
No issue criteria or invalid-mass checks were waived.

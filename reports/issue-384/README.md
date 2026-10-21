# Issue #384 — Portable food-gathering and starvation balance

Adds six bounded deterministic scenarios plus reproducible raw JSON. Production
mining/F pickup/E intake/E crafting/E tray/Q consumption, exact starvation,
blackout, full inventory/tray, save/reload, offline fatigue-only, exhausted gates
and moving-EVA return are exercised. No production balance values change.

See [contracts, assumptions and reproduction](../../docs/survival-balance.md).
Travel/setup are declared time budgets with fixture avatar placement at stage
boundaries, not measured player navigation or discovery time. Availability is
80 bounded samples (16 seeds on each of five solid bodies), not a global bound.

- [balance.json](balance.json): four sensitivity rows. The long travel route is
  net-negative for one +40 ration; baseline/slow setup/low-health-fatigue routes
  have positive margins. Actual total Core debits include service loads.
- [availability.json](availability.json): Earth samples contain both feedstocks;
  other current planets lack biomass. Missing food is retained as a finding.
- Three initial/initially craftable rations provide at most 36 extra minutes
  without waste. No-supply hunger/starvation reaches terminal state deterministically.

Remaining owner approval of numerical budgets/provisional tuning and actual
physical route/native difficulty evidence transfer to
[#496](https://github.com/mr-exception/salimon/issues/496); native HUD/Q remains
#376. Source closure records portable delivery and that transfer, not approval.

Linux x86_64 / Rust 1.99.1 validation results are recorded below after execution.

- `cargo test -p salimon-client survival_balance --locked`: PASS, 6 tests; JSON
  generated with absolute report paths as documented.
- `cargo test --workspace --locked`: PASS.
- `cargo clippy --workspace --all-targets --locked -- -D warnings`: PASS.
- `cargo build --workspace --locked`: PASS.
- `cargo fmt --all -- --check` and `git diff --check`: PASS.
- Native M1/GUI/subjective route timing: UNRUN; tracked in #496/#376.

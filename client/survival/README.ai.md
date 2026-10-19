# Survival maintenance

Read [architecture](architecture.md), [invariants](invariants.md), root conventions
and the active issue. `src/lib.rs` owns portable meter and clock contracts. Keep
native clock admission and whole-session persistence in runtime (#371). Run
`cargo test -p salimon-survival --locked`, then workspace build/tests/Clippy/fmt
for cross-crate changes. Add deterministic boundary and partition tests for rules.

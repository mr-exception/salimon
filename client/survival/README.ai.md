# Survival maintenance

Read [architecture](architecture.md), [invariants](invariants.md), root conventions
and the active issue. `src/lib.rs` owns portable meter and clock contracts. Keep
native clock admission and whole-session persistence in runtime (#371). Run
`cargo test -p salimon-survival --locked`, then workspace build/tests/Clippy/fmt
for cross-crate changes. Add deterministic boundary and partition tests for rules.

`src/restrictions.rs` owns #369 thresholds, exact penalty DTOs, permission policy
and validated numeric tuning; `tests/restrictions.rs` checks fractional boundaries.
Runtime applies the policy in `session/survival.rs`; character and ship own the
physical consumers. Do not duplicate thresholds in input handlers or the HUD.

`src/consumables.rs` owns #370 item recovery data/validation and atomic application;
`tests/consumables.rs` covers timed food/medical benefits, registry conflicts,
terminal ordering, treatment, exact restoration and offline freeze. Runtime owns
item selection/removal, never this crate. See the canonical consumable contract.


`effect_snapshots()` (#373) is the immutable presentation seam for captured effect
rules, stacks and live timers. Keep descriptions/native visibility in runtime,
not the survival owner, and preserve stable EffectId iteration and offline freeze.

#378 extends the existing catalogs with four food definitions and captured food
modifiers; see the owning architecture for provisional tuning and regressions.
Keep advanced food recipes locked and preserve exact-slot consumption; no new
progression acquisition or native Q-wheel input is introduced.

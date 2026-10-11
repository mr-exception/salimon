# Physics maintenance guide

Read [architecture](architecture.md), [invariants](invariants.md), the canonical
[ownership decision](../../docs/technical-architecture.md#physical-object-simulation-decision-114),
[root conventions](../../docs/coding-conventions.md) and
[validation](../../docs/validation.md) before editing.

`salimon-physics` owns small physical-object rules; its only dependency is
`salimon-math`. No renderer, native events, resource identities, catalog or
character/ship controllers belong here. Future compatible cargo/equipment/debris
adapters should use this crate. Keep feature identity/lifetime and environmental
selection with their caller. Tests in `src/lib.rs` and `src/convex.rs` protect motion/contact policy;
runtime fragment tests protect integration with the existing resource session.
Run `cargo test -p salimon-physics --locked` while iterating and workspace gates
for contract changes. Record task evidence under root `reports/`.


Persistent activation policy lives in `src/sleep.rs`; its stable-ID cache is
owned by each caller. Read the [sleep contract](architecture.md#persistent-contact-island-sleep-156).
`cargo run --release --locked -p salimon-physics --example sleep_cost` isolates
sleep/active solver work on identical supported cube snapshots (500/1,000,
100 samples, mixed masses). It does not replace authored cargo/native #155 runs.

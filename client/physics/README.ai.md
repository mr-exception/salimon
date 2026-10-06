# Physics maintenance guide

Read [architecture](architecture.md), [invariants](invariants.md), the canonical
[ownership decision](../../docs/technical-architecture.md#physical-object-simulation-decision-114),
[root conventions](../../docs/coding-conventions.md) and
[validation](../../docs/validation.md) before editing.

`salimon-physics` owns small physical-object rules; its only dependency is
`salimon-math`. No renderer, native events, resource identities, catalog or
character/ship controllers belong here. Future compatible cargo/equipment/debris
adapters should use this crate. Keep feature identity/lifetime and environmental
selection with their caller. Tests in `src/lib.rs` protect motion/contact policy;
runtime fragment tests protect integration with the existing resource session.
Run `cargo test -p salimon-physics --locked` while iterating and workspace gates
for contract changes. Record task evidence under root `reports/`.

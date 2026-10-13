# Physics maintenance guide

Read [architecture](architecture.md), [invariants](invariants.md), the canonical
[ownership decision](../../docs/technical-architecture.md#physical-object-simulation-decision-114),
[root conventions](../../docs/coding-conventions.md) and
[validation](../../docs/validation.md) before editing.

`salimon-physics` owns small physical-object rules; its only dependency is
`salimon-math`. No renderer, native events, resource identities, catalog or
character/ship controllers belong here. Future compatible cargo/equipment/debris
adapters should use this crate. Keep feature identity/lifetime and environmental
selection with their caller. Tests in `src/lib.rs`, `src/broad_phase.rs` and `src/convex.rs` protect motion/contact policy;
runtime fragment tests protect integration with the existing resource session.
Run `cargo test -p salimon-physics --locked` while iterating and workspace gates
for contract changes. Record task evidence under root `reports/`.

`ConvexHull::complexity` is read-only geometry instrumentation. The runtime's
opt-in [geometry evaluation](../../scripts/FRAGMENT_BENCHMARKS.md#resource-geometry-evaluation-160)
combines these counts with authored vertices; asset loading remains outside physics.


Persistent activation policy lives in `src/sleep.rs`; its stable-ID cache is
owned by each caller. It also owns stable-ID warm contact reactions; changes to
snapshot matching, support invalidation or substep timing must preserve both
activation, reaction-cache safety and exact retained-projection equivalence.
Read the [sleep contract](architecture.md#persistent-contact-island-sleep-156).
`cargo run --release --locked -p salimon-physics --example sleep_cost` isolates
sleep/active solver work on identical supported cube snapshots (500/1,000,
100 samples, mixed masses). It does not replace authored cargo/native #155 runs.

Spatial candidate/cache maintenance follows
[the broad-phase contract](architecture.md#deterministic-spatial-candidates-157).
`candidate_filtering_cost` is an ignored release test isolating sparse all-pairs
versus grid filtering; retain exact-pose all-pairs and old-projection oracles.

The convex motion target is recomputed after each response; preserve shared
progression, swept waking and correction-created contact refresh. `tick_cost`
profiles settled/fast-impact/active 500/1,000 CPU cases; it does not establish native
reference-machine performance. See [active-group evaluation](architecture.md#active-group-evaluation-159).

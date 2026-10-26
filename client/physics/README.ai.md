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
The benchmark guide's [spatial attribution commands](../../scripts/FRAGMENT_BENCHMARKS.md#spatial-candidate-attribution-157)
also cover `separated_sphere_contact_cost` for empty-row, grid-refresh and
all-awake activation costs (#255/#288).

The convex motion target is recomputed after each response; preserve shared
progression, swept waking and correction-created contact refresh. `tick_cost`
profiles settled/fast-impact/active 500/1,000 CPU cases; it does not establish native
reference-machine performance. See [active-group evaluation](architecture.md#active-group-evaluation-159).

## Terrain integration (#179–#181)

Follow the canonical [integration/budgets/validation guide](../../docs/terrain-validation.md)
and [world definition contract](../world/terrain-contract.md). Keep local rules
and their consumers aligned; flat deterministic fixtures are explicit.

The #559 unique-visit query optimization and epoch rollover have exact bounds,
all-pairs and correction-created-contact regressions in `broad_phase.rs`.
`StepStats::grid_query_time` is a subset of profiled contact time. Historical
runtime stress reproduction lives in `reports/issue-559`; preserve geometry and
full inventories when comparing versions of the supported-deck fixture.


## Exact impulse fixed points (#592)

One-point ground/pair response loops retain the sixteen-iteration policy. After
one complete iteration, the implementation may elide the remaining iterations
only if every mutable iterative input has identical IEEE bits: both linear and
angular velocities (one body for ground), accumulated normal impulse, tangent
impulse and rolling torque. Normal/arms/targets, mass/inertia and all geometric
inputs remain constant within this pure loop; position/environment processing
stays outside it. An exact fixed point deterministically repeats the same state.
No epsilon, approximate convergence, active/sleep/contact rule, damping or time
budget changes. Multi-point manifolds retain every original iteration.

`iteration_oracle.rs` retains the old sixteen-iteration ground/pair response for
exact serialized-state and warm-continuation checks. Existing spatial all-pairs,
correction-created contact, retained projection and sleep regressions remain.
Small shared arithmetic primitives are inlinable across the crate boundary;
their expressions and IEEE ordering are unchanged. See the #592 report for
archived mass/radius-bit pools, partial/timeouts, profiles and exact comparisons.


Contact response (#644) lives in `src/response.rs`; root `src/lib.rs` schedules
substeps and broad-phase traversal. Read the [response owner](architecture.md#contact-response-owner-644)
and frozen iteration oracle before changing impulse order, tolerances, warm
state or allocations. `convex.rs` remains the coherent hull/SAT/feature owner.

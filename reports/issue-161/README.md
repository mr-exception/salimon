# Issue #161 — custom solver / Rapier decision

## Outcome — 2026-10-13

**Keep the optimized custom fragment solver for the accepted Phase 0 iteration;
defer the isolated Rapier comparison and production adoption.** This uses #161's
explicit documented keep/defer option, not its alternative prototype-benchmark
route. No Rapier prototype was built, no Rapier timing/memory/stability result is
claimed, and no migration issues are warranted without comparative evidence.

The decision rests on the accepted dense **500** correctness and naturally
settled evidence from #156, plus merged #157/#159 improvements. It does **not**
mean active dense piles meet a performance target or that the custom solver is
faster than Rapier. The full active/stress matrix remains in
[#165](https://github.com/mr-exception/salimon/issues/165). Rapier remains the
first alternative to evaluate if that work requires a solver replacement.

At reviewed main `04d8120`, #155/#156/#157/#159 are closed and their implementations
are present (#159 merged in `f538d4e`). Historical reports' pending-acceptance
language is dated evidence, not an unresolved implementation dependency. The
user requested a review PR, waived new M1 verification and explicitly allowed
issue closure once the PR exists. This decision does not waive #165's separate
requirements or alter its scope.

## Why defer now

1. #156's revised acceptance already establishes usable, accurate, individually
   interactable settled 500 cargo. Replacing its solver for idle cost alone has
   no demonstrated benefit.
2. Active contact cost is still a real problem: #159's fast-drop probe improves
   timestep pressure but fully active work remains expensive, and #160 rejects
   cheap box proxies on accuracy grounds. Neither warrants inventing a Rapier
   speedup or assuming equivalent stack behavior.
3. A meaningful comparison requires persistent body/cache adapters, authored
   hulls, matching floor/containment and radial-gravity policy, supported waking,
   frame transfers and comparable timing boundaries. A generic falling-box demo
   would not answer those questions. Defer that engineering until an active
   workload/budget is selected; preserve the existing accepted behavior meanwhile.

This is a scoped keep/defer recommendation, not a claim that the original full
100/500/1,000 comparative experiment has run. Existing optimization evidence
satisfies the revised settled primary-load goal only. Active scalability and
long-term maintenance do not have a conclusive keep-versus-adopt answer yet.

## Evidence reviewed, not rerun

[Machine-readable review](evidence-review.json) records source hashes, original
revision/binary identity and extracted values. No evidence was copied between
machines or relabeled as a current benchmark. Separate histories are not a
single before/after comparison.

| Evidence / workload | Observations | Limit |
| --- | --- | --- |
| [#156 native dense settled 500](../issue-156/native-500-dense-settled-2026-10-12-summary.json) | Six facing/away windows, 1,801–1,802 intervals each; physics median 0.437–0.443 ms, p95 1.150–1.170 ms; frame medians 16.731–16.758 ms (about 59.67–59.77 FPS) | Apple M1/Metal, 1920×1080, scale factor 2, AutoVsync; older physics `047105c`, source `8f8fe61`; not today's build or a strict 60 FPS floor |
| [#156 native 500 interaction route](../issue-156/native-500-floor-navigation-2026-10-13.json) | Natural sleep, 25,000 m/s rotating flight, landing/takeoff, zero local drift, support 162 removed and dependent 238 falls, unrelated sleepers preserved, three pickup/drop cycles, 625 kg retained | Historical fixed-step correctness route, not active wall-clock timing |
| [#157 authored dense active 500 / 1,000](../issue-157/README.md#authored-5001000-matrix) | Median physics 1,299.86 / 2,848.69 ms; candidate visits 371,992 / 794,480, greatly below all-pairs counts | 20 exploratory samples, concurrent shared CPU work; no stable tails or reference hardware comparison |
| [#159 scattered cube probe](../issue-159/after.csv), 500 settled / one fast drop / all active | p95 0.093 / 31.627 / 137.401 ms; 0 / 3 / 22 substeps | 100 independent 16 ms replays, setup/cloning excluded; cubes on unlimited floor, not authored dense cargo |
| Same #159 probe, 1,000 stress | p95 0.220 / 64.619 / 347.950 ms | Separate stress evidence; 1,000 active p95 increased versus its baseline; not an adoption justification by itself |
| 100 authored load | Existing harness supports 100; no new measurement retained here | Do not infer a current budget from larger-load medians |
| Memory | #157 reports sparse pair/projection capacity and removal of the N×N matrix | Structural requested capacity excludes other scratch, allocator overhead and total RSS; Rapier memory unknown |

Settled and active rows must remain separate. The accepted historical 500 native
settled route is stronger evidence for the limited decision than zero-warmup
`settled_requested` rows that still contain moving objects. No cross-machine
speedup ratio is computed.

## Behavior and integration matrix

Rapier column below describes **requirements and feasibility**, not tested parity.
Its documented features cannot establish compatibility with Salimon policy.

| Concern | Current custom ownership/evidence | Rapier comparison requirement / risk |
| --- | --- | --- |
| Dense piles and authored geometry | Convex envelopes, angular/friction/manifold response; #156 dense correctness and #160 support-accuracy audit | Use all six material/variant hull vertex sets with identical scale, mass, pose and seed; match support/penetration accuracy, not merely sphere or cuboid count. Exact-volume inertia/solver defaults may change settling |
| Sleeping / support removal | Stable-ID `SleepTracker`, supported 0.5 s dwell; `support_removal_wakes_only_the_connected_pile_and_reordering_keeps_identity`; native 500 dependent fall | Persistent body sets; normal Rapier sleep differs from the custom point-envelope policy. Verify removed-support wake and unrelated sleepers; explicit adapter waking may be needed |
| Fast impacts / CCD | Adaptive substeps; `off_center_ground_contact_tips_and_fast_drop_does_not_cross_floor`, `fast_deck_impact_releases_substep_pressure_without_waking_remote_cargo`; no general CCD guarantee | Exercise deck and dynamic-pile impacts with translation and rotation, plus CCD configuration/cost. Rapier's dynamic-body CCD is opt-in and documented limitations still need tests |
| Streaming / session lifetime | Runtime/world retain IDs, material, mass and in-memory session state; `sleeping_ship_fragment_follows_rigid_motion_and_streaming_return_wakes_safely` | Maintain external ID-to-handle map; remove absent bodies/caches and recreate in stable order. Rapier handles/snapshots cannot become authoritative world IDs or disk persistence |
| Carrying / frame transfer | Carried piece excluded; release/transfer bypass incompatible pose history; `adapter_excludes_carried_and_distant_fragments`, `adapter_preserves_entity_contract_and_maps_rotated_ship_frame` | Preserve exactly one carried object; remove/reinsert body without duplicate identity, mass loss or implicit ejection; transform linear/angular state at transfer boundary |
| Flight / landing / takeoff | Ship-local poses follow rigid ship transform; historical native 500 zero-drift route | Simulate local cargo rather than absolute moving ship at universe scale; preserve today's effective interior gravity and wake only actual force/support changes |
| Precision / future frames | Absolute `f64` world positions, relative subtraction before narrowing; one ship frame per physics call | Use `rapier3d-f64`, separate local simulation worlds per ship/body frame and explicit transfer; no cross-frame collisions. Radial planetary gravity needs per-body forces/local geometry rather than one global -Y vector |
| Deterministic repeatability | Stable ordered snapshots and deterministic candidate/response traversal; exact oracle/cache tests | Fixed timestep and insertion/removal order; pin compiler/library/features. Local replay first, cross-platform feature separately; different solver poses need tolerance-based behavior checks rather than custom bit equality |
| Rendering / clocks | Renderer-neutral DTOs; #159 shared 16 ms ticks, retained backlog, bounded calls and frame-local interpolation | Keep runtime clocks/presentation adapters; time full adapter + world step + writeback, not only engine internals; engine worlds never own renderer or mining state |

Named tests are in [physics sleep](../../client/physics/src/sleep.rs),
[convex contacts](../../client/physics/src/convex.rs) and
[runtime adapter](../../client/runtime/src/fragment_physics.rs). They were inspected,
not rerun in this documentation-only change.

## Dependency, platform and maintenance assessment

Upstream references reviewed on 2026-10-13:
[getting started](https://rapier.rs/docs/user_guides/rust/getting_started/),
[determinism](https://rapier.rs/docs/user_guides/rust/determinism/),
[sleeping](https://rapier.rs/docs/user_guides/rust/rigid_body_sleeping/),
[CCD](https://rapier.rs/docs/user_guides/rust/rigid_body_ccd/) and
[license/overview](https://rapier.rs/docs/).

Rapier is a low-level Rust physics library under Apache-2.0, compatible with the
no-full-engine direction. It offers an f64 crate, sleep and CCD; that establishes
candidate feasibility, not measured performance or stack stability. The live
Rust guide covers 0.36; package discovery listed `rapier3d-f64` 0.36.1. **No version
is selected or installed by this decision.** A future prototype must pin an exact
version and lockfile, verify its minimum Rust version against Salimon's 1.89,
and review transitive licenses/features. Version-specific manifest retrieval was
unavailable during this review, so exact MSRV/dependency count is unverified.

The 0.x API can change. Update cost includes Rapier/geometry/math transitive
packages, handle lifetime, cache invalidation, waking and force/frame adapters.
Cross-platform determinism is optional and constrains feature configuration;
initially avoid parallel execution and alternate SIMD configuration to keep the
comparison simple. Native macOS/Windows/Linux compilation, release size/build
time, WASM viability and deterministic replay must be measured for the selected
version; upstream platform support is not evidence those Salimon builds passed.
No production Cargo dependency, lockfile, public contract or build profile changes.

The custom solver has substantial maintained SAT/manifold/cache code and a dense
worst case. Rapier could reduce ownership of those algorithms but cannot remove
Salimon's frame/session/policy adapters. No person-day estimate or maintenance
saving is defensible from a documentation review.

## When to reopen the comparison

Reopen #161 or create a separate evaluation issue when repeated, isolated active
500 measurements miss an agreed interaction budget, ordinary drop/support routes
stall gameplay, stability needs general CCD, or maintaining the custom response
outweighs adapter cost. #165 is the existing matrix follow-up, not an automatic
Rapier migration task. This deferral adds no cargo cap/deletion or new dependency.

The reopened experiment must live outside production dependencies and reuse
#155's fixture/seed/authored hulls at **100, 500 primary and 1,000 stress**. Compare
scattered/dense/surface, actual naturally settled versus active (never force
sleep to satisfy a result), facing/away native cases and the behavior matrix
above. Keep reference Apple M1 1920×1080 settings from the
[benchmark guide](../../scripts/FRAGMENT_BENCHMARKS.md); use >=100 samples and three
isolated repeats, report median/p95/p99/max, preparation cost, process memory,
allocations, build size/time, repeatability and exact source/binary/features.
Separate equal-initial-state replays from naturally evolved steady worlds;
include setup/reconciliation separately and total production tick cost.

Select adopt only after an equivalent-accuracy prototype demonstrates repeatable
active benefit without settled regression, passes all ownership/frame/carrying
routes and has acceptable dependency/build/maintenance impact. Otherwise keep or
defer with explicit gaps. Only then create reviewable migration tasks for the
physics adapter, frame/identity lifecycle and production parity/rollout; do not
replace the solver as part of an evaluation.

## Validation and limitations

Passed for this change:

- Python 3 evidence review: parsed six historical native settled windows and the
  500 correctness route, checked sample counts, zero moving samples, completed
  support removal/three pickup cycles/zero drift, and recorded SHA-256 for four
  source files in `evidence-review.json`.
- Relative Markdown file links and named behavior tests checked against checkout.
- `git diff --check`.

Not run: Rust builds/tests/Clippy, new CPU/native benchmarks, Rapier prototype,
Rapier memory/stack/CCD parity or graphical tests. This is a documentation/evidence
review; runtime and dependencies are unchanged. Rust toolchain is not on this
session's PATH. **New M1 verification skipped at the user's explicit request.**
Historical M1 evidence is reused with its original identity/limits, not claimed
as fresh validation. A current head-to-head result remains deliberately deferred.

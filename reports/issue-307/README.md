# Issue #307 — shared resource queries and complete preparation accounting

2026-10-18. Base: `c14a6d6`. Linux x86_64, Xeon Platinum 8573C,
Rust 1.99.1; CPU-only observations, with other compilation/tests on the host.

## Delivered

- One immutable nearby result per exact position/seed/terrain mode. Mining,
  inspection and redraw share it; mass changes invalidate reconciliation using
  the world-owned deposit revision. Aim, obstruction and input release stay live.
- Candidate generation, authored placement misses/time, reconciliation, LOD/key
  selection, generation, complete CPU preparation and renderer upload CPU
  preparation have distinct inspection observations. GPU execution is separate.
- Cold LOD selection/balancing/key construction now advances in cooperative
  slices under the same 2 ms admission clock as generation. Queues are bounded
  (128 pending tree cells; 512 leaves/keys). Exact target changes discard stale
  work. Only complete caps publish; the far sphere/previous cap covers loading.
  No worker thread or background job infrastructure was needed.
- Existing identities, nominal candidate selection, terrain vertex support,
  deterministic topology and depleted tombstones are preserved.

## Measured budgets

The [release traversal log](release-cpu-profile.txt) records Earth/Moon/Mars
at 0/256/1200 m and return, all at actual sampled terrain height. Cold first
resource queries measured 27.40/14.86/44.74 ms respectively. Terrain steady
preparation after warmup measured 0.095–1.532 ms in this run. Complete terrain
preparation peaked at **39.670 ms** under host contention: the 2 ms admission
budget is soft, with the last indivisible unit/patch and scheduling delays
included in overrun counts. This is not a claim of a 2 ms wall-clock ceiling.
A generous 50 ms cold-resource observation envelope and 2 ms steady/preparation
admission target describe this corpus only, not universal hardware guarantees.
No test asserts wall-clock timings. Placement still has a synchronous first-query
cost; caching does not hide it. M1 budgets/FPS remain unverified in #313.

The initial unoptimized probe exposed approximately 0.06–0.33 s synchronous
selection, motivating cooperative work; exact-position repeats performed no new
candidate generation/reconciliation. New regressions cover streamed return,
depleted/restored copies, position/seed/mode/body invalidation, zero-time aim
loss/release, stale target rejection and equality to the synchronous reference.

## Validation

- **Passed:** `cargo test -p salimon-client -p salimon-world --locked`
  (155 runtime tests plus world/unit/integration tests; 4 opt-in runtime tests
  ignored), before the final cooperative scheduler addition.
- **Passed:** `cargo test --workspace --locked` before that scheduler addition.
- **Passed:** `cargo test -p salimon-client --locked terrain_patches -- --nocapture`
  after the scheduler addition: all 6 topology/coverage/cache/stale-target tests.
- **Passed:** `cargo test -p salimon-client --release --locked preparation_profile -- --ignored --nocapture`
  after the scheduler addition; all 12 cold/steady traversals converge.
- **Passed:** `cargo clippy --workspace --all-targets --locked -- -D warnings`,
  `cargo build --workspace --locked`, `cargo fmt --all -- --check`, `git diff --check`.
- Initial release compilation failed with zero-length archive objects and a
  subsequent stale-object link failure. `cargo clean -p gltf -p jiff --release`
  and `CARGO_BUILD_JOBS=2` rebuilt successfully; no dependency/source workaround.
- **Not run:** live GPU upload measurements, graphical scenarios, native Metal
  visuals/FPS. Upload counters are implemented, not fabricated measurements.
  [Native #313](https://github.com/mr-exception/salimon/issues/313) remains pending.

Portable delivery closes #307; it does not close or certify native #313.

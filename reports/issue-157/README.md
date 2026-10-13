# Issue #157 — deterministic fragment spatial candidates

## Outcome

Implemented a conservative uniform-grid broad phase, sparse exact SAT pair
storage, reusable transformed hull/candidate buffers, and spatial sleep wake /
support queries. The solver keeps the original ordered response law. Candidates
refresh after every positional/ground correction, including newly reachable
later pairs in the same pass. Separate calls retain ship-frame boundaries and
existing planetary/environment gates.

#155 is closed and its harness is available on the source baseline. Native Apple
M1 baseline/acceptance evidence is still absent. This implementation is reviewable,
but **native performance acceptance remains pending**; the PR relates to #157
without automatically closing it. CPU results below do not establish native
60 FPS or active-interaction budgets.

## Contracts and correctness

- Cell width is twice median radius. Radius AABBs cover rotations, contact skin
  and f64 rounding. More than 512 occupied cells per body/query uses an exact
  linear fallback, with no collision/population cap.
- Queries sort/deduplicate stable input indices; hash iteration never sets
  response order. Bounds update before querying later IDs after correction.
  Pre-integration wake queries include linear/angular/gravity travel, and the
  support graph uses the final pose with its existing 1 mm gap.
- Sparse geometry prunes distant pairs. Exact orientation/scale/hull changes
  refresh transformed vertices/normals/edges; rotation invalidates pair spans,
  while unchanged center-only geometry and warm reactions retain their policy.
  Stable-ID snapshot validation continues to handle external changes/removal.
- Candidate/grid/transformed-hull/SAT-index buffers reuse capacity. Fully sleeping
  groups release projections. Ground/manifold/response/motion temporaries remain.
- Tests compare exact poses against an all-pairs candidate oracle over dense,
  scattered, deterministic randomized, rotated, mixed-size, mixed sphere/convex,
  high-speed and correction-created-contact cases. An independent retained copy
  of the old projection arithmetic verifies cache shape/rotation refresh exactly.
  Existing sleep, support removal, growth, one-object carrying, IDs/mass/session
  and large-world ship-frame tests remain passing.
- Runtime profiler parity still compares exact fragment poses/accounting each
  update; its old N×N visit/allocation assertions now check candidate bounds.

Canonical behavior lives in
[physics architecture](../../client/physics/architecture.md#deterministic-spatial-candidates-157).
[Benchmark guide](../../scripts/FRAGMENT_BENCHMARKS.md#spatial-candidate-attribution-157)
records metric compatibility and attribution commands.

## Measurement environment and interpretation

2026-10-13 UTC; Linux 6.18.44 / Ubuntu 24.04 container, AMD EPYC 9V74,
9 visible CPUs, approximately 9.97 GB visible memory, Rust 1.99.0.
Baseline source: `a206afbdf425c424afaa9af93ef3a2f0ba85c751` (main at checkout).
Spatial source: this PR's dirty implementation on that base. Exact measured
executable SHA-256:

- Baseline: `768b90846fc1fc5fd4eec106e858f73ac8c39008b1e673a4d546530586783398`.
- Spatial: `a888d5656361064c07a553555f4afbe075997c6c8d6789883c0a6c93b969c5c3`.

No native window, GPU, rendering, presentation or FPS measurement. CPU matrix
uses 16 ms production simulation deltas, seed 155, zero warm-up, minimum 20
updates and at least one wall second per case. Requested settled authored loads
are **not verified settled**; report them separately from actual settled cube
snapshots. Twenty samples are exploratory, insufficient for trustworthy tails.
The matrix processes and some build/test work overlapped on shared CPUs; wall
numbers are descriptive only, not isolated performance gates or stable speedup
claims. Pair-work/storage counters provide structural attribution independent
of CPU contention. A dense pile remains quadratic in the worst case.

Reproduce both binaries in separate worktrees (build/copy baseline first so it
cannot overwrite the proposed executable), then run the following against each:

```sh
CARGO_PROFILE_RELEASE_CODEGEN_UNITS=1 cargo build --release --workspace --locked -j 2
python scripts/benchmark_fragments.py --binary /absolute/path/to/copied/salimon-client --mode physics --counts 500 1000 --layouts scattered dense surface --states settled active --warmup-seconds 0 --sample-seconds 1 --minimum-samples 20 --timeout 180 --machine-label container-cpu-only --output artifacts/issue-157/<baseline-or-spatial>
cargo run --release --locked -p salimon-physics --example sleep_cost
cargo test --release -p salimon-physics candidate_filtering_cost --locked -- --ignored --nocapture
```

Run benchmark processes sequentially without concurrent builds for actual
comparative wall budgets. Use #155's native settings, verified preparation,
100+ samples, three repeats and actual reference hardware for acceptance.

## Authored 500/1,000 matrix

Both binaries completed all 12 cases, each with at least 20 updates. Exact
accounting (IDs, source IDs, material, fragment/extracted mass) matches, and
contact, moving/awake/sleeping, substep and integrated-object counters match
for every corresponding retained update. See [parity](accounting-parity.json),
[baseline summary](baseline-summary.json), [spatial summary](spatial-summary.json)
and [raw JSON](raw/). This is bounded CPU evidence, not complete flight/landing/
streaming graphical acceptance or a verified settled authored-pile baseline.

The following are median total physics milliseconds and median pair visits.
`settled_requested` is explicitly an initially posed, still-active fixture.

| Count | Layout | Initial state | Main ms | Spatial ms | Main visits | Spatial visits | Main requested storage bytes | Spatial requested storage bytes |
| --- | --- | --- | ---: | ---: | ---: | ---: | ---: | ---: |
| 500 | scattered | settled_requested | 421.56 | 264.64 | 7,984,000 | 143,064 | 36,288 | 0 |
| 500 | scattered | active | 848.52 | 233.72 | 7,984,000 | 128,352 | 0 | 0 |
| 500 | dense | settled_requested | 2083.41 | 1545.18 | 7,984,000 | 381,394 | 8,546,304 | 8,546,568 |
| 500 | dense | active | 1328.28 | 1299.86 | 7,984,000 | 371,992 | 2,559,584 | 2,559,860 |
| 500 | surface | settled_requested | 269.66 | 129.84 | 7,984,000 | 32,192 | 2,012,000 | 0 |
| 500 | surface | active | 258.04 | 216.44 | 7,984,000 | 30,976 | 2,012,000 | 0 |
| 1000 | scattered | settled_requested | 1133.51 | 1186.40 | 31,968,000 | 301,528 | 36,288 | 0 |
| 1000 | scattered | active | 1065.80 | 555.34 | 31,968,000 | 272,136 | 0 | 0 |
| 1000 | dense | settled_requested | 3408.47 | 3029.69 | 31,968,000 | 800,594 | 9,388,272 | 8,546,868 |
| 1000 | dense | active | 3236.37 | 2848.69 | 31,968,000 | 794,480 | 5,710,208 | 5,710,208 |
| 1000 | surface | settled_requested | 839.52 | 227.82 | 31,968,000 | 66,048 | 8,024,000 | 0 |
| 1000 | surface | active | 840.66 | 192.68 | 31,968,000 | 63,360 | 8,024,000 | 0 |

The structural reduction is large; these exploratory concurrent wall windows do
not define a speedup budget. Dense exact response remains expensive. Storage
values exclude grid/hash-control/transformed-hull/response scratch and allocator
overhead; they measure requests, not total heap use.

## Separate filtering and storage attribution

[Filtering attribution](filtering-attribution.csv) runs a sparse-table solver
with all-pairs candidates versus the spatial filter, keeping response and
transformed geometry identical. Twenty replayed supported-floor or active-air
cube snapshots at 500/1,000 have zero pair geometry in either path. Pair visits
fall from 7,984,000 / 31,968,000 to zero. The all-pairs oracle retains grid refresh
bookkeeping, so this isolates candidate filtering within this implementation;
it is not the original matrix solver or a native performance reference.

[Original cube timings](baseline-cubes.csv) and [spatial cube timings](spatial-cubes.csv)
use 100 replay samples with mixed masses. Settled rows first pass real sleep dwell;
active rows replay actual airborne production inputs. Both fully sleeping paths
skip integration/pair storage, while stateless rows isolate active solver work.
These are simpler cubes, not authored cargo.
[Original storage samples](baseline-storage.csv) use five replays of the same
example (the added CSV storage fields were copied into the detached baseline
example only). Original stateless matrix requests are 4,024,000 bytes at 500 and
16,048,000 at 1,000, with 1,002 / 2,002 outer/row vector allocations per update.
Spatial stateless rows request zero pair-table/projection bytes and zero growth
events. This demonstrates matrix removal separately from pair filtering; total
allocation reduction still requires a process allocator profiler. Both active
and actual settled rows are preserved separately in the CSVs.

## Validation

Passed:

- `cargo fmt --all -- --check` and `git diff --check`.
- `CARGO_PROFILE_DEV_CODEGEN_UNITS=1 cargo clippy --workspace --all-targets --locked -j 2 -- -D warnings`.
- `CARGO_PROFILE_DEV_CODEGEN_UNITS=1 CARGO_PROFILE_TEST_CODEGEN_UNITS=1 cargo test --workspace --locked -j 2`: **361 passed**, four opt-in benchmark/GPU tests ignored.
- Final mixed sphere/convex oracle addition:
  `CARGO_INCREMENTAL=0 CARGO_PROFILE_DEV_CODEGEN_UNITS=1 CARGO_PROFILE_TEST_CODEGEN_UNITS=1 cargo test -p salimon-physics --locked -j 2`: **55 passed**, one opt-in attribution benchmark ignored.
- Final physics Clippy with incremental disabled and one codegen unit: passed.
- `python -m unittest discover -s scripts -p 'test_*.py'`: **55 passed**.
- `CARGO_PROFILE_RELEASE_CODEGEN_UNITS=1 cargo build --release --workspace --locked -j 2`: passed for both source trees.
- Both 12-case CPU matrices, cube replay examples and ignored filtering benchmark: passed.

Default Rust 1.99 multi-codegen builds failed locally while archiving unrelated
`ttf-parser`, `wgpu-core` and `naga` objects (`memory map must have a non-zero
length`). A later default incremental physics link also reported corrupt cache
and missing hidden symbols. One codegen unit succeeded; final physics tests
additionally disabled incremental compilation. These are local invocation
settings, not repository/profile/dependency changes. The compiler warning and
initial failed attempts are not counted as successful validation.

Not run: native graphical/E2E, packaged smoke, GPU timing or M1 performance.
This container has no display, Xvfb or Vulkan ICD. CI provides the existing
Linux graphical suite; Apple M1 1920×1080 reference runs remain necessary for
native acceptance. No screenshot evidence is claimed.

## Remaining acceptance and limitations

- Run #155's native reference settings with verified settled authored 500/1,000
  populations, 100+ samples and three isolated repeats; include active pickup/
  drop, support removal, takeoff/flight/landing and streaming return routes.
- Profile total allocations separately from these structural storage counters.
  Transformed-hull/SAT scratch reuse is implemented and correctness-tested, but
  its isolated native allocation/time contribution is not established here.
- Dense contacts and fallback queries retain quadratic worst-case work. Grid
  width is a conservative initial median-radius choice, not a universal optimum.
  Frequent exact correction refresh and existing manifold/ground response remain
  costly. This PR introduces no cargo cap, deletion, CCD promise or disk persistence.

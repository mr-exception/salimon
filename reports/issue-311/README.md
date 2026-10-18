# Issue #311 — coordinate boundaries and sphere dimensions

Implemented 2026-10-18 against `f058dbd` (merged #309 / #310).

## Outcome

Completed the portable acceptance criteria. The maintained
[boundary inventory](../../docs/technical-architecture.md#position-boundaries-and-resource-dimensions-311)
identifies persistent cargo, interpolation, solver, body-local terrain and GPU
rebasing owners and explains the narrow migration choices.

- Added character-owned `WorldPoint` / `ShipLocalPoint` DTOs and typed
  `ShipFrame` conversion methods. Persistent runtime ship anchors require the
  local type; drop, synchronization, simulation writeback, fixture/benchmark
  creation and JSON consumers explicitly cross their appropriate boundary.
  Character presentation interpolates ship-local point DTOs before unprojection;
  ship-attached fragment remapping uses the same pair.
- Renamed world sphere helpers to `fragment_diameter_meters` / `diameter_meters`.
  Renderer mesh instances and physics authored-hull consumers now name their
  dimensionless multiplier `authored_mesh_scale`. The shared sphere GLB radius
  has one exported constant. Analytic contact/support uses radius, and render
  history invalidation compares radius rather than an unused legacy hull slot.
- Kept existing f64 operations/axis order, domain normalization/quaternion rules,
  renderer dependency boundaries, mass/volume and scenarios stable. Terrain's
  existing body-local sample/patch DTOs and renderer's private prepared GPU DTOs
  remain with their current owners; no generic math framework was added.
- Preserved JSON field names and numeric meanings, including legacy fragment
  diameter `side_meters`, authored-deposit mesh scale `authored_visual.side_meters`
  and benchmark solver snapshots. The unused analytic-sphere solver scale slot
  retains its historical diameter value for snapshot compatibility; it does not
  determine contact geometry. There is no supported disk-save schema to migrate.

Updated canonical architecture/maintenance, coordinate/resource contracts and
character/runtime/renderer/physics guides. No assets, dependencies, lockfile,
terrain version, CI triggers or baseline corpus changes.

## Verification

Linux x86_64 container; rustc 1.99.1, Cargo 1.99.1. Commands from repository root.

| Check | Result |
| --- | --- |
| `cargo test --workspace --locked` | **Passed:** 519 tests, zero failures; 22 existing opt-in/doc tests ignored |
| `cargo test -p salimon-client --locked` after final adapter/JSON assertions | **Passed:** 164 tests; five existing opt-in tests ignored |
| `cargo clippy --workspace --all-targets --locked -- -D warnings` | **Passed** |
| `cargo build --workspace --locked` | **Passed** |
| `cargo fmt --all -- --check` | **Passed** |
| `git diff --check` | **Passed** |
| Changed canonical guide relative file links | **Passed:** checked Markdown file targets exist |
| `./target/debug/salimon-client --benchmark --mode physics --count 10 --layout onboard --state active --warmup-seconds 0 --sample-seconds 1` | **Passed:** ten-fragment production-session accounting smoke; no renderer |
| Native graphics/captures/FPS | **Not run:** no native desktop verification in this task; portable tests do not establish appearance or performance |
| Apple M1/Metal signoff | **Not run:** remains pending in [#314](https://github.com/mr-exception/salimon/issues/314) |

The [compact smoke result](accounting-smoke.json) retains the observed IDs, mass,
configuration and accounting result; disposable timing samples were not promoted.

Six added tests cover typed rotated/distant-anchor round trips, two compile-fail
world/local argument substitutions, the checked-in sphere's authored vertex radius,
render/diameter/pickup/contact agreement for every resource at local and 1e12 m
anchors, and analytic contact independence from authored mesh scale. Existing
composition tests additionally assert compatible JSON diameter/radius/local-array
fields through the mining/carrying/Core route. Existing interpolation, sleep,
quaternion, terrain and rebasing regressions passed.

During development an exact-equality assertion on an oblique round trip failed
at floating-point rounding precision. It was corrected to an explicit 0.5 mm
absolute tolerance covering f64 spacing at the 1e12 m anchor, with exact parity
still asserted against the original one-way transform. All final checks pass.

## Remaining verification

[Native #314](https://github.com/mr-exception/salimon/issues/314) remains open for
Metal visuals, baseline review, interpolation/lifecycle, latency and native
performance. No native signoff was attempted or inferred from CPU checks. This
PR closes #311 on merge without changing that follow-up's pending status.

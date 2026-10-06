# Issue #112 — focused character modules

Date: 2026-10-06. Base: `01d6106514b5564c6dcd2edd77155b7f713f4404`.

## Outcome

Decomposed `salimon-character` without changing its crate dependencies, public
root imports, method signatures, frame contracts, constants or gameplay rules.
`src/lib.rs` is now a 30-line public facade rather than a 3,500-line implementation.

- `controller/mod.rs` retains private mutable state and samples input/frame data
  once before dispatching one movement mode. Children own interior/cockpit,
  doorway, surface, EVA/nearby-body and camera behavior.
- `state.rs` and `input.rs` contain portable contracts and normalized controls.
- `layout.rs`, `collision.rs` and `queries.rs` separate shared geometry, collision
  sweeps and public sight/placement queries. `math.rs` retains domain vector
  policies while primitive arithmetic still comes from `salimon-math`.
- All 60 existing character regression tests remain intact, grouped with mode,
  camera, query and math owners plus controller collision integration tests.
  Shared fixtures are test-only; controller fields remain private.
- Generated `ship_anchors.rs` and `thruster_collision.rs` are unchanged.
- Character maintenance/architecture/invariant guides, the feature maintenance
  map and its coding-convention reference reflect the new locations.

## Validation

Environment: Linux x86_64; `rustc 1.99.0 (b940084d7 2026-09-28)`;
`cargo 1.99.0 (5f94df478 2026-08-27)`.

| Check | Result |
| --- | --- |
| `cargo fmt --all -- --check` | Passed |
| `cargo clippy --workspace --all-targets --locked -- -D warnings` | Passed |
| `cargo test -p salimon-character --locked` | Passed: 60 character tests |
| `cargo test --workspace --locked` | Passed: 260 tests, 0 failures; 1 existing GPU test explicitly ignored |
| `cargo build --workspace --locked` | Passed |
| `git diff --check` | Passed |
| Mechanical source comparison | All 60 original character test bodies unchanged apart from whitespace; moved helper/public method bodies and all four extracted movement-arm bodies preserved apart from whitespace |

The ignored renderer test is `resource_mesh::gpu_tests::headless_resource_pipeline`,
which explicitly requires a GPU backend. Native graphical/E2E runs were not run:
this environment has no Xvfb or installed Vulkan ICD. This report establishes
portable regression and workspace build/lint coverage, not graphical or hardware
performance evidence. No visual assets or generated spatial contracts changed,
so no screenshot or Blender regeneration is included. No implementation blocker
remains; the PR awaits review and merge.

# Issue #170 — stage 1: portable terrain contract

Date: 2026-10-13. Baseline: `f3270118915344a3b58707c9e6fbedc8fa8643df`.

## Outcome

Implemented the narrow definition/height-query stage permitted by #170's
sequencing notes. **This does not complete #170 or add visible terrain yet.**
Gameplay/presentation still use nominal spheres. The PR must not close #170.

World now owns immutable version-1 definitions, bounded hills/ridge layers,
sparse signed features (including crater depressions), conservative elevation
bounds, pure body-local height/position/normal/slope queries and explicit codec
validation. Representative Earth/Moon/Mars profiles serialize to 106 bytes each;
the maximum definition is 1,448 bytes. No runtime/GPU/serialization dependencies,
networking or full-planet allocations are introduced.

[Terrain contract](../../client/world/terrain-contract.md) documents coordinates,
normal approximation, format, cache identity/version migration, numerical
compatibility, preliminary cube-sphere choice and future consumer boundaries.
World guides and the maintenance map identify the new owners/tests.

## Verification

Linux x86_64; stable rustc/cargo 1.99.0, rustfmt and Clippy. Commands from root:

| Check | Result |
| --- | --- |
| `cargo fmt --all -- --check` | Passed |
| `cargo test -p salimon-world --locked` | Passed: 63 tests including seven terrain tests |
| `cargo clippy -p salimon-world --all-targets --locked -- -D warnings` | Passed |
| `cargo test --workspace --locked` | Passed; existing opt-in ignored benchmarks remain ignored |
| `cargo build --workspace --locked` | Passed |
| `cargo clippy --workspace --all-targets --locked -- -D warnings` | Passed |
| `git diff --check` | Passed |

Tests cover decode/encode and size budgets, unknown versions, invalid/nonfinite
inputs and radius bounds, truncated/corrupt records, same samples after reload,
reverse query order and multiple threads, world-center translation, cube seams
and poles, conservative extrema/outward normals, crater depth/support/slope and
version-1 golden heights/normals (including interpolated lattice samples).
Golden tolerances: 1e-7 m height and 1e-6 normal components. macOS/Windows
compatibility execution was not available in this Linux turn; existing platform
CI runs the workspace tests. No native screenshots or M1 performance measurement
were run: this stage has no renderer/physical-contact integration to validate.

## Remaining acceptance and dependencies

1. [#179](https://github.com/mr-exception/salimon/issues/179): bounded stable
   cube-sphere patches, LOD stitching, camera-relative precision, near/far
   presentation including masking nominal sphere over negative terrain.
2. [#180](https://github.com/mr-exception/salimon/issues/180): shared contact,
   traversal policies, footprint-aware touchdown/feet/ramp and resource/fragment
   anchoring; coordinate with #168/#169 and preserve 2 m hull clearance.
3. [#181](https://github.com/mr-exception/salimon/issues/181): native scenarios,
   screenshots, query/geometry tolerances, patch/triangle/memory/CPU/GPU budgets
   and measured comparison on reference hardware before parent closure.

No material/biome masks or erosion controls ship in version 1. Ridge normals
are finite-difference approximations; terrain material and gameplay policy remain
integration work. Representative +Z pits are fixtures and require spawn-site
integration before activation. No performance/FPS improvement is claimed from
these CPU-only query tests. Caves/tunnels/overhangs/through-holes remain explicitly
post-Phase-0 volumetric work, with no placeholder mechanics.

# Issue #89 — Blender-authored iron fragments

## Outcome

Iron ore fragments now render two editable Blender-authored variants through the
existing generic resource pipeline and batched opaque mesh renderer. Even stable
fragment IDs choose the original chunk; odd IDs choose the taller, narrower
asymmetric shard. All fragments use authored geometry, so the final procedural
fragment cuboid fallback is removed.

The original #84 iron work is incorporated: its 42 vertices, 80 triangles,
material assignments, hierarchy and logical asset ID remain. Recentring its
bottom pivot and applying a uniform 2.2 scale adapts it to the existing centered
unit-cube presentation contract. The second source evolves the same faceted
mesh rather than adding a disconnected copy of legacy work. All vertices fit
inside ±0.48 m, leaving margin inside the authoritative physical cube.

World mass/volume, mining output, one-object carrying, pickup/drop, ship-local
motion/contact and in-memory session persistence are unchanged. Runtime passes
world-owned center and physical side, deriving variant only from identity.
Tests cover selection across identities, masses and large-coordinate movement,
distinct actual exported vertices, authoritative sizing and scaled bounds.

## Assets and inspection

| Export | Triangles | Primitives/materials | Textures | GLB bytes |
| --- | ---: | ---: | ---: | ---: |
| `resource.iron-fragment` | 80 | 3 / 3 | 0 | 11,492 |
| `resource.iron-fragment-shard` | 80 | 3 / 3 | 0 | 11,492 |

Re-imported exported GLBs in Blender and visually inspected the distinct chunk
and shard silhouettes. Reusable asset previews and editing contracts:

- [Chunk source and preview](../../models/assets/resources/iron-fragment/README.md)
- [Shard source and preview](../../models/assets/resources/iron-fragment-shard/README.md)

Blender previews show its material lighting; the game uses the existing flat
base-color/local-facet shading, with no new shader or gameplay collision mesh.

## Validation

Environment: Ubuntu 24.04, Rust/Cargo 1.91.1, Python 3.12.14, Blender 4.0.2 and
Ubuntu jsonschema 4.10.3. The repository recommends Blender 4.5 LTS and pins
jsonschema 4.26.0; those versions were unavailable locally. The original 4.5.3
source was opened and saved with 4.0.2; mesh/material/hierarchy contracts passed
export and consumer checks.

Passed:

- `python models/tools/export_asset.py resource.iron-fragment`
- `python models/tools/export_asset.py resource.iron-fragment-shard`
- `python models/tools/validate_asset.py resource.iron-fragment`
- `python models/tools/validate_asset.py resource.iron-fragment-shard`
- `BLENDER=/path/to/blender python -m unittest discover -s models/tests -v` —
  45 tests including real Blender resource/item exports.
- `python -m unittest discover -s scripts -p 'test_*.py'` — 34 tests.
- `cargo fmt --all -- --check`
- `cargo test --workspace --locked -j 2` — 245 passed; 1 GPU-dependent tests ignored by default.
- `cargo clippy --workspace --all-targets --locked -j 2 -- -D warnings`
- `cargo build --workspace --locked -j 2`
- `WGPU_BACKEND=vulkan VK_DRIVER_FILES=/path/to/lvp_icd.json cargo test
  -p salimon-renderer --locked headless_resource_pipeline -- --ignored` —
  passed on Mesa software Vulkan; all six embedded fragment meshes load and
  the resource pipeline/shader validates without a window.

Rust checks use `CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0
CARGO_INCREMENTAL=0` and two build jobs to reduce temporary build resource use.
An initial build-artifact failure was cleared with `cargo clean` before retry.

Local native E2E/capture is unavailable: Xvfb fails to establish its local/unix
listening sockets in this execution environment. This is an environment failure,
not passing gameplay evidence. No native screenshots are claimed. The normal
PR CI workflow remains the cross-platform and Linux graphical verification gate,
including resource collection/transfer/streaming scenarios. Apple M1 performance
and macOS/Windows graphics fidelity were not measured here.

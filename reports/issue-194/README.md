# Issue #194 — Physical Core status display

Implemented a distinct Blender-authored Core-local screen, ship-owned live
consumption/conversion/depletion telemetry, cached rendering, low-energy warnings
at 10%, and automatic blank/restart on explicit bus power. Mechanical intake
continues without electricity through the existing #189 contract.

The screen is inside the unchanged Core collider and faces the starboard walking
lane. Source/export validation protects exact corners, normals, top-left UVs,
component ownership and budget: 6,102 triangles, 116 primitives, 13 materials,
513,096 GLB bytes; still two runtime ship draws. The shared instrument texture is
2048 × 256 (2 MiB), uploaded only when displayed values change.

Depletion assumes unchanged ordinary load and only the current fragment. It
accounts for conversion ending/capacity waste, excludes isolated assist energy,
and has no finite estimate for zero load. Consumption includes active reserved
assist power; storage/depletion describe the ordinary bus.

Validation (Linux x86_64, Rust 1.99.0, Python 3.12, Blender 4.5.3 LTS):

- `cargo fmt --all -- --check` — passed.
- `cargo test --workspace --locked` — passed (existing opt-in GPU tests ignored).
- `cargo clippy --workspace --all-targets --locked -- -D warnings` — passed.
- `cargo build --workspace --locked` — passed.
- `python models/assets/ships/salimon-scout/export.py --blender /tmp/blender-4.5.3-linux-x64/blender` — passed.
- `BLENDER=/tmp/blender-4.5.3-linux-x64/blender python -m unittest discover -s models/tests -v` — 55 passed, including source/interchange and modular ownership checks.
- `python -m unittest discover -s scripts -p 'test_*.py'` — 55 passed.

Graphical GPU/native smoke was not run: this host has no display/Vulkan ICD and
package setup could not establish one. CPU/source checks do not establish visual
fidelity or hardware performance. Apple M1 visual/regression signoff is outside
this issue and remains in the native follow-up.

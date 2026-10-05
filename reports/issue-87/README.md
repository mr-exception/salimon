# Issue #87: authored silicate deposits

Four editable Blender sources and validated generic GLB exports replace the
silicate cuboid: boulder, layered slab, ridge and scree. Muted opaque olive/grey
materials match silicate fragments and differ from rusty iron and blue ice.
Local deposit ID modulo four selects the variant independently of streaming,
camera, remaining mass and query order. Depleted deposits emit no visual.

Runtime supplies the absolute world center and uniform scale
`bounds_radius_meters / (0.48 * sqrt(3))`. All baked vertices fit ±0.48 m, so
visuals stay inside the existing authoritative spherical bound. Generation,
mining, mass, in-memory persistence and streaming controllers are unchanged.
Renderer adds the exports to its existing opaque resource batch and preserves
subtract-before-narrowing precision. Automation exposes `authored_visual`
(mesh, center, scale); scenarios now assert mesh routing and depleted absence
while retaining existing identity, mass, mining and streaming assertions.

![Four exported deposit variants](silicate-deposit-preview.png)

This comparison plots actual checked-in GLB triangles/materials. It is asset
inspection, not a native-game screenshot or hardware evidence.

## Validation

Environment: Linux x86_64, Python 3.12.14, Blender 4.5.3 LTS,
jsonschema 4.26.0; standalone rustfmt 1.99.0.

- For each of `boulder slab ridge scree`, ran
  `/tmp/blender-4.5.3-linux-x64/blender --background --python-exit-code 1 --python models/assets/resources/silicate-deposit-<variant>/create_source.py`,
  then `python models/tools/export_asset.py resource.silicate-deposit-<variant> --blender /tmp/blender-4.5.3-linux-x64/blender`:
  passed; generic validation executes before atomic export replacement.
- `python models/tools/validate_asset.py resource.silicate-deposit-<variant>`:
  passed for all four exports.
- `BLENDER=/tmp/blender-4.5.3-linux-x64/blender python -m unittest discover -s models/tests -v`:
  passed, 47 tests including real Blender integration and new deposit validation.
- `python -m unittest discover -s scripts -p 'test_*.py'`:
  passed, 34 tests, including a final rerun after scenario assertion updates.
- Standalone rustfmt applied to changed Rust files; `git diff --check`: passed.
- [Native builds run 37312274224](https://github.com/mr-exception/salimon/actions/runs/37312274224):
  passed on macOS 14, Windows 2022 and Ubuntu 24.04 for implementation commit
  `08d6d85e92c4a943e4708dc8e05f1a3344903a94`. All three ran
  `cargo fmt --all -- --check`,
  `cargo clippy --workspace --all-targets --locked -- -D warnings`,
  `cargo test --workspace --locked`, model/script checks and debug/release staging.
  Linux also passed deterministic baseline, lower-cockpit screenshot,
  resource-collection and ship-EVA evidence suites, and real X11 packaged smoke.
  Native screenshots/state/build artifacts are attached to that run.
- Cargo was not installed locally; CI provided workspace and native coverage.
- Re-exported all four saved compressed Blender sources through the generic
  exporter: passed with byte-identical GLBs (`sha256sum -c`).
- Changed documentation relative-link checks: passed.

This final report-only update does not change the successfully tested implementation.

| Variant | Triangles | GLB bytes | Materials/primitives | Texture bytes |
| --- | ---: | ---: | --- | ---: |
| Boulder | 38 | 4,028 | 2 / 2 | 0 |
| Slab | 76 | 5,932 | 2 / 2 | 0 |
| Ridge | 114 | 7,848 | 2 / 2 | 0 |
| Scree | 190 | 11,664 | 2 / 2 | 0 |

Regression tests cover exported geometry uniqueness, opacity/materials, baked
transforms, spherical bounds, stable selection through rematerialization and
partial extraction, depletion, iron routing and large-origin precision.

## Limits

Existing axis-independent placement remains centered at the surface anchor;
no radial orientation or collider policy is introduced. Software Vulkan does
not establish macOS/Windows hardware fidelity or reference-machine performance.
The issue remains open until the linked PR is reviewed and merged.

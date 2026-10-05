# Issue #88: authored water-ice deposits

Four editable Blender sources and checked-in generic GLB exports replace the
water-ice deposit cuboid: spire, crown, ridge and shelf. Each has two opaque
blue/frost materials, no textures or transparency and a 240-triangle/32-KiB
budget. Local deposit ID modulo four selects the variant; session mass changes,
streaming and camera movement do not affect selection. Depleted deposits emit
no visual. Iron/silicate deposit presentation is unchanged.

Runtime composition supplies absolute world centers and a uniform visual scale
of `bounds_radius_meters / (0.48 * sqrt(3))`. All baked vertices fit ±0.48 m, so
the entire visual stays inside the existing spherical gameplay bound on every
body/latitude. No generation, mining, mass, persistence, contact or streaming
code is changed. Renderer loads these assets into the existing opaque resource
batch and retains subtract-before-narrowing camera precision.

![Four authored deposit exports](ice-deposit-preview.png)

This comparison plots actual checked-in GLB triangles and material colors;
it is asset inspection, not a native-game screenshot or hardware evidence.

## Validation

Environment: Linux x86_64, Python 3.12.14, Blender 4.5.3 LTS (67807e1800cc),
jsonschema 4.26.0.

- For each of `spire crown ridge shelf`, ran
  `/tmp/blender-4.5.3-linux-x64/blender --background --python-exit-code 1 --python models/assets/resources/water-ice-deposit-<variant>/create_source.py`,
  then `python models/tools/export_asset.py resource.water-ice-deposit-<variant> --blender /tmp/blender-4.5.3-linux-x64/blender`:
  passed; generic semantic validation executes before atomic export replacement.
- `BLENDER=/tmp/blender-4.5.3-linux-x64/blender python -m unittest discover -s models/tests -v`:
  passed, 46 tests including real Blender round trips and all four deposit assets.
- `python -m unittest discover -s scripts -p 'test_*.py'`:
  passed, 34 tests.
- `git diff --check`: passed.
- [Native builds run 37308925737](https://github.com/mr-exception/salimon/actions/runs/37308925737):
  passed on macOS 14, Windows 2022 and Ubuntu 24.04 for implementation commit
  `2cb822a5a9d61465e5c70f8d2bcedf02ce82eeff`. All three ran
  `cargo fmt --all -- --check`,
  `cargo clippy --workspace --all-targets --locked -- -D warnings`,
  `cargo test --workspace --locked`, model/script tests, and debug/release staging.
  Linux also passed the deterministic baseline, lower-cockpit screenshot,
  resource-collection and ship-EVA evidence suites, and real X11 packaged smoke.
  Build/screenshot/state artifacts are attached to that run.
- Rust was not installed locally: automatic approval review rejected executing
  the downloaded Rust installer, so the existing CI provided Rust/native coverage.
  Initial CI caught two formatting differences and a test slice-iteration warning;
  both were corrected before the successful run.
- Changed documentation relative-link checks: passed.
- Re-exported all four compressed editable sources: passed with identical GLBs.

Measured generic export budgets:

| Variant | Triangles | GLB bytes | Materials/primitives | Texture bytes |
| --- | ---: | ---: | --- | ---: |
| Spire | 32 | 3,716 | 2 / 2 | 0 |
| Crown | 128 | 8,620 | 2 / 2 | 0 |
| Ridge | 96 | 6,992 | 2 / 2 | 0 |
| Shelf | 64 | 5,348 | 2 / 2 | 0 |

This final report-only update does not change the successfully tested implementation.

Regression coverage checks opaque materials, baked transforms, distinct exported
geometry, actual vertex bounds, the spherical scaling rule, stable identity
across rematerialization/partial extraction, depletion, non-ice routing and
large-origin precision. Existing world/mining/session tests and resource native
scenarios exercise unchanged gameplay through CI.

## Limits

The existing axis-independent deposit placement remains centered at the surface
anchor; there is no new radial orientation. CPU/asset checks alone do not prove
native presentation. Linux software Vulkan CI does not establish macOS/Windows
hardware fidelity or reference-machine performance. The issue stays open until
its linked PR is reviewed and merged.

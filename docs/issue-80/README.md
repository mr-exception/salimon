# Issue #80 — Blender scout visual migration

The checked-in scout GLB is exported from the committed Blender 4.5.3 source.
The runtime destination and Rust renderer/gameplay code are unchanged. The ship
adapter uses shared headless export, shared semantic/budget checks, a preserved
ship extension and the retained detailed validator before publishing outputs.
Asset-level legacy metadata is preserved separately; no procedural geometry is
used to produce the exported visual meshes or materials. The old Python CLI
can regenerate only the two Rust spatial layouts pending #81–#83.

## Comparison with the previous runtime

[geometry-comparison.json](geometry-comparison.json) compares every triangle
against the pre-migration GLB, including winding, position, normal, UV and
material role, independent of vertex reordering/merging. All 120 visual meshes
and 5,938 triangles match. Position delta is zero; maximum UV delta is
2.9802322387695312e-8 and maximum normal component delta is
0.0003999967884737998 from DCC normal encoding. There is no intentional visual
or spatial change. Node names, parentage, transforms, node extras, door metadata,
all monitor contracts, materials and glass behavior remain preserved.

The GLB shrinks from 513,824 to 485,332 bytes. Existing caps remain unchanged:
6,000 triangles, 120 primitives, 13 materials and 512 KiB. The shader's two-draw
opaque/glass submission and runtime door/monitor behavior are unchanged.
`models/assets/ships/salimon-scout/export-report.json` records the exact source
and output hashes plus measured budgets.

## Validation

- Headless Blender source verification: 150 objects, 5,938 triangles, 13 materials.
- Shared validation and legacy ship validation: pass, including geometry bounds,
  monitor planes/UVs/normals, sightlines, cargo/engine collision and Rust layouts.
- Models: 26 tests pass with Blender enabled, including the real shared export
  integration and scout provenance/material/spatial/failure-preservation tests.
- Build/E2E tooling: 34 Python tests pass.
- `cargo fmt --all -- --check`: pass.
- `cargo clippy --workspace --all-targets --locked --offline -- -D warnings`: pass.
- `cargo test --workspace --locked --offline`: 231 tests pass.
- `cargo build --workspace --locked --offline`: pass.

### Native graphics follow-up

The original local X11 launch failed before ready. Subsequent push CI for the
exact implementation commit `2be215c41f2452c43ce9d8f0a63522517a455543`
completed successfully on 2026-10-02:
[Native builds run 37019407848](https://github.com/mr-exception/salimon/actions/runs/37019407848).

The downloaded Linux artifact was verified against its published SHA-256.
[native-ci-validation.json](native-ci-validation.json) records every scenario
result, screenshot count, artifact ID and digest. All 27 deterministic/evidence
scenario runs passed, as did the packaged launch/real X11 input smoke. This
includes the lower-cockpit window/landing/takeoff route, space airlock door
close/open and reentry, moving/nearby EVA, cockpit-to-cargo navigation, physical
cargo during flight, and the complete resource loop. Screenshot checkpoints
were captured successfully. Reviewed forward/standing cockpit screenshots show
readable live monitor textures; the cargo screenshot shows the preserved room
and the closed-door checkpoint shows the blocking door surface.

Builds, Rust quality gates and Python tooling tests passed on Linux, macOS and
Windows. Native graphical scenarios ran on Linux Xvfb with Mesa software Vulkan;
macOS and Windows graphics, reference-hardware performance, resize and
minimize/restore were not exercised by this CI run. Those broader platform
smokes remain separate coverage limitations. The Linux native behavior and
visual evidence resolve the outstanding #80 migration validation; no runtime
code or asset changes were needed in this follow-up.

## Reproduce

```sh
python3 -m pip install -r models/tools/requirements.txt
python3 models/assets/ships/salimon-scout/export.py --blender /path/to/blender
python3 client/assets/ship/source/validate_salimon_phase0_ship.py
BLENDER=/path/to/blender python3 -m unittest discover -s models/tests -v
cargo test --workspace --locked
python3 scripts/salimon-test run scenarios/evidence/lower-cockpit-windows.json
python3 scripts/salimon-test suite --group ship-eva --evidence
```

The last two commands require a native display and working graphics driver.

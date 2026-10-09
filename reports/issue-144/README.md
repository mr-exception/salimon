# Issue #144 — resource gameplay size

Implemented against main `2af99bb2af27801ef3ee8da84f78f78e9d1538c3` on 2026-10-09.
The review branch enlarges every deposit/fragment variant by **10× on each axis**
without changing mass, density-derived solid volume, mining rate, yield or the
single-object carrying rule. Native gameplay screenshots remain pending; the
provided visual comparison is a CPU render of the actual exported vertices.

## Changes

- World `resource_size` is the shared geometry authority. A 2 kg fragment's
  gameplay cube side is approximately 0.830 m iron, 0.905 m silicate, 1.295 m ice.
  Material volume is still mass / density; bounding volume grows by 1,000×.
- Deposits retain initial geometry through partial mining and disappear at
  depletion. Runtime places their authored lower support 5 mm above terrain;
  the same center feeds the mesh, targeting and object prompt. Seed arithmetic,
  IDs, variant selection and generation query membership remain unchanged.
- Growing fragments use current mass; pickup still seals growth. Spawn lanes
  reserve full size and wider spacing. Physics receives a circumscribed cube
  sphere for pair/hull clearance and separate actual mesh support for ground/deck
  contact. Pair response remains an approximate sphere solver, without spinning.
- Carrying keeps the full cube ahead of the player's vertical capsule at every
  pitch and clamps its lower support above the floor/terrain. Invalid ship floor
  drops preserve the carried object. F input/equipment exclusivity is unchanged.
- Production-control baseline/evidence routes were recalibrated for larger
  fragment spawn positions, pickup aim, carrying clearance and cargo positions.
  Surface extraction, depletion, streaming, F exclusivity and cargo round trips
  remain protected by runtime walkthrough tests.

## Measured authored geometry

[All 18 before/after measurements](measurements.json) use every GLB POSITION
vertex with baked identity transforms. Fragment mass is 2 kg; deposit mass is
30 kg. Axes are world X/Y/Z. Every axis of every variant has exactly a 10×
visible extent ratio, independent of nominal cube/sphere bounds.

![Actual authored geometry at equal mass with 1.75 m player reference](authored-size-comparison.png)

This is an orthographic CPU mesh render, **not a native gameplay screenshot**.
Meshes are shown entirely above a comparison ground line; original deposits in
production were partly buried, so the visible in-game increase can exceed 10×.

| Variant | Mass (kg) | Before X/Y/Z (m) | After X/Y/Z (m) |
| --- | ---: | --- | --- |
| iron-deposit-ledge | 30 | 0.130/0.121/0.112 | 1.297/1.212/1.117 |
| iron-deposit-nodule | 30 | 0.123/0.138/0.100 | 1.231/1.382/0.996 |
| iron-deposit-rubble | 30 | 0.132/0.119/0.115 | 1.320/1.189/1.152 |
| iron-deposit-vein | 30 | 0.117/0.138/0.101 | 1.169/1.382/1.009 |
| iron-fragment | 2 | 0.073/0.055/0.058 | 0.730/0.548/0.584 |
| iron-fragment-shard | 2 | 0.040/0.070/0.043 | 0.404/0.701/0.426 |
| silicate-deposit-boulder | 30 | 0.125/0.151/0.105 | 1.247/1.507/1.047 |
| silicate-deposit-ridge | 30 | 0.140/0.151/0.091 | 1.399/1.507/0.915 |
| silicate-deposit-scree | 30 | 0.144/0.130/0.127 | 1.445/1.297/1.265 |
| silicate-deposit-slab | 30 | 0.136/0.113/0.111 | 1.363/1.132/1.114 |
| silicate-fragment-ridge | 2 | 0.066/0.067/0.054 | 0.661/0.670/0.543 |
| silicate-fragment-slab | 2 | 0.079/0.052/0.062 | 0.787/0.525/0.615 |
| water-ice-deposit-crown | 30 | 0.199/0.200/0.195 | 1.987/2.002/1.955 |
| water-ice-deposit-ridge | 30 | 0.210/0.205/0.095 | 2.103/2.050/0.950 |
| water-ice-deposit-shelf | 30 | 0.213/0.188/0.196 | 2.135/1.883/1.956 |
| water-ice-deposit-spire | 30 | 0.136/0.219/0.131 | 1.355/2.193/1.311 |
| water-ice-fragment-cluster | 2 | 0.095/0.109/0.084 | 0.955/1.088/0.836 |
| water-ice-fragment-shard | 2 | 0.067/0.114/0.058 | 0.674/1.140/0.583 |

## Validation

Environment: Linux x86_64, Rust/Cargo 1.99.0, Python 3.12.14, software Vulkan
lavapipe. All commands ran from the repository root with locked dependencies.

| Command | Result |
| --- | --- |
| `cargo fmt --all -- --check` | Passed |
| `cargo clippy --workspace --all-targets --locked -- -D warnings` | Passed |
| `cargo test --workspace --locked` | Passed: 288 tests; one GPU test ignored in the default suite |
| `cargo build --workspace --locked` | Passed |
| `python3 -m unittest discover -s scripts -p 'test_*.py'` | Passed: 34 tests |
| `WGPU_BACKEND=vulkan VK_DRIVER_FILES=/usr/share/vulkan/icd.d/lvp_icd.json cargo test --locked -p salimon-renderer headless_resource_pipeline -- --ignored` | Passed: resource pipeline initializes and validates on Vulkan without a window |
| `cargo test --locked -p salimon-client terrain_support_and_default_spacing -- --nocapture` | Passed: actual terrain support and deterministic restoration; proximity audit below |
| `git diff --check` | Passed |

Focused regressions include all three materials at small/growing/full mass,
all 18 scaled authored extents, axis/diagonal terrain support, partial/depleted
state, permanent one-object carrying, extreme up/down pitch, safe/unsafe cargo
drops and separate physical ground support versus pair radius. Runtime
walkthroughs exercise actual movement, mining, pickup/drop and streaming controls.
No assets or generated exports changed, so Blender regeneration was not needed.

## Spacing audit and limitations

At seed 0, 120 m queries on +Y, +Z and the positive cube seam sampled 1,695
deposits across the five solid bodies. The actual authored AABB audit found
four possible overlapping pairs: two among 273 Earth seam deposits and two
among 184 Mars seam deposits; all other sampled areas had zero. AABB overlap
is conservative and does not establish triangle intersection. Existing seeded
positions/IDs and all mass are retained; this change does not silently remove
neighbors or change the world seed by altering profile spacing. See
[spacing audit log](spacing-audit.log). This is sampled coverage, not a global
non-overlap guarantee. A separate spacing policy would need an explicit decision
about moving existing deposits while preserving stable identities.

Native graphical runs were attempted on the original binary before editing:

```sh
WGPU_BACKEND=vulkan VK_DRIVER_FILES=/usr/share/vulkan/icd.d/lvp_icd.json \
  xvfb-run -a -s '-screen 0 1280x800x24' python3 scripts/salimon-test suite \
  --group resource-collection --evidence --binary artifacts/issue144/before-client \
  --artifacts artifacts/issue144/before \
  --screenshot-command '["python3","scripts/capture_settled.py","{path}"]'
```

All six launches failed before readiness: `Failed to open connection to X
server`. Xvfb could not establish UNIX listening sockets. A TCP-only Xvfb
fallback started but clients still could not connect. Native rendering, input,
terrain appearance and cargo screenshot acceptance therefore remain **unverified**
in this environment; CPU/controller and headless GPU coverage do not replace
those checks. Existing synchronized evidence routes remain available for CI or
a desktop run. Do not interpret the authored comparison as screenshot evidence.
Reference macOS/Windows visual fidelity and hardware performance were not tested.

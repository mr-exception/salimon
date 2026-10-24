# Modular interior fixtures (#575)

Third change in the six-issue stack, based on #597. The existing moving berth is
now a stable, independently rendered fixture (ID 1001) driven only by saved
habitation and ship pose/power. The v1 layout explicitly maps the original mesh
names. Schema 10, original physical clearance/mass/IDs and legacy save migrations
are preserved. The original whole-ship presentation remains available for older
reference fixtures; native composition replaces its berth once.

Renderer-neutral local nodes resolve parents in linear time using f64 ship/room
transforms before camera rebasing. Duplicate IDs, invalid material/transforms,
missing parents and cycles fail. Immutable indexed templates share geometry;
48-byte instances batch by template, cull conservatively and retain GPU capacity.
Opaque fixtures precede glass. Unchanged instance bytes avoid repeated uploads.
The explicit 16,384-record staging guard fails before allocation; it does not
silently truncate physical populations.

A separate, linked Blender 4.5.3 source library provides shelf, seat, panel and
explicit damaged-panel prototypes. These are render templates, not installed
physical objects or invented damage. Original ship source/export budgets are
unchanged: 6,767 triangles, 134 primitives, 13 materials, 581,380 bytes. The
historical issue budget predates the reviewed parent caps (6,900/140/13/600 KiB).
The separate kit is 60 triangles, four primitives, three shared materials and
8,060 bytes; limits are 80/four/three/32 KiB. Each isolated template passes the
normal geometry diagnostics without overlap exemptions. Five resident templates
including the original berth occupy 5,488 indexed geometry bytes.

See [the maintained contract](../../docs/modular-interior-fixtures.md) for
ownership, API, migration, admission and reproduction commands. Runtime/renderer
guides, maintenance map and asset guides are updated in this PR. Supporting
physical domains never depend on renderer types. This does not implement ship
construction, damage simulation, cosmetic aging or future room gameplay.

## Validation and retained evidence

Actual host: macOS 26.2, Apple M1 / Metal, debug Rust build. The fixture GPU lane
is isolated production-wgpu offscreen rendering at 1280×800; native game captures
are 2560×1600 at DPI 2. These contexts are separate.

- `cargo test --workspace`: pass (runtime 366 passed / 13 ignored; renderer 80 passed / 15 ignored). `cargo clippy --workspace --all-targets -- -D warnings`, formatting, diff and documentation validation pass. Original results in [validation](validation/).
- Five renderer CPU tests cover hierarchy/errors, far-origin rebasing/culling,
  template sharing/damage and portable WGSL validation. Runtime save restoration
  verifies the stable ID, exact pose, unchanged mass and actual blackout source.
- Supported Blender source author/export, ordinary asset validation and isolated
  template quality checks pass. Full model suite retained in validation.
  Its first run exposed a macOS `/var` versus `/private/var` comparison in the
  disposable modular-source verifier. Resolving both paths fixes the verifier;
  production source geometry is unchanged by that portability repair.
- [Native route](native/scenario.json): all 17 steps pass after real window
  restoration before each capture. Original protocol/state/result/log bytes are
  compressed with [SHA-256 inventory](native/sha256.json). The earlier capture
  timeout is retained separately in [native-timeout](native-timeout/): the game
  presented only two frames and the capture stayed pending while backgrounded.
  No focus/occlusion or display-admission rule was bypassed.
- [GPU original rows](gpu/timings.csv): 16 samples for each of intact, independently
  moved, damaged, blackout and expanded-500 cases. Repeated identical cases have
  byte-identical PNGs; moved/damaged/blackout cases differ. No image is promoted
  as a human-approved baseline.

| Fixture workload | CPU prepare median / max (µs) | GPU median / max (ms) |
|---|---:|---:|
| Intact | 15.209 / 287.209 | 0.065 / 0.238 |
| Independent move | 16.625 / 42.542 | 0.065 / 0.068 |
| Damaged | 14.667 / 43.625 | 0.066 / 0.069 |
| Blackout | 19.021 / 52.417 | 0.067 / 0.073 |
| Expanded 500 | 1669.146 / 1997.292 | 0.084 / 0.087 |

The synthetic expanded lane places 500 fixtures on 3 m cells over five decks:
all 500 visible, five draws, 8,400 submitted triangles, 40,960 retained instance
buffer bytes and 64,960 staging/upload-copy capacity bytes. First upload is
24,064 bytes including the camera; unchanged frames upload only the 64-byte
camera uniform. CPU prepare includes validation, culling and staging, not domain
simulation or whole-frame work. The actual game berth adds one fixture draw to
the original two ship draws. Spikes remain in the raw CSV.

[Native stowed](native/step-007-bunk-stowed.png) ·
[Native deployed](native/step-012-bunk-deployed.png) ·
[Native restowed](native/step-017-bunk-restowed.png) ·
[Independent move](gpu/independent-move.png) · [Damaged](gpu/damaged.png) ·
[Blackout](gpu/blackout.png) · [500 fixtures](gpu/expanded-500.png)

These measurements establish functional integration and bounded fixture storage,
not a whole-game frame-budget pass (#563), human appearance approval (#561),
physical input/platform qualification (#557), or hosted Linux evidence. No PR is
merged by this task.

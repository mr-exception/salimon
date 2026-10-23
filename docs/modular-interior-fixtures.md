# Modular interior fixture boundary (v1, #575)

Runtime now composes a ship/room-rooted hierarchy of independently addressable
fixture observations. `FixtureNode` contains stable caller-owned ID, optional
parent, named template, f64 local pose/scale, bounded opaque tint and actual power
state. Resolution validates root/pose/material, rejects duplicate IDs, missing
parents and cycles, processes parent edges in linear time, and returns original
input order. Renderer types have no dependency on world/ship/character ownership.

A room can instantiate reusable fixtures at any approved 3 m structural cell or
multi-deck pose. This layer does not decide connection/clearance, install physical
objects, count mass, allocate gameplay IDs, animate mechanics, simulate damage or
assign cosmetic age. Those owners supply source state. A damaged template is a
rendering option, never evidence that physical damage occurred.

## Assets and v1 migration

`client/assets/ship/interior-layout-v1.json` binds the original named
Starter_BunkFrame/Starter_BunkCushion pair to stable fixture 1001 and existing
`session.habitation` source. All old supported runtime saves retain their explicit
migrations; schema 10 is unchanged. Mapping a restored state produces the same
ID/pose/power without ticking, mass changes or input replay. Native composition
selects independent presentation; the old paired surfaces are clipped only in
that mode. The retained legacy whole-ship DTO path supports reference fixtures.

The independent berth extracts original vertices/materials relative to its
existing generated hinge. Static hull, cockpit glass/monitor, Core, Fabricator,
Archive, ramp and collision/anchor contracts remain unchanged. Named template
coverage is mandatory and malformed geometry fails initialization. The other
four opaque prototypes have their own small linked Blender source/export and
can change without touching or re-exporting unrelated hull meshes. Future room
instances reuse the library at runtime; new mesh types register a library entry.

## Rendering and measured limits

Templates share immutable indexed geometry. Instances carry 48 bytes of rebased
center/scale, quaternion and tint/power. Root composition remains f64; camera
subtraction happens before GPU narrowing. Frustum admission uses measured
pivot-centered template radii. Opaque batches are grouped by template and render
before cockpit glass. Camera uniform uploads remain 64 bytes; unchanged instance
bytes are retained rather than uploaded again. Counts, triangle work, resident
geometry, instance capacities, staging capacities and preparation/upload time are
observable through `Renderer::fixture_batch_stats` and native debug inspection.

The renderer rejects a scene above 16,384 fixture records before staging. This
is an explicit memory-admission guard, not a physical object limit or silent
population cap. World streaming/physical owners still decide the visible source
set. Allocation capacities grow by powers of two and are retained for reuse.

The original #575 text quotes older 6,500 triangle / 124 primitive / 512 KiB
figures. The reviewed parent #538 already uses caps of **6,900 triangles, 140
primitives, 13 materials and 600 KiB**; its actual 6,767/134/13/581,380 export is
unchanged here. No inherited cap is raised. The separate kit is 60 triangles,
four primitives, three reused materials, 8,060 bytes, with an 80-triangle / four
primitive / three material / 32 KiB cap. Five resident templates including the
original 24-triangle berth occupy 5,488 indexed vertex/index bytes.

The opt-in GPU workload renders independently moved shelf/seat, an authored
panel variant, blackout and 500 synthetic fixtures over 3 m cells/five decks.
The 500-instance lane uses five draws and 40,960 retained instance-buffer bytes;
the actual one-berth game route adds one fixture batch to the original two ship
draws. See the task report for exact raw timings and context. These are fixture
workloads, not a whole-game 60 FPS pass or human scene approval. #563 owns hardware
budgets and #561 human appearance; #557 retains physical platform/input checks.

## Checks

```sh
cargo test -p salimon-renderer --locked fixtures::tests
cargo test -p salimon-client --locked session::fixtures
SALIMON_FIXTURE_EVIDENCE_DIR=/absolute/new/output cargo test -p salimon-renderer --locked modular_fixture_gpu_evidence -- --ignored --nocapture
python3 scripts/salimon-test run scenarios/evidence/modular-fixtures.json --binary target/debug/salimon-client --screenshot-command '["native"]'
```

The native scenario restores the actual window before captures. It does not fake
focus, disable occlusion or bypass display admission. Retain failed captures and
context; headless production-wgpu images remain separate from window/OS evidence.

# Canonical planet terrain streaming (#424)

`world::canonical` adapts the exact frozen U11 Vesper publication into a system-
local body. Its full catalog ID and galactic anchor remain in the publication;
local coordinates never flatten the distant galaxy anchor into global f64 meters.
The compressed six-body catalog and F2 fixtures remain separate and unchanged.
Normal startup migration is #426, with gameplay terrain consumers in #425.

`TerrainDefinition::canonical_start` delegates every height to the frozen U11
height evaluator. It does not substitute a new procedural noise recipe or revise
published geography. The same fixed 0.25 m normal stencil samples that evaluator.
The immutable cache codec uses distinct `SLMCAN01` magic plus the exact full-world
SHA256 and validates the known Vesper identity/radius. All `SLMTERR1` fixture bytes,
seeds and semantics are unchanged. There is no runtime disk dependency or data
regeneration; missing/corrupt catalog data still follows the #422 fail-closed path.
This bundled selected definition remains usable without a full real-data pack.

Runtime's existing cube-sphere quadtree now supports all six faces of the
canonical world, from distant orbit through surface detail. Patch edge size
tracks camera distance, with a 64 m minimum, the existing 16×16 vertex cells,
2:1 neighbor balancing and exact coarse-boundary midpoint stitching. Full sphere
coverage avoids a local-cap hole during orbital traversal. Only a complete target
set becomes active; stale preparation never publishes after a target change.
Vesper uses its physical published radius, 0–1,800 m relief and complete definition
identity. Mesh normals/heights are body-local; camera subtraction precedes f32.

The renderer uses the declared mask radius for terrain morphing, so whole-planet
meshes retain their relief rather than flattening everything beyond a fixture's
4,096 m cap. The canonical patch and far sphere use the same Slate presentation
material; biome/geology/material authoring remains its separately planned scope.

Existing bounds remain: 512 active / 1,024 cached CPU patches, eight generated
per frame and 2 ms cooperative preparation admission (a final unit may overrun
and is recorded). Renderer buffers retain the 32 MiB hard bound and 2 MiB upload
bound. Cache eviction/LOD never changes the physical evaluator or canonical ID.
Old complete geometry or the base sphere remains visible while preparing a new
set. Surface collision and resource queries are not derived from visual LOD.

## Reproduce

- `cargo test -p salimon-world --locked canonical`
- `cargo test -p salimon-client --locked canonical_ -- --nocapture`
- `cargo test -p salimon-client --locked terrain`
- Production Linux Xvfb/Mesa software-Vulkan route:
  `python3 scripts/salimon-test run scenarios/evidence/universe-terrain.json --binary target/debug/salimon-client`

The `canonical-terrain` E2E fixture changes presentation conditions only; its
`universe-orbit`, `universe-surface` and `universe-edge` reference views are
absolute system-local cameras. Fixed overhead illumination is declared capture
context, not an evolving astronomical sky or a claim of ephemeris light accuracy
(#423). The regular character/ship state remains an isolated fixture in this
rendering PR; #425/#426 integrate playable canonical spawning and interaction.
Screenshots and CPU/GPU preparation/memory observations belong in the issue report.
Software Vulkan establishes actual production rendering, not M1/Metal 60 FPS;
subjective presentation and hardware budgets remain #561/#563.

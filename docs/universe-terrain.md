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
material. [Geological shading](geological-materials.md) now supplies versioned
non-displacing material observations for other profiles while preserving Vesper;
climate and biospheres retain their separately planned scope.

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
absolute system-local cameras. The face-edge eye samples the authoritative
canonical terrain height and stays 500 m above it; a nominal-radius offset was
675 m underground at that location. Native `reference_camera` observations name
the view and report physical terrain clearance; the reviewer rejects nonpositive
clearance or a mismatched view. Fixed overhead illumination is declared capture
context, not an evolving astronomical sky or a claim of ephemeris light accuracy
(#423). The regular character/ship state remains an isolated fixture in this
rendering PR; #425/#426 integrate playable canonical spawning and interaction.
Screenshots and CPU/GPU preparation/memory observations belong in the issue report.
Software Vulkan establishes actual production rendering, not M1/Metal 60 FPS;
subjective presentation and hardware budgets remain #561/#563.


## Geological surface materials (#613)

Follow the [geological material contract](geological-materials.md) for terrain-bound
profile mapping, immutable material cache keys, the 52-byte vertex layout,
filtered stratification and original software-Vulkan evidence. Runtime owns
geology-to-DTO mapping; renderer owns shading only. Vesper and resource palettes,
physical queries, geometry and save identities are preserved.

## Cloud integration acceptance (#436)

The required `universe-render` evidence suite always selects production capture
routes: `universe-terrain.json`, `universe-journey.json` and `universe-mining.json`.
It covers orbit → surface → cube-face edge → orbit revisit, ordinary canonical
spawn/boarding and physical extraction/carry/reload. Reference camera changes
are explicit rendering probes, not a flown orbital trajectory.

```sh
python3 scripts/salimon-test suite --group universe-render --binary target/debug/salimon-client --artifacts artifacts/universe-render --screenshot-command '["native"]' --summary
python3 scripts/review_universe_captures.py artifacts/universe-render/RUN/result.json --output artifacts/universe-render/image-review.json
```

Replace `RUN` with the terrain route's actual run directory from the suite
summary. Run under the documented Linux Xvfb/Mesa environment. The reviewer
requires original GPU surfaces, actual viewport dimensions, the orbit disk
diameter/center at the declared 2.5-radius distance and 70-degree FOV (four
physical pixels allow raster edge rounding and published relief), settled nonempty
canonical geometry, four exact frozen RGB repeats and an exact orbit revisit.
It enforces existing 512 active / 1,024 cached patch, eight-generated-per-frame,
32 MiB GPU cache and 2 MiB upload limits; it does not relax those limits or certify
an artistic baseline. Blank/degenerate images, missing views, incompatible
contexts and failed runs reject. Review original images alongside these checks.

`terrain_patches.rs` tests full-sphere coverage, 2:1 topology, actual stitched
triangle-boundary pairing at canonical face edges/corners, exact geometry on
cache eviction/revisit and unchanged physical queries. Renderer sphere tests
cover sub-meter depth and trillion-meter translation; Universe sky projection
and the opt-in production `sky_production_reference` test cover physical angular
radius, motion, exact translated pixels and reverse-Z occlusion. Run the sky
capture separately with `SALIMON_SKY_DIR` set; it is a production-shader probe,
not a claim of complete real-star catalog coverage.

Runtime `terrain_observations.rs` retains the last 3,600 raw frame samples with
explicit column names: the existing bounded presented interval, CPU update/render, optional GPU pass
milliseconds and scene/total draw counts. Unavailable GPU samples serialize as
null. Presented intervals inherit `FrameClock`'s 250 ms cap and zero after an
interval reset; they are not uncapped wall-clock frame pacing. CPU update/render
and returned GPU duration samples retain their supplied values. The existing
`performance_reset` clears the sample window. Inspect includes
raw samples plus existing aggregate timing, allocator, terrain preparation,
cache and upload observations. This diagnostic window has no simulation or save
authority and is not a hardware performance threshold.

The [#436 report](https://github.com/mr-exception/salimon/blob/f1cd0b3e878fac487ce898547a0c8c35aa40519b/reports/issue-436/README.md) records actual commands, results,
originals and model limits. Fixed host illumination, authored Milky Way and optional
bounded star sources remain as documented in [the sky contract](universe-sky.md).
Arbitrary real ephemeris-frame gameplay and complete scientific catalogs are not
established by these captures. Subjective presentation remains #561, physical
platform behavior #557 and real hardware/M1 budgets #563, separately unverified.

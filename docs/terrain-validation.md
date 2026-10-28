# Terrain integration and validation

The canonical definition, coordinate/version rules and compact wire format live in
[the world terrain contract](../client/world/terrain-contract.md). This guide owns
integration budgets, approximation limits and the reproducible native workflow.

## Ownership and activation

Normal native gameplay activates immutable version-1 representative definitions
on Earth, Moon and Mars. Mercury/Venus retain zero elevation; Sun is visual-only.
Runtime supplies the same definition to character ground callbacks, ship contact/
landing, deposit anchoring, fragment contact and patch generation. Renderer has
no world dependency. Existing named flat scenarios and the F2 engineering camera
retain their spherical fixtures, including the precision markers. F2 changes
presentation, not the live gameplay definition or session journal.

Runtime passes generated scout sole grids, hull/appendage envelopes and the
deployed ramp tip from character to a ship-owned landing policy. It does not edit
Blender geometry or generated contracts. Landing aligns to the sampled normal;
fixed geometry accepts at most a 0.10 m sole/ramp support gap and a 20 degree
center slope. Samples enforce the existing 2 m non-gear hull clearance, with a
0.5 mm placement bias and a 1 mm validation tolerance for world-coordinate
rounding. Unsafe sites report a typed rejection before an assist begins. Startup
searches a bounded 41×41 lattice at 25 m spacing around the requested direction,
rejecting unsafe footprints; failure aborts startup explicitly. Assists remain
uncancellable and partition-independent. Free-flight movement uses bounded
continuous segment sweeps: elevation extrema are broad bounds, and version-1
heightfield gradient bounds drive chronological contact refinement. The existing
conservative 16 m sphere retains local-normal clearance; see the ship-owned
[collision/crash contract](../client/ship/architecture.md#swept-collision-and-crash-settling-202)
for resolution, conservative work-cap behavior and crash-state response.

Character eye height is 1.75 m along radial gravity up. Walking follows the view
tangent at 3.8 m/s; uphill/contour traversal over 40 degrees and upward steps over
0.30 m are rejected. Downhill escape remains available. Terrain jumping launches
at 4.4 m/s and uses 9.81 m/s² radial gravity, including falling down large steps;
held jump cannot relaunch. Flat legacy fixtures retain their prior walking policy.

Deposits preserve the original generation lattice, IDs, mass and variant choice.
Runtime queries candidates on the nominal anchor, then orients and supports their
actual vertices on terrain with a 5 mm target clearance. A 1,024-entry placement
cache uses body, resource seed, local ID and the full terrain definition; the
mining journal is applied afterward and never lives in that cache. Depleted
tombstones survive cache eviction. Held pieces also clear sampled terrain; new
fragment emission reserves the full growing envelope on a real authored facet
and rejects unsafe ground. Physics receives pure full/height-only queries,
samples rotated convex vertices, uses radial gravity and sampled contact normals,
and checks the same support for sleep. Runtime memoizes at most 4,096 exact
normal queries per native thread, retaining every input bit; eviction changes
cost, never a returned sample. Definitions are immutable for that process; live
definition edits/migration remain unsupported.

A fill residual at or below 1e-12 kg after successful emission is left unextracted
in the source instead of allocating a roundoff-only microscopic body. It is not
credited, discarded or buffered. Initial small requests remain valid. See
[resource output](../client/world/resource-contracts.md#handheld-extraction).

## Patch selection and near/far presentation

For representative fixtures, cube-sphere quadtree leaves cover a local 8,192 m cap, with 16×16 cells per patch.
Selection uses actual spherical cell edge lengths; near patches approach 64 m
width and distant widths grow with distance up to 4,096 m. Selection is cached
at a stable approximately 128–256 m body-local anchor. Stable keys contain the
opaque body identity, entire encoded definition, face, level, u/v and boundary
refinement. Only selection changes regenerate geometry. All edge intervals are
checked when balancing to 2:1; coarse boundaries add exact midpoint samples and
fan triangles that match fine grids, including across cube faces and poles.
This choice preserves stable cache identities and global coverage through seams.
Tangent clipmaps would simplify regular grid updates, but require moving-frame
coverage, pole/face transitions and a separate cache/stitching design.

Patches carry separate f64 body centers/local anchors and f32 local offsets;
renderer subtracts center-camera in f64 before adding the local anchor and casting.
Vertex/index buffers stay immutable. One aligned uniform arena is uploaded per
frame; indexed draws share the sphere material textures and lighting. Reverse-Z,
distant spheres and the Sun remain intact.

The far sphere transitions smoothly from nominal radius at 50 km altitude to
the conservative lower elevation envelope at 20 km, before a cap can activate
below 10 km. Inside 4,096 m of the cap anchor, settled geometry uses the actual
query. Between 4,096 and 6,144 m, geometry/normals blend to that lower envelope;
the unmasked sphere fills the exterior. This distant approximation can differ
from true elevation by the declared elevation span and is not a physical query.
Once GPU buffers are ready, a 250 ms visual blend grows the cap from the same
far envelope, avoiding a cold-load height pop. Automated evidence waits for
presentation_ready. Ordinary ship exit takes longer than this warmup; startup
keeps the player on the supported cabin floor.

## Budgets and tolerances

| Resource | Bound |
| --- | --- |
| Active patches | 512 |
| Active triangles | 393,216 (768 per patch upper bound) |
| CPU patch cache | 1,024 patches; measured requested buffers/metadata below 48 MiB in regressions |
| GPU patch buffers and uniform arena | 32 MiB hard preparation bound |
| GPU uploads, including uniforms | 2 MiB per frame |
| Complete CPU preparation | 2 ms admission target from target/LOD selection through publication; at most 8 generated patches/frame. A final bounded selection unit or patch may overrun; counted explicitly |
| Deposit placement cache | 1,024 immutable placements |
| Exact normal-query cache | 4,096 entries/native thread |
| Observational frame history | 3,600 samples |

These are requested application buffers, not total driver/RSS allocations.
Record process RSS separately; Metal may not expose wgpu allocator totals.
Selection/key construction, generation, full CPU preparation (including cold work) and renderer upload preparation have separate native inspection timings. The 2 ms clock starts before selection; selection, balancing and key construction advance in cooperative slices before admitting generation. Queues are capped at 128 pending tree nodes and 512 leaves/keys; changed body/definition/anchor discards partial work. The final indivisible selection unit or patch can cause a soft-budget overrun, not a hidden pass. `preparation_overruns` and peak include them; the existing far sphere/previous complete cap remains until the new cap is ready. Renderer upload timing measures CPU validation/buffer/queue preparation, not GPU execution.
No full-planet mesh or unbounded generation queue is allocated.

CPU tests measure vertex/shared-border error below 0.2 mm at catalog radii and
the 1e12 m world anchor. Triangle-center radial error is gated at 0.25 m within
150 m of the six plain/crater fixtures; measured maxima are approximately
2.1/15.2 mm (Earth), 0.3/37.9 mm (Moon), and 1.4/29.9 mm (Mars). This is a
fixture measurement, not a universal bound for arbitrary future configurations
or every unsampled triangle location. Deposits target 5 mm vertex clearance,
checked within 0.5 mm. Convex environmental support uses sampled vertices and a
1 mm contact grouping/sleep gap; it is a heightfield approximation, not volumetric
collision or general CCD. Native images/state verify the playable fixtures.
Sharper future definitions require new mesh/contact tolerance validation before
activation. Caves, tunnels, overhangs, networking and disk persistence remain
excluded.

## Commands and evidence

Run the root CPU/build/lint gates from [validation](validation.md), including world
golden/codec/reload/thread tests, character slope/jump tests, ship footprint/
uncancellable-assist tests, physics support/sleep tests and runtime border,
topology, budget, resource-cache and cross-domain regressions.

After native release staging:

    python3 scripts/salimon-test suite --group terrain --binary artifacts/build/release/salimon-client
    python3 scripts/salimon-test suite --group terrain --evidence --binary artifacts/build/release/salimon-client --screenshot-command '["native"]'
    python3 scripts/salimon-test suite --group resource-collection --evidence --binary artifacts/build/release/salimon-client --screenshot-command '["native"]'
    python3 scripts/salimon-test suite --group ship-eva --evidence --binary artifacts/build/release/salimon-client --screenshot-command '["native"]'

The native screenshot option copies the actual completed GPU color surface,
including overlays, before presentation. It requires surface COPY_SRC and
RGBA8/BGRA8, has bounded GPU waits, never reads unrelated desktop content and
fails explicitly on unsupported capture or file errors. Ordinary frames do not
read back. Runtime creates only new absolute .png paths. Desktop captures remain
available through the existing external command option.

On a desktop with a window manager, also run:

    python3 scripts/salimon-test run scenarios/reference/terrain-render-smoke.json --binary artifacts/build/release/salimon-client --screenshot-command '["native"]'

This reference route exercises native resize/minimize/restore, F3 and the F2
flat-precision round trip. It is excluded from the default Xvfb suite because
bare Xvfb has no window manager. The packaged OS-input smoke is separate.

Reference measurements use Apple M1/Metal, release builds and a stated physical
1920×1080 drawable. SALIMON_REFERENCE_SIZE=1 also gives flat E2E controls that
size. Warm up, issue performance_reset, sample an isolated scene, then inspect
performance and terrain counts/bytes. GPU durations are the latest asynchronous
completed pass observations; presented intervals are application submission/
presentation observations. A locked desktop can validate GPU pixels and native
controllers but cannot establish visible display refresh or a playable FPS floor.
Record that distinction, machine/revision, sample counts, CPU/GPU timings,
cold generation cost and requested/RSS memory. Never infer an FPS floor from
averages or software Vulkan.

Completion evidence lives in [#179](../reports/issue-179/README.md),
[#180](../reports/issue-180/README.md) and [#181](../reports/issue-181/README.md).

## Option A shader evidence (#278)

Visual palette/face-normal policy is maintained in the
[art bible](visual-style.md#procedural-body-and-terrain-recipes-278).
`terrain_style_reference` is an opt-in renderer-only synthetic hill/crater and
analytic-body fixture, independent of authoritative world terrain. It compares
frozen frames and 2 cm crossings around both shading fade boundaries. It records
actual offscreen GPU pass timestamps when supported (explicit NaN/no samples
otherwise), patch buffer/uniform costs and PNG captures. It does not simulate
walking, certify physical support or replace native gameplay/M1 scenarios.

```sh
SALIMON_TERRAIN_STYLE_DIR=/absolute/path/to/artifacts/terrain-style cargo test -p salimon-renderer --locked terrain_style_reference -- --ignored --nocapture
python scripts/compare_terrain_style.py --before artifacts/terrain-before --after artifacts/terrain-style --output artifacts/terrain-comparison
```

Use the same renderer test harness and camera fixtures for before/after. The
comparison rejects changed silhouette/background coverage, geometry/draw/memory
costs, or inconsistent scene sets; palette/color changes are expected. No
comparison promotes a pixel baseline. Run existing topology/shared-border and
world ground/ship/character/physics regressions alongside this visual-only probe.

## Nearby query reuse (#307)

Runtime retains one immutable nearby candidate/result snapshot, keyed by exact
position bits, seed and terrain mode (definitions are immutable). Within a tick
and subsequent unchanged-position redraws, targeting and mesh presentation share
it. A world-owned deposit revision invalidates reconciliation only when mass
changes; fragment movement and input release do not regenerate candidates. A new
position/seed/mode regenerates; streaming return reapplies depleted tombstones.
Aim/ship obstruction/equipment gates always use current snapshots, including
zero-duration look updates. No ray-result cache can delay aim loss or release.
`resource_queries` separates candidate/placement/reconciliation costs and counts;
placement misses include cold authored-vertex terrain support. The single result
cache and 1,024-placement limit keep memory bounded. No background queues were
introduced. Run the opt-in CPU traversal probe with:

    cargo test -p salimon-client --locked preparation_profile -- --ignored --nocapture

CPU observations and measured cold/steady costs are recorded in
[the #307 report](../reports/issue-307/README.md); hardware budgets/FPS remain
pending [native #313](https://github.com/mr-exception/salimon/issues/313).

Canonical Vesper adds [planet-wide orbit-to-ground streaming](universe-terrain.md)
with the same renderer budgets and the exact frozen publication evaluator.


## Canonical surface integration (#425)

The [canonical gameplay contract](universe-gameplay.md) maps current body/frame,
resource/physics/Fabricator integration and schema-24 definition binding to owners
and tests. It preserves the frozen terrain and explicit historical fixtures.

Canonical new journeys (#426) use `session/canonical.rs::new_canonical_run`: frozen Vesper ID, published traveler offset [14,0], conservative authored-hull clearance and a view facing the landed ship. Fresh terminal restarts preserve run serial/seed policy and grant existing supplies once; restore never grants them. Flat fixtures remain explicit, old active checkpoints remain resumable until #427. Oxygen depletion stays deferred. Tests and native evidence: `reports/issue-426/README.md`; setup `canonical-journey`.

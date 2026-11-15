# Canonical browser planet surface (#847 / #848)

The browser owns three **illustrative packaged fixtures**, not observed scientific
bodies: `vesper` (6,200,000 m), `vesper-moon` (1,400,000 m) and `caligo`
(9,000,000 m). They retain the shipped sample catalog IDs. There is no archived
Rust import or scientific satellite ephemeris. P02 adds deterministic geological
terrain and a minimal physical ground preview; illustrated streaming art is P03/P04.

## Publication and compatibility

`client/src/planetDefinition.ts` publishes schema/record version 1, spherical
geometry, `surface:v1` coordinates and independent terrain, geology, resources and
visual domain versions, explicit 64-bit hexadecimal seeds, algorithms, parameters
and content digests. Revision 1 remains the original flat sphere. Revision 2
publishes `geological-sphere:v2` consuming `provinces:v2` inputs; resources remain
authored-only and visual parameters remain palette identities. `planetDefinition`
without a reference deliberately resolves legacy v1; `geologicalDefinition`
selects the separately pinned v2 record. No runtime seed selection occurs.

Digests use sorted-key compact JSON, UTF-8 and unsigned FNV-1a-64, displayed as
16 lowercase hex digits. Full-record hashing includes all domain digests; the
terrain parameters also bind the complete authored physical fixture, including
spawn, Scout pose, object/source IDs and boxes. These are content identifiers,
not authentication. `validatePlanetDefinition` compares the entire incoming record
against the frozen pack, beyond its digest. Published golden values are pinned
separately from the definition builder. Unknown bodies, changed radii, unsupported
versions and content/reference mismatches throw. No fallback generator exists.

Changing renderer/cache/viewport/order does not change a body, site, terrain or
original mined source ID. A visual-domain change cannot redefine physical truth.
A changed publication requires an explicitly approved coordinate/entity/depletion
migration; updating hashes alone is not a migration.

## Coordinates and region ownership

`surfaceCoordinates.ts` exposes `surfaceRegion`, `surfaceToLocal`,
`localToSurface` and `neighboringRegions`. `SurfaceLocation` is a body ID plus a
body-fixed Cartesian radial vector in metres. It deliberately has no longitude
wrap, pole singularity, camera origin, ship yaw, wall clock or astronomical origin.
The adapter preserves #842's navigation shell at **ground radius + 5 m**, the
existing conservative Scout bound. Height is relative to the physical ground,
not an extra 5 m of terrain. Radial magnitude must match the shell within 1 mm;
local-to-surface returns the exact shell. Do not pass arbitrary altitude here.

The shipped `surface:v1` cubed-sphere ratio grid uses `ceil(shellRadius / 64)`
steps, dominant-axis ties X/Y/Z, nearest integer cells and stable signed face IDs.
The 64 m value is a ratio-grid scale, not an equal-area tile dimension. There is
one coordinate-based region lookup on either side of a seam. Two face charts may
share a boundary centre, but querying a physical coordinate has one deterministic
owner. Tangent frames come from the stable cell, not the camera or last loaded
neighbor. `neighboringRegions` returns sorted, deduplicated cardinal neighbors;
face-edge half cells are retained, and corners do not jump across a whole cell.
The authored Vesper rectangle is an inset with the enclosing grid cell as its
neighbor, preserving the existing special site ID.

Local coordinates are orthographic projections onto a region's deterministic
orthonormal east/north basis. The inverse reconstructs the front sphere hemisphere
with a positive radial component. Back-side, non-finite, zero-radial, wrong-body,
forged-frame and out-of-chart coordinates throw. Use another region before the
hemisphere horizon; these are local charts, not a promise of whole-sphere walking.
Re-express a physical location through both charts when crossing a boundary;
never reinterpret the same local x/y as another region's coordinates.

Only the final metre-scale displacement is combined with the existing split
`Position` astronomical body origin in `surfaceSite.ts`. The adapter never
flattens the astronomical origin into a large floating-point world coordinate.

## One physical query and mutable overlays

`samplePlanetSurface(location, reference?, probe?, delta?)` validates the
publication and returns definition/body/region/site/terrain identity, height,
slope, solid ground, collision, swept walking fraction, optional landing clearance
and original source IDs at the queried source footprint. Landing clearance is
`null` until `landingYaw` is supplied. Probe motion/yaw are in the canonical
region's basis. Revision 2 also returns `normalBody` (unit body-fixed normal),
`grade`, `groundType`, continuous province `weights` and a geological domain ID.
No query invents collectible sources.

`authoredSurface.ts` owns immutable physical positions and boxes. Artwork in
`planetarySite.ts` reads them instead of choosing collision from sprites.
`surfacePhysics.ts` is the common metre-scale surface query used by the canonical
adapter and existing swept walking in `topDownInteraction.ts`, including ship
fixtures in their local frame. `validateSurfaceClearance` now consumes the
canonical adapter. Both paths use the same physical boxes and collision arithmetic;
landing additionally checks the existing 5 m hull bound, illustrated hull margin
and aft access apron. Authored obstacles are sampled across the inset boundary,
so site ownership cannot erase physical collision.

`SurfaceDelta` binds a definition reference and bounded authored local anchor /
removal deltas. It cannot invent source identities, change physical dimensions,
remove required original sources, alter immutable definitions or apply a Vesper
overlay to another body. Resource quantities/depletion remain exclusively in
`resourceSession.ts`'s stable-ID ledger. The adapter does not copy or mine sources.

## #842 integration and legacy saves

`surfaceSite.ts` delegates its shipped region/frame mapping to the adapter and
uses its sample for touchdown clearance. It retains the original Vesper
`site:illustrated-landing:v1`, outdoor spawn `[-6, 3]`, Scout pose `[2, -1, -.12]`,
`source:landing:mineral:01` and `source:landing:biology:01`. Site-state restoration
still uses the existing validated entity anchors/removals. The approved P02
migration keeps absent `SurfaceAddress.definition` references bound to v1;
pre-v11 saves also explicitly follow the legacy starting binding. New expeditions
start with a compact `{bodyId, version, digest}` v2 reference. Geometry, original
anchors/IDs and depletion do not move, and migration does not rewrite on read.

The first saved binding for a **body** governs all subsequent landing regions in
that save, including previously unseen neighbors. This prevents a v1/v2 border
seam. Old saves keep existing bodies at v1; first visits to other bodies adopt v2.
Conflicting body revisions, unknown hashes and changed complete records fail
closed. There is no automatic upgrade of a previously visited body's geography;
a later such upgrade needs a separately approved physical migration. New v2
sites persist their reference across takeoff, landing, depletion and reload.

A renderer-independent consumer can use the adapter directly:

```ts
const definition = geologicalDefinition('vesper-moon')
const location = { bodyId: definition.bodyId, radialM: [definition.radiusM + 5, 0, 0] as const }
const region = surfaceRegion(location, definition)
const nextLocation = localToSurface(region, [12, -3])
const sample = samplePlanetSurface(nextLocation, definition, { landingYaw: 0 })
if (!sample.solid || !sample.landingClear) throw Error('Unsafe surface')
```

Use `SurfaceAddress` with a validated yaw for the existing landing planner;
`resolveSurface` supplies the current-time split universe frame. This is surface
approval only: collision/crash is not a landing authorization, and no new flight
lifecycle or arrival telemetry is introduced here.

## Verification ownership

`planetSurface.test.ts` covers publication golden values and tampering,
fresh-process/reordered queries, all face seams/corners, poles and antimeridian,
reversible sub-millimetre local vectors, reciprocal neighbors, malformed input,
shared walking/landing collision and original source anchors. Existing
`surfaceSite.test.ts` covers legacy saves, stable depletion, local deltas and
moon/Caligo journeys. The rotating landing test now uses a published sample moon
instead of an unversioned body; unknown bodies are deliberately unsupported.

Run `cd client && npm test` and `npm run build`. Affected browser routes are
`production.spec.ts`, `surface-sites.spec.ts`, `gathering.spec.ts`, and the
`TD-3`, `PLANET-1`, `TRAVERSE-1` cases in `top-down.spec.ts`. Their existing original,
repeat, reference and negative-checkpoint evidence remains applicable; no
reference images are changed. New execution logs/captures belong in ignored
`client/test-results/` and existing CI artifacts, with the criterion matrix in
the delivering PR. CI results and final acceptance await the owner's report in
the active chat. No physical M1/Metal performance claim is made.

## P02 geological field and prototype movement

`planetGeology.ts` evaluates smooth global Cartesian waves in body-fixed metres.
Kilometre-scale provinces drive relief, closed impact bowls/rims, highlands and
fractured ridges. Fixed seeds and body-specific relief produce distinct Vesper,
moon and Caligo fixtures. Only the packaged cold moon enables polar surface ice;
these are illustrative geology parameters, not measured climate claims. Barren
areas have no mandatory organisms. Province weights and height blend continuously;
material labels discretize those weights. Region IDs, chart ownership and visual
LOD never enter the field. Normals use a fixed 0.125 m central difference, not
renderer mesh normals. A flat authored disk of radius 200 m protects every
original footprint and blends smoothly into relief over the next 1,800 m.

Initial prototype movement is overhead walking: a 0.46 m player disk, maximum
32 degree grade, and swept 0.25 m query intervals with a conservative 0.2 m/m²
curvature bound and 0.25 m sweep margin. Existing box collision still owns ship
fixtures and authored objects; slope collision composes with it in `stepTopDown`.
Movement uses the world's chart across region boundaries, never reinterprets its
axes as the next region's. No jumping, platforming, caves or vertical character
simulation is introduced. Generated obstacles are steep rock/ridge footprints,
not independently randomized collectible boulders.

Scout landing requires at most 8 degree grade throughout the conservative
10 × 17 m enclosing hull/access box, sampled at 0.5 m with a half-diagonal
curvature margin, and no more than 0.5 m height variation from its centre. The
existing authored-box clearance remains authoritative too. `surfaceContactRadial`
adds sampled height to the canonical navigation shell for actual Scout placement
and the landing endpoint; the stored coordinate and tangent basis do not change.
The continuous landing Bezier must remain above a conservative 2 m/m terrain
cone (with an analytical tighter bound inside the authored blend), rejecting
obstructed lateral approaches instead of clipping an intermediate ridge. The
landing HUD subtracts the same sampled ground height, using the session’s
body binding even before a landing plan is admitted. Contact departure tolerates
0.1 mm of split-position rounding, matching takeoff preflight. Existing flight funding and timing rules remain intact.

Single probes reject radius >4 m and motion >64 m; the special landing probe
has its fixed bounded grid. `sampleGeologyBatch` accepts at most 2,048 queries.
There is no planetary heightfield allocation. Immutable baseline landing
clearance has a 32-entry cache; mutable authored save deltas are still checked on
load/commands. `GeneratedGround.tsx` displays at most 1,600 2 m cells, cached by
8 m camera shifts. Its material and conservative steep-rock cracks/shading sample the
same field; the rock marking includes footprint/subcell margins and is not an exact
walk boundary. It creates neither collision nor source entities. This is a
bounded physical preview for P03 streaming/P04 illustration, not new approved
art baselines.

`planetGeology.test.ts` covers publication, fresh-process/order/batch stability,
seams, coherent transitions, normal/slope, physical sweep/landing negatives,
height-bound contact, invalid definitions and workload limits.
`surfaceSite.test.ts` covers body-wide legacy binding and new-v2 flight/reload.
`tests/terrain.spec.ts` runs the built production keyboard/collision/render/save
path, captures original/repeat images and compares displayed slope/material with
the domain. It runs in the cockpit project and bounded PR smoke; existing
surface-site/gathering browser journeys protect the authored opening site.

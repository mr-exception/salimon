# Canonical browser planet surface (#847)

The browser owns three **illustrative packaged fixtures**, not observed scientific
bodies: `vesper` (6,200,000 m), `vesper-moon` (1,400,000 m) and `caligo`
(9,000,000 m). They retain the shipped sample catalog IDs. There is no archived
Rust import, procedural geology, satellite ephemeris or new exploration artwork.

## Publication and compatibility

`client/src/planetDefinition.ts` publishes schema/record version 1, spherical
geometry, `surface:v1` coordinates and independent terrain, geology, resources and
visual domain versions, explicit 64-bit hexadecimal seeds, algorithms, parameters
and content digests. Terrain is a flat sphere; geology is unspecified; resources
are authored-only; visual parameters are illustrative palette identities. None
selects random content. No domain currently consumes another domain's generated
output. Future upstream dependencies must be explicit in a new publication.

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
region's basis. The current terrain is flat, solid and undeformed; other positions
and bodies receive no invented resources.

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
still uses the existing validated entity anchors/removals; no new save schema or
write-on-read migration is introduced. Legacy v1–v11 saves remain bound to this
exact `surface:v1` publication and its pinned definitions. They must never be
silently assigned a future default publication. A future save schema should store
`PlanetReference` explicitly; any transition from the legacy binding requires an
approved migration and preservation of the resource ledger.

A renderer-independent consumer can use the adapter directly:

```ts
const definition = planetDefinition('vesper-moon')
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

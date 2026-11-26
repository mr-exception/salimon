# Canonical browser planet surface (#847 / #848)

The browser owns three **illustrative packaged fixtures**, not observed scientific
bodies: `vesper` (6,200,000 m), `vesper-moon` (1,400,000 m) and `caligo`
(9,000,000 m). They retain the shipped sample catalog IDs. There is no archived
Rust import or scientific satellite ephemeris. P02 adds deterministic geological
terrain and a minimal physical ground preview; P03 adds continuous regional walking and a bounded baseline cache; full illustrated terrain remains P04.

## Publication and compatibility

`client/src/planetDefinition.ts` publishes schema/record version 1, spherical
geometry, `surface:v1` coordinates and independent terrain, geology, resources and
visual domain versions, explicit 64-bit hexadecimal seeds, algorithms, parameters
and content digests. Revision 1 remains the original flat sphere. Revision 2
publishes `geological-sphere:v2` consuming `provinces:v2` inputs; its original
resource record remains authored-only, extended by the separately pinned P05
publication below. Visual parameters remain palette identities. `planetDefinition`
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
forged-frame and out-of-chart coordinates throw. The P03 walking runtime changes charts before the hemisphere horizon; individual charts remain local.
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

## Continuous regional walking and cache (#849)

`topDownRuntime.rechartWalking` preserves body-fixed player/camera positions while
changing the working tangent chart at 256 m. It uses canonical `surfaceRegion`
keys on every face, including poles and the antimeridian, without clamping domain
movement to authored map edges. The nearby Scout restores the original landing
chart within 128 m, preserving exact authored collision and doorway transforms.
Physical motion, collision and reach use these charts; camera/viewport changes
cannot select different terrain truth. Screen input and presentation transport their heading locally between charts,
avoiding remote heading-vector singularities. The view aligns gradually before
the Scout becomes visible on return, keeping its actual bow up without a snap.

`topDownFrames.walkingFrames` projects the actual landed Scout and existing
landmarks into the working chart. Back-side landmarks remain distant and drawing
culls distant objects rather than cloning a Scout in each region. The Scout's
original surface address, ship-local cabin and flight lifecycle are unchanged.
When airborne the planet frame remains unavailable. `Anchor.chart` is an optional
canonical, publication-bound chart on walking player/camera anchors; untagged
anchors retain the original landing chart. Save v12 retains this chart
metadata and rejects forged axes, another body or an incompatible publication.
Legacy v1–v11 saves migrate losslessly, preserving anchor meaning; older clients reject v12 rather than silently discarding its chart. Reopen restores the physical
coordinate and working chart, not the camera or a newly selected landing site.

`regionStream.ts` owns a disposable immutable terrain cache, separate from site
anchor journals, stable source IDs, depletion and player saves. A physical-player
neighbourhood of three cardinal rings prefetches nearby region centres and retains
regions behind the player for hysteresis. On each request batch stale work is
replaced synchronously; no outstanding asynchronous generator can publish an old
body or neighbourhood. Lexical keys and last-use sequence break eviction ties.
At most 32 regions × 2,048 samples are retained, with at most 2,048 new samples
per batch. Exact physical query keys preserve the sampled field across chart,
viewport and load-order changes. A changed body/publication clears the baseline
cache, never mutable journals. No planet-wide array is allocated.

`GeneratedGround` requests its existing 1,600-cell physical preview through that
cache when the 8 m render origin changes, then prewarms neighbours within the
remaining budget. Cache generation/hit/eviction/residency counters are exposed
as `data-stream` for measurement. The world background extends across the
viewport, so ordinary boundaries expose no finite authored canvas edge or loading
screen. This retains P02's material/slope preview; it does not introduce P04 art,
P05 collectible generation or P07 wider persistence transactions.

`regionStream.test.ts` covers four-direction out/back routes through multiple
regions and chart changes, actual Scout visibility, face/pole/winding routes,
deterministic cache stress/budget rejection, save replay and forged-chart
rejection. `tests/streaming.spec.ts` exercises the built production route with
real keyboard input in all four directions, original/repeat/negative captures,
chart handoff and reload. `tests/terrain.spec.ts` retains the v2 shared
collision/render route; existing gathering/traversal/flight tests protect source,
door and detached cabin semantics. Full world CI includes streaming in the
existing cockpit project and uploads ignored `client/test-results/` artifacts
with 14-day retention. CI acceptance awaits the user's report in this chat.

## Geology-driven mineral deposits (#851)

`planetDeposits.ts` publishes the supplemental `geological-deposits:v1` resource
rules (`DEPOSIT_RULES`, digest `d703783c21cc35a6`). This extends the authored-only
resource domain of **geological v2** bodies without changing any published terrain,
coordinate or visual record. Legacy flat v1 bodies remain authored-only. Reading
an older save adopts this additive resource publication in memory; the next
successful transaction writes `depositPublication` alongside the existing v12
schema. Unknown publication hashes fail closed; original sources, quantities,
anchors and work remain unchanged. No physical geography is migrated.

Four bounded candidates per canonical ratio-grid cell use the body's resource
seed, publication, cell coordinates and slot. Canonical ownership rejects seam
duplicates. A mixed 64-bit deterministic hash chooses subcell position, presence,
rarity and integer grams. Highlands and fractured provinces raise the probability;
polar ice, unsafe solid-ground footprints and the original 200 m Vesper inset
are excluded. The prototype only generates existing `mineral`, never biomass:
common deposits contain 2,000–12,000 g; rare deposits contain 30,000–60,000 g at
4% of accepted candidates. These are illustrative prototype material rules,
not a chemical/economic simulation. Barren cells are valid. Camera, player identity,
LOD, neighboring query order and cache residency do not enter candidate identity.

`ResourceSession` retains only the current two-ring immutable deposit neighborhood
(at most 17 cells / 68 candidates, rebuilt on region change). Ship-footprint
exclusion is a visit overlay: deposits within the Scout's conservative 12×20 m
clearance rectangle are suppressed, never moved or rekeyed. The same entity anchor,
0.65 m half-box and 2.3 m reach drive collision, selection and the scaled mineral
illustration above the ground raster. A body-fixed east direction rotates both
collision and illustration together when walking charts change, including cube-face
seams. Depleted silhouettes keep their collider
and existing dimmed cue. The active world frame reprojects neighboring source
anchors; sources are absent in flight and on other bodies.

Only starting work registers a procedural source in the existing atomic ledger.
Initial mass is regenerated from the canonical ID on validation/reload, never
trusted from saved metadata. Source IDs must bind a visited body's exact v2
publication. Unworked baselines are disposable and do not accumulate in saves.
Worked sources and fractional work survive eviction/revisit/reload; the same
1 kg/s baseline, survival modifiers, partial weight/slot capacity, interruptions
and 50 kg Scout container apply. No fragments, volume storage or new item types
are created. `sourceDeltas()` exposes exact cumulative debited grams by source
(including authored sources, omitting zero deltas) for P07 aggregate persistence.

`planetDeposits.test.ts` owns fixed barren/common/dense/rare vectors, fresh-process
replay, body/seam validation, bounded queries, conservation, survival/capacity,
interruption, failed writes and eviction/reload. `tests/deposits.spec.ts` exercises
built Chromium input, source originals/repeats, missing-source negative comparison,
depletion and a 240 m out/back eviction-and-reload journey. The gathering, surface-site
and terrain-art suites retain their authored-source and physical/art checks;
old assertions that all generated regions contain zero resources now prohibit
copied authored sources and generated biology instead. New originals remain in
ignored `client/test-results/`, with the validation record in the PR. Hardware
and subjective art qualification remain separate from Chromium CPU/SVG evidence.

## Field exploration and local Scout guidance (#852)

`exploration.ts` interprets existing authored boulders and isolated pronounced
fractured grades in the published v2 physical sampler as geological landmarks.
Canonical cell ownership, body/publication IDs and a versioned rule identify
these places; no collision, resource, biological population or quest reward is
added. Flat legacy regions and unsuitable terrain have no generated landmarks.
The ridge footprint is a 3 m half-extent inspection area on the existing relief,
not a new rectangular obstacle; physical slope collision remains authoritative.
Authored boulder footprints and saved positions/removals come from actual entities.
The bounded neighboring-cell query is shared with deposit residency; it never
allocates a planet-wide landmark list. Rendering fidelity does not enter identity.

Help exposes **Inspect / mark** and **Return to Scout**; M opens the notebook
and G toggles guidance from world focus. The **Inspect / mark** notebook lists only features within 12 m of the surface
player. Loading or seeing a region never adds a known place. Explicit inspection
records a stable physical-source ID; **Mark current location** records the player's
body-fixed coordinate. Optional 48-character names and 160-character notes are
plain text. Records distinguish known physical sources, personal position estimates
and user notes. Re-inspecting the same landmark updates its record; the notebook
admits at most 256 records and reports capacity failure without evicting old notes.
Diamond-and-cross cues project only deliberately recorded local marks, retain
contrast against terrain and cannot intercept world input. The native modal uses
the existing focus/keyboard/Escape stack; closing returns focus to Help. The closed
HUD leaves walking, gathering, doors and cockpit interaction with their owners.

`ResourceSession.fieldMarks` reads the schema-v13 expedition journal. Explicit
marks save immediately with source balances, inventory, survival, player frame
and the shared Scout state. Reload and cache eviction retain notes; failed or
stale writes publish no new mark. No second storage key or writer is introduced.
See [the expedition save contract](save-contract.md#browser-expedition-journal-853).

**Return to Scout** enables an on-demand local walking estimate; it never moves
the player. A nearby hull (within 24 m) reacquires a fix. The estimate's uncertainty
starts at 5 m, grows with accumulated walking distance and expires after 384 m of
travel or beyond that range. The screen-relative eight-sector direction uses the
actual landed Scout transform, including transported walking charts; range is
rounded to 10 m. A reload away from the hull has no position fix. Returning within
24 m can reacquire it; simply loading a distant region cannot. Detached flight,
cabin-only position, changed landing identity and exhausted/absent fixes give an
explicit unavailable message. This prototype dead-reckoning policy is not a
scanner, GPS, chart, route planner or claim of hardware capability. The durable
journal preserves observations and coordinates, while the walking
fix intentionally remains session-local: exact world coordinates alone do not
grant a reliable local dead-reckoning fix after reload.

`exploration.test.ts` covers identity, terrain agreement, command admission,
capacity, chart handoff and fix loss. `tests/exploration.spec.ts` owns built real
keyboard discovery, focus, non-color marker originals/repeats/negative comparisons
and a bounded multi-region out/back route. Evidence stays in ignored test output
and existing CI uploads; no approved visual references are silently replaced.

## Deterministic planetary environment (#916)

`planetEnvironment.ts` publishes supplemental environmental v1, digest
`f9512d32ccbcfd7e`, without rewriting terrain, geology, resources or visual hashes.
It supports the existing canonical Vesper, Vesper Moon and Caligo bodies. These
are fictional packaged values, not atmosphere inferred from a body's name or
scientific observations. The canonical packaged world currently has frozen
per-body seeds, not a player-selected world-seed input.

The exact v1 key is sorted-key canonical JSON (UTF-8 FNV-1a-64 via
`definitionDigest`) of `{publication, seed, bodyId, field, cell?}`. `seed` is the
body's **original v1 terrain seed**, retained as immutable world identity even
when a saved body uses geological v2. `field` is `temperature`, `radiation` or
`toxicity`. Temperature includes `surfaceRegion(...).cell` with its axis, sign,
i, j and steps; rates omit the cell. Interpret the 64-bit digest as an unsigned
integer: temperature is `hash % 361 − 180` °C; each rate is `hash % 21` HP/s.
There is no mutable RNG, time, viewport, cache or landing-position input.
Queries validate canonical body references and finite body-fixed shell positions,
then perform bounded work independent of world/cache size.

Vesper's safe start overrides the regional temperature and both body baselines:
positive-Y hemisphere, inclusive `abs(radialM.x) <= 128` and
`abs(radialM.z) <= 128`. These are east/north metres in the fixed +Y starting
tangent chart. It remains 24°C and zero rates even after changing the walking
chart or landing elsewhere. Exactly at an edge/corner is safe; any amount beyond
an edge uses generated data. The rest of Vesper retains its body-wide baselines.
The separately exported safe Scout sample has the same values; #917 owns the
actual interior/exterior adapter and damage, so publishing this query adds no
exposure yet.

`samplePlanetEnvironment` returns a local sample with publication/body/region
identity. `bodyEnvironmentSummary` returns the supported temperature range
−180..180°C, body baselines and whether a safe starting override exists. The
range is a conservative publication range, not a sampled local reading or a
computed claim about exact extrema. #921 owns scanner authorization and the
separate labeling of range versus actual supported landing-site sample. The raw
world query itself neither scans nor reveals anything in the current HUD.

| Body | +X cell temperature °C | Radiation HP/s | Toxicity HP/s |
| --- | ---: | ---: | ---: |
| Vesper | 85 | 12 | 16 |
| Vesper Moon | 62 | 11 | 13 |
| Caligo | 72 | 18 | 6 |

These golden vectors bind v1; algorithm, seed-policy, bounds or safe-boundary
changes require a new environmental publication and an explicit migration.
Existing saves lacking `environmentPublication` bind explicitly to this v1,
lazily persisted on the next successful atomic commit. Unknown values reject
with original bytes preserved; catalog revisions continue using each saved
canonical body reference. No region caches or regenerated baseline values are
saved. `ResourceSession.environmentAt` and `environmentSummary` use that binding.
No weather/day-night/pressure/oxygen, damage or scanner presentation ships here.

`planetEnvironment.test.ts` covers published vectors in separate Node processes,
bounds/regional variation, exact safe edges, streaming eviction/return, catalog
references, reload/migration and failed writes. Existing surface/stream/site
suites remain owners of physical coordinate and saved terrain continuity.

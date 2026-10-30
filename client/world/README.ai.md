# World AI Maintenance Guide

## Read before changing

1. Fetch the active GitHub issue, project specification, and technical
   architecture as required by the root `AGENTS.md`.
2. Read [solar-system-layout.md](solar-system-layout.md),
   [coordinate-strategy.md](coordinate-strategy.md),
   [architecture.md](architecture.md), and [invariants.md](invariants.md).
3. Read the runtime and renderer maintenance guides before changing their typed
   integration boundary.

## Ownership

Keep the immutable compressed body catalog, absolute coordinates, body/landing
geometry, reference speed, and portable camera behavior here. Use `f64` meters
until after origin subtraction, and expose renderer-neutral snapshots. Keep the
three engineering precision markers separate from the six canonical bodies. Do not add
window events, GPU types, renderer projections, disk/backend persistence,
networking, textured sphere rendering, or lighting. Portable resource contracts
belong in `resources`; read [resource-contracts.md](resource-contracts.md) before
extending them. Portable generation, extraction, fragment output, and one-object carrying live
in their own modules. Runtime composes character/ship placement and fragment
motion/contact; this crate owns validated physical identities, mass, poses, and
mining session retention across local streaming. Read the mining contract and focused tests
before changing aim, rate, session mutation, or obstruction semantics.
`resource_fragments` owns conservative output and surface poses; do not move
fragment creation or mass splitting into runtime or renderer. Querying output
must remain read-only. Interactive mining uses `spherical_mining` plus the session adapter in
`sphere_session.rs`; attached entries cannot be picked up or simulated loose.
Legacy helper output tails are sealed on pickup. Read the canonical lifecycle
before changing caps, release precedence or session conservation.

## Source navigation

Use the [architecture source map](architecture.md#source-map) for coordinates,
catalog, geometry, proximity, precision and engineering-camera code/tests.
`src/lib.rs` preserves the root public API through reexports; add rules to their
owning module rather than expanding the facade. Resource module paths are unchanged.

## Change checklist

- Preserve the exact ordered compressed-fixture membership: Sun, Mercury, Venus, Earth, Moon, Mars.
- Keep the Sun visual-only and all five `1.15R` solid landing volumes disjoint.
- Preserve the 24-second Earth-to-Mars surface trip at the catalog reference
  maximum speed unless a source-of-truth decision changes it.
- Preserve the subtract-before-cast precision contract.
- Keep tour commands and updates deterministic and independently testable.
- Keep all catalog values finite, positive where required, and immutable.
- Preserve stable resource keys and private validated mass/pose state. Run the
  resource contract tests when changing catalog entries or domain structures.
- Update the layout table, measured float spacings, tests, and runtime mapping
  together when catalog geometry changes.
- Run the workspace build, format, lint, and test gates plus the native visual
  scene/precision/depth smoke test after integration.

## Shared maintenance rules

Follow the root [coding conventions](../../docs/coding-conventions.md),
[validation matrix](../../docs/validation.md) and
[feature map](../../docs/maintenance-map.md). Update affected contracts/guides
with behavior changes and record completion evidence under root `reports/`.

## Terrain

Read [terrain-contract.md](terrain-contract.md) and the
[integration guide](../../docs/terrain-validation.md) before editing definitions
or consumers. All enabled consumers must agree on the same immutable definition;
run golden/codec/compatibility tests and affected cross-domain/native regressions.

## Terrain integration (#179–#181)

Follow the canonical [integration/budgets/validation guide](../../docs/terrain-validation.md)
and [world definition contract](../world/terrain-contract.md). Keep local rules
and their consumers aligned; flat deterministic fixtures are explicit.

## Physical mounts (#576)

The shared carry slot also accepts externally owned physical fixture IDs; fixture source/motion/attachments remain with ship/runtime owners. See [the canonical contract](../../docs/physical-mounts.md).


Instant Core upgrades #548: [canonical input, source, mass and schema-14 contract](../../docs/core-upgrades.md). Ship owns validated capacity-only transactions; world returns original caller-owned reserved entities through the one carry slot; runtime maps the data catalog/physical controls and reconstructs pending+installed material mass. No timer, joule grant, Core replacement or flight command.


## Canonical terrain streaming (#424)

See the [canonical terrain contract](../../docs/universe-terrain.md) for the exact U11
evaluator, separate frozen identity/codec, planet-wide quadtree, stitching and
CPU/GPU bounds. Compressed six-body fixtures remain unchanged. Gameplay adoption
and normal alien-world startup belong to #425/#426.


## Canonical surface gameplay and schema 24 (#425)

Follow the [canonical surface contract](../../docs/universe-gameplay.md):
selected body/terrain context, immutable U11 deposits, shared physical ground,
conserved fragment/cargo/Fabricator ownership and exact U05 definition binding.
Historical fixtures remain explicit; new-run startup is #426.


## Canonical geology (#612)

World `terrain::geology` owns versioned terrain-bound province/history/substrate
queries. The returned physical sample is the exact existing terrain; geology
never displaces ground or reshuffles resource identities. See the
[geology contract](../../docs/planetary-geology.md) for codec, profiles, compatibility and tests.
Vesper remains a homogeneous silicate interpretation with unchanged published
terrain/resources; rendering and later environmental systems retain their owners.


## Regional climate and weather (#615)

Follow the [climate contract](../../docs/planetary-climate.md) for immutable geology-bound climate inputs,
strict versioned publication and stateless TT epoch queries. World owns pressure,
temperature, coherent storm/cloud/precipitation metadata and body-fixed tangent wind;
presentation cannot rewrite them. Vacuum has no airborne effects. Existing terrain,
resource identities, saves and drag physics remain unchanged. Weather visuals are #616.


## Active geological sources (#617)

The [active geology contract](../../docs/active-geology.md) owns explicit heat/mechanism inputs,
stable terrain-bound sites, strict publication restore and shared-TT source levels.
Spatial queries cannot create/rekey sources. Vacuum permits ballistic plumes and
contact coupling, never atmospheric wind/sound. Source radii do not add colliders.


## Rare biospheres and ecological regions (#618)

Follow the [ecology contract](../../docs/planetary-ecology.md) for independent fixed life decisions,
geology/climate-bound potential, continuous realms and stable patch/grammar IDs.
Suitability never grants life; observed bodies retain no procedural life claim
and Vesper stays sterile. Strict catalog/save bindings reject incompatible
publications. Organism meshes, physical instances and harvesting remain #619–#621.


## Optical atmosphere (#614)

The [atmosphere contract](../../docs/planetary-atmosphere.md) maps immutable climate
optics, runtime light inputs and depth-limited production scattering. Vacuum
remains empty; presentation never changes gas physics, weather or canonical identity.


## Coherent weather presentation (#616)

See [weather visuals](../../docs/planetary-weather-visuals.md): shared-TT source
fields, fixed body-frame maps, bounded optical rendering and exact source/quality
separation. Vacuum stays empty; no terrain, hydrology, weather truth or sound is invented.


## Organism family grammar and presentation (#619)

Follow the [organism contract](../../docs/planetary-organisms.md) for stable
family/part identities, cosmetic tile sampling, neutral segment mapping, bounded
instanced geometry and source-driven GPU evidence. Physical population remains #620.


## Physical organisms and static clearance (#620)

The [physical organism contract](../../docs/physical-organisms.md) maps canonical
placement/part/baseline identity, physics-owned swept boxes, character/ship typed
adapters, runtime residency and matching renderer evidence. Quality never changes
physical populations. Explicit obstruction safety overrides ordinary assist continuation.

## Geological site identity (#623)

[Landmark source and validation](../../docs/planetary-landmarks.md) binds sparse
sites to actual published terrain relief. Keep physical source, renderer vertices,
immutable site IDs and save/reference binding consistent across LOD and eviction.
No chart/scanner discovery, duplicate terrain surface or invented cave geometry.

## Natural signature sites (#624)

[Biological signature contract](../../docs/biological-signature-sites.md) maps rare
site metadata onto exact physical organisms and geological/activity provenance.
No source geometry, collision/LOD anatomy, identity or player knowledge changes;
strict publication binding survives reconstruction and rejects mismatched sources.


## Biological harvesting and schema 29 (#621)

Follow the [yield, atomic transfer, geometry and persistence contract](../../docs/biological-harvesting.md).
World owns explicit source designations, revision/depletion journals and physical
publication; runtime composes F/tool input, current anatomy and source-bound restore.
Schema 29 preserves missing/partial parts and natural object identity. Vesper
remains sterile; positive sources are explicit fictional geometry fixtures.

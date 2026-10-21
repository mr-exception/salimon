# Canonical planetary generation and multi-seed world identity

Status: **approved architectural direction (2026-10-21), documentation-only; future universe/catalog and multiplayer implementations are not shipped by this document**. This is the authoritative contract for permanent base-world identity; the current compressed six-body prototype and its version-1 terrain/resource generators retain their existing behaviors until deliberately migrated. See [approved universe expansion](universe-expansion-approved-design.md), [world and realism](world-and-realism.md), [terrain contract](../client/world/terrain-contract.md), [resource contracts](../client/world/resource-contracts.md), and [multiplayer](multiplayer-and-npc-interaction.md).

## Player-visible invariant

A planet or moon has **one shared canonical base definition**, independent of player, client process, session, device, load order, and whether its data is read from the installed offline universe package or later from the authoritative backend. Two players visiting the same location must encounter the same underlying mountains, canyons, craters, natural resource identities and designated fixed discovery sites. Terrain patches, render LOD, streaming order, and local graphics settings may vary, but must not alter physical world identities, collision geometry or what is canonically present.

No generation step may obtain a fresh player-specific, time-of-launch, process-random, unordered-iteration, or system-clock seed for **immutable base-world content**. "No runtime randomness" here means **no nondeterministic or uncommitted world-identity decisions at runtime**; the engine may evaluate deterministic noise, hashes, and fixed-seed pseudorandom samplers on demand from a published canonical definition. This preserves current resource sampling (FNV-1a / SplitMix64) and terrain evaluation (hash-based lattice noise). Dynamic, time-varying simulations are separate from immutable geography and must use their own synchronized/authoritative state.

## Canonical planet record and independent seed domains

Each future planet has:
- **Stable canonical body ID**, disjoint from display name, catalog ordering, runtime indices and existing compressed-prototype enum discriminants.
- **Immutable physical and astronomical inputs** as relevant (radius, identity, provenance, observational validity) and a versioned planet record/schema.
- **Published multi-seed map**, with independent named domains and their evaluation/algorithm versions. The seed representation and derivation must be a reproducible, platform-independent binary/packed contract; it need not be a free-form seed string. A reproducible root seed **may** be used by the offline pack builder to derive distinct domain seeds, but the resulting per-domain seeds and versions are part of the canonical record. Use explicit domain separation, fixed byte ordering and a specified stable hash/PRF; never rely on default language hashers or a mutable RNG sequence.
- **Full canonical-definition content digest** and preferably subsystem-specific digests for exact validation/cache invalidation; a seed alone is not a sufficient cache or compatibility key.
- **Authored overrides and provenance** where a scientific source or designed location takes precedence over procedural proposals. Versions and overrides are part of the definition/digest.

Proposed domain breakdown (implement only when the respective system is designed):

| Domain | Owns | Separation rule |
| --- | --- | --- |
| Terrain | Base heightfield and stable geological surface geometry | Changes never silently perturb resource or discovery identity |
| Geology | Higher-level geological landform distribution / physical composition when introduced | Not an independent render-only displacement; physical effects must use the canonical sampled surface |
| Resources | Stable deposit lattice, resource selection and original deposit IDs | Preserve existing per-material stable-key derivation or explicitly version/migrate it; biome/resource parameters also belong in its definition |
| Visual | Planet color/texture/material recipes and safe purely presentational variation | Cannot determine collision, resource or gameplay identity; renderer LOD remains local |
| Biomes / ecology | Future environmental regions and baseline biological distribution | Only if implemented; biome keys cannot depend on player position or session randomness |
| Discoveries | Future canonical natural/artificial site placement and stable site IDs | Authored site overrides and rare-event placement are shared, not per-run random rewards |

A domain's seed **does not by itself** define its output. Bind its algorithm version, parameters, authored feature set, scientific-input provenance where applicable, and stable identities into its versioned definition. Adding a new domain must not reshuffle outputs in older domains. Do not assume new geology, erosion, vegetation, dynamic weather or discoveries are implemented today.

For the existing version-1 terrain implementation, the validated `TerrainDefinition` already includes `u64` seed, generator version, radius, ordered layers and sparse authored features; retain that exact code/codec contract. Existing resources derive stable per-material inputs via FNV-1a and fixed SplitMix64 lanes. Do not silently rekey currently saved deposits/fixtures or treat the current six-body tag codec as the future galactic body-ID format.

## Offline package and future backend: one data contract

1. A reproducible **offline catalog/pack builder** selects/derives fixed parameters and seeds for fictional populations and writes the published canonical record and digest. Generation for a new catalog version happens during content production, **not independently for each player at runtime**. For measured Solar System bodies, favor real scientific/geographic data where practical, with source provenance and versions, rather than synthesizing different fictional terrain.
2. The client reads canonical records from the **installed versioned universe pack**, and only evaluates their expensive local procedural geometry/resources/visual recipes on demand. This is not a requirement to store full meshes or entire heightfields in the package.
3. In future multiplayer, a backend database/catalog serves **the same canonical definitions** (or authenticated digest + version/ID referencing an exactly matching local record). It owns definition publication/compatibility and gameplay authority; it does not independently invent new terrain when a player arrives. Never silently accept mismatched client/server definitions.
4. Catalog/version IDs and full definition digests flow into caches and saved state. On missing/unsupported/mismatched definitions, fail clearly, request a compatible published catalog, or run an **explicitly approved migration**; never quietly reinterpret a visited world.
5. A canonically published planet is stable within its published definition revision. Whether revisions may ever change an **already published/visited** baseline is **unresolved owner policy** (see decision register). In all cases, protect physical positions, authored sites and persisted player deltas; upgrades may not invisibly overwrite them.

The source of a definition (package file or database) is an adapter concern, not two separate generators. The future server may validate sampled surfaces/deposits independently using portable Rust world code. Cross-platform golden vectors and collision tolerances, not identical render screenshots, establish compatibility.

## Immutable baseline versus mutable shared world

- **Baseline**: planet identity, canonical terrain and physics surfaces, original deposit IDs/distribution, authored fixed landmarks/discovery sites and baseline environmental descriptors. Query/render it on demand without allocating an entire planet.
- **Mutable state**: consumed/mined resources, moved cargo, wrecks, changed structures, player and NPC actions, and any future terrain deformation. Persist changes as stable-ID deltas and versioned spatial events, authoritative on the backend when multiplayer exists. The existing mining depletion journal is an early local precedent.
- **Time-dependent simulation**: orbit/rotation, weather (if implemented), transient signals, renewable-resource schedules, mobile actors and similar events are not frozen into a planet seed. Their behavior must be consistent with shared simulated time and backend authority; fixed baseline does not mean a frozen scene.
- **Presentation**: mesh density, LOD, cache residency, local shading approximations and quality settings are not part of shared gameplay truth.

Do not infer that planetary bases are approved from the examples above: current [world rules](world-and-realism.md) prohibit permanent player-built planetary bases, with permanent construction limited to spaceships. This architecture supports changes only where separately authorized.

A player's **known navigation/discovery chart** remains separate from the full canonical catalog. Access to a planet record cannot automatically reveal Earth's unknown route, hidden coordinates, unverified signals or narrative evidence in gameplay UI; offline package secrecy against deliberate data inspection is a separate unresolved concern.

## Required future compatibility and validation

- Pack-builder golden hashes across repeated builds, reordered records and machines; canonical serialization, domain separation, collision checks, version rejection and input/provenance validation.
- Compare same planet + same coordinates across Linux/macOS/Windows for sampled heights, normals, physical contact and stable resource/site identities, within explicit numeric tolerances. Current version-1 terrain allows defined floating-point tolerances, not unconditional bitwise equality.
- Verify that changing only visual seed/palette cannot change terrain, collision or deposits; changing or adding a biome domain does not reshuffle independent resource IDs; the same published definition behaves identically after eviction, save/load, offline reopen and multiplayer reconnect.
- Test deliberately mismatched client/server catalog/algorithm versions; reject silent divergence. Test immutable baseline + depletion deltas against two simulated clients.
- Treat existing compressed six-body catalog, version-1 body tag and local resource sessions as historical/active prototype contracts, not claims that arbitrary galactic body identities or shared backend are implemented.
- Owners: #402 catalog schema and identity, #406 universe-aware saves, #410 inference, #413 pack builder, #414 indexed reader, #424 terrain LOD, #425 resource/landing integration; future backend and ecological/discovery design need separate scoping. Review issue descriptions against this contract at implementation time without claiming they are already complete.

## Conflict and owner-decision register (not silently resolved)

| Topic | Existing document tension / gap | Proposed interpretation or decision needed |
| --- | --- | --- |
| Backend-generated fictional planets | `world-and-realism.md` historically says fictional planets are generated authoritatively **on the backend**, while the later universe plan commits to a permanent offline package. | **Reconciled**: canonical generation/publication occurs in a reproducible pack-build process now; later backend distributes/validates **the same record** and shared deltas, rather than spawning an alternative planet. |
| "No random at runtime" vs current deterministic sampling | Current resource and terrain implementations use fixed-seed noise, FNV-1a and SplitMix64 during queries. | **Reconciled**: deterministic evaluation on demand is allowed; player/session/time-driven randomness affecting base-world identity is not. |
| Existing resource seeds, visual seeds, terrain seeds | These already have independent derivation and cache contracts; swapping to a new root/seed algorithm could move deposits. | **Reconciled**: preserve version-1 semantics, introduce named domains in a future version, and never silently reinterpret saved IDs. |
| Real observed geography vs generated worlds | `world-and-realism.md` asks for recognizable Earth/Mars terrain where practical. | **Reconciled**: real terrain/geographic sources take precedence for measured worlds; the multi-seed model is especially for fiction and compatible procedural detail. |
| Future world/catalog upgrades | Immutable canonical fiction and explicit migrations coexist with later scientific-data corrections and better generators. | **Owner decision 1**: freeze already published planet baselines forever (recommended for shared worlds), or permit explicit, carefully migrated revisions? No automatic replacements meanwhile. |
| Scientific terrain updates | Improved Earth/Mars data could shift collision sites and player state. | **Owner decision 2**: keep existing published versions and layer only additive detail, or allow approved remapping with versioned migration? |
| Discovery secrecy in offline pack | Fixed shared site placement is compatible with deterministic generation but a user can inspect packaged data even before discovering sites in-game. | **Owner decision 3**: accept datamining risk for offline canonical locations, keep only select endgame secrets server-only, or adopt another explicit reveal scheme? Never leak the Hall through normal pre-completion UI. |

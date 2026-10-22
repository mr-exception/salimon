# Real-scale universe expansion — approved design (2026-10-20)

Status: **approved design; portable U01–U11 foundations delivered with the limits below; remaining expansion not implemented**. Source: ten rounds of owner Q&A. This contract supersedes compressed-playable-world assumptions *for the future expansion*, not existing implementation until migrated. Work begins after the existing planned gameplay implementation queue completes. Keep separate native verification, no implementation PRs as part of planning.

## Narrative alignment (2026-10-21)

The approved alien homeworld becomes the traveler's **awakening location**; they retain ship operation knowledge and memory of Earth as home but not the journey. The canonical scientific catalog still contains astronomical identities; the **player-known navigation chart must not reveal Earth/Solar System coordinates or route** until discovered. Intact departure/Mars/system-exit ship messages and corrupted distress/warning leads belong to the physical Archive and future energy-intensive Recovery Module; the alleged wormhole's actual use is unresolved. Absenat and its Hall are endgame/backend scope, not part of this expansion's claimed implementation. See [narrative](narrative-and-lost-journey.md), [Archive and chart](archive-recovery-and-navigation.md), [Absenat](absenat-and-hall.md). Preserve the scientific and scanner rules below. The **later approved** [Spatial Jump Drive, shared-clock and impulse design](jump-drive-and-interstellar-travel.md) **supersedes the earlier blanket no-FTL statement and velocity-dependent-relativity expectation** for future gameplay without authorizing engine/Jump Drive implementation in this documentation PR. This addition does not authorize new implementation outside the planned issue queue.

## World / catalog
- 1:1 real size, mass, spatial distances and advancing orbital positions. Solar System positions via high-quality JPL ephemerides within published validity; documented lower-accuracy models outside. Time starts at real system calendar date for each new run; gameplay advances 1:1 in real time, no time acceleration.
- Only Milky Way playable/data-packaged now; schema should permit future galaxies. Prioritize real usable Gaia observations; merge NASA/JPL celestial and exoplanet information with provenance, quality flags, uncertainty. Observations do not imply all objects have known full 3D states. Fixed, immutable shared canonical fiction completes plausible missing populations using scientifically informed density/age/metallicity. Stable identifiers and explicit migrations across release versions; source corrections must not silently overwrite visited player changes.
- Initially one versioned offline binary pack, partitioned/indexed, demand-loaded; install budget aim 2–5 GB, initial target <=10 GB unless measurements prove unacceptable and owner approves larger pack. Benchmark actual record size, compression, throughput, quality and storage; no unsupported assumptions. Individually store observed/selected stars, planets, major moons and cataloged/important minor bodies. Represent smaller populations via compact aggregate/statistical descriptors; deterministic detailed terrain at streaming time from stable inputs. A standalone Rust build pipeline imports, cross-matches, validates and packages canonical data with checksums/manifests, licenses/attribution and reproducible golden hashes.
- Start in ONE generated, permanently selected, scientifically plausible fictional star system >=30 light years from Earth; player stands outside landed ship on visually distinctive alien planet. Scientific-style IDs with separate display names. Surfaces landable if physical conditions permit; unlimited suit oxygen *temporarily* (TODO for future equipment/survival). Preserve mining, fabricator, Core, survival and carrying systems.
- Old compressed playable Solar System replaced; keep engineering fixtures. Old prototype saves do not need migration; new saves must track catalog versions and immutable baseline vs player deltas.
- **Canonical multi-seed world identity (approved 2026-10-21):** each published fictional planet has stable ID, versioned full definition, independent named seeds for terrain, resources, visuals and future geology/biomes/discoveries, plus hashes and authored/scientific provenance. Fixed seeds and versions are selected by the reproducible pack builder; client runtime only evaluates deterministic recipes on demand. Future backend returns/verifies **the same definitions** rather than creating player-specific planets. Shared player changes are separate authoritative deltas; output is stable across players, clients and render LOD. Preserve existing version-1 terrain/resource derivations until an explicit versioned migration. See [canonical generation contract and approved owner policies](canonical-planet-generation.md). Published fictional worlds may change **only via explicit validated all-player-consistent versioned migrations**, including safe remapping of persisted player/object/resource state and a single active multiplayer revision. **All observed solid bodies** preserve physical geography and collision after their **first authoritative real-scale surface publication** (not the compressed six-body prototype); better scientific data may add non-displacing details only, absent a new owner approval. Versioned geology inputs feed a **single authoritative terrain evaluator**, and resource distributions may intentionally depend on versioned geology/biome inputs with migration of affected deposit identities. Offline-package site datamining is accepted; gameplay discovery and Hall pre-completion UI secrecy remain enforced.

## Gravity / physics / space
- Ship gravity only this phase; existing EVA/fragment gravity remains, but reusable celestial gravity service permits later migration. Four dynamically strongest gravitational contributors initially (configurable); continuous combined accelerations, stable selection without discontinuities and reference-integrator tests; quantify acceptable orbital numerical drift. Ephemeris-driven celestial motion, no gravitational backreaction from ship. Ship's physically integrated motion permits orbits, assists, escape, decay, landings, crashes, atmospheric density/drag; detailed heating/tidal damage deferred.
- Startup ship inherits local planet rotation and orbital state in an inertial reference frame. Landing support forces and physically funded assist maneuvers. Recalibrate real thrust and Core where necessary; no artificial gravity-defying assists. Continuous swept collision broad phase and local terrain contact. Hierarchical coordinate frames + camera-relative GPU f32 after f64 local subtraction; galaxy-scale distances never raw global f64-meters-only.
- The existing Newtonian flight prototype remains its current implementation until changed; future normal thrusters use **exact Core-joule-to-impulse accounting** (baseline 1 N·s/J; implementation issue #493), not kinetic-energy-balance billing. This gameplay convention is a fictional exception to physical energy accounting. Continuous sub-light travel and offline coasting remain important; future ordinary engine upgrades can improve acceleration. Whether continuous engines deliberately exceed light speed remains **undecided** and a numerical superluminal bug is not approval.
- **One global universe clock:** 1 second of actual real elapsed time = 1 second of universe time for all ships and players, regardless of speed/online status. **No relativistic time dilation or per-player elapsed-time differences.** The separately approved reusable **Spatial Jump Drive** is an intentional FTL displacement exception: Core joules pay an N=20% virtual speed-gap/impulse cost **per light-second of distance**, it moves the ship forward without changing its real velocity, and it uses a quantum-themed loading transition. Its physical operation and formula are in [the canonical future design](jump-drive-and-interstellar-travel.md). No ordinary warp acceleration or time acceleration is implied. Black holes/compact bodies use approximate models with explicit validity limits, not implied fully relativistic accuracy.
- Navigation instruments include trajectory prediction, distances, closing speed, power/acceleration estimates; no autopilot yet. The future owner-authorized Jump Drive panel has separate light-distance years/months/days/hours/minutes/seconds inputs, shows Core cost before activation, and **rejects a jump while engines are on** (even at zero selected throttle); the engine-off switch is now implemented by #493; the Jump Drive panel remains future work.

## Rendering / scanner
- Hybrid beautiful authored Milky Way skybox plus physical/catalog-derived local stars, physically informed atmospheric/astronomical effects, real projected sizes during flight; enhanced markers only in navigation UI. Seamless terrain tile LOD from space to surface; bounded cache, prefetch and stall budgets. Performance target 60 FPS on reference Apple M1 iMac, configurable render LOD/streaming budgets without changing canonical content.
- ONE powered omnidirectional scanner module, auto-emitting a pulse every **5 seconds**. **No separate active/passive modes or manual scan mode**. Detect celestial objects at 100% upgradeable range, including small bodies regardless of size; future ships/stations at 40% range, future distress/broadcasting. No occlusion. Game-world catalog is not player-known; scan and identify contacts in range. Exactly identified contacts are visible on physical 3D cockpit hologram. Signal propagation details must be documented as an intentional gameplay abstraction for astronomy-scale detection, while avoiding claims of physical five-second radar roundtrips over interstellar distances. Continuous Core power expense + pulse costs per approved power model.
- Contact is refreshed on successful pulse. Remove from live radar after 3 hours without successful returns; rediscovery required, encounter history retained. Scanner runs along *offline* propagated ship path; avoid replaying each five-second frame individually by analytically evaluating detection/contact interval events. Other entities/signals are later-phase extension points.

## Offline progression / persistence
- Closing the game **immediately stops engines**; retain inertial velocity, orientation and physical gravity. Offline coasting continues without an automatic Jump Drive activation; entering a quantum jump-loading environment is a player-triggered, atomic position transfer, not a simulation time skip. On reopen, use system clock only (online authoritative time future), propagate elapsed real time including gravitational dynamics, orbits, drag, physical collision/impact/death, fatigue-only recovery, active timed fabrication/Core conversion and scanner contacts. Freeze loose cargo relative to ship; hunger/health/status effects and all other survival effects remain frozen while closed per existing rule.
- Main menu opens while propagation calculates ship and current region. A detailed loading bar shows stage/real measured completion percentage where known; Resume is prohibited until ship/current-region calculated and verified. No correctness-compromising approximate teleport. Distant region caches and player-owned state may refresh on low-priority background work, but immutable celestial ephemerides are on-demand, not universally ticked. Extreme elapsed times remain valid subject to documented ephemeris fallback/accuracy and progress. Save on success by atomic replacement of the active checkpoint: preserve recoverability on process crash; no extra public save slot necessary. Existing pause contract freezes *running* gameplay; distinguish from app-closed wall-clock progression.

## Engineering / verification / rollout
- Native Rust client and portability only; do not implement server now. Keep small crates and existing domain boundaries, avoid renderer authority and unnecessary frameworks.
- Use independent dependency-blocked implementation issues and separate native Apple M1/Metal hardware signoff. Linux portable/headless/CI evidence never silently substitutes for native M1 tests. Benchmark catalog, memory, streaming and 60 FPS under representative conditions.
- Existing gameplay issues (survival, save/pause, visuals) are prerequisites for beginning the expansion. Reconcile #375, #381, #382, #373, #377, #379 and native #312–#314, #348, #376 as applicable; do not duplicate their scopes.

## Deferred / explicit TODOs
Jump Drive implementation and balance, explicit at/above-c behavior and high-speed protection (ordinary relativistic time dilation is not planned); breathable-atmosphere toxicity and suit oxygen supply; detailed reentry heating, structural tides, full EVA gravity migration, artificial ships/stations/signals/distress broadcasts, radar upgrades gameplay/progression, other galaxies, online data downloads/multiplayer/server authority, autopilot.


## Implemented U01 boundary (#402)

The [isolated portable schema](../client/universe/architecture.md) now implements
immutable source IDs, typed records and provenance/uncertainty validation. It does
not connect canonical truth to player discovery or replace the playable catalog.
U02–U34 retain their dependency gates. Existing gameplay implementation work is
delivered; separate native review is not claimed complete.

## Ongoing documentation implementation gate (#440)

The [U35 contributor/reviewer policy](documentation-maintenance.md) and GitHub
issue/PR templates are implemented. Every U01–U34 code change still updates its
own affected guides in the same PR, with explicit reviewed exceptions and checked
links. Automatic link/path/stale-contract validation remains #442 and final audit
remains #441. This policy setup does not certify future changes or native gates.

## U02 implementation status (#403)

Portable split-cell galactic positions, typed instantaneous frame trees and
calendar epoch/scale gates are delivered independently of playable migration.
See [limits and ownership](../client/universe/architecture.md#hierarchical-frames-and-epoch--u02-403).
Actual ephemerides, astrometry ingestion and UTC leap-table conversion remain
explicit downstream work; no reference-machine/real-dataset approval is implied.

U05 (#406) delivers a separate catalog-bound new-run save envelope, with verified
slot fallback and explicit incompatibility errors. Production pack verification,
scanner payloads, world migration and runtime adoption remain downstream. See
[contract](save-contract.md#future-universe-new-run-contract--u05-406).

U06 (#407) delivers pinned development source manifests, typed metadata and
reproducible offline raw staging. It does not distribute a scientific universe:
[reuse/full-snapshot follow-up #513](https://github.com/mr-exception/salimon/issues/513)
remains required for production Gaia inputs and pack publication. See
[source coverage and restrictions](universe-sources.md).

## U04 implementation status (#405)

The [binary partition prototype](universe-storage.md) delivers bounded versioned
compressed dependency-closed cells, strict immutable source/record indexes,
lossless U01 precision, corruption checks and random-access Read/Seek decoding.
Observed development sample measurements do not establish the 2–5 GB goal or
<=10 GB full-pack target. Production construction/publication, large-system
cross-cell format evolution and runtime caching remain U12/U13. Source reuse
remains #513; no runtime migration or M1 approval is implied.

## U03 implementation status (#404)

The [scientific reference harness](universe-reference-fixtures.md) delivers frozen
Horizons and analytic coordinate/time goldens, complete strict SI/frame/epoch
comparison, test-level numerical budgets by flight regime, a candidate-file CLI
and repeatable Linux CPU workloads. It implements no orbit/gravity solver and
approves no future scientific model drift. Reference M1 profiling remains
[native follow-up #521](https://github.com/mr-exception/salimon/issues/521);
full-game/flight/rendering acceptance remains #435–#439. Fixture/source-use and
production pack limitations remain intact.

U07 now implements local streaming Gaia selection and exact reviewed-ID matching
with typed source-unit measurements, missing-data preservation and synthetic
storage sensitivity evidence. See [implemented contract](universe-gaia-ingestion.md).
It is not a full observed catalog or production pack; #513 remains required.

U08 imports frozen Horizons geometric ICRF/SSB/TDB state/physical measurements and
PS exoplanet quantities with Gaia host reconciliation and explicit errors/limits.
See [implemented validity and unit policy](universe-observed-ingestion.md).
Exact samples only; arbitrary-date/lower-accuracy models remain U14, full reviewed
source coverage #526/#513 and real pack integration U12. No native world migration.

## U09 implementation status (#410)

[Offline inference](universe-inference.md) now completes missing physical/age/partial
orbital data for explicitly classified supported single-star systems, with
versioned heuristic priors, qualitative confidence, source evidence and bounded
physical screening. It preserves measurements and refuses unsupported/inconsistent
inputs. This is not empirical population calibration, a stability proof or a
complete ephemeris. Sidecar packaging remains U12, state evaluation U14 and
production-source/native gates remain independent.

## U10 implementation status (#411)

The [portable offline fictional-system builder](universe-fiction-builder.md) now
publishes stable individually materialized stellar/body records and compact minor
populations from explicit residual-density/age/metallicity regions. Actual
generation recipes and catalog bytes are hashed reproducibly. Validation uses
synthetic heuristic cohorts, not a calibrated production Milky Way census. Full
selected-world definitions are delivered by U11 below; other worlds' full
definitions remain downstream. Production batching/packaging U12 and native
real-scale integration/signoff remain downstream. Runtime does not generate a
new canonical world on player arrival.


## U11 implementation status (#412)

The [permanent starting publication](universe-starting-system.md) now freezes
Nacre Reach / Vesper with separate IDs/names, conservative 37.5500 ly Earth-distance
proof, physical screening, complete selected-world inputs/digests and portable
single-surface evaluation. Authored nearby outcrops cover all five material keys
with checked ship-clearing walk paths. This is a cold airless fictional baseline,
not a biosphere, climate model, production Milky Way pack or native screenshot.
U12 packaging, U14 evolving states and U23/U24 playable terrain/arrival/mining
remain downstream. Current runtime startup, discovery/chart and save semantics
are unchanged; native #436–#439 retain separate acceptance.

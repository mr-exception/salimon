# Canonical surface gameplay (#425)

Runtime composes the published Vesper body and exact frozen terrain with existing
ship landing/continuous contact, radial walking, doorway/cabin gravity, resource
meshes, continuous spherical mining, fragment support/carry and Fabricator intake.
`TerrainMode::Canonical` selects the published body list; representative/flat
modes preserve their existing isolated compressed fixtures. Ship snapshots carry
the selected terrain mode so downstream frames cannot accidentally select Earth.
Normal new-run startup and ordinary play use this integration (#426/#427);
compressed worlds remain explicit engineering fixtures.

The surface adapters do not flatten Vesper's distant galactic anchor. Coordinates
are body/system-local metres, with separate canonical identity from U11. Ground,
height-only physics, normals, landing footprint support and visual patches all
consume the same immutable evaluator. Internal cabin gravity remains controlled
by ship power; the existing radial walking/fragment gravity rules are preserved.
Canonical ship gravity and closed progression are implemented by #432–#434.
Atmosphere damage, oxygen depletion and observed-body playable adoption retain
their separate limits.

## Resources and physical ownership

Only the five published U11 outcrops materialize on Vesper. Exact semantic local
IDs are iron=1, silicate=2, ice=3, organics=4, compounds=5, under the full Vesper
catalog ID and world digest. These explicit mappings do not depend on iteration,
world/session seed, camera, LOD or a runtime hash. Original published material
keys/masses and positions are retained; resource poses receive the existing
sampled authored-mesh support adjustment exactly once. There is no new lottery,
biomass everywhere, material conversion or free inventory.

The ordinary mining journal owns depletion, attached/released physical identities
and conserved mass. Cache eviction replays this journal before queries. Carrying,
fragment gravity/support and Fabricator intake use the existing physical object
and material contracts, so accepting a fragment consumes that identity once.
CPU tests exercise canonical mining to real fragments, cargo transfer, Fabricator
intake and restore; native routes exercise actual input, mining, pickup/drop and
save/reconciliation on the published world.

## Canonical binding (introduced in schema 24; current schema 30)

Physical source now carries the existing U05 `DefinitionBinding` for canonical
runs: full body ID, exact definition version and SHA256. It must equal the frozen
publication; unknown/missing/mismatched bindings fail before publication. Ship and
world terrain modes must agree. Depletion IDs must belong to published outcrops
and remaining mass cannot exceed their initial source mass. Character/fragment
surface callbacks are reconstructed from the same selected world after restore.

Historical fixture saves 1–23 remain explicitly admitted and re-encode as version
24 in explicit fixture reconstruction without changing their source values. The independent U05 universe envelope
and its catalog/epoch requirements are unchanged; this surface binding does not
claim a complete real-data pack, a new ephemeris or general save migration.
Changing a published definition requires explicit versioned migration, never a
silent substitution or regeneration on load. #427 excludes compressed prototype saves from ordinary play; #432–#434 add
versioned motion migration, chronological timed work and verified resume.

`CharacterController::on_surface` supplies a validated standing constructor,
using the sampled ground plus radial eye height and rejecting invalid/steep
initial ground. Runtime still owns hull/obstacle clearance and initial facing.
No user-facing teleport or navigation automation is added.

## Verification

Run `cargo test -p salimon-client --locked session::canonical`, historical save
contracts and affected world/ship/character/persistence tests. Native production
wgpu/software-Vulkan routes are `scenarios/evidence/universe-boarding.json`,
`universe-mining.json` and `universe-flight.json`. The `canonical-surface`,
`canonical-mining` and `canonical-flight` setups declare initial conditions;
subsequent actions use production command admission/controllers. Canonical host lighting now follows the approved motion source; weather, terrain
and biological presentation follow their implemented contracts. Hardware/Metal/FPS, subjective presentation and physical
platform checks remain #563/#561/#557, separately unverified.

Canonical new journeys (#426) use `session/canonical.rs::new_canonical_run`: frozen Vesper ID, published traveler offset [14,0], conservative authored-hull clearance and a view facing the landed ship. Fresh terminal restarts preserve run serial/seed policy and grant existing supplies once; restore never grants them. Flat fixtures remain explicit; normal launch rejects prototype saves under #427. Oxygen depletion stays deferred. Tests and native evidence: `reports/issue-426/README.md`; setup `canonical-journey`.


## Physical organisms and clearance (#620)

[Physical organism placement](physical-organisms.md) owns stable source/part
bindings, root/exclusion rules, common physics sweeps and live character/ship
clearance. Vesper remains sterile; mutable harvesting/removal remains #621.


## Ordinary canonical play (#427)

Normal startup accepts only schema 24+ canonical checkpoints with the exact
immutable definition binding. Prototype/engineering saves fail as incompatible
without overwriting either slot; select a separate save directory for a fresh
journey. Explicit test reconstruction still supports the historical fixtures.
Canonical rendering constructs only published local bodies and no precision
markers; F2 remains an explicit isolated engineering camera view. Canonical
ship telemetry and preview selection have no compressed 3 Mm neighborhood cutoff;
local collision/interaction ranges keep their own physical meaning.

Source/tests: runtime `checkpoints.rs` canonical gate tests, `app/scene.rs`
canonical scene test and ship `controller/telemetry.rs` range regression.
Ordinary canonical flight/offline trajectory use the approved two-body motion
publication. Selected observed JPL/PS source coverage is implemented by #526;
full Gaia acceptance remains #524. Generic observed ephemerides are not silently
substituted for the fictional playable system.

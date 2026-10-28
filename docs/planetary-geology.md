# Canonical planetary geology (#612)

World `terrain::geology` owns immutable version-1 geological interpretations of
validated terrain. `TerrainDefinition::geology()` selects a documented built-in
recipe: weathered rock for the historical Earth fixture, dry rock for other
representative fixtures, and homogeneous silicate for frozen Vesper. These are
stylized authored interpretations, not measured geological reconstructions.
Explicit authoring also supports ice and homogeneous bodies. No atmosphere,
tectonic activity, deposit, liquid, damage or survival effect is granted.

`GeologyDefinition` owns its upstream terrain snapshot. `SLMGEO01` stores algorithm
version, geology seed, profile and complete upstream terrain bytes; SHA-256 of
those bytes identifies the definition. Province IDs append a fixed region index.
The bounded codec rejects unknown versions/profiles, wrong bodies/radii, malformed
terrain and trailing bytes. `compatible_with` requires both the exact geology
identity and upstream terrain bytes. A changed recipe is a new identity; no
implicit migration or reinterpretation of old saves is supplied.

Eight seeded centers in body-fixed unit-sphere coordinates define broad coherent
provinces. All centers contribute inverse-distance weights to continuous extent,
weathering and layer-depth fields, including triple junctions and cube-face
edges. Dominant and neighboring IDs identify regions; categorical transitions
are expected at region/landform boundaries. Homogeneous bodies use one province.
No runtime random generator, query order, tile, render quality, camera or clock
enters the evaluator. Weights and process extents are dimensionless modeling
parameters, not measured fractions of a planet's age or geological mass.

The ordered conceptual history describes impacts, volcanic bedrock/deposition,
and, only in the explicit weathered profile, deformation/weathering. Existing
negative terrain features identify impact basins; existing relative elevation
and slope classify highlands, plains and fractured exposure. Ice has an icy
expanse classification. Substrate/layer/exposure describe existing ground; they
do not generate a second surface, excavate material or change resource recipes.
Future terrain/resource recipes may deliberately consume geology only through
new published definitions with explicit compatibility/migration ownership.

`sample` returns `GeologicalSurface`: its `surface` is exactly the upstream
`TerrainDefinition::sample`, including normals and height. Existing collision,
landing, deposits and published U11 bytes remain unchanged. In particular Vesper
keeps its published relief, landing cap, resources, seeds and palette. The
geology sidecar is reconstructed from the immutable terrain; session save schema
26 and U11 publication hashes do not change. It is not player-discovered knowledge.
Surface rendering consumes this boundary in #613; climate, active geology,
ecologies, terrain destruction and caves remain their separate scopes.

## Validation and source mapping

- `client/world/src/terrain/geology.rs`: recipe, identity/codec, history and query.
- `client/world/src/terrain/geology/tests.rs`: frozen identity, roundtrip/order,
  cube-edge/triple-junction continuity, profiles, malformed inputs, compatibility,
  exact terrain agreement and Vesper publication preservation.
- `cargo test -p salimon-world --locked geology`
- Run workspace build/tests/Clippy/format and `python3 scripts/check_docs.py`.

The [world architecture](../client/world/architecture.md) and
[invariants](../client/world/invariants.md) own this portable boundary.
[Approved direction](planetary-environment-art-direction.md) retains future
requirements. No renderer changes occur in #612; actual rendered appearance is
#613, with subjective/hardware qualification retained in #561/#563.

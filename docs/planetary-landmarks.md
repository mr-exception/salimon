# Geological landmark sites (#623)

`world::environment::landmarks::Landmarks` publishes a bounded immutable site
inventory over **one existing physical terrain**. It never adds geometry,
modifies source geography, deposits, rewards, chart entries or scanner contacts.
Terrain generates the actual relief; a site gives meaningful relief a stable
identity, geological interpretation and support radius. Runtime and renderer
continue to consume the exact physical heightfield, preserving existing real-body
geography from its authoritative publication. No separate landmark mesh/collider
or idealized duplicate heightfield can drift from walking/landing contact.

Publication inputs are complete geology/terrain identity, explicit observed vs
fictional geography, seed and at most four sorted authored signatures. A fixed
64-direction survey independently admits 1/16 candidates before relief validation;
no minimum, player-distance quota or camera/load-time RNG exists. Observed bodies
receive only explicitly authored anchors to existing relief. Homogeneous Vesper
currently has no procedural sites and no invented signature. General fictional
landforms are selectable only after actual relief measurements pass thresholds.

Local (64 m), regional (0.001 radius, bounded), and planetary (0.02 radius,
bounded) surveys compare actual center/eight perimeter heights. At least
2/10/40 m relief respectively is required. Geology/history/substrate and physical
slope classify supported basins, ridges, scarps or ice ridges. These are bounded
stylized interpretations, not scientific reconstructions or proof of a local
maximum. An authored key/anchor must have real relief and cannot duplicate a key
or overlap an admitted site's support; impossible flat claims reject. Arches,
caves, overhangs, terrain destruction and POI rewards remain unsupported.

IDs hash version, complete upstream identity, publication inputs and candidate
slot/authored key. Querying nearby sites, changing visual LOD or evicting meshes
cannot create/rekey sites. `binding`/`verify_binding` provides a strict immutable
reference for saves/future observations, rejecting changed source/version,
trailing bytes or corruption. Reconstruct from publication inputs before checking;
it is not a replacement for the terrain codec or a new mutable player save.
Current canonical checkpoints already bind frozen Vesper source and reconstruct
its same empty site inventory; no save-layout migration is required.

## Owners and validation

- World `environment/landmarks.rs`: publication, sparse survey, actual relief,
  source-bound IDs, nearby queries and strict immutable binding.
- World tests: reproducibility after codec reconstruction, rarity audits, empty
  worlds, observed geography, invalid signatures, exact physical source/normal,
  malformed/changed bindings and Vesper preservation.
- Runtime `terrain_patches.rs::landmark_tests`: actual production patch generation
  at three quadtree LODs agrees with terrain contact to <1 mm and reconstructs
  the same authored site/binding. Quality never selects permanent sites.
- Runtime opt-in `landmark_production_patch_export` exports a declared physical
  basin fixture through the real mesh/material generator. Renderer opt-in
  `landmark_production_reference` reads those exact vertices and captures the
  production terrain pipeline at three approach distances and a 1e12 m anchor;
  repeat frames must match. Keep original JSON/binding and media hashes.

Run affected landmark/terrain tests, workspace build/tests/Clippy/fmt and
`python scripts/check_docs.py --base <parent>`. Production `universe-terrain.json`
verifies canonical orbit/surface/edge streaming and revisit. Evidence lives in
`reports/issue-623/README.md`. Human art review #561, physical platform #557 and
hardware/M1/Metal #563 remain unverified. [Biological signature sites](biological-signature-sites.md) extend this inventory;
chart/field observations #625/#626, never auto-seeded from this private inventory.

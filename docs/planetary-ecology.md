# Rare biospheres and ecological regions (#618)

World `environment::ecology` owns an immutable version-1 biosphere publication
bound to an exact catalog body and full [climate/geology definition](planetary-climate.md).
Suitability is potential, **not evidence of life**. A separate fixed decision is
Unknown, Sterile, MicrobialOnly or Developed. Observed bodies in this procedural
model must remain Unknown; they never receive invented confirmed alien life or
proven sterility. Fictional publications can explicitly select a suitable state,
or use deterministic rare selection. Vesper is explicitly sterile regardless of
seed and its frozen terrain/resources remain unchanged. Existing carbonaceous
feedstocks/medicinal material names do not establish a biosphere.

The declared potential model screens stellar energy, broad temperature range,
pressure, atmospheric chemistry and condensable availability. It combines local
canonical substrate weathering, relief/slope, latitude and a fixed reference-epoch
climate. These are bounded authored heuristics, not an astrobiological probability,
measured habitability, proof of surface liquids, food/medicine safety or a model of
all possible alien biochemistry. Zero light/vacuum/unsuitable inputs cannot
procedurally acquire a developed surface biosphere. No oxygen, survival hazard,
hydrology, fauna, autonomous ecosystem or resource recipe is introduced.

Fictional generation first checks potential, then uses an independent ecology
seed/domain: under 2% of suitable seeds select life, with under 0.4% selecting
developed communities and the remainder microbial-only. These are explicit
procedural art-direction rarity parameters, not scientific occurrence rates.
A finite seed batch is not guaranteed a living world. The selected state is encoded
once; visiting, loading, weather, rendering and device quality never roll again.
Suitability never overrides an explicit sterile decision. Microbial-only worlds
have no implied visible vegetation or large-organism sites.

Eight seed-bound spherical ecological realms blend through continuous regional
weights. Local exposed/sheltered/cold substrate community weights sum to one;
realm labels may change at dominance boundaries without snapping cover/potential.
Biological family grammar identity has its own version/domain, body and seed,
separate from terrain/resource/weather seeds. At most 32 fixed candidate patch
anchors are selected from continuous developed cover. They retain original slot
IDs, canonical terrain positions and realm provenance through eviction/reload.
They are ecological region anchors, **not rendered organisms, colliders, guaranteed
landmarks, treasure, chart markers or harvestable material**. Mesh/family generation,
physical organism placement and harvesting remain #619–#621. Seasons/weather may
later change appearance but cannot rewrite this immutable baseline or identities.

## Publication, catalog and save binding

`SLMECO01` stores algorithm version, independent seed, nature/decision, validated
catalog identity and complete climate bytes. SHA-256 identifies the entire
publication. Strict known-body decode rejects unsupported versions, malformed
identities, impossible decision/context combinations, bad upstream definitions,
truncation and trailing data. Changing a seed/decision/upstream climate creates
a new definition; `compatible_with` requires exact catalog body, identity and
climate, so revisions require explicit migration.

`BiosphereCatalog` is a bounded sorted inventory (maximum 1,024 publications).
`SLMECAT1` rejects duplicate bodies, wrong order, missing physical metadata,
malformed lengths and trailing bytes. A host-provided authoritative catalog-body
resolver supplies physical metadata; the codec never guesses a radius or falls
back to another body. Save consumers retain the catalog-body/definition-ID pair
and use `resolve` to reject missing or stale definitions before publishing state.
Roundtrip restores reconstruct exactly the same decisions, realms and patch IDs.
This additive immutable publication format changes no U11 hash or current session
schema and grants no player discovery, chart entry or new live save owner.
Mutable biological depletion/organisms and their runtime save integration remain
#620/#621; this task does not pretend those features already exist.

## Validation and sources

- `client/world/src/environment/ecology.rs`: decisions/potential, strict publication
  and inventory codecs, continuous realm queries, grammar and patch identity.
- `client/world/src/environment/ecology/tests.rs`: frozen seed-batch rarity and
  independent numerical potential, potential/existence separation, sterile and
  observed-body rules, spherical seams and climatic variation, independent seeds,
  original physical anchors, concurrent/stream/save reconstruction, malformed data,
  catalog mismatch/revision rejection and Vesper preservation.
- `cargo test -p salimon-world --locked environment::ecology`
- Run affected world/ship tests, workspace build/Clippy/fmt and committed-diff docs
  checks in [validation](validation.md). No renderer/runtime mesh, source resource,
  collider or audio code changes; graphical captures are not applicable. Appearance
  and hardware qualification remain separately unverified under #561/#563.


## Organism family presentation (#619)

[Procedural organism families](planetary-organisms.md) map the world grammar,
stable cosmetic surface tiles, runtime adapter and bounded renderer/test owners.
Physical colliders/landing safety are #620; biological material response is #622.


## Physical organisms and clearance (#620)

[Physical organism placement](physical-organisms.md) owns stable source/part
bindings, root/exclusion rules, common physics sweeps and live character/ship
clearance. Vesper remains sterile; mutable harvesting/removal remains #621.

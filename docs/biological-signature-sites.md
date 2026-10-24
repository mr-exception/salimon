# Rare biological and mixed natural sites (#624)

`world::environment::signature_sites::SignatureSites` publishes rare site metadata
for **existing physical organisms**. Giant coherent structures reuse the PE08
Giant anatomy and PE09 stable roots/parts/collision. Dense colonies require at
least two real Colony organisms in one ecological patch within 8 m; metadata
never manufactures extra specimens to meet a quota. Ecological sample/realm,
family pigment, material, terrain and immutable publication provenance remain
with their original owners. A site supplies no treasure, automatic chart marker,
new organism, growth, terrain displacement or additional collider.

Each eligible physical source has a fixed 1/8 signature admission, independent
of viewpoint, renderer LOD, player distance, visit/load order and hardware. This
is conditional on the already rare published developed biosphere/population;
sterile, microbial and observed-unknown bodies remain empty. No minimum signature
count exists. Up to two explicit authored signature keys may refer to eligible
real sources; duplicate/unknown/excluded sources or keys reject. No authored
owner signature is silently invented or promoted to an approved art baseline.

A mixed geological site must intersect the actual support radius of a published
PE12 geological site. Geothermal/ecological labeling additionally requires the
same climate-bound active geological publication, positive declared heat input
and a physical source inside its real site radius. Ancient volcanism or a color
palette is insufficient; this does not create heat, hydrology or ecological
simulation. Candidate sites retain exact geological/activity IDs. Dormant or
nonintersecting sources never grant geothermal classification.

The site bounds are the union of the exact existing physical part boxes. Runtime
renders those same part endpoints/radii/pigments through the production organism
pipeline and existing six/three-sided LOD; quality cannot remove the physical
anatomy. Ship/resource/other-organism exclusion already occurs in the immutable
PE09 publication. Naming it does not alter clearance or add a visit-time placement
rule around a moving ship. Cosmetic cover cannot become a physical signature.

## Identity, persistence and ownership

Publication identity binds algorithm version, full biosphere, physical population,
geological inventory, optional activity and sorted authored source keys. Source
organism IDs stay unchanged; site IDs bind their complete sorted source group.
`binding`/`verify_binding` supplies an immutable save/reference contract, checked
after reconstructing the source publication; changed definitions or malformed
bindings reject. It does not replace the existing organism/terrain codecs or add
mutable player save state. Current frozen Vesper saves reconstruct the same empty
`canonical::NATURAL_SITES`, without a schema bump. Future #621 removal/depletion
must preserve site identity while rendering actual missing parts, never respawn
biology just because a site is named.

World `signature_sites.rs` owns publication/grouping/provenance; organism placement
exposes its original biosphere identity for compatibility validation, preserving
all existing IDs/binding bytes. World tests audit rarity/reconstruction, exact
physical bounds, mixed geological provenance, invalid/excluded sources and
sterile/canonical behavior. Runtime `app/organisms.rs::natural_site_evidence`
compares every rendered endpoint with its source collider and exports actual
rare giant/dense-colony geometry, IDs and publication/source binding. The existing
production renderer reference captures that same data at true physical scale;
no special detached render-only landmark is invented.

Run `cargo test --workspace --locked natural_`, the workspace build/test/Clippy/fmt
gates and documentation checks. Opt-in `natural_site_production_export` writes
`SALIMON_NATURAL_SITES_FILE`; feed that absolute path as
`SALIMON_ORGANISMS_FILE` with `SALIMON_ORGANISM_DIR` to renderer
`organism_production_reference`. Retain originals, source bytes and hashes in
`reports/issue-624/README.md`. Production vacuum/boarding and parent terrain
streaming routes exercise ordinary runtime preservation. Mixed geological source
logic is automated; combined positive-world art acceptance and all subjective
review remain #561, physical platform #557 and actual hardware/M1/Metal budgets
#563. Software Vulkan proves neither human art approval nor hardware FPS.

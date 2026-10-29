# Procedural organism families (#619)

World `environment::organisms` derives a version-1 family grammar from the
published PE07 family ID. Stable SHA-256 specimen IDs bind grammar/form/variant;
part IDs are stable local ordinals, independent of render density, load order or
camera. Six families (mats, ringed columns, fronds, filaments, colonies and giant
structural forms) use reviewable parametric tapered segments, related palette
and bounded morphology variation. Forms have 5–14 individually identified parts;
variants 0–63 reject outside that range. No downloaded model/service, Blender
export, authored asset replacement, new fauna or growth simulation is used.

Developed fictional biospheres alone create macro-organism grammars. Sterile,
unknown observed-life and microbial-only definitions do not get large plants.
Vesper explicitly retains its published sterile decision. Species do not
reshuffle at visit/load; ecology/climate/geology and original terrain remain
immutable. The existing macro-cover/realm/community metadata is the coarse
coverage authority; fine visible cosmetic candidates sample that same ecology.
Detailed biological material/animation consumers remain #622.

A `CoverTile` is a fixed level-16 cube-face cell, not a viewport or terrain LOD
key. Up to 64 small mat/filament candidates have stable source IDs, body-fixed
surface positions and normals from authoritative terrain. Render density admits
a stable candidate prefix, preserving matching positions/parts. These candidates
have no colliders, mass, resource availability or save authority. The runtime
canonical adapter selects the current tile; world tile APIs support a bounded
terrain stream's neighboring-cell selection without regenerating definitions.
Physical large organisms/roots, placement exclusions and landing safety are #620.

Runtime `app/organisms.rs` maps local segment endpoints through the supplied
surface basis and f64 world anchor. Renderer owns closed faceted tapered-prism
geometry and lighting only, with six-sided near and three-sided unresolved LODs.
LOD changes only below 0.75 projected radius pixels, retains every segment/root
and endpoint, and never changes source population or physical bounds. It does
not enlarge distant objects into navigation markers. At most 16,384 segments,
two draws, 48 B per instance and two fixed 768 KiB instance buffers bound work.
A body-relative f64 subtraction precedes f32 upload; buffer/radius/color/end-point
validation rejects malformed input. Opaque reverse-Z geometry participates in
normal atmosphere/weather occlusion. No translucency/glow/weather motion is
introduced here; #622 owns those enhancements.

## Source and validation

World tests verify all six related forms, bounded parts/palettes, stable IDs,
sterile/microbial exclusions, immutable source bytes and exact cosmetic density
subsets. Runtime tests verify sterile startup and f64 large-anchor mapping.
Renderer tests verify closed bounded LOD meshes and unchanged part counts.

Export actual source specimens with `SALIMON_ORGANISMS_FILE=... cargo test -p
salimon-client --locked organism_specimen_export -- --ignored --nocapture`, then
pass the same file and `SALIMON_ORGANISM_DIR` to renderer
`organism_production_reference`. Twelve original software-Vulkan images capture
all forms/variants at true generated scale with separately fitted cameras;
far-LOD preparation keeps identical part counts. Run affected organism/ecology
checks, workspace build/Clippy/fmt, tooling/docs and native vacuum startup.
Original media remains review candidates for #561; hardware #563 and physical
platform #557 remain unverified. No rendered specimen is silently published as
a canonical new world or promoted to an approved art baseline.

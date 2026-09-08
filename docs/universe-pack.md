# Incremental offline packs (#413)

`pack` owns a version-1 content-addressed local pack. `pack::pipeline::build`
(or `cargo run -p salimon-universe --example pack_build --release --locked --
BUILD_REQUEST.json OUTPUT_DIR`) is the standalone offline entry point. It reads
selected local U01 catalogs or producer bundles, validates exact original source
snapshot lengths/SHA256s, packages all bundle observation/inference/generation
fields as sidecars and accepts additional exact-byte full-world sidecars. It
processes one dependency-closed cell at a time, preserving IDs and values under
the explicitly requested combined publication revision.

A build request has `version:1`, `revision`, a root-relative `source_manifest`,
`cells:[{key:{galactic:[x,y,z],shard:n},input:"producer.json",sidecars:{name:"file"}}]`
and an optional `migrations` object mapping old IDs to replacement IDs. Paths are
relative to the request directory; source artifacts are relative to their source
manifest. Absolute/traversing paths and escaping symlinks reject. Producer
bundles contain `catalog` plus named JSON fields; plain U01 catalogs also work.
U07/U08 import adapters, U09/U10 publications and the U11 complete world manifest
remain their owning producers; this builder preserves their outputs rather than
inventing observations, orbital phase, terrain or physical models.

Immutable SHA256 blobs are reused on incremental builds and checked before reuse.
The old manifest remains active until all cells validate and the replacement
manifest is atomically published. Global duplicate identities/source conflicts
reject; removed IDs require an explicit migration against the previous manifest.
Failures may leave unreachable immutable blobs, but never publish a partial
manifest. Private real observed data is supported without source-review flags.
Public/commercial redistribution review remains #659; source statuses are not
rewritten. The format's `production_eligible:false` is release eligibility, not a
restriction to synthetic input.

Bounds are explicit and fail closed: 4096 cells, 16 MiB manifest, 16 MiB U01 raw
catalog per cell and 256 MiB per blob. Split large inputs into dependency-closed
cells before building; unsupported larger layouts fail rather than silently
omit data. The cloud workload measures complete installed bytes, manifest hash
and build throughput for original/expanded Earth/Proxima samples. Full selected
Gaia scientific coverage and real Milky Way size acceptance remain #524;
selected JPL/PS coverage is documented in [#526](observed-source-coverage.md); measured fixture density is not extrapolated into
a claim of meeting 2–5 GB or the 10 GB full-release target. Exceeding that target
requires the already specified measurement/escalation workflow there.

Run universe tests, all-target universe Clippy, formatting and docs/base-impact
checks. Run `pack_bench OUTPUT_DIR` in release mode for repeatable storage and
throughput evidence. Playable world adoption is owned by later expansion issues;
no human gameplay or hardware signoff is required for this portable builder.

Producer bundles retain raw JSON field bytes. Duplicate root fields reject, and
the nested catalog is decoded from its original bytes so duplicate/unknown U01
fields cannot be hidden by intermediate JSON normalization. Source hashes and
the previous active manifest remain unchanged on these failures.

## Raw observed-source input

Instead of `input`, a cell may supply `observed` to run the existing U07/U08
production adapters directly: `gaia` and `exoplanets` each select
`{source:"dataset:id",artifact:"frozen/path.csv"}` from the source manifest;
`horizons` is a list of `{source,artifact,system,parent,body_kind}` selections.
Optional `stellar_cross_matches:[[source_id,canonical_id],...]` supplies explicit
exact-ID associations. No name/proximity matching or missing measurement values
are invented. Body classification/parents are explicit input, validated by U01.

Selected raw cell artifacts are bounded to 8 MiB before collector allocation and
rehash-checked into immutable input bytes immediately before import. Larger
selections must be partitioned into dependency-closed batches; they are never
silently truncated. Gaia measurements/selection evidence, Archive observations
and Horizons states/raw responses are packaged as typed producer sidecars.
Exactly one of `input` and `observed` is required per cell; explicit additional
sidecars can provide full canonical world manifests alongside either route.

Selected real JPL/PS builds, expected target IDs, star classification, source-inventory
cell limits and independent verification follow [the observed coverage contract](observed-source-coverage.md).

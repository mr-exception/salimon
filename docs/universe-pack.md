# Incremental offline packs (#413)

`pack` writes a version-2 content-addressed local pack and reads versions 1 and 2. `pack::pipeline::build`
(or `cargo run -p salimon-universe --example pack_build --release --locked --
BUILD_REQUEST.json OUTPUT_DIR`) is the standalone offline entry point. It reads
selected local U01 catalogs or producer bundles, validates exact original source
snapshot lengths/SHA256s, packages all bundle observation/inference/generation
fields as sidecars and accepts additional exact-byte full-world sidecars. It
processes one dependency-closed cell at a time, preserving IDs and values under
the explicitly requested combined publication revision.

Version 2 losslessly zlib-compresses a sidecar only when that reduces its size.
The blob's SHA256 and `bytes` describe the stored stream; optional
`zlib_raw_bytes` declares its exact decoded length, bounded to 256 MiB before
allocation. `pack::read_blob` and the normal read-only reader return the original
producer bytes. Complete checksum-verified single streams are required; truncated,
trailing/concatenated streams and false decoded lengths reject. Unknown versions
reject; version 1 remains raw-only. Catalog partitions already have their own
compression and are not recompressed. Incremental builds may upgrade a v1 pack
without changing record IDs, and corruption in a reused compressed blob prevents
publication. Old readers cannot open v2 packs; retain the old publication when
using an older binary. Installed-byte measurements count encoded blob sizes,
not inflated sidecars.

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

Bounds are explicit and fail closed: 4096 cells, 16 MiB legacy-v1 / 64 MiB v2 manifest, 16 MiB U01 raw
catalog per cell and 256 MiB per blob. Split large inputs into dependency-closed
cells before building; unsupported larger layouts fail rather than silently
omit data. The cloud workload measures complete installed bytes, manifest hash
and build throughput for original/expanded Earth/Proxima samples. Full selected
Gaia delivery now uses [#524's 1 GB nearby selection](nearby-celestial-catalog.md);
full Milky Way scientific/size acceptance remains future expansion. The
selected JPL/PS coverage is documented in [#526](observed-source-coverage.md); measured fixture density is not extrapolated into
a claim of meeting 2–5 GB or the 10 GB full-release target. Exceeding that target
requires the already specified measurement/escalation workflow there.

V2 keeps the root's literal-ID index and bounded cell catalogs. The 64 MiB root
admits the first nearby catalog; it is not a billion-source design. Builder
global identity checks are incremental and a new cell is validated before commit,
avoiding a full manifest clone/validation for every batch. Finish still validates
the complete publication and preserves atomic replacement/migration rules.
Future larger regions must respect those limits or implement a scalable paged
index, with exact source/science/storage acceptance appropriate to that scope.

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

`csv_gzip` source artifacts are hashed as stored, then checksum-verified and
inflated to at most 8 MiB. Trailing/concatenated members and truncated streams
reject before import. `votable_gzip` archives original TAP responses and metadata
for independent transcription verification; the production CSV adapter consumes
the separate exact-string CSV transcription.

Selected real JPL/PS builds, expected target IDs, star classification, source-inventory
cell limits and independent verification follow [the observed coverage contract](observed-source-coverage.md).

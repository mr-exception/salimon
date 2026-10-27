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
Gaia/JPL/PS scientific coverage and real Milky Way size acceptance retain their
existing #524/#526 ownership; measured fixture density is not extrapolated into
a claim of meeting 2–5 GB or the 10 GB full-release target. Exceeding that target
requires the already specified measurement/escalation workflow there.

Run universe tests, all-target universe Clippy, formatting and docs/base-impact
checks. Run `pack_bench OUTPUT_DIR` in release mode for repeatable storage and
throughput evidence. Playable world adoption is owned by later expansion issues;
no human gameplay or hardware signoff is required for this portable builder.

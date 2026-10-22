# Issue 407 — pinned sources and offline importer

Delivered typed source metadata, official release/query/coverage/uncertainty/credit
references, SHA256/size-pinned development fixtures and atomic offline raw staging.
JPL/exoplanet fixtures are actual frozen API responses; Gaia fixture is explicitly
synthetic. No full catalog ingestion, inferred science or runtime network added.
[Source contract](../../docs/universe-sources.md) and
[owning architecture](../../client/universe/architecture.md#source-manifests--u06-407)
record exact evidence and boundaries.

Checks: four Python importer contracts pass, including socket-disabled two-build
byte reproducibility, tamper/path/version refusal and production rejection.
Rust universe tests, all-target universe Clippy, fmt and documentation lint pass;
Ubuntu PR CI executes offline import and both typed/tooling regression suites.
No runtime source/dependency changes since #406; prior workspace gates apply.

Actual official API fetches succeeded on 2026-10-21; endpoint/query/response
metadata remain in the manifest/raw snapshots. Gaia license/credits full pages
returned HTTP 451; official indexed policy states CC BY-NC 3.0 IGO. Do not claim
commercial or general production reuse approval. #513 owns source-specific
reuse review and full input snapshot approval. Production mode deliberately fails
until reviewed eligibility and actual data replace restricted/synthetic inputs.
U07/U12 production dependencies must follow #513 rather than assuming U06 closure
waives this gate. No native/performance or scientific-dataset acceptance claimed.

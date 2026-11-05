# Predictive spatial streaming (#422)

> **Corpus migration status (#791): Historical native implementation.** The current [React overhead 2D world](current-client-status.md) uses a separate **cockpit-only 3D radar**. Any older Rust, wgpu, 3D assets, first-person view or issue-delivery statements below concern the [archived native tree](https://github.com/mr-exception/salimon/tree/8f112bb6e6be9f813174efb47d706a8601930d04) rather than shipped browser features. Keep approved versioned identities, environmental and simulation rules; represent planets in authoritative top-down illustrated layers and footprint collision, with faceted celestial contacts only in cockpit radar. Native mesh/camera/GPU and Rust source details are archived. See the [migration register](2d-migration-register.md) for readiness and ownership.


`salimon-universe::streaming::Stream` composes the verified U13 filesystem
reader with split-cell galactic navigation. The native runtime opens the optional
`SALIMON_UNIVERSE_PACK` directory at startup and polls it during presentation.
Invalid pack startup fails explicitly. No catalog truth is exposed as discovered
navigation. Canonical gameplay/startup is implemented by #425/#426; streaming
uses the #432 inertial observer and orbital velocity without exposing pack truth.

The current and five-second projected positions select a deterministic nearest
set of at most sixteen cells (configurable 1–64). Integer cell differences precede
floating conversion. Catalogs include system/body detail; ephemeris evaluation
continues to use U14's explicit epoch/validity contracts. No per-object global tick,
network fetch, source mutation or new world generation occurs during travel.

One dedicated worker performs verified disk reads and decoding; the frame thread
uses bounded nonblocking channels without waiting for disk or a reader lock.
There is at most one requested/result cell in flight. A generation number rejects
old results after traversal, and active catalogs outside the target are evicted.
A corrupt/missing required cell leaves an observable `last_error` and readiness
false; changing the target permits retry. Initialization and shutdown may wait for
bounded manifest/current-cell IO; no wall-clock disk-latency guarantee is claimed.

Budgets are explicit: default encoded cache is 16 MiB; decoded residency is bounded
by active-cell count and existing per-cell schema limits, with at most one decode
scratch/result beyond it. Manifest/index and decoded catalogs are outside the
encoded byte budget. At most eight results may be polled per call; default two.
No GPU allocation is made by this service. Terrain/GPU budgets belong to #424.
Quality changes only selection and residency; identical keys return identical
canonical bytes. `ready()` is false while loading or after error; consumers must
not invent missing bodies or treat partial loading as verified world state.

Run `cargo test -p salimon-universe --locked streaming` for full filesystem
traversal, quality-invariance, stale-result, corruption and budget regressions.
Full-data release coverage remains #524/#526 and public distribution review #659.
Hardware timing/Metal qualification remains #563/#557; the service tests establish
portable contracts, not real GPU frame rate.

The native `inspect.universe_stream` observation reports readiness, active/wanted
counts, verified IO and encoded residency. The opt-in
`scenarios/evidence/universe-streaming.json` route expects the single-cell output
of `pack_bench` and asserts an actual worker load in the running wgpu client.

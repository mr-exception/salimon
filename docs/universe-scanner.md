# Scanner discovery contract

`salimon-universe::scanner::Contacts` owns player discovery independently of immutable
catalog truth. It starts empty. Only a successful powered, catalog-validated
observation refreshes an exact celestial identity, explicitly framed position and
SI velocity. Unknown records, systems/statistical populations, duplicate
batches, nonfinite velocities, revision mismatch and regressing clocks reject
atomically. History is bounded to 65,536 identities and ordered by stable ID.

Fresh contacts expire at exactly three hours since the last successful refresh.
Power loss and misses never refresh or erase history. Rediscovery updates state
and retains first encounter and refresh count. Observations are sampled states,
not extrapolated current positions. No occlusion or prior knowledge is inferred.

UTC labels use supplied nondecreasing POSIX gameplay milliseconds, matching
checkpoint wall-clock labels. This is explicitly not a UTC-to-TT/TDB astronomy
conversion; source evaluation retains its explicit scientific epoch. No OS clock
is read by the domain. Pulse/range/Core funding is #429; checkpoint integration
is #431; physical presentation is #430 and chart provenance is #469.

Validation: `cargo test -p salimon-universe --locked scanner`, universe/workspace
checks, Clippy, fmt and docs checks. Tests cover distant split-coordinate identity,
expiry edges, blackout, history/rediscovery and atomic malformed batches.

## Pulses, frames and Core funding (#429)

`scanner/pulses.rs` admits positive configurable range, continuous watts and joules
per pulse. Pulses lie on the absolute five-second lattice; no manual/passive mode
exists. Celestial range is inclusive at 100%; the future noncelestial contract is
40%. No size/occlusion filtering or light-speed roundtrip is implied: identification
is an intentional astronomy-scale gameplay abstraction, not physical radar.

`scan_linear` atomically consumes certified affine trajectory pieces split at
power/range/model changes, intersects the relative line with the inclusive range
sphere, and finds the last lattice pulse in the crossing interval analytically.
Work is bounded by supplied objects/pieces, independent of elapsed pulse count.
Unpowered intervals advance freshness without discoveries. The chronological
U31/U32 host (#432/#433) supplies certified nonlinear path subdivisions and actual
funding windows; the scanner never invents a gravitational trajectory.

Bulk offline scans preserve the earliest successful sighting, latest sampled
state and exact successful pulse count analytically (at most two chronological
updates per object/piece). Near-zero nonzero motion clips traveled distances
before time division, preventing overflow roots from inventing a range crossing.

Current canonical surface gameplay samples Vesper in its explicitly named
body-centered frame; absent generated orbital phase/velocity is not zero galactic
motion. `Position` separates body-local coordinates from true split galactic
states; mixed frames reject. Only the currently represented body has a production
state adapter. Other objects require actual evaluated same-frame source states,
not assumed missing astrometry. Offline pack loading alone reveals nothing.

Runtime samples the completed authoritative tick when a pulse boundary is crossed
(up to one 16 ms tick of latency); it never backdates a fabricated state. Continuous
and per-pulse costs debit the real Core after other powered work. Insufficient
funding does not refresh contacts; available partial service energy is spent,
without queuing unpaid pulses. Pause/terminal freezes the shared running clock.
Parameters are in `client/runtime/config/scanner.json`: initial 1e9 m range,
1 kW continuous and 10 kJ/pulse. These configurable implementation defaults are
not hardware/balance approval or a new upgrade acquisition mechanic. Checkpoint
mapping/launch freshness follows in #431; generic closed-game timed integration
remains #433 as explicitly scoped in that issue.

## Checkpoints and launch (#431)

`scanner/checkpoint.rs` owns version-1 contact source (catalog revision, logical
UTC watermark and sorted encounter records). Reconstruction revalidates celestial
IDs, frames/states, first/last-seen ordering, nonzero counts, duplicates and size.
No contact is created merely by opening a catalog. Runtime
`session/scanner/save.rs` owns schema-25 mapping of parameters, clock/remainder
and those records; presentation selection/power/error are transient.

Canonical schema 25 requires scanner source. Canonical versions 1–24 migrate to
an empty scanner at their checkpoint UTC, preserving other source and granting
no historic discoveries. Historical compressed fixtures omit scanner state.
Wrong catalog revision is an incompatible-catalog error, not permission to load a
new galaxy or silently discard history. Restore validates before session publication;
a corrupt source cannot debit Core or partially replace a live session.

Normal startup/restart binds an empty scanner to the supplied checkpoint UTC.
Running simulation uses the shared admitted clock (including pause/terminal
freeze) at 1:1 rate. Launch-only nonnegative elapsed time advances scanner
freshness and retains history, after the existing chronological powered-work
reconciliation. The launch watermark prevents duplicate credit and rollback.
No offline observations are fabricated while the trajectory/event host #433 is
absent. Expired contacts can be rediscovered by ordinary paid scans.

Tests: `cargo test -p salimon-universe --locked scanner`,
`cargo test -p salimon-client --locked scanner`, persistence/launch regressions,
workspace build/tests/Clippy/fmt and parent-relative docs validation.

## Cockpit hologram (#430)

The console projector sits on the existing console, with a 0.30 m projection
radius centered at ship-local [5.2, 1.75, -0.9] m. Three orthogonal dotted rings,
a ship-origin marker, contact cubes and elevation stems provide actual 3D depth
through the ordinary production depth-tested instance pass. Logarithmic radial
scaling compresses astronomical distances; direction/elevation remain 3D. Real
sampled distance and age are shown in the cockpit hint. This is navigation
presentation, never a physically rescaled celestial object or collision volume.

M selects the next contact through the same native/automation input path. Only
fresh same-frame identified contacts are projected; unknown catalog objects never
appear. Power loss suppresses all projected rings/contacts and dims the housing;
encounter history is unaffected. Selection is transient, does not control flight
or scanner cadence, and is gated by cockpit, pause/terminal and modal admission.
The existing flight/navigation monitor remains available. Mappings preserve the
current interpolated ship pose and subtract/rebase through the ordinary renderer.

Renderer geometry is bounded to 64 contacts and at most 227 cuboids, with the
selected contact included even beyond the first page. No new draw pipeline,
postprocessing, shadow, model service, authoritative collider or periodic asset
regeneration is introduced. The base is procedural decorative projector hardware
on the existing console. Projection uses ordinary scene lighting/depth; it is a
low-poly holographic instrument, not a physical optics simulation.

Validate renderer projection/blackout/bounds, runtime observation nonmutation,
input gates, actual software-Vulkan captures and the maintained
`scenarios/evidence/scanner-hologram.json` route. `scanner` is a close reference
camera; it changes presentation only. Baseline/appearance, real physical controls
and hardware/Metal costs remain separately unverified (#561/#557/#563).

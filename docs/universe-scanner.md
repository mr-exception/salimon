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

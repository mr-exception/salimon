# Scanner discovery contract

`salimon-universe::scanner::Contacts` owns player discovery independently of immutable
catalog truth. It starts empty. Only a successful powered, catalog-validated
observation refreshes an exact celestial identity, split galactic position and
inertial SI velocity. Unknown records, systems/statistical populations, duplicate
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

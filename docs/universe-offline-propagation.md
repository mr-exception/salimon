# Closed-game motion and canonical frame (#432)

Status: canonical motion, schema-30 migration and gravity-only launch integration are implemented on the batch branch. Scanner catch-up (#433), asynchronous menu publication (#434), the final audit (#441) and integrated evidence (#435) follow in the same PR.

## Independent propagation core

`client/ship/src/offline.rs` composes the existing gravity/drag/contact trajectory
operators without granting thrust or spending Core energy. A caller supplies a
complete epoch-aware `Environment`, an explicit starting mode, physical ship
parameters and a requested Duration. `poll` bounds work per call rather than
truncating the elapsed interval. Only `verified` on a completed job exposes a
result for publication. Progress retains actual steps, impacts and the largest
contact witness uncertainty. That witness is not a global numerical error bound.

Coasting uses the existing four-contributor gravity selector and passive drag.
Grounded states require the provider to verify moving-body support at each new
epoch. Impacts dispatch rebound, grounded or terminal consequences. No fatality
threshold or automatic safe landing is invented. Provider errors remain errors;
missing ephemerides, air or collision geometry cannot become frozen bodies,
vacuum or empty space. The selected environment owns coverage and model validity.

The trajectory and a private cloned `Consumer` commit together after a successful
poll. Failure at a later boundary rolls back both, including earlier work in that
poll. Consumer clones must own their mutable sources; callbacks must not perform
IO or mutate shared state. The native host still owns crash-safe checkpoint
publication, engine shutdown, clock validation and admission to Resume.

## Approved canonical motion publication

The owner explicitly selected “Use that motion model and continue the five-task PR” on 2026-10-31. `client/universe/src/starting_motion.rs` publishes version 1 separately from the immutable terrain/resource definition. Its SHA256 binds all model fields and the unchanged geographic SHA256.

The publication retains Vesper's existing orbit size, circular eccentricity and period. It fixes zero phase at 2026-01-01 UTC, whose explicit reference is 2026-01-01 00:01:09.184 TT, and right-handed orbital axes (+X, +Z, -Y). Axial spin is zero. The star is at the system origin; Vesper follows the authored analytic circle. These are fictional model inputs, not inferred scientific observations.

The trusted local POSIX clock admits uniform elapsed SI seconds from that reference. This is an explicitly defined game-clock mapping, **not a general UTC-to-TT conversion or a claim about future leap-second announcements**. Epoch calendar bounds are checked before advancing; unavailable stellar-interior physics and invalid frame/definition sources reject resume. Imported ephemerides keep their own validity/fallback policies; they are not extrapolated by this model.

Runtime and ship compose in nonrotating, translating Vesper axes. Saved velocity is relative to that frame. Inertial position/velocity reconstruct from the motion epoch plus the body's analytic position/velocity and the split galactic system anchor, without flattening the galaxy into f64 metres. Grounded support retains body-relative poses while its inertial position follows the orbit.

`ShipController::advance_celestial` stages Core, motion and clock together. Active flight and closed coasting share symmetric gravity kicks around the existing paid-impulse and continuous terrain contact path. Relative acceleration subtracts the analytic frame acceleration. The current Vesper publication is explicit vacuum; no atmosphere is guessed. The generic trajectory/atmosphere/contact APIs remain available for other declared environments.

Far coasting uses steps at most one second on the exact epoch grid. Within a potentially intersected terrain shell it uses at most 16 ms; the published <=12 m/s² surface field gives <0.4 mm chord sag, below the existing 1 mm sweep tolerance. Canonical terrain now uses the actual folded-wave/smoothstep height-gradient bound, rather than the empty generic-noise layers. Work-limit exhaustion rejects the candidate instead of accepting a fabricated collision. Existing normal-clearance, rebound, commissioned-assist and crash policies are retained. The one-hour coarse/fine fixture requires <25 m position and <0.05 m/s velocity difference; this is a measured numerical regime, not a certified universal error bound.

Closing snapshots explicitly disable engines/throttle/held steering while retaining velocity, orientation, Core and committed paid assists. An abrupt process loss is also safe: launch disables engines before admitting closed time. Invalid motion prevents saving or resuming the candidate, preserving the last verified checkpoint.

## Schema 30 and migration

Canonical sessions require a `state.motion` source with exact version, definition SHA256 and TT game epoch. Schema 24–29 canonical saves without it migrate once from their recorded checkpoint time, preserving the entire physical/geographic/resource source and body-relative velocity. Current missing/unknown/mismatched motion fails atomically. Engineering worlds retain their existing independent frames and cannot carry canonical motion. Reconstruction alone advances no time or grants supplies.

See the [save contract](save-contract.md) and [source/report evidence](../reports/issue-432/README.md). This changes no observed source quantity, terrain definition, resource ID, Core conversion coefficient, Jump Drive rule or backend authority.

## Remaining batch work

#432 supplies ordinary saved-journey motion and close-time shutdown. #433 needs
chronological timed-system/scanner integration. #434 needs native asynchronous
progress, verified Resume and checkpoint failure/restart routes. #441 follows the
implemented U01–U33 audit; #435 follows that audit and integrated source/physics/
performance evidence. The combined batch is not ready until its remaining implementations and acceptance checks complete.

## Checks

Focused iteration: `cargo test -p salimon-ship --locked offline::`.
The new tests exercise actual production trajectory/contact operators, a complete
orbit against an independent analytic solution and half-step reference, progress
partitioning, first impact/support and atomic provider/consumer failure. Fixture
models are declared test inputs, not publication of Vesper motion or full-data
scientific coverage. See the [work log](../reports/issue-432/README.md).

# Closed-game propagation work in progress (#432–#435)

Status: **local, incomplete implementation; no delivery or resume claim**.

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

## Pending canonical motion decision

Current Vesper gameplay stores body-local positions and velocities. Its frozen
publication provides mass, radius, semi-major axis and orbital period, but no
orbital phase, orbital axes, epoch or spin. The ordinary flight controller does
not apply celestial gravity. A portable integrator passing its fixtures does not
establish ordinary online/offline consistency.

The proposed additional motion publication would explicitly freeze:

- Vesper's existing fictional orbit size, zero eccentricity and period;
- a reference epoch, zero reference phase and fixed right-handed orbital axes;
- zero axial spin, preserving the current nonrotating body-local frame;
- the exact motion-publication identity in saves, with an explicit migration
  preserving existing terrain, resources, object identities and body-local poses;
- reconstruction of inertial position and velocity from the body frame at that
  saved epoch, with no conversion of missing observed quantities to zero.

**This proposal is pending the owner's answer; it is not implemented or approved.**
It must not silently alter the frozen physical-world definition. The existing
[canonical-world migration requirement](universe-expansion-approved-design.md)
and [save contract](save-contract.md) apply. Scientific-source model validity,
UTC/TT conversion, authoritative terrain coverage and continuation of committed
assists also require explicit implementation and verification in the host.

## Remaining batch work

#432 still needs ordinary saved-journey integration, close-time engine shutdown,
canonical motion binding and full event/source validity acceptance. #433 needs
chronological timed-system/scanner integration. #434 needs native asynchronous
progress, verified Resume and checkpoint failure/restart routes. #441 follows the
implemented U01–U33 audit; #435 follows that audit and integrated source/physics/
performance evidence. None of these five issues is complete from this core alone.

## Checks

Focused iteration: `cargo test -p salimon-ship --locked offline::`.
The new tests exercise actual production trajectory/contact operators, a complete
orbit against an independent analytic solution and half-step reference, progress
partitioning, first impact/support and atomic provider/consumer failure. Fixture
models are declared test inputs, not publication of Vesper motion or full-data
scientific coverage. See the [work log](../reports/issue-432/README.md).

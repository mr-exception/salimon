# Real-scale surface support and funded assists (#418)

`salimon_ship::surface` is the portable real-scale surface-flight owner on top of
[#417's trajectory state](universe-trajectory.md). It accepts canonical body IDs,
split galactic coordinates and explicit TT/TDB epochs. `BodyMotion` is an
instantaneous ephemeris result with center position, velocity and acceleration;
its epoch must match the query. `Site` fixes a surface offset and constant rigid
spin at a reference epoch. Orbital ephemerides remain the caller's responsibility;
missing/invalid epochs fail rather than freezing a moving body.

## Support and motion

A site's inertial velocity is center velocity plus omega cross radius. Its
acceleration includes center acceleration and centripetal acceleration. Contact
support is m(a_site-g); no Core joules are billed for the ground's constraint
force. Negative normal force (requiring attachment/tension) or tangential force
beyond the supplied static friction limit rejects support. The surface adapter
must select an actually supported safe hull footprint, including clearance above
the physical surface; the site radius is the ship origin's radius, not a newly
invented planetary radius. This module does not infer missing mass, spin or
terrain properties from the compressed catalog.

## Maneuvers and energy

`Maneuver::activate` checks cockpit/engine authority and existing-assist exclusion;
takeoff also requires closed doors, construction readiness and a supported start
matching site position/velocity. Landing checks full relative speed (including
spin and tangential motion) and configured range. The default 20 m/s cap remains.
Repeated commands cannot mutate a running maneuver; its sole update advances the
already admitted plan and needs no continuing cockpit authority.

The body-fixed cubic Hermite path preserves initial velocity and ends with zero
body-fixed velocity, with rotating-frame Coriolis/centripetal terms and orbital
acceleration included. The default 6 s / 50 m lift is physically feasible only if
the Core can supply the required force. Landing duration/range are explicit
configuration, not a promise to descend arbitrary planetary distances in 8 s.
Supported durations are positive multiples of 10 ms up to 120 s. Each plan checks
surface-plane clearance and required acceleration minus gravity at 5 ms samples.
The body/gravity providers must supply smooth ephemerides on this resolution;
this is not arbitrary discontinuous input or unmodeled-terrain clearance proof.

Required watts are |a_required-g| / delta_v_per_joule, using ledger-derived mass
and the existing Core impulse coefficient. Available assist thrust is bounded by
the smaller configured forward/reverse power. Simpson quadrature on 10 ms
intervals produces one isolated full-path Core reservation before activation;
no fixed mass-independent fee, kinetic-energy billing, double debit or free
hover is used. Failed provider, clearance, power or funding admission leaves Core
unchanged. Active reservations are non-clonable and inaccessible to ordinary bus
consumers. Dropping a plan does not refund or restore already reserved energy.
Ordinary service loads are still settled by the composition owner.

Cached 10 ms inertial states use Hermite interpolation for intervening times.
Velocity is the path derivative, including nonzero surface/orbital motion at
landing and takeoff handoff. Update results return unused time for ordinary
`Trajectory` gravity flight or surface support; they never drop caller time.
No runtime engine switch or throttle is turned on by a completion observation.

## Source, validation and integration boundary

- Source: `client/ship/src/surface/mod.rs`; regressions: `surface/tests.rs`.
- Run `cargo test -p salimon-ship --locked surface`, then workspace
  build/tests/Clippy/fmt and `python3 scripts/check_docs.py --base <parent-sha>`.
- Tests independently integrate required force, compare Newtonian support on
  Earth/Moon/Mars parameters, exercise mass and impulse scaling, rejection and
  rollback, partitioning, path derivatives and a production #417 coast/landing
  composition. Inputs are declared physical fixtures, not scientific coverage.

These are real portable domain APIs, not a new compressed-world gameplay mode.
The current playable controller/save format/render/input route remains unchanged,
as in #417. Later canonical world/terrain/new-run adoption (#424–#427) consumes
these states and owns native session composition. Moving-body impact is #420;
closed-game event orchestration is #432–#434. No renderer output changes here,
so numerical/domain acceptance does not require graphical captures. Actual
Metal/physical controls/hardware qualification remains #557/#563 and human
presentation remains #561, separate from routine cloud delivery.


## Real-scale atmospheric drag (#419)

See the [atmosphere contract](universe-atmosphere.md) for planet-specific density,
co-rotating gas/wind, atomic online/offline drag integration and physically paid
assist compensation. Model validity, tests and playable-adoption limits are explicit.

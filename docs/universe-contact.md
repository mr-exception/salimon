# Moving-body and terrain contact (#420)

`salimon_ship::contact` owns real-scale continuous contact and body-relative
impact response. `Trajectory::step_with_contacts` composes it with the production
#417 gravity/funded-thrust and #419 atmospheric operators. It intercepts the
motion **before** requesting endpoint gravity, including when the proposed
endpoint lies inside a planet. No compressed catalog or nearest-body cutoff is
used by this path. Existing compressed gameplay remains a separate mode until
canonical streaming/new-run migration (#424–#427).

## Motion, geometry and identities

`CubicPath::between` describes a bounded <=60 s TT/TDB Hermite segment from actual
position/velocity endpoints. The equivalent Bezier controls give convex-hull
spatial bounds and derivative/acceleration bounds. Providers own fitting error
and validity of ephemerides; a cubic segment is not a universal arbitrary-date
orbit approximation. Split/refine curved ephemerides to the scientific error
budget. Do not replace moving centers with frozen snapshots or endpoint chords.

Spheres carry canonical body IDs, physical solid radii and rigid surface spin.
Triangles carry canonical body ID plus nonzero stable collision-mesh primitive
ID, and one motion segment per vertex. That includes actual body translation,
rotation or deformation, rather than assuming a frozen local tile. Surface
velocity is interpolated with closest-point barycentric weights, including edge
and corner contacts. Sphere surface velocity includes spin cross radius.
Sphere primitive ID zero and all other IDs are unique per body; duplicate IDs
reject. Canonical identity order breaks exact event-time ties independently of
input insertion order. Degenerate sampled mesh geometry and stale intervals fail.

The streaming adapter must supply the complete authoritative collision shell for
the queried route and report missing tiles as an error, never as declared empty
space. For a terrain body, use its actual collision shell/guaranteed inner solid,
not a nominal outer sphere that fills negative terrain valleys. Contact does not
weaken the gravity source's interior guard: gravity exclusion radii and mesh must
be consistent with the approved terrain/gravity model. The later streaming owner
supplies those inputs; this module does not silently clamp interior gravity,
invent unseen terrain or alter a scientific body's radius.

## Continuous query

A motion-envelope AABB pass rejects far objects before narrow phase. Distance
search uses two independent clear-interval certificates: endpoint distance with
a relative-speed Lipschitz bound, and tighter interval AABBs with curvature error
bounded by max acceleration * dt²/8. Spheres use actual center-distance bounds,
not their loose outer AABB for this narrow-phase certificate. An instantaneous
common inertial velocity is subtracted before interval bounds, avoiding loose
bounds caused solely by a shared orbital velocity.

Uncertain intervals are bisected and searched earliest first. This covers an
interior crossing with clear endpoints, curved body paths, grazing contacts and
triangle edges/corners. It is not point sampling at a fixed frame rate. Sphere
and triangle queries evaluate actual path geometry at event time. Ordinary
spatial tolerance is 1 mm; with uncapped extreme speeds, a nanosecond epoch step
can exceed that distance. The hit exposes the conservative spatial uncertainty
(including unresolved one-nanosecond width), rather than claiming impossible
millimeter timing precision. Callers must respect their own stricter numerical
error budgets and refuse an insufficiently precise event.

Scene size is bounded to 65,536 colliders and query work to 16,384 distance samples.
Exhaustion returns an explicit refine/split-scene error and publishes no movement,
not a fabricated collision, a clear route or a successful test. Query statistics
retain broad/narrow candidate and distance-sample counts. The Earth-scale tangent
and 1 cm near-miss tests each finish within 200 samples; 1,000 far bodies produce
zero extra narrow-phase candidates in the overlap/tie fixture. These are bounded
cloud query checks, not hardware frame-time or full streaming budgets.

## Atomic integration and impact

The dense drift is exactly the positional part of the #417/#419 integrator at
each possible truncated duration: half fixed-gas drag, initial gravity and
proportional funded impulse. Its certified derivative bounds account for the
rational drag drift, not an unjustified straight chord. The query needs no
endpoint force sample inside a solid. The selected elapsed duration is then
integrated with the real gravity/drag operators; its position must match the hit
witness before response can publish. Any provider, geometry, arithmetic, witness
or work-limit failure leaves position, velocity, epoch and gravity selector intact.
Clear-space results match `step_with_drag` exactly.

At impact, funded thrust is consumed only in proportion to elapsed time. The
returned unused-joule receipt and remaining nanoseconds belong to the composition
owner: do not spend the receipt twice or continue the remaining tick as powered
free flight. The API follows #417's already-funded-thrust boundary and never
performs a second Core debit. `disable_propulsion` is a required control effect
for the caller's engine/throttle/steering owner, not permission for an automatic
restart. The owner dispatches the returned grounded/rebounding/fatal mode instead
of blindly advancing another airborne tick. Grounded motion thereafter uses
surface support; a rebound uses subsequent ordinary gravity/contact steps.

Response uses velocity relative to the moving contact point. Default restitution
0.35, 8 m/s rebound cap and 0.1 m/s minimum retain established ship contact values;
other relative motion is dissipated. The result records body-frame dissipated
mechanical energy. Rebounds receive an explicit two-tolerance separation bias to
avoid repeatedly resolving the same zero-time contact; the original hit witness
is retained. Fatal speed thresholds are supplied explicitly by the health/gameplay
owner; default `None` retains the legacy nonfatal ship-contact policy. This
implements the fatal consequence mechanism without inventing a new damage/balance
threshold. No collision grants a completed safe landing, Core energy, new material
or player immortality. Initial invalid interior states remain recovery/validation
errors under the gravity guard; they are not silently projected out of a planet.

## Ownership and cloud validation

- `client/ship/src/contact/path.rs`: split-coordinate cubic/dense paths and bounds.
- `client/ship/src/contact/geometry.rs`: moving solids, terrain closest points,
  surface velocity and interval distance bounds.
- `client/ship/src/contact/mod.rs`: continuous earliest query, IDs and work guards.
- `client/ship/src/contact/response.rs`: body-relative consequences and policy.
- `client/ship/src/trajectory.rs`: staged gravity/drag/contact integration.
- `client/ship/src/contact/tests.rs`: actual production composition, independent
  crossing times, curved motion, terrain face/edge/corner, production terrain
  samples, orbital-frame invariance, tangent/miss, broad-phase work, response and
  failure rollback. Terrain samples remain declared representative-world fixtures,
  not evidence that future canonical tile streaming already exists.

Run `cargo test -p salimon-ship --locked contact::`, workspace tests/build,
all-target Clippy, fmt and docs/base-impact checks. No rendered output or current
save/UI/input behavior changes. Offline event orchestration and fatal/grounded
save publication remain #432–#434; canonical tile production remains #424/#425.
Physical input/OS #557, human presentation #561 and actual hardware/Metal #563 are
separate, explicitly unverified release qualification, not routine cloud gates.

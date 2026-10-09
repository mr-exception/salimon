# Physics architecture

The crate advances an ordered slice of `ObjectState` values: f64 metre position,
m/s velocity, positive spherical contact radius, positive ground-support distance
and explicit `Surface`. It does not
allocate identities or store session state. Callers validate finite state and
positive finite surface radii, select geometry and retain orientation/material/mass.

`Surface::Sphere` supplies radial gravity and ground geometry in the object's
absolute or relative frame. `Surface::Floor` supplies a +Y floor height in one
shared local frame; a pure containment callback supplies hull/furniture bounds.
Physics owns rejection/bounce and pile correction, not the callback. Unsupported
objects preserve legacy -Y gravity without ground contact. The caller must use
separate calls for different ship-local frames. Sphere centers distinguish
planet contact groups; unsupported and sphere objects retain the legacy shared
non-floor contact group when only one has a body.

`advance` integrates velocity/position, resolves ground/hull contact, performs
three ordered pair passes and damps velocity per substep. Equal-weight spherical
contacts intentionally do not consume mass. Orientation does not spin. The
1/90 s target is capped at 48 substeps, with all elapsed time distributed across
them; this preserves the existing large-delta limitation, not guaranteed CCD.
There is no physics engine, ECS, platform service or presentation dependency.

Release helpers own the existing impulses; the runtime projects ship-forward
components and supplies a stable ejection variant from fragment identity. For
future objects, extend policy only with an explicit feature contract and tests.
See the [canonical decision](../../docs/technical-architecture.md#physical-object-simulation-decision-114).

Ground support is independent of pair-contact radius: callers can supply actual
mesh support along the environmental normal while retaining a conservative
sphere for object contacts and hull containment. Ground contact uses support;
pair separation and floor containment use radius. Runtime refreshes support
from the current frame/normal before every advance; physics owns no mesh data.

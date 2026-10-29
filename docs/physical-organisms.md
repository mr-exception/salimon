# Physical organism placement and clearance (#620)

World `environment::organism_placement` evaluates a version-1 canonical baseline
from the immutable PE07 biosphere, PE08 grammar and authoritative terrain. It uses
fixed ecological sites and four bounded candidates per site; neither camera,
render LOD, quality, device configuration nor player presence selects population.
Supported columns/fronds/colonies and rare giant forms have stable object/site/
part identities and exactly the same specimen endpoints/radii as rendering.
Microbial/sterile/unknown worlds create no physical macro organisms. Cosmetic
cover remains independent and cannot supply colliders, materials or resources.

Root candidates sample real terrain height/normal. Slope >0.4 rejects; every
root must remain within 0.35 m of its supporting surface. Unsupported geometry
rejects instead of creating caves, deforming terrain or floating an organism.
Per-part conservative boxes enclose both endpoints plus maximum taper radius.
Objects overlapping previously admitted physical organisms or immutable named
exclusion boxes reject. Exclusions cover authored openings, hull/ramp/player
routes, fixed resources, pads and authored protected spaces; their IDs/bounds are
publication inputs, never visit-time exclusions around a moving ship/player.
The starting publication includes explicit opening/ramp and resource clearances
and preserves Vesper's sterile status/geography. Other physical populations are
reproducible declared fictional fixtures until later catalog/body adoption.

Publication identity binds placement version, organism grammar version, complete
biosphere/terrain/climate identity and sorted length-delimited exclusion IDs and
bounds. `Population::binding` additionally records exact object/part identities;
`verify_binding` rejects changed source/version/corruption. This is the immutable
baseline binding, not a new mutable harvest save format. Existing Vesper
checkpoints reconstruct the same empty baseline; future #621 part-removal/resource
state must bind this exact source and explicit migrations. Changing an exclusion
or recipe is a versioned all-player publication change, never a silent relocation.

## Common collision owner and live consumers

`salimon-physics::static_boxes` owns validated static boxes, a 4,096-box maximum
and exact continuous slab sweeps for caller-supplied axis-aligned extents. Slabs
use f64 in one explicit caller metre frame; no renderer geometry or domain state
enters the kernel. Initial overlap permits outward retreat, not deeper movement.
Source order resolves contact ties deterministically. Invalid/bounded input
rejects. Existing fragment/convex contact and numerical contracts are unchanged.

Character and ship now consume this shared portable physics kernel through
explicit typed dependencies. Character surface/EVA substeps sweep conservative
full-body capsule boxes and stop at the first source collider, with an f64-scale
separation margin and removal of inward velocity. Ship uses a conservative
circumsphere enclosing the full authored/current installed-hull bounds, independent
of visual LOD. This can reject narrow clearances conservatively; it does not
promise exact convex hull contact. A swept hit preserves the separated pose,
stops velocity/engines/throttle/assist and reports the existing unsafe-site cue;
spent Core energy is not refunded. Physical obstruction is an explicit safety
exception to ordinary uninterrupted assists. Target landing and terrain spawn
also reject blocked full-hull sites before accepting them.

Runtime `session/organisms.rs` installs the same canonical baseline field before
movement, including after restore. Completed construction solids extend the hull
bound. Unsafe initial overlap holds the pose with the unsafe-site cue, preserves
the organism and does not advance/spend a simulation tick; it never removes a
canonical tree or invents a clear landing. Collider residency is independent of
render selection. `app/organisms.rs` renders nearby physical specimens from the
same source position/basis. Current body-fixed Vesper is supported; moving/orbital
pack/body-frame adoption remains #427, not an implicit new ephemeris transform.

## Verification

World tests verify exact reconstruction/part identity, cosmetic-quality independence,
root/box validity, exclusion clearance and source/version binding rejection.
Physics tests cover high-speed crossing, tangent misses, retreat and 1e12-m frame
translations. Character and ship tests exercise live sweeps and landing clearance.
Runtime positive-source integration drives both controllers into the *same*
real generated organism; both stop before its source collider. Unsafe restore
holds preserve pose/energy/source; native Vesper startup/boarding remain intact.

`SALIMON_PHYSICAL_ORGANISMS_FILE=... cargo test -p salimon-client --locked
physical_population -- --include-ignored --nocapture` exports the actual fixture
population, physical part bounds/identities and binding. Pass that file as
`SALIMON_ORGANISMS_FILE` to production renderer `organism_production_reference`
with `SALIMON_ORGANISM_DIR` to capture the same anchored/basis-oriented geometry.
Keep source JSON/binding, original captures and hashes. Run workspace tests,
build/all-target Clippy/fmt, tooling and documentation checks. Actual art review
#561, physical platform #557 and hardware/M1/Metal budgets #563 remain unverified.
No harvesting, mass allocation, breakage, regrowth, fauna or soft-body simulation
is implied; those remain their explicitly owned downstream features.

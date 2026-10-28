# Canonical active geological sources (#617)

World `environment::activity` binds immutable version-1 activity inputs to the
complete [climate/geology publication](planetary-climate.md). The author declares
mechanism, independent seed, bounded candidate count (maximum 64), heat flux,
source radius, period and active duty fraction. Ancient geological history alone
never authorizes active geology: zero declared heat or duty creates no sources.
These are explicit authored physical models, not measured activity on observed
bodies. Existing body definitions receive no automatic activity.

Rock vents/volcanism require a rock profile with suitable local volcanic history;
seismic sources require deformation history; cryovolcanic sources require ice.
Steep unsupported terrain excludes candidates. Placement samples fixed spherical
candidate slots independent of player location, camera, time, tiles and query
order. IDs bind the complete publication plus original slot; excluded candidates
do not renumber survivors. Each source anchor and normal comes from the exact
existing terrain evaluator. A site radius describes an emitter, **not a new solid
collider**, crater, resource, deposit or altered landing surface. Vesper's frozen
homogeneous surface accepts the dormant model and gains no arbitrary volcanism.

`SLMACT01` stores version, seed, inputs and full upstream climate bytes. Strict
restore validates against the known physical body, rejects unsupported versions,
malformed/nonfinite inputs, bad mechanisms and trailing data, then reconstructs
the same sources. `compatible_with` requires exact identity and upstream climate;
revision changes require explicit migration. No mutable event replay list or
source generation based on a loaded region enters saves. Existing runtime saves,
U11 terrain and resource publications remain unchanged.

`sample` and `events_within` take the same uniform TT Epoch as climate. A smooth
periodic envelope yields bounded continuous intensity/thermal-power metadata.
Direct queries work online, after eviction or after an arbitrary offline interval
without stepping a global simulation or allocating another source. The spatial
query returns each matching fixed source once in candidate order. Events are
continuous levels, not commands to replay an eruption each frame or on reload.
Unknown/revised source IDs fail closed.

Plume metadata retains source kind, substrate, local pressure, canonical position,
normal, bounded height and body-fixed velocity. Vacuum supports ballistic material
emission and contact-only acoustic coupling, with **zero atmospheric advection or
airborne sound**. Atmospheric sources consume the same local climate wind and
expose air/contact coupling; seismic events have no plume. Thermal power is a
bounded declared source model, not resource extraction, ship energy credit or a
new environmental damage rule. Render/audio consumers must transform the existing
body-fixed source once, cull/downsample presentation without changing site truth,
and use actual transmission/occlusion rules. Audio mixing remains #627; this
feature exposes source interfaces rather than introducing an unrelated renderer,
particle asset, audio loop, terrain destruction, hazards or deformation.

## Source and validation

- `client/world/src/environment/activity.rs`: inputs, placement, codec, continuous
  event and read-only spatial interfaces.
- `client/world/src/environment/activity/tests.rs`: physical terrain agreement,
  stable IDs and unique spatial results after stream/restore, periodic/offline
  queries, bounds, suitable mechanisms, vacuum versus atmosphere and invalid
  publication/migration failures.
- `cargo test -p salimon-world --locked environment::activity`
- Run affected world/ship tests and workspace build, Clippy, fmt and docs checks
  from [validation](validation.md). No production renderer or collider changes
  occur; graphical capture is not applicable to these metadata interfaces.
  Subjective/hardware qualification remains separately unverified in #561/#563.

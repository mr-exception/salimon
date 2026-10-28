//! Frozen canonical publication adapter. Positions are local to the selected system,
//! never a flattened galaxy coordinate; U02 retains its separate galactic anchor.
use crate::{BodyRole, CelestialBody, CelestialBodyId, WorldPosition};
use salimon_universe::starting_system::{self, FrozenStart};
use std::sync::LazyLock;
pub static START: LazyLock<FrozenStart> =
    LazyLock::new(|| starting_system::permanent().expect("checked permanent publication"));
pub static BODIES: LazyLock<Vec<CelestialBody>> = LazyLock::new(|| {
    vec![CelestialBody::new(
        CelestialBodyId::Vesper,
        "Vesper",
        WorldPosition::new(0., 0., 0.),
        START.manifest().physical.radius_meters,
        [0.22, 0.24, 0.27, 1.],
        BodyRole::Solid,
    )]
});
pub fn body(id: CelestialBodyId) -> Option<&'static CelestialBody> {
    BODIES
        .iter()
        .find(|b| b.id == id)
        .or_else(|| crate::CELESTIAL_BODIES.iter().find(|b| b.id == id))
}

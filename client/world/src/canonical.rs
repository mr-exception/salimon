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

/// Explicit stable mapping from published material identity to local world ID.
/// The full publication binding gates restores; ordering never determines identity.
pub fn outcrop_local_id(material: salimon_universe::starting_system::Material) -> u64 {
    use salimon_universe::starting_system::Material::*;
    match material {
        IronOre => 1,
        SilicateRock => 2,
        WaterIce => 3,
        OrganicBiomass => 4,
        MedicinalCompounds => 5,
    }
}
pub fn deposits(
    position: [f64; 3],
    range: f64,
) -> Result<
    Vec<crate::resource_generation::SurfaceDeposit>,
    crate::resource_generation::GenerationError,
> {
    use crate::{
        resource_generation::{GenerationError, SurfaceDeposit},
        resources::{DepositId, RawMaterial, ResourceDeposit, ResourceId},
    };
    if !position.iter().all(|v| v.is_finite()) || !range.is_finite() || range <= 0. {
        return Err(GenerationError::InvalidArea);
    }
    let radius = START.manifest().physical.radius_meters;
    let mut output = Vec::new();
    for site in &START.manifest().resources.outcrops {
        let physical = START
            .surface_point(site.tangent_xz_meters)
            .map_err(|_| GenerationError::InvalidArea)?;
        // The ordinary placement adapter expects the nominal sphere anchor and
        // applies the same canonical height + authored mesh support exactly once.
        let n = physical[0].hypot(physical[1]).hypot(physical[2]);
        let local = physical.map(|v| v * radius / n);
        let distance = (0..3)
            .map(|i| (physical[i] - position[i]).powi(2))
            .sum::<f64>()
            .sqrt();
        if distance > range {
            continue;
        }
        let resource = match site.material {
            salimon_universe::starting_system::Material::IronOre => ResourceId::IronOre,
            salimon_universe::starting_system::Material::SilicateRock => ResourceId::SilicateRock,
            salimon_universe::starting_system::Material::WaterIce => ResourceId::WaterIce,
            salimon_universe::starting_system::Material::OrganicBiomass => {
                ResourceId::OrganicBiomass
            }
            salimon_universe::starting_system::Material::MedicinalCompounds => {
                ResourceId::MedicinalCompounds
            }
        };
        let material = RawMaterial::new(resource, site.initial_mass_kg)
            .map_err(|_| GenerationError::InvalidMaterial)?;
        output.push(SurfaceDeposit {
            deposit: ResourceDeposit::new(
                DepositId {
                    body: CelestialBodyId::Vesper,
                    local: outcrop_local_id(site.material),
                },
                material,
                WorldPosition::new(local[0], local[1], local[2]),
                site.initial_mass_kg,
            )
            .map_err(|_| GenerationError::InvalidMaterial)?,
            body_local_position_meters: local,
            bounds_radius_meters: crate::resource_size::deposit_radius_meters(material),
        });
    }
    Ok(output)
}

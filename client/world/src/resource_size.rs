//! Shared gameplay geometry policy. Material volume remains mass / density.
use crate::resources::RawMaterial;

/// Deliberate linear enlargement: 10× on each axis, 1,000× bounding volume.
/// Deposit scale; fragments use their separate scale. Neither changes yield.
pub const RESOURCE_LINEAR_SCALE: f64 = 10.0;

pub fn fragment_side_meters(material: RawMaterial) -> f64 {
    2.0 * fragment_contact_radius_meters(material)
}

/// Initial deposit geometry is retained through partial mining; depletion hides it.
pub fn deposit_radius_meters(material: RawMaterial) -> f64 {
    (3.0 * material.volume_m3() / (4.0 * std::f64::consts::PI)).cbrt() * RESOURCE_LINEAR_SCALE
}

/// Exact bounded sphere shared by visual, pickup, carrying and physical support.
pub fn fragment_contact_radius_meters(material: RawMaterial) -> f64 {
    (3.0 * material.volume_m3() / (4.0 * std::f64::consts::PI))
        .cbrt()
        .clamp(
            crate::spherical_mining::MIN_SPHERE_RADIUS_METERS,
            crate::spherical_mining::MAX_SPHERE_RADIUS_METERS,
        )
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::resources::RESOURCE_CATALOG;

    #[test]
    fn geometry_enlarges_every_axis_without_changing_material_volume_or_mass() {
        for resource in RESOURCE_CATALOG {
            for mass in [0.001, 0.032, 1.0, 2.0, 5.0, 80.0] {
                let material = RawMaterial::new(resource.id, mass).unwrap();
                let solid_volume = mass / resource.density_kg_per_m3;
                assert_eq!(material.mass_kg(), mass);
                assert_eq!(material.volume_m3(), solid_volume);
                assert_eq!(
                    fragment_side_meters(material),
                    2.0 * fragment_contact_radius_meters(material)
                );
                let original_radius = (3.0 * solid_volume / (4.0 * std::f64::consts::PI)).cbrt();
                assert!((deposit_radius_meters(material) / original_radius - 10.0).abs() < 1e-12);
                assert!(
                    (fragment_contact_radius_meters(material)
                        - original_radius.clamp(
                            crate::spherical_mining::MIN_SPHERE_RADIUS_METERS,
                            crate::spherical_mining::MAX_SPHERE_RADIUS_METERS
                        ))
                    .abs()
                        < 1e-12
                );
            }
        }
    }
}

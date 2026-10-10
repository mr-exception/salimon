//! Shared gameplay geometry policy. Material volume remains mass / density.
use crate::resources::RawMaterial;

/// Deliberate linear enlargement: 10× on each axis, 1,000× bounding volume.
/// Deposit scale; fragments use their separate scale. Neither changes yield.
pub const RESOURCE_LINEAR_SCALE: f64 = 10.0;

/// Fragment-only 50% linear reduction after the initial resource enlargement.
pub const FRAGMENT_LINEAR_SCALE: f64 = RESOURCE_LINEAR_SCALE * 0.5;

pub fn fragment_side_meters(material: RawMaterial) -> f64 {
    material.volume_m3().cbrt() * FRAGMENT_LINEAR_SCALE
}

/// Initial deposit geometry is retained through partial mining; depletion hides it.
pub fn deposit_radius_meters(material: RawMaterial) -> f64 {
    (3.0 * material.volume_m3() / (4.0 * std::f64::consts::PI)).cbrt() * RESOURCE_LINEAR_SCALE
}

/// Conservative sphere around a world-axis fragment cube, for broad-phase bounds/clearance, never final convex contacts.
pub fn fragment_contact_radius_meters(material: RawMaterial) -> f64 {
    fragment_side_meters(material) * 0.5 * 3.0_f64.sqrt()
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
                assert!(
                    (fragment_side_meters(material) / solid_volume.cbrt() - 5.0).abs() < 1e-12
                );
                let original_radius = (3.0 * solid_volume / (4.0 * std::f64::consts::PI)).cbrt();
                assert!((deposit_radius_meters(material) / original_radius - 10.0).abs() < 1e-12);
                assert!(
                    fragment_contact_radius_meters(material)
                        >= fragment_side_meters(material) * 0.5
                );
            }
        }
    }
}

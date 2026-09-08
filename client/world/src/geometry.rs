//! Spherical body, landing-volume and radial-velocity geometry.

use crate::{BodyRole, CelestialBody, WorldPosition};

/// Height above a solid body's surface at which landing assist becomes available.
pub const LANDING_RANGE_ALTITUDE_RADIUS_FACTOR: f64 = 0.15;

impl CelestialBody {
    /// Outer radius of the landing volume (`1.15R`) for solid bodies.
    #[must_use]
    pub fn landing_outer_radius(self) -> Option<f64> {
        (self.role == BodyRole::Solid)
            .then_some(self.radius_meters * (1.0 + LANDING_RANGE_ALTITUDE_RADIUS_FACTOR))
    }

    /// Nonnegative distance from a point to the nominal spherical surface.
    #[must_use]
    pub fn surface_distance_from(self, position: WorldPosition) -> f64 {
        (self.center.distance_to(position) - self.radius_meters).max(0.0)
    }

    /// Rate at which a point's center distance is changing. Negative values
    /// approach the body, positive values recede, and zero is stationary.
    #[must_use]
    pub fn radial_speed_meters_per_second(
        self,
        position: WorldPosition,
        velocity_meters_per_second: [f64; 3],
    ) -> f64 {
        let offset = position.offset_from(self.center);
        let distance = vector_length(offset);
        if distance <= f64::EPSILON {
            return 0.0;
        }
        offset
            .into_iter()
            .zip(velocity_meters_per_second)
            .map(|(component, velocity)| component * velocity)
            .sum::<f64>()
            / distance
    }

    #[must_use]
    pub fn center_distance_to(self, other: Self) -> f64 {
        self.center.distance_to(other.center)
    }

    /// Signed distance between nominal spherical surfaces: negative overlaps, zero
    /// tangency, and positive separation.
    #[must_use]
    pub fn surface_separation_from(self, other: Self) -> f64 {
        sphere_surface_separation(
            self.center,
            self.radius_meters,
            other.center,
            other.radius_meters,
        )
    }

    /// Signed distance between solid landing volumes. Visual-only bodies have
    /// no landing volume and therefore return `None`.
    #[must_use]
    pub fn landing_volume_separation_from(self, other: Self) -> Option<f64> {
        let own_radius = self.landing_outer_radius()?;
        let other_radius = other.landing_outer_radius()?;
        Some(sphere_surface_separation(
            self.center,
            own_radius,
            other.center,
            other_radius,
        ))
    }
}

fn vector_length(vector: [f64; 3]) -> f64 {
    vector.iter().map(|value| value * value).sum::<f64>().sqrt()
}

/// Signed surface separation for two spheres.
#[must_use]
pub fn sphere_surface_separation(
    first_center: WorldPosition,
    first_radius_meters: f64,
    second_center: WorldPosition,
    second_radius_meters: f64,
) -> f64 {
    first_center.distance_to(second_center) - first_radius_meters - second_radius_meters
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{CELESTIAL_BODIES, CelestialBodyId};

    #[test]
    fn every_pair_of_solid_landing_volumes_is_disjoint() {
        let solids: Vec<_> = CELESTIAL_BODIES
            .iter()
            .copied()
            .filter(|body| body.role == BodyRole::Solid)
            .collect();

        for (index, first) in solids.iter().enumerate() {
            for second in &solids[index + 1..] {
                assert!(
                    first.landing_volume_separation_from(*second).unwrap() > 0.0,
                    "{} and {} landing volumes overlap",
                    first.name,
                    second.name
                );
            }
        }
    }

    #[test]
    fn sphere_surface_separation_distinguishes_overlap_tangency_and_gap() {
        let origin = WorldPosition::new(0.0, 0.0, 0.0);

        assert_eq!(
            sphere_surface_separation(origin, 2.0, WorldPosition::new(3.0, 0.0, 0.0), 2.0),
            -1.0
        );
        assert_eq!(
            sphere_surface_separation(origin, 2.0, WorldPosition::new(4.0, 0.0, 0.0), 2.0),
            0.0
        );
        assert_eq!(
            sphere_surface_separation(origin, 2.0, WorldPosition::new(7.0, 0.0, 0.0), 2.0),
            3.0
        );
    }

    #[test]
    fn radial_speed_distinguishes_approaching_receding_and_zero() {
        let earth = body(CelestialBodyId::Earth);
        let position = earth
            .center
            .translated([earth.radius_meters + 10.0, 0.0, 0.0]);
        assert_eq!(
            earth.radial_speed_meters_per_second(position, [-25.0, 0.0, 0.0]),
            -25.0
        );
        assert_eq!(
            earth.radial_speed_meters_per_second(position, [25.0, 0.0, 0.0]),
            25.0
        );
        assert_eq!(
            earth.radial_speed_meters_per_second(position, [0.0, 25.0, 0.0]),
            0.0
        );
    }

    fn body(id: CelestialBodyId) -> CelestialBody {
        *CELESTIAL_BODIES
            .iter()
            .find(|body| body.id == id)
            .expect("catalog body must exist")
    }
}

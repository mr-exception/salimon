//! Deterministic surface-distance selection for catalog bodies.

use crate::{BodyRole, CELESTIAL_BODIES, CelestialBody, WorldPosition};

/// Shared inclusive surface-distance limit for nearby-body gameplay.
pub const NEARBY_BODY_MAX_SURFACE_DISTANCE_METERS: f64 = 3_000_000.0;

/// Returns the nearest body whose nominal surface is no farther than `maximum`.
/// Catalog order resolves exact ties deterministically.
#[must_use]
pub fn nearest_celestial_body_within_surface_distance(
    bodies: &[CelestialBody],
    position: WorldPosition,
    maximum_distance_meters: f64,
) -> Option<(&CelestialBody, f64)> {
    if !maximum_distance_meters.is_finite() || maximum_distance_meters < 0.0 {
        return None;
    }

    bodies
        .iter()
        .filter_map(|body| {
            let distance = body.surface_distance_from(position);
            (distance <= maximum_distance_meters).then_some((body, distance))
        })
        .min_by(|(_, left), (_, right)| left.total_cmp(right))
}

/// Select solid bodies by surface distance, preserving catalog order on ties.
#[must_use]
pub fn nearby_solid_body(position: WorldPosition) -> Option<(&'static CelestialBody, f64)> {
    CELESTIAL_BODIES
        .iter()
        .filter(|body| body.role == BodyRole::Solid)
        .filter_map(|body| {
            let distance = body.surface_distance_from(position);
            (distance <= NEARBY_BODY_MAX_SURFACE_DISTANCE_METERS).then_some((body, distance))
        })
        .min_by(|(_, a), (_, b)| a.total_cmp(b))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::CelestialBodyId;

    #[test]
    fn nearby_body_selection_uses_surface_distance_and_an_inclusive_threshold() {
        let bodies = [
            CelestialBody::new(
                CelestialBodyId::Earth,
                "First",
                WorldPosition::new(0.0, 0.0, 0.0),
                100.0,
                [1.0; 4],
                BodyRole::Solid,
            ),
            CelestialBody::new(
                CelestialBodyId::Moon,
                "Second",
                WorldPosition::new(1_000.0, 0.0, 0.0),
                100.0,
                [1.0; 4],
                BodyRole::Solid,
            ),
        ];

        let first = nearest_celestial_body_within_surface_distance(
            &bodies,
            WorldPosition::new(400.0, 0.0, 0.0),
            300.0,
        )
        .unwrap();
        assert_eq!(first.0.name, "First");
        assert_eq!(first.1, 300.0);

        let second = nearest_celestial_body_within_surface_distance(
            &bodies,
            WorldPosition::new(600.0, 0.0, 0.0),
            300.0,
        )
        .unwrap();
        assert_eq!(second.0.name, "Second");
        assert_eq!(second.1, 300.0);

        assert!(
            nearest_celestial_body_within_surface_distance(
                &bodies[..1],
                WorldPosition::new(400.001, 0.0, 0.0),
                300.0,
            )
            .is_none()
        );
    }
}

#[cfg(test)]
mod nearby_solid_tests {
    use super::*;
    #[test]
    fn solid_influence_is_inclusive_shared_and_deterministic_for_every_body() {
        for body in CELESTIAL_BODIES
            .iter()
            .filter(|b| b.role == BodyRole::Solid)
        {
            for (offset, expected) in [(-1.0, true), (0.0, true), (1.0, false)] {
                let c = body.center.meters();
                let position = WorldPosition::new(
                    c[0],
                    c[1] + body.radius_meters + NEARBY_BODY_MAX_SURFACE_DISTANCE_METERS + offset,
                    c[2],
                );
                assert_eq!(
                    nearby_solid_body(position).map(|(b, _)| b.id),
                    expected.then_some(body.id)
                );
                assert_eq!(nearby_solid_body(position), nearby_solid_body(position));
            }
        }
        let sun = CELESTIAL_BODIES
            .iter()
            .find(|b| b.role != BodyRole::Solid)
            .unwrap();
        assert_eq!(nearby_solid_body(sun.center), None);
    }
}

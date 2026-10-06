//! Immutable compressed Solar System identities and ordered fixture.

use crate::WorldPosition;

pub const SURFACE_ANCHOR: WorldPosition =
    WorldPosition::new(1_000_000_000_000.0, -750_000_000_000.0, 250_000_000_000.0);
pub(crate) const EARTH_RADIUS_METERS: f64 = 6_000_000.0;
const EARTH_CENTER: WorldPosition = SURFACE_ANCHOR.translated([0.0, 0.0, -EARTH_RADIUS_METERS]);

/// Phase 0's reference maximum travel speed, used to tune compressed distances.
pub const PHASE_ZERO_REFERENCE_MAX_SPEED_METERS_PER_SECOND: f64 = 2_500_000.0;
/// The immutable, ordered Phase 0 Solar System catalog.
pub static CELESTIAL_BODIES: &[CelestialBody] = &[
    CelestialBody::new(
        CelestialBodyId::Sun,
        "Sun",
        EARTH_CENTER.translated([-90_000_000.0, 18_000_000.0, 0.0]),
        15_000_000.0,
        [1.0, 0.63, 0.10, 1.0],
        BodyRole::VisualOnly,
    ),
    CelestialBody::new(
        CelestialBodyId::Mercury,
        "Mercury",
        EARTH_CENTER.translated([-58_000_000.0, -22_000_000.0, 0.0]),
        2_400_000.0,
        [0.52, 0.49, 0.45, 1.0],
        BodyRole::Solid,
    ),
    CelestialBody::new(
        CelestialBodyId::Venus,
        "Venus",
        EARTH_CENTER.translated([-30_000_000.0, 20_000_000.0, 0.0]),
        5_500_000.0,
        [0.91, 0.56, 0.22, 1.0],
        BodyRole::Solid,
    ),
    CelestialBody::new(
        CelestialBodyId::Earth,
        "Earth",
        EARTH_CENTER,
        EARTH_RADIUS_METERS,
        [0.08, 0.32, 0.78, 1.0],
        BodyRole::Solid,
    ),
    CelestialBody::new(
        CelestialBodyId::Moon,
        "Moon",
        EARTH_CENTER.translated([20_000_000.0, -12_000_000.0, 0.0]),
        1_600_000.0,
        [0.68, 0.67, 0.63, 1.0],
        BodyRole::Solid,
    ),
    CelestialBody::new(
        CelestialBodyId::Mars,
        "Mars",
        EARTH_CENTER.translated([69_500_000.0, 0.0, 0.0]),
        3_500_000.0,
        [0.78, 0.24, 0.12, 1.0],
        BodyRole::Solid,
    ),
];
/// Stable identity for one canonical Phase 0 body.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum CelestialBodyId {
    Sun,
    Mercury,
    Venus,
    Earth,
    Moon,
    Mars,
}

/// Whether a body's compressed volume is solid for collision and landing rules.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum BodyRole {
    Solid,
    VisualOnly,
}

/// One immutable body in the compressed client-side Solar System.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct CelestialBody {
    pub id: CelestialBodyId,
    pub name: &'static str,
    pub center: WorldPosition,
    pub radius_meters: f64,
    pub display_color: [f32; 4],
    pub role: BodyRole,
}

impl CelestialBody {
    #[must_use]
    pub const fn new(
        id: CelestialBodyId,
        name: &'static str,
        center: WorldPosition,
        radius_meters: f64,
        display_color: [f32; 4],
        role: BodyRole,
    ) -> Self {
        Self {
            id,
            name,
            center,
            radius_meters,
            display_color,
            role,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::LANDING_RANGE_ALTITUDE_RADIUS_FACTOR;
    use std::collections::HashSet;

    #[test]
    fn catalog_has_exact_membership_order_and_unique_identity() {
        let expected = [
            (CelestialBodyId::Sun, "Sun"),
            (CelestialBodyId::Mercury, "Mercury"),
            (CelestialBodyId::Venus, "Venus"),
            (CelestialBodyId::Earth, "Earth"),
            (CelestialBodyId::Moon, "Moon"),
            (CelestialBodyId::Mars, "Mars"),
        ];
        let actual: Vec<_> = CELESTIAL_BODIES
            .iter()
            .map(|body| (body.id, body.name))
            .collect();
        let ids: HashSet<_> = CELESTIAL_BODIES.iter().map(|body| body.id).collect();
        let names: HashSet<_> = CELESTIAL_BODIES.iter().map(|body| body.name).collect();

        assert_eq!(actual, expected);
        assert_eq!(ids.len(), CELESTIAL_BODIES.len());
        assert_eq!(names.len(), CELESTIAL_BODIES.len());
    }

    #[test]
    fn catalog_values_are_finite_positive_and_visually_distinct() {
        let radii: HashSet<_> = CELESTIAL_BODIES
            .iter()
            .map(|body| body.radius_meters.to_bits())
            .collect();
        let colors: HashSet<_> = CELESTIAL_BODIES
            .iter()
            .map(|body| body.display_color.map(f32::to_bits))
            .collect();
        let centers: HashSet<_> = CELESTIAL_BODIES
            .iter()
            .map(|body| body.center.meters().map(f64::to_bits))
            .collect();

        assert_eq!(radii.len(), CELESTIAL_BODIES.len());
        assert_eq!(colors.len(), CELESTIAL_BODIES.len());
        assert_eq!(centers.len(), CELESTIAL_BODIES.len());
        for body in CELESTIAL_BODIES {
            assert!(body.center.is_finite());
            assert!(body.radius_meters.is_finite() && body.radius_meters > 0.0);
            assert!(!body.name.is_empty());
            assert!(
                body.display_color
                    .iter()
                    .all(|channel| channel.is_finite() && *channel > 0.0 && *channel <= 1.0)
            );
        }
    }

    #[test]
    fn catalog_has_five_solids_and_one_visual_only_sun() {
        let solids = CELESTIAL_BODIES
            .iter()
            .filter(|body| body.role == BodyRole::Solid)
            .count();
        let visual_only: Vec<_> = CELESTIAL_BODIES
            .iter()
            .filter(|body| body.role == BodyRole::VisualOnly)
            .collect();

        assert_eq!(solids, 5);
        assert_eq!(visual_only.len(), 1);
        assert_eq!(visual_only[0].id, CelestialBodyId::Sun);
        assert_eq!(visual_only[0].landing_outer_radius(), None);

        let earth = body(CelestialBodyId::Earth);
        assert_eq!(LANDING_RANGE_ALTITUDE_RADIUS_FACTOR, 0.15);
        assert_eq!(
            earth.landing_outer_radius(),
            Some(earth.radius_meters * 1.15)
        );
    }

    #[test]
    fn earth_surface_is_the_camera_anchor() {
        let earth = body(CelestialBodyId::Earth);

        assert_eq!(earth.radius_meters, EARTH_RADIUS_METERS);
        assert_eq!(
            earth.center.translated([0.0, 0.0, earth.radius_meters]),
            SURFACE_ANCHOR
        );
    }

    #[test]
    fn earth_to_mars_surface_gap_tunes_to_24_seconds() {
        let earth = body(CelestialBodyId::Earth);
        let mars = body(CelestialBodyId::Mars);
        let surface_gap = earth.surface_separation_from(mars);

        assert_eq!(earth.center_distance_to(mars), 69_500_000.0);
        assert_eq!(surface_gap, 60_000_000.0);
        assert_eq!(
            surface_gap / PHASE_ZERO_REFERENCE_MAX_SPEED_METERS_PER_SECOND,
            24.0
        );
    }

    fn body(id: CelestialBodyId) -> CelestialBody {
        *CELESTIAL_BODIES
            .iter()
            .find(|body| body.id == id)
            .expect("catalog body must exist")
    }
}

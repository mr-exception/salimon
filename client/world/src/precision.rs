//! Noncanonical precision fixtures and representative scalar spacing.

use crate::catalog::EARTH_RADIUS_METERS;
use crate::{CameraPrototype, SURFACE_ANCHOR, WorldPosition};

pub(crate) static PRECISION_MARKERS: &[PrecisionMarker] = &[
    PrecisionMarker::new(
        SURFACE_ANCHOR.translated([-2.0, 0.4, 1.25]),
        [0.5, 0.5, 0.5],
        [0.24, 0.91, 0.82, 1.0],
    ),
    PrecisionMarker::new(
        SURFACE_ANCHOR.translated([2.2, -0.6, 2.0]),
        [0.35, 0.75, 0.35],
        [0.88, 0.36, 0.71, 1.0],
    ),
    PrecisionMarker::new(
        SURFACE_ANCHOR.translated([0.0, 1.8, 4.0]),
        [0.25, 0.25, 0.25],
        [0.98, 0.82, 0.24, 1.0],
    ),
];

/// One noncanonical meter-scale cuboid retained from Task 4 for precision QA.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct PrecisionMarker {
    pub absolute_center: WorldPosition,
    pub half_extents_meters: [f32; 3],
    pub color: [f32; 4],
}

impl PrecisionMarker {
    #[must_use]
    pub const fn new(
        absolute_center: WorldPosition,
        half_extents_meters: [f32; 3],
        color: [f32; 4],
    ) -> Self {
        Self {
            absolute_center,
            half_extents_meters,
            color,
        }
    }
}

/// Numeric spacing at the representative coordinates exercised by the tour.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct PrecisionReport {
    pub anchor_f64_ulp_meters: f64,
    pub far_relative_f32_ulp_meters: f64,
    pub near_relative_f32_ulp_meters: f64,
    pub near_earth_proxy_center_f32_ulp_meters: f64,
    pub near_earth_proxy_center_rounding_error_bound_meters: f64,
}

impl PrecisionReport {
    #[must_use]
    pub fn for_prototype() -> Self {
        let largest_anchor_component = SURFACE_ANCHOR
            .meters()
            .into_iter()
            .map(f64::abs)
            .fold(0.0, f64::max);
        let near_earth_proxy_center_distance_meters =
            EARTH_RADIUS_METERS + CameraPrototype::NEAR_ALTITUDE_METERS;
        let near_earth_proxy_center_f32_ulp_meters =
            positive_f32_ulp(near_earth_proxy_center_distance_meters);
        Self {
            anchor_f64_ulp_meters: positive_f64_ulp(largest_anchor_component),
            far_relative_f32_ulp_meters: positive_f32_ulp(CameraPrototype::FAR_ALTITUDE_METERS),
            near_relative_f32_ulp_meters: positive_f32_ulp(CameraPrototype::NEAR_ALTITUDE_METERS),
            near_earth_proxy_center_f32_ulp_meters,
            near_earth_proxy_center_rounding_error_bound_meters:
                near_earth_proxy_center_f32_ulp_meters * 0.5,
        }
    }
}

fn positive_f64_ulp(value: f64) -> f64 {
    debug_assert!(value.is_finite() && value >= 0.0);
    f64::from_bits(value.to_bits() + 1) - value
}

fn positive_f32_ulp(value: f64) -> f64 {
    let rounded = value as f32;
    debug_assert!(rounded.is_finite() && rounded >= 0.0);
    f64::from(f32::from_bits(rounded.to_bits() + 1) - rounded)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{CELESTIAL_BODIES, CelestialBody, CelestialBodyId};

    #[test]
    fn precision_report_matches_the_representative_float_spacings() {
        let report = PrecisionReport::for_prototype();

        assert_close(report.anchor_f64_ulp_meters, 0.000_122_070_312_5, 1.0e-15);
        assert_close(report.far_relative_f32_ulp_meters, 8.0, TEST_EPSILON);
        assert_close(
            report.near_relative_f32_ulp_meters,
            0.000_000_953_674_316_406_25,
            1.0e-18,
        );
        assert_close(
            report.near_earth_proxy_center_f32_ulp_meters,
            0.5,
            TEST_EPSILON,
        );
        assert_close(
            report.near_earth_proxy_center_rounding_error_bound_meters,
            0.25,
            TEST_EPSILON,
        );
        assert!(report.anchor_f64_ulp_meters < 0.001);
        assert!(report.near_relative_f32_ulp_meters < 0.000_01);
        assert!(report.far_relative_f32_ulp_meters <= 8.0);
    }

    #[test]
    fn earth_proxy_near_face_exposes_its_reported_center_rounding_limit() {
        let camera = SURFACE_ANCHOR.translated([0.0, 0.0, 12.3]);
        let earth = body(CelestialBodyId::Earth);
        let expected_face_z = earth.center.offset_from(camera)[2] + earth.radius_meters;
        let gpu_face_z =
            f64::from(earth.center.camera_relative_f32(camera)[2] + earth.radius_meters as f32);
        let error = (gpu_face_z - expected_face_z).abs();
        let report = PrecisionReport::for_prototype();

        assert!(error > report.near_relative_f32_ulp_meters);
        assert!(error <= report.near_earth_proxy_center_rounding_error_bound_meters);
    }

    #[test]
    fn near_camera_rebase_stays_within_the_reported_local_budget() {
        let mut prototype = CameraPrototype::default();
        prototype.advance(CameraPrototype::APPROACH_DURATION);
        let snapshot = prototype.snapshot();
        let target_relative = snapshot
            .camera
            .target
            .camera_relative_f32(snapshot.camera.position);

        assert_close(f64::from(target_relative[0]), 0.0, TEST_EPSILON);
        assert_close(f64::from(target_relative[1]), 0.0, TEST_EPSILON);
        assert_close(
            f64::from(target_relative[2]),
            -CameraPrototype::NEAR_ALTITUDE_METERS,
            snapshot.precision.near_relative_f32_ulp_meters,
        );
    }

    const TEST_EPSILON: f64 = 1.0e-9;

    fn assert_close(actual: f64, expected: f64, tolerance: f64) {
        assert!(
            (actual - expected).abs() <= tolerance,
            "expected {actual:.16e} to be within {tolerance:.3e} of {expected:.16e}"
        );
    }

    fn body(id: CelestialBodyId) -> CelestialBody {
        *CELESTIAL_BODIES
            .iter()
            .find(|body| body.id == id)
            .expect("catalog body must exist")
    }
}

//! Portable engineering camera state and deterministic tour sampling.

use std::time::Duration;

use crate::precision::PRECISION_MARKERS;
use crate::{
    CELESTIAL_BODIES, CelestialBody, CelestialBodyId, PrecisionMarker, PrecisionReport,
    WorldPosition,
};

const CAMERA_UP: [f64; 3] = [0.0, 1.0, 0.0];

/// Camera values expressed independently of a graphics API's clip convention.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct CameraSnapshot {
    pub position: WorldPosition,
    pub target: WorldPosition,
    pub up: [f64; 3],
    pub vertical_field_of_view_radians: f64,
    pub physical_near_plane_meters: f64,
    pub altitude_meters: f64,
}

/// Current segment of the repeating validation-camera tour.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum TransitionPhase {
    Approach,
    NearDwell,
    Retreat,
    FarDwell,
}

impl TransitionPhase {
    #[must_use]
    pub const fn is_moving(self) -> bool {
        matches!(self, Self::Approach | Self::Retreat)
    }
}

/// User intent accepted by the portable prototype.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum CameraCommand {
    TogglePause,
    Restart,
    JumpToNear,
    /// Restart the validation approach at a body's positive-Z surface.
    InspectBody(CelestialBodyId),
}

/// Renderer-neutral, borrowed view of the current continuous world scene.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct WorldSnapshot<'a> {
    pub camera: CameraSnapshot,
    pub celestial_bodies: &'a [CelestialBody],
    pub precision_markers: &'a [PrecisionMarker],
    pub transition_phase: TransitionPhase,
    /// Fraction through the current phase, inclusive of zero and below one
    /// until the next phase begins.
    pub phase_progress: f64,
    pub paused: bool,
    pub precision: PrecisionReport,
}

/// Deterministic camera tour between far-space and near-surface validation views.
#[derive(Debug)]
pub struct CameraPrototype {
    elapsed_in_cycle: Duration,
    paused: bool,
    inspected_body: CelestialBodyId,
}

impl Default for CameraPrototype {
    fn default() -> Self {
        Self {
            elapsed_in_cycle: Duration::ZERO,
            paused: false,
            inspected_body: CelestialBodyId::Earth,
        }
    }
}

impl CameraPrototype {
    pub const FAR_ALTITUDE_METERS: f64 = 120_000_000.0;
    pub const NEAR_ALTITUDE_METERS: f64 = 12.0;
    pub const PHYSICAL_NEAR_PLANE_METERS: f64 = 0.05;
    pub const VERTICAL_FIELD_OF_VIEW_RADIANS: f64 = std::f64::consts::FRAC_PI_3;
    pub const APPROACH_DURATION: Duration = Duration::from_secs(8);
    pub const NEAR_DWELL_DURATION: Duration = Duration::from_secs(2);
    pub const RETREAT_DURATION: Duration = Duration::from_secs(8);
    pub const FAR_DWELL_DURATION: Duration = Duration::from_secs(2);
    pub const CYCLE_DURATION: Duration = Duration::from_secs(20);

    /// Advances the tour. Large deltas wrap deterministically within the cycle.
    pub fn advance(&mut self, delta: Duration) {
        if self.paused || delta.is_zero() {
            return;
        }

        let elapsed_nanos = self.elapsed_in_cycle.as_nanos() + delta.as_nanos();
        let remainder = elapsed_nanos % Self::CYCLE_DURATION.as_nanos();
        self.elapsed_in_cycle = Duration::from_nanos(remainder as u64);
    }

    /// Applies a portable camera command. Restart resumes the tour, while a
    /// near jump selects the exact near-dwell endpoint and pauses there.
    pub fn apply_command(&mut self, command: CameraCommand) {
        match command {
            CameraCommand::TogglePause => self.paused = !self.paused,
            CameraCommand::Restart => {
                self.elapsed_in_cycle = Duration::ZERO;
                self.paused = false;
            }
            CameraCommand::JumpToNear => {
                self.elapsed_in_cycle = Self::APPROACH_DURATION;
                self.paused = true;
            }
            CameraCommand::InspectBody(body) => {
                self.inspected_body = body;
                self.elapsed_in_cycle = Duration::ZERO;
                self.paused = false;
            }
        }
    }

    #[must_use]
    pub const fn is_paused(&self) -> bool {
        self.paused
    }

    #[must_use]
    pub const fn elapsed_in_cycle(&self) -> Duration {
        self.elapsed_in_cycle
    }

    #[must_use]
    pub fn snapshot(&self) -> WorldSnapshot<'static> {
        let timeline = sample_timeline(self.elapsed_in_cycle);
        let body = CELESTIAL_BODIES
            .iter()
            .find(|body| body.id == self.inspected_body)
            .expect("all inspection identities are catalog members");
        let surface_anchor = body.center.translated([0.0, 0.0, body.radius_meters]);
        let camera = CameraSnapshot {
            position: surface_anchor.translated([0.0, 0.0, timeline.altitude_meters]),
            target: surface_anchor,
            up: CAMERA_UP,
            vertical_field_of_view_radians: Self::VERTICAL_FIELD_OF_VIEW_RADIANS,
            physical_near_plane_meters: Self::PHYSICAL_NEAR_PLANE_METERS,
            altitude_meters: timeline.altitude_meters,
        };

        WorldSnapshot {
            camera,
            celestial_bodies: CELESTIAL_BODIES,
            precision_markers: PRECISION_MARKERS,
            transition_phase: timeline.phase,
            phase_progress: timeline.phase_progress,
            paused: self.paused,
            precision: PrecisionReport::for_prototype(),
        }
    }
}

#[derive(Clone, Copy, Debug)]
struct TimelineSample {
    phase: TransitionPhase,
    phase_progress: f64,
    altitude_meters: f64,
}

fn sample_timeline(elapsed: Duration) -> TimelineSample {
    let elapsed_seconds = elapsed.as_secs_f64();
    let approach_end = CameraPrototype::APPROACH_DURATION.as_secs_f64();
    let near_dwell_end = approach_end + CameraPrototype::NEAR_DWELL_DURATION.as_secs_f64();
    let retreat_end = near_dwell_end + CameraPrototype::RETREAT_DURATION.as_secs_f64();

    if elapsed_seconds < approach_end {
        let progress = elapsed_seconds / approach_end;
        TimelineSample {
            phase: TransitionPhase::Approach,
            phase_progress: progress,
            altitude_meters: logarithmic_transition(
                CameraPrototype::FAR_ALTITUDE_METERS,
                CameraPrototype::NEAR_ALTITUDE_METERS,
                progress,
            ),
        }
    } else if elapsed_seconds < near_dwell_end {
        TimelineSample {
            phase: TransitionPhase::NearDwell,
            phase_progress: (elapsed_seconds - approach_end)
                / CameraPrototype::NEAR_DWELL_DURATION.as_secs_f64(),
            altitude_meters: CameraPrototype::NEAR_ALTITUDE_METERS,
        }
    } else if elapsed_seconds < retreat_end {
        let progress =
            (elapsed_seconds - near_dwell_end) / CameraPrototype::RETREAT_DURATION.as_secs_f64();
        TimelineSample {
            phase: TransitionPhase::Retreat,
            phase_progress: progress,
            altitude_meters: logarithmic_transition(
                CameraPrototype::NEAR_ALTITUDE_METERS,
                CameraPrototype::FAR_ALTITUDE_METERS,
                progress,
            ),
        }
    } else {
        TimelineSample {
            phase: TransitionPhase::FarDwell,
            phase_progress: (elapsed_seconds - retreat_end)
                / CameraPrototype::FAR_DWELL_DURATION.as_secs_f64(),
            altitude_meters: CameraPrototype::FAR_ALTITUDE_METERS,
        }
    }
}

fn logarithmic_transition(start: f64, end: f64, progress: f64) -> f64 {
    if progress <= 0.0 {
        return start;
    }
    if progress >= 1.0 {
        return end;
    }

    let eased = progress * progress * (3.0 - 2.0 * progress);
    (start.ln() + (end.ln() - start.ln()) * eased).exp()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn inspection_tours_reach_every_catalog_surface_and_keep_the_scene_immutable() {
        let mut prototype = CameraPrototype::default();
        for body in CELESTIAL_BODIES {
            prototype.apply_command(CameraCommand::InspectBody(body.id));
            let far = prototype.snapshot();
            assert!(!far.paused);
            assert_eq!(
                body.surface_distance_from(far.camera.position),
                CameraPrototype::FAR_ALTITUDE_METERS
            );
            assert_eq!(
                far.camera.target,
                body.center.translated([0.0, 0.0, body.radius_meters])
            );
            prototype.advance(CameraPrototype::APPROACH_DURATION);
            let near = prototype.snapshot();
            assert_eq!(
                body.surface_distance_from(near.camera.position),
                CameraPrototype::NEAR_ALTITUDE_METERS
            );
            assert_eq!(near.celestial_bodies, CELESTIAL_BODIES);
            prototype.apply_command(CameraCommand::JumpToNear);
            assert!(prototype.is_paused());
            prototype.apply_command(CameraCommand::Restart);
            assert_eq!(prototype.snapshot().camera, far.camera);
        }
    }

    #[test]
    fn default_snapshot_starts_at_the_far_approach_endpoint() {
        let snapshot = CameraPrototype::default().snapshot();

        assert_eq!(snapshot.transition_phase, TransitionPhase::Approach);
        assert_close(snapshot.phase_progress, 0.0, TEST_EPSILON);
        assert_close(
            snapshot.camera.altitude_meters,
            CameraPrototype::FAR_ALTITUDE_METERS,
            TEST_EPSILON,
        );
        assert_close(
            snapshot.camera.physical_near_plane_meters,
            CameraPrototype::PHYSICAL_NEAR_PLANE_METERS,
            TEST_EPSILON,
        );
        assert_eq!(snapshot.celestial_bodies.len(), 6);
        assert_eq!(snapshot.precision_markers.len(), 3);
        assert!(!snapshot.paused);
    }

    #[test]
    fn approach_is_monotonic_and_reaches_the_near_dwell() {
        let mut prototype = CameraPrototype::default();
        let mut previous_altitude = prototype.snapshot().camera.altitude_meters;

        for _ in 0..8 {
            prototype.advance(Duration::from_secs(1));
            let altitude = prototype.snapshot().camera.altitude_meters;
            assert!(altitude < previous_altitude);
            previous_altitude = altitude;
        }

        let snapshot = prototype.snapshot();
        assert_eq!(snapshot.transition_phase, TransitionPhase::NearDwell);
        assert_close(
            snapshot.camera.altitude_meters,
            CameraPrototype::NEAR_ALTITUDE_METERS,
            TEST_EPSILON,
        );
    }

    #[test]
    fn logarithmic_midpoint_is_the_geometric_mean() {
        let mut prototype = CameraPrototype::default();
        prototype.advance(CameraPrototype::APPROACH_DURATION / 2);

        let expected =
            (CameraPrototype::FAR_ALTITUDE_METERS * CameraPrototype::NEAR_ALTITUDE_METERS).sqrt();
        assert_close(
            prototype.snapshot().camera.altitude_meters,
            expected,
            expected * 1.0e-12,
        );
    }

    #[test]
    fn near_and_far_dwells_hold_their_endpoints() {
        let mut prototype = CameraPrototype::default();
        prototype.advance(CameraPrototype::APPROACH_DURATION);
        prototype.advance(CameraPrototype::NEAR_DWELL_DURATION / 2);
        let near = prototype.snapshot();
        assert_eq!(near.transition_phase, TransitionPhase::NearDwell);
        assert_close(
            near.camera.altitude_meters,
            CameraPrototype::NEAR_ALTITUDE_METERS,
            TEST_EPSILON,
        );

        prototype.advance(CameraPrototype::NEAR_DWELL_DURATION / 2);
        prototype.advance(CameraPrototype::RETREAT_DURATION);
        prototype.advance(CameraPrototype::FAR_DWELL_DURATION / 2);
        let far = prototype.snapshot();
        assert_eq!(far.transition_phase, TransitionPhase::FarDwell);
        assert_close(
            far.camera.altitude_meters,
            CameraPrototype::FAR_ALTITUDE_METERS,
            TEST_EPSILON,
        );
    }

    #[test]
    fn retreat_is_monotonic_and_cycle_wraps_to_the_far_endpoint() {
        let mut prototype = CameraPrototype::default();
        prototype
            .advance(CameraPrototype::APPROACH_DURATION + CameraPrototype::NEAR_DWELL_DURATION);
        let mut previous_altitude = prototype.snapshot().camera.altitude_meters;

        for _ in 0..8 {
            prototype.advance(Duration::from_secs(1));
            let altitude = prototype.snapshot().camera.altitude_meters;
            assert!(altitude > previous_altitude);
            previous_altitude = altitude;
        }

        assert_eq!(
            prototype.snapshot().transition_phase,
            TransitionPhase::FarDwell
        );
        prototype.advance(CameraPrototype::FAR_DWELL_DURATION);
        let wrapped = prototype.snapshot();
        assert_eq!(wrapped.transition_phase, TransitionPhase::Approach);
        assert_close(wrapped.phase_progress, 0.0, TEST_EPSILON);
        assert_eq!(prototype.elapsed_in_cycle(), Duration::ZERO);
    }

    #[test]
    fn a_large_delta_wraps_to_the_same_deterministic_sample() {
        let mut direct = CameraPrototype::default();
        direct.advance(Duration::from_millis(3_250));

        let mut wrapped = CameraPrototype::default();
        wrapped.advance(CameraPrototype::CYCLE_DURATION * 1_000 + Duration::from_millis(3_250));

        assert_eq!(wrapped.elapsed_in_cycle(), direct.elapsed_in_cycle());
        assert_eq!(wrapped.snapshot(), direct.snapshot());
    }

    #[test]
    fn pause_freezes_the_timeline_and_toggle_resumes_it() {
        let mut prototype = CameraPrototype::default();
        prototype.advance(Duration::from_secs(1));
        prototype.apply_command(CameraCommand::TogglePause);
        let paused_at = prototype.snapshot();

        prototype.advance(Duration::from_secs(30));
        assert!(prototype.is_paused());
        assert_eq!(prototype.snapshot(), paused_at);

        prototype.apply_command(CameraCommand::TogglePause);
        prototype.advance(Duration::from_secs(1));
        assert!(!prototype.is_paused());
        assert_ne!(prototype.snapshot(), paused_at);
    }

    #[test]
    fn restart_returns_to_the_far_endpoint_and_resumes() {
        let mut prototype = CameraPrototype::default();
        prototype.advance(Duration::from_secs(7));
        prototype.apply_command(CameraCommand::TogglePause);
        prototype.apply_command(CameraCommand::Restart);

        let snapshot = prototype.snapshot();
        assert_eq!(prototype.elapsed_in_cycle(), Duration::ZERO);
        assert!(!prototype.is_paused());
        assert_eq!(snapshot.transition_phase, TransitionPhase::Approach);
        assert_close(
            snapshot.camera.altitude_meters,
            CameraPrototype::FAR_ALTITUDE_METERS,
            TEST_EPSILON,
        );
    }

    #[test]
    fn jump_to_near_selects_the_exact_near_dwell_endpoint_and_pauses() {
        let mut prototype = CameraPrototype::default();
        prototype.advance(Duration::from_secs(3));
        prototype.apply_command(CameraCommand::JumpToNear);

        let snapshot = prototype.snapshot();
        assert_eq!(
            prototype.elapsed_in_cycle(),
            CameraPrototype::APPROACH_DURATION
        );
        assert!(prototype.is_paused());
        assert_eq!(snapshot.transition_phase, TransitionPhase::NearDwell);
        assert_close(snapshot.phase_progress, 0.0, TEST_EPSILON);
        assert_close(
            snapshot.camera.altitude_meters,
            CameraPrototype::NEAR_ALTITUDE_METERS,
            TEST_EPSILON,
        );
        assert!(snapshot.paused);

        prototype.advance(Duration::from_secs(30));
        assert_eq!(prototype.snapshot(), snapshot);
    }

    #[test]
    fn snapshots_remain_finite_through_an_entire_cycle() {
        let mut prototype = CameraPrototype::default();
        let steps = CameraPrototype::CYCLE_DURATION.as_millis() / 17;

        for _ in 0..=steps {
            let snapshot = prototype.snapshot();
            assert!(snapshot.camera.position.is_finite());
            assert!(snapshot.camera.target.is_finite());
            assert!(
                snapshot
                    .camera
                    .up
                    .iter()
                    .all(|component| component.is_finite())
            );
            assert!(snapshot.camera.vertical_field_of_view_radians.is_finite());
            assert!(snapshot.camera.physical_near_plane_meters.is_finite());
            assert!(snapshot.camera.altitude_meters.is_finite());
            assert!((0.0..1.0).contains(&snapshot.phase_progress));

            for body in snapshot.celestial_bodies {
                assert!(body.center.is_finite());
                assert!(body.radius_meters.is_finite() && body.radius_meters > 0.0);
                assert!(
                    body.display_color
                        .iter()
                        .all(|channel| channel.is_finite() && (0.0..=1.0).contains(channel))
                );
                assert!(
                    body.center
                        .camera_relative_f32(snapshot.camera.position)
                        .iter()
                        .all(|component| component.is_finite())
                );
            }

            for marker in snapshot.precision_markers {
                assert!(marker.absolute_center.is_finite());
                assert!(
                    marker
                        .half_extents_meters
                        .iter()
                        .all(|extent| extent.is_finite() && *extent > 0.0)
                );
                assert!(
                    marker
                        .color
                        .iter()
                        .all(|channel| channel.is_finite() && (0.0..=1.0).contains(channel))
                );
                assert!(
                    marker
                        .absolute_center
                        .camera_relative_f32(snapshot.camera.position)
                        .iter()
                        .all(|component| component.is_finite())
                );
            }

            prototype.advance(Duration::from_millis(17));
        }
    }

    #[test]
    fn snapshots_borrow_stable_immutable_scene_sets() {
        let mut prototype = CameraPrototype::default();
        let first_bodies = prototype.snapshot().celestial_bodies;
        let first_markers = prototype.snapshot().precision_markers;
        prototype.advance(Duration::from_secs(11));
        let second_bodies = prototype.snapshot().celestial_bodies;
        let second_markers = prototype.snapshot().precision_markers;

        assert!(std::ptr::eq(first_bodies.as_ptr(), second_bodies.as_ptr()));
        assert!(std::ptr::eq(
            first_markers.as_ptr(),
            second_markers.as_ptr()
        ));
        assert_eq!(first_bodies, second_bodies);
        assert_eq!(first_markers, second_markers);
    }

    #[test]
    fn near_camera_is_twelve_meters_above_earth_surface() {
        let mut prototype = CameraPrototype::default();
        prototype.apply_command(CameraCommand::JumpToNear);
        let snapshot = prototype.snapshot();
        let earth = body(CelestialBodyId::Earth);

        assert_eq!(
            earth.surface_distance_from(snapshot.camera.position),
            CameraPrototype::NEAR_ALTITUDE_METERS
        );
    }

    #[test]
    fn transition_phase_reports_whether_the_camera_is_moving() {
        assert!(TransitionPhase::Approach.is_moving());
        assert!(TransitionPhase::Retreat.is_moving());
        assert!(!TransitionPhase::NearDwell.is_moving());
        assert!(!TransitionPhase::FarDwell.is_moving());
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

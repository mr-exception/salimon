//! Portable large-scale coordinate and camera-transition prototype.
//!
//! Authoritative positions remain `f64` meters. Presentation code must subtract
//! the camera position in `f64` before converting the small relative result to
//! `f32`; [`WorldPosition::camera_relative_f32`] is the canonical conversion.
//! This crate deliberately has no windowing or GPU dependency.

use std::time::Duration;

const SURFACE_ANCHOR: WorldPosition =
    WorldPosition::new(1_000_000_000_000.0, -750_000_000_000.0, 250_000_000_000.0);
const CAMERA_UP: [f64; 3] = [0.0, 1.0, 0.0];
const PROTOTYPE_PRIMITIVES: [PrototypePrimitive; 6] = [
    PrototypePrimitive::new(
        SURFACE_ANCHOR.translated([0.0, 0.0, -6_000_000.0]),
        [6_000_000.0, 6_000_000.0, 6_000_000.0],
        [0.08, 0.22, 0.42, 1.0],
    ),
    PrototypePrimitive::new(
        SURFACE_ANCHOR.translated([35_000_000.0, 10_000_000.0, 25_000_000.0]),
        [2_000_000.0, 2_000_000.0, 2_000_000.0],
        [0.96, 0.58, 0.20, 1.0],
    ),
    PrototypePrimitive::new(
        SURFACE_ANCHOR.translated([-50_000.0, 2_000.0, 20_000.0]),
        [500.0, 300.0, 500.0],
        [0.34, 0.82, 0.93, 1.0],
    ),
    PrototypePrimitive::new(
        SURFACE_ANCHOR.translated([-2.0, 0.4, 1.25]),
        [0.5, 0.5, 0.5],
        [0.24, 0.91, 0.82, 1.0],
    ),
    PrototypePrimitive::new(
        SURFACE_ANCHOR.translated([2.2, -0.6, 2.0]),
        [0.35, 0.75, 0.35],
        [0.88, 0.36, 0.71, 1.0],
    ),
    PrototypePrimitive::new(
        SURFACE_ANCHOR.translated([0.0, 1.8, 4.0]),
        [0.25, 0.25, 0.25],
        [0.98, 0.82, 0.24, 1.0],
    ),
];

/// A canonical world-space position in meters.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct WorldPosition {
    meters: [f64; 3],
}

impl WorldPosition {
    #[must_use]
    pub const fn new(x_meters: f64, y_meters: f64, z_meters: f64) -> Self {
        Self {
            meters: [x_meters, y_meters, z_meters],
        }
    }

    #[must_use]
    pub const fn meters(self) -> [f64; 3] {
        self.meters
    }

    #[must_use]
    pub const fn translated(self, offset_meters: [f64; 3]) -> Self {
        Self::new(
            self.meters[0] + offset_meters[0],
            self.meters[1] + offset_meters[1],
            self.meters[2] + offset_meters[2],
        )
    }

    /// Subtracts an origin while both values still have double precision.
    #[must_use]
    pub fn offset_from(self, origin: Self) -> [f64; 3] {
        [
            self.meters[0] - origin.meters[0],
            self.meters[1] - origin.meters[1],
            self.meters[2] - origin.meters[2],
        ]
    }

    /// Produces GPU-friendly meters relative to the current camera origin.
    ///
    /// The order is intentional: subtracting global positions after converting
    /// them to `f32` destroys meter-scale offsets at this prototype's anchor.
    #[must_use]
    pub fn camera_relative_f32(self, camera_position: Self) -> [f32; 3] {
        let relative = self.offset_from(camera_position);
        [relative[0] as f32, relative[1] as f32, relative[2] as f32]
    }

    #[must_use]
    pub fn is_finite(self) -> bool {
        self.meters.iter().all(|component| component.is_finite())
    }
}

/// One immutable cuboid used to expose scale and depth failures visually.
///
/// Centers are absolute `f64` world positions. Half extents and colors are
/// presentation attributes; the borrowed prototype slice is never authoritative
/// simulation state.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct PrototypePrimitive {
    pub absolute_center: WorldPosition,
    pub half_extents_meters: [f32; 3],
    pub color: [f32; 4],
}

impl PrototypePrimitive {
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
}

/// Numeric spacing at the representative coordinates exercised by the tour.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct PrecisionReport {
    pub anchor_f64_ulp_meters: f64,
    pub far_relative_f32_ulp_meters: f64,
    pub near_relative_f32_ulp_meters: f64,
    pub near_giant_proxy_center_f32_ulp_meters: f64,
    pub near_giant_proxy_center_rounding_error_bound_meters: f64,
}

impl PrecisionReport {
    #[must_use]
    pub fn for_prototype() -> Self {
        let largest_anchor_component = SURFACE_ANCHOR
            .meters()
            .into_iter()
            .map(f64::abs)
            .fold(0.0, f64::max);
        let near_giant_proxy_center_distance_meters =
            f64::from(PROTOTYPE_PRIMITIVES[0].half_extents_meters[2])
                + CameraPrototype::NEAR_ALTITUDE_METERS;
        let near_giant_proxy_center_f32_ulp_meters =
            positive_f32_ulp(near_giant_proxy_center_distance_meters);
        Self {
            anchor_f64_ulp_meters: positive_f64_ulp(largest_anchor_component),
            far_relative_f32_ulp_meters: positive_f32_ulp(CameraPrototype::FAR_ALTITUDE_METERS),
            near_relative_f32_ulp_meters: positive_f32_ulp(CameraPrototype::NEAR_ALTITUDE_METERS),
            near_giant_proxy_center_f32_ulp_meters,
            near_giant_proxy_center_rounding_error_bound_meters:
                near_giant_proxy_center_f32_ulp_meters * 0.5,
        }
    }
}

/// Renderer-neutral, borrowed view of the current validation scene.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct PrototypeSnapshot<'a> {
    pub camera: CameraSnapshot,
    pub primitives: &'a [PrototypePrimitive],
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
}

impl Default for CameraPrototype {
    fn default() -> Self {
        Self {
            elapsed_in_cycle: Duration::ZERO,
            paused: false,
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
    pub fn snapshot(&self) -> PrototypeSnapshot<'static> {
        let timeline = sample_timeline(self.elapsed_in_cycle);
        let camera = CameraSnapshot {
            position: SURFACE_ANCHOR.translated([0.0, 0.0, timeline.altitude_meters]),
            target: SURFACE_ANCHOR,
            up: CAMERA_UP,
            vertical_field_of_view_radians: Self::VERTICAL_FIELD_OF_VIEW_RADIANS,
            physical_near_plane_meters: Self::PHYSICAL_NEAR_PLANE_METERS,
            altitude_meters: timeline.altitude_meters,
        };

        PrototypeSnapshot {
            camera,
            primitives: &PROTOTYPE_PRIMITIVES,
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
    use super::{
        CameraCommand, CameraPrototype, PROTOTYPE_PRIMITIVES, PrecisionReport, SURFACE_ANCHOR,
        TransitionPhase, WorldPosition,
    };
    use std::time::Duration;

    const TEST_EPSILON: f64 = 1.0e-9;

    #[test]
    fn subtracting_in_f64_preserves_local_offsets_at_the_large_anchor() {
        let origin = WorldPosition::new(1_000_000_000_000.0, 0.0, 0.0);
        let nearby = origin.translated([1.0, 0.125, -4.0]);

        assert_eq!(nearby.camera_relative_f32(origin), [1.0, 0.125, -4.0]);

        let cast_then_subtract = nearby.meters()[0] as f32 - origin.meters()[0] as f32;
        assert_eq!(cast_then_subtract, 0.0);
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
        assert_eq!(snapshot.primitives.len(), 6);
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

            for primitive in snapshot.primitives {
                assert!(primitive.absolute_center.is_finite());
                assert!(
                    primitive
                        .half_extents_meters
                        .iter()
                        .all(|extent| extent.is_finite() && *extent > 0.0)
                );
                assert!(
                    primitive
                        .color
                        .iter()
                        .all(|channel| channel.is_finite() && (0.0..=1.0).contains(channel))
                );
                assert!(
                    primitive
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
            report.near_giant_proxy_center_f32_ulp_meters,
            0.5,
            TEST_EPSILON,
        );
        assert_close(
            report.near_giant_proxy_center_rounding_error_bound_meters,
            0.25,
            TEST_EPSILON,
        );
        assert!(report.anchor_f64_ulp_meters < 0.001);
        assert!(report.near_relative_f32_ulp_meters < 0.000_01);
        assert!(report.far_relative_f32_ulp_meters <= 8.0);
    }

    #[test]
    fn giant_proxy_near_face_exposes_its_reported_center_rounding_limit() {
        let camera = SURFACE_ANCHOR.translated([0.0, 0.0, 12.3]);
        let proxy = PROTOTYPE_PRIMITIVES[0];
        let expected_face_z =
            proxy.absolute_center.offset_from(camera)[2] + f64::from(proxy.half_extents_meters[2]);
        let gpu_face_z = f64::from(
            proxy.absolute_center.camera_relative_f32(camera)[2] + proxy.half_extents_meters[2],
        );
        let error = (gpu_face_z - expected_face_z).abs();
        let report = PrecisionReport::for_prototype();

        assert!(error > report.near_relative_f32_ulp_meters);
        assert!(error <= report.near_giant_proxy_center_rounding_error_bound_meters);
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

    #[test]
    fn snapshots_borrow_one_stable_immutable_primitive_set() {
        let mut prototype = CameraPrototype::default();
        let first = prototype.snapshot().primitives;
        prototype.advance(Duration::from_secs(11));
        let second = prototype.snapshot().primitives;

        assert!(std::ptr::eq(first.as_ptr(), second.as_ptr()));
        assert_eq!(first, second);
    }

    #[test]
    fn transition_phase_reports_whether_the_camera_is_moving() {
        assert!(TransitionPhase::Approach.is_moving());
        assert!(TransitionPhase::Retreat.is_moving());
        assert!(!TransitionPhase::NearDwell.is_moving());
        assert!(!TransitionPhase::FarDwell.is_moving());
    }

    fn assert_close(actual: f64, expected: f64, tolerance: f64) {
        assert!(
            (actual - expected).abs() <= tolerance,
            "expected {actual:.16e} to be within {tolerance:.3e} of {expected:.16e}"
        );
    }
}

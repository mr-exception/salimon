//! Portable compressed Solar System and large-scale camera prototype.
//!
//! Authoritative positions remain `f64` meters. Presentation code must subtract
//! the camera position in `f64` before converting the small relative result to
//! `f32`; [`WorldPosition::camera_relative_f32`] is the canonical conversion.
//! This crate deliberately has no windowing or GPU dependency.

use std::time::Duration;

pub mod mining;
pub mod resource_distribution;
pub mod resource_generation;
pub mod resources;

pub const SURFACE_ANCHOR: WorldPosition =
    WorldPosition::new(1_000_000_000_000.0, -750_000_000_000.0, 250_000_000_000.0);
const CAMERA_UP: [f64; 3] = [0.0, 1.0, 0.0];
const EARTH_RADIUS_METERS: f64 = 6_000_000.0;
const EARTH_CENTER: WorldPosition = SURFACE_ANCHOR.translated([0.0, 0.0, -EARTH_RADIUS_METERS]);

/// Phase 0's reference maximum travel speed, used to tune compressed distances.
pub const PHASE_ZERO_REFERENCE_MAX_SPEED_METERS_PER_SECOND: f64 = 2_500_000.0;
/// Height above a solid body's surface at which landing assist becomes available.
pub const LANDING_RANGE_ALTITUDE_RADIUS_FACTOR: f64 = 0.15;

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

static PRECISION_MARKERS: &[PrecisionMarker] = &[
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

    /// Euclidean center-to-center distance in authoritative `f64` meters.
    #[must_use]
    pub fn distance_to(self, other: Self) -> f64 {
        let offset = self.offset_from(other);
        offset
            .iter()
            .map(|component| component * component)
            .sum::<f64>()
            .sqrt()
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
        BodyRole, CELESTIAL_BODIES, CameraCommand, CameraPrototype, CelestialBody, CelestialBodyId,
        EARTH_RADIUS_METERS, LANDING_RANGE_ALTITUDE_RADIUS_FACTOR,
        PHASE_ZERO_REFERENCE_MAX_SPEED_METERS_PER_SECOND, PrecisionReport, SURFACE_ANCHOR,
        TransitionPhase, WorldPosition, nearest_celestial_body_within_surface_distance,
        sphere_surface_separation,
    };
    use std::collections::HashSet;
    use std::time::Duration;

    const TEST_EPSILON: f64 = 1.0e-9;

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
}

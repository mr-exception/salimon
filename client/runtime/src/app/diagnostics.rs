//! Observational snapshot/renderer measurements and log formatting.

use crate::frame_clock::FrameTiming;
use salimon_diagnostics::{BodyDistance, DomainMetrics, FrameSample, GpuMemory, GpuTime};
use salimon_renderer::{GpuFrameTime, RenderStats};
use salimon_ship::ShipSnapshot;
use salimon_world::{TransitionPhase, WorldPosition, WorldSnapshot};
use std::time::Duration;

pub(super) fn format_metric_speed(meters_per_second: f64) -> String {
    if meters_per_second >= 1_000_000.0 {
        format!("{:.2} Mm/s", meters_per_second / 1_000_000.0)
    } else if meters_per_second >= 1_000.0 {
        format!("{:.2} km/s", meters_per_second / 1_000.0)
    } else {
        format!("{meters_per_second:.1} m/s")
    }
}

#[cfg(test)]
pub(super) fn camera_body_distances(snapshot: WorldSnapshot<'_>) -> Vec<BodyDistance<'static>> {
    camera_body_distances_from(snapshot, snapshot.camera.position.meters())
}

pub(super) fn camera_body_distances_from(
    snapshot: WorldSnapshot<'_>,
    position_meters: [f64; 3],
) -> Vec<BodyDistance<'static>> {
    let position = WorldPosition::new(position_meters[0], position_meters[1], position_meters[2]);
    snapshot
        .celestial_bodies
        .iter()
        .map(|body| BodyDistance {
            name: body.name,
            distance_meters: body.surface_distance_from(position),
        })
        .collect()
}

pub(super) fn gameplay_domain_metrics<'a>(
    player_position: [f64; 3],
    ship: ShipSnapshot,
    body_distances: &'a [BodyDistance<'static>],
) -> DomainMetrics<'a> {
    DomainMetrics {
        camera_position: Some(player_position),
        player_position: Some(player_position),
        ship_position: Some(ship.pose.position_meters),
        ship_velocity: Some(ship.velocity_meters_per_second),
        ship_speed_mps: Some(ship.speed_meters_per_second),
        thruster_percent: Some(ship.thruster_percentage),
        nearby_bodies: body_distances,
        ..DomainMetrics::default()
    }
}

pub(super) fn camera_domain_metrics<'a>(
    snapshot: WorldSnapshot<'_>,
    body_distances: &'a [BodyDistance<'static>],
) -> DomainMetrics<'a> {
    DomainMetrics {
        camera_position: Some(snapshot.camera.position.meters()),
        camera_altitude_meters: Some(snapshot.camera.altitude_meters),
        camera_phase: Some(transition_phase_name(snapshot.transition_phase)),
        camera_paused: Some(snapshot.paused),
        nearby_bodies: body_distances,
        ..DomainMetrics::default()
    }
}

const fn transition_phase_name(phase: TransitionPhase) -> &'static str {
    match phase {
        TransitionPhase::Approach => "approach",
        TransitionPhase::NearDwell => "near dwell",
        TransitionPhase::Retreat => "retreat",
        TransitionPhase::FarDwell => "far dwell",
    }
}

pub(super) fn frame_sample(
    timing: FrameTiming,
    render_stats: RenderStats,
    cpu_update_time: Duration,
) -> FrameSample {
    FrameSample {
        frame_number: timing.frame_number,
        frame_interval: timing.frame_interval,
        cpu_render_time: render_stats.cpu_render_time,
        cpu_update_time: Some(cpu_update_time),
        gpu_time: match render_stats.gpu_frame_time {
            GpuFrameTime::Unsupported => GpuTime::Unsupported,
            GpuFrameTime::Pending => GpuTime::Pending,
            GpuFrameTime::Measured(duration) => GpuTime::Measured(duration),
        },
        visible_objects: render_stats.visible_objects,
        rendered_objects: render_stats.rendered_objects,
        scene_draw_calls: render_stats.scene_draw_calls,
        total_draw_calls: render_stats.total_draw_calls,
        gpu_memory: render_stats.gpu_memory.map(|memory| GpuMemory {
            allocated_bytes: memory.allocated_bytes,
            reserved_bytes: memory.reserved_bytes,
        }),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use salimon_world::{CELESTIAL_BODIES, CameraCommand, CameraPrototype, CelestialBodyId};
    #[test]
    fn cockpit_speed_uses_practical_metric_units() {
        assert_eq!(format_metric_speed(12.0), "12.0 m/s");
        assert_eq!(format_metric_speed(12_500.0), "12.50 km/s");
        assert_eq!(format_metric_speed(2_500_000.0), "2.50 Mm/s");
    }

    #[test]
    fn diagnostics_reports_every_body_and_near_view_earth_is_closest() {
        let mut prototype = CameraPrototype::default();
        prototype.apply_command(CameraCommand::JumpToNear);
        let snapshot = prototype.snapshot();
        let distances = camera_body_distances(snapshot);

        assert_eq!(distances.len(), CELESTIAL_BODIES.len());
        assert_eq!(
            distances.iter().map(|body| body.name).collect::<Vec<_>>(),
            CELESTIAL_BODIES
                .iter()
                .map(|body| body.name)
                .collect::<Vec<_>>()
        );
        assert!(distances.iter().all(|body| body.distance_meters >= 0.0));
        let nearest = distances
            .iter()
            .min_by(|first, second| first.distance_meters.total_cmp(&second.distance_meters))
            .expect("catalog is nonempty");
        let earth = CELESTIAL_BODIES
            .iter()
            .find(|body| body.id == CelestialBodyId::Earth)
            .expect("Earth is canonical");

        assert_eq!(nearest.name, earth.name);
        assert!((nearest.distance_meters - 12.0).abs() <= f64::EPSILON);
    }
}

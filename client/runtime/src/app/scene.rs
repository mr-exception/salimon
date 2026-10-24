//! Domain snapshots mapped to renderer-owned, absolute-metre DTOs.

use salimon_renderer::{
    CameraFrame, CockpitInstruments, CoreInstruments, NavigationInstruments, NearbyBodyInstruments,
    PointLight, SceneInstance, ShipMeshInstance, SphereInstance, SurfaceMaterial,
};
use salimon_ship::ShipSnapshot;
use salimon_world::{CelestialBodyId, WorldSnapshot};

pub(super) fn map_ship_to_renderer(ship: ShipSnapshot) -> ShipMeshInstance {
    ShipMeshInstance {
        machinery: Default::default(),
        fabricator_output_present: false,
        fabricator_output_medical: false,
        fabricator_output_component: false,
        position_meters: ship.pose.position_meters,
        orientation: ship.pose.orientation.map(|value| value as f32),
        door_open_fraction: ship.door_open_fraction as f32,
        bunk_open_fraction: 0.0,
        independent_interior: false,
        instruments: CockpitInstruments {
            engines_off: !ship.engines_enabled,
            fabricator: Default::default(),
            navigation: NavigationInstruments {
                direction_local: ship.navigation.velocity_direction_local,
                angle_degrees: ship.navigation.heading_velocity_angle_degrees,
                acceleration: ship.navigation.acceleration_meters_per_second_squared,
                mass_kg: ship.total_mass_kg,
                relative_speed: ship.nearby_body.map(|_| ship.speed_meters_per_second),
                braking: match ship.navigation.braking {
                    salimon_ship::BrakingEstimate::Aligned {
                        seconds,
                        distance_meters,
                        energy_joules,
                        ..
                    } => Some([seconds, distance_meters, energy_joules]),
                    _ => None,
                },
                braking_status: match ship.navigation.braking {
                    salimon_ship::BrakingEstimate::Stationary => "STOPPED",
                    salimon_ship::BrakingEstimate::Aligned {
                        sufficient_energy: true,
                        ..
                    } => "AVAILABLE",
                    salimon_ship::BrakingEstimate::Aligned {
                        sufficient_energy: false,
                        ..
                    } => "LOW ENERGY",
                    salimon_ship::BrakingEstimate::AlignHeading => "ALIGN NOSE TO VELOCITY",
                    salimon_ship::BrakingEstimate::Unavailable => "UNAVAILABLE",
                },
            },
            power_off: !ship.power_available,
            core_status: CoreInstruments {
                consumption_watts: ship.core_status.consumption_watts,
                conversion_watts: ship.core_status.conversion_watts,
                conversion_remaining_joules: ship.core_status.conversion_remaining_joules,
                depletion_seconds: ship.core_status.depletion_seconds,
                low_energy: ship.core_status.low_energy,
            },
            speed_meters_per_second: ship.speed_meters_per_second,
            thruster_percentage: ship.thruster_percentage,
            core_energy_capacity_joules: ship.energy_core.capacity_joules,
            core_energy_stored_joules: ship.energy_core.stored_joules,
            nearby_body: ship.nearby_body.map(|body| NearbyBodyInstruments {
                name: body.name,
                surface_distance_meters: body.surface_distance_meters,
                radial_speed_meters_per_second: body.radial_speed_meters_per_second,
            }),
        },
    }
}

pub(super) fn map_world_to_renderer(
    snapshot: WorldSnapshot<'_>,
) -> (
    CameraFrame,
    Vec<SceneInstance>,
    Vec<SphereInstance>,
    PointLight,
) {
    let camera = CameraFrame {
        position_meters: snapshot.camera.position.meters(),
        target_meters: snapshot.camera.target.meters(),
        up: snapshot.camera.up.map(|component| component as f32),
        vertical_fov_radians: snapshot.camera.vertical_field_of_view_radians as f32,
        near_plane_meters: snapshot.camera.physical_near_plane_meters as f32,
    };
    let spheres = snapshot
        .celestial_bodies
        .iter()
        .map(|body| SphereInstance {
            center_meters: body.center.meters(),
            radius_meters: body.radius_meters,
            material: match body.id {
                CelestialBodyId::Sun => SurfaceMaterial::Emissive,
                CelestialBodyId::Mercury => SurfaceMaterial::Stone,
                CelestialBodyId::Venus => SurfaceMaterial::Ochre,
                CelestialBodyId::Earth => SurfaceMaterial::Oceanic,
                CelestialBodyId::Moon => SurfaceMaterial::Slate,
                CelestialBodyId::Mars => SurfaceMaterial::Rust,
            },
        })
        .collect();
    let instances = snapshot
        .precision_markers
        .iter()
        .map(|marker| SceneInstance {
            orientation: [0., 0., 0., 1.],
            center_meters: marker.absolute_center.meters(),
            half_extents_meters: marker.half_extents_meters,
            color: marker.color,
        })
        .collect();
    let sun = snapshot
        .celestial_bodies
        .iter()
        .find(|body| body.id == CelestialBodyId::Sun)
        .expect("world catalog always contains the Sun");
    let light = PointLight {
        position_meters: sun.center.meters(),
        color: [1.0, 0.96, 0.88],
    };

    (camera, instances, spheres, light)
}

/// Hollow blueprint edges use the ordinary oriented instance pass. No domain state
/// is owned here, and these visual markers never become collision geometry.
pub(super) fn construction_wireframes(
    session: &crate::session::GameplaySession,
    frame: salimon_character::ShipFrame,
    orientation: [f32; 4],
) -> Vec<SceneInstance> {
    let mut result = Vec::new();
    for (space, color) in session.construction_boxes() {
        for axis in 0..3 {
            let other: Vec<_> = (0..3).filter(|i| *i != axis).collect();
            for a in [space.min[other[0]], space.max[other[0]]] {
                for b in [space.min[other[1]], space.max[other[1]]] {
                    let mut center = std::array::from_fn(|i| (space.min[i] + space.max[i]) / 2.);
                    center[other[0]] = a;
                    center[other[1]] = b;
                    let mut half = [0.012; 3];
                    half[axis] = ((space.max[axis] - space.min[axis]) / 2.) as f32;
                    result.push(SceneInstance {
                        center_meters: frame.local_to_world(center),
                        half_extents_meters: half,
                        color,
                        orientation,
                    });
                }
            }
        }
    }
    result
}

/// Bounded procedural mechanical leaves and fixed manual release grips. All
/// geometry follows character-owned bounds; it carries no interaction authority.
pub(super) fn cargo_door(
    session: &crate::session::GameplaySession,
    ship: ShipSnapshot,
) -> Vec<SceneInstance> {
    let frame = crate::session::frames::character_ship_frame(ship.pose);
    let mut instances = Vec::with_capacity(4);
    for b in salimon_character::cargo::cargo_door_leaves(session.cargo_door.open_fraction) {
        instances.push(SceneInstance {
            center_meters: frame.local_to_world([
                (b[0] + b[1]) * 0.5,
                (b[2] + b[3]) * 0.5,
                (b[4] + b[5]) * 0.5,
            ]),
            half_extents_meters: [
                (b[1] - b[0]) as f32 * 0.5,
                (b[3] - b[2]) as f32 * 0.5,
                (b[5] - b[4]) as f32 * 0.5,
            ],
            orientation: ship.pose.orientation.map(|v| v as f32),
            color: [0.13, 0.15, 0.17, 1.0],
        });
    }
    for x in [-0.14, 0.14] {
        let mut p = salimon_character::cargo::CARGO_DOOR_TARGET;
        p[0] += x;
        instances.push(SceneInstance {
            center_meters: frame.local_to_world(p),
            half_extents_meters: [0.035, 0.14, 0.035],
            orientation: ship.pose.orientation.map(|v| v as f32),
            color: [0.45, 0.24, 0.10, 1.0],
        });
    }
    instances
}

#[cfg(test)]
mod tests {
    use super::super::diagnostics::{camera_body_distances, camera_domain_metrics};
    use super::super::native::{INITIAL_HEIGHT, INITIAL_WIDTH};
    use super::*;
    use salimon_ship::{FlightState, ShipController};
    use salimon_world::CameraPrototype;
    #[test]
    fn cockpit_instruments_follow_live_power_and_landing_after_leaving_the_seat() {
        let mut pose = ShipController::default().snapshot().pose;
        pose.position_meters[1] += 100.0;
        let mut ship = ShipController::flying(pose, 0);
        let stopped = map_ship_to_renderer(ship.snapshot());
        assert_eq!(stopped.instruments.speed_meters_per_second, 0.0);
        assert_eq!(stopped.instruments.thruster_percentage, 0);
        assert_eq!(
            stopped.instruments.core_energy_capacity_joules,
            salimon_ship::DEFAULT_CORE_ENERGY_CAPACITY_JOULES
        );
        assert_eq!(stopped.instruments.nearby_body.unwrap().name, "Earth");

        ship.set_stored_core_energy_joules(25_000_000_000);
        let changed_energy = map_ship_to_renderer(ship.snapshot());
        assert_eq!(
            changed_energy.instruments.core_energy_stored_joules,
            25_000_000_000
        );

        ship.adjust_thruster(1);
        let minimum_power = map_ship_to_renderer(ship.snapshot());
        assert_eq!(minimum_power.instruments.thruster_percentage, 1);
        assert_eq!(minimum_power.instruments.speed_meters_per_second, 0.0);

        ship.adjust_thruster(99);
        let full_power = map_ship_to_renderer(ship.snapshot());
        assert_eq!(full_power.instruments.thruster_percentage, 100);
        assert_eq!(full_power.instruments.speed_meters_per_second, 0.0);
        ship.advance(std::time::Duration::from_millis(10));
        assert_eq!(
            map_ship_to_renderer(ship.snapshot())
                .instruments
                .speed_meters_per_second,
            10.0
        );

        ship.trigger_landing_action();
        ship.set_cockpit_control(false);
        let assisting = map_ship_to_renderer(ship.snapshot()).instruments;
        assert_eq!(assisting.core_energy_stored_joules, 24_198_998_900);
        assert_eq!(
            assisting.speed_meters_per_second,
            full_power.instruments.speed_meters_per_second
        );
        // Landing starts with an alignment hold, even when already at the
        // surface. The live monitor must report the actual stationary phase.
        assert_eq!(
            assisting
                .nearby_body
                .unwrap()
                .radial_speed_meters_per_second,
            0.0
        );
        ship.advance(std::time::Duration::from_millis(100));
        assert!(matches!(
            ship.snapshot().flight_state,
            FlightState::AssistedLanding { .. }
        ));
        ship.advance(salimon_ship::LANDING_ASSIST_DURATION - std::time::Duration::from_millis(100));
        assert!(matches!(
            ship.snapshot().flight_state,
            FlightState::Landed { .. }
        ));
        let landed = map_ship_to_renderer(ship.snapshot()).instruments;
        assert_eq!(landed.speed_meters_per_second, 0.0);
        assert_eq!(landed.thruster_percentage, 0);
        assert_eq!(landed.core_energy_stored_joules, 24_198_118_900);

        let out_of_range = ShipController::flying(
            salimon_ship::ShipPose {
                position_meters: [0.0; 3],
                orientation: [0.0, 0.0, 0.0, 1.0],
            },
            0,
        );
        assert!(
            map_ship_to_renderer(out_of_range.snapshot())
                .instruments
                .nearby_body
                .is_none()
        );
    }

    #[test]
    fn cockpit_proximity_monitor_tracks_eased_assists_without_pilot_authority() {
        let mut ship = ShipController::default();
        ship.set_cockpit_control(true);
        ship.trigger_landing_action();
        ship.set_cockpit_control(false);
        ship.advance(std::time::Duration::from_secs(1));
        let lifting = map_ship_to_renderer(ship.snapshot())
            .instruments
            .nearby_body
            .unwrap();
        assert!((lifting.radial_speed_meters_per_second - 7.5).abs() < 1.0e-9);
        ship.advance(std::time::Duration::from_secs(1));
        let lifted = map_ship_to_renderer(ship.snapshot())
            .instruments
            .nearby_body
            .unwrap();
        assert!(lifted.surface_distance_meters > lifting.surface_distance_meters);
        assert_eq!(lifted.radial_speed_meters_per_second, 0.0);
        ship.advance(std::time::Duration::from_secs(4));
        let handoff = map_ship_to_renderer(ship.snapshot())
            .instruments
            .nearby_body
            .unwrap();
        assert_eq!(handoff.radial_speed_meters_per_second, 0.0);
        assert!(
            (handoff.surface_distance_meters - lifted.surface_distance_meters - 40.0).abs() < 0.001
        );

        let mut pose = ShipController::default().snapshot().pose;
        pose.position_meters[1] += 100.0;
        let mut ship = ShipController::flying(pose, 100);
        ship.trigger_landing_action();
        ship.set_cockpit_control(false);
        ship.advance(std::time::Duration::from_secs(2));
        let aligned = map_ship_to_renderer(ship.snapshot())
            .instruments
            .nearby_body
            .unwrap();
        assert_eq!(aligned.radial_speed_meters_per_second, 0.0);
        ship.advance(std::time::Duration::from_secs(1));
        let descending = map_ship_to_renderer(ship.snapshot())
            .instruments
            .nearby_body
            .unwrap();
        assert!(descending.surface_distance_meters < aligned.surface_distance_meters);
        assert!(descending.radial_speed_meters_per_second < 0.0);
    }

    #[test]
    fn runtime_mapping_preserves_spheres_light_camera_and_separate_markers() {
        let snapshot = CameraPrototype::default().snapshot();
        let (camera, instances, spheres, light) = map_world_to_renderer(snapshot);
        let body_distances = camera_body_distances(snapshot);
        let metrics = camera_domain_metrics(snapshot, &body_distances);

        assert_eq!(camera.position_meters, snapshot.camera.position.meters());
        assert_eq!(camera.target_meters, snapshot.camera.target.meters());
        assert_eq!(instances.len(), 3);
        assert_eq!(spheres.len(), 6);
        assert_eq!(snapshot.celestial_bodies.len(), 6);
        assert_eq!(snapshot.precision_markers.len(), 3);
        for (instance, body) in spheres.iter().zip(snapshot.celestial_bodies) {
            assert_eq!(instance.center_meters, body.center.meters());
            assert_eq!(instance.radius_meters, body.radius_meters);
            assert_eq!(
                instance.material == salimon_renderer::SurfaceMaterial::Emissive,
                body.id == CelestialBodyId::Sun
            );
        }
        assert_eq!(
            light.position_meters,
            snapshot.celestial_bodies[0].center.meters()
        );
        for (instance, marker) in instances.iter().zip(snapshot.precision_markers) {
            assert_eq!(instance.center_meters, marker.absolute_center.meters());
            assert_eq!(instance.half_extents_meters, marker.half_extents_meters);
            assert_eq!(instance.color, marker.color);
        }
        assert_eq!(metrics.camera_position, Some(camera.position_meters));
        assert_eq!(
            metrics.camera_altitude_meters,
            Some(snapshot.camera.altitude_meters)
        );
        assert_eq!(metrics.camera_phase, Some("approach"));
        assert_eq!(metrics.camera_paused, Some(false));
        assert_eq!(metrics.nearby_bodies, body_distances);
    }

    #[test]
    fn initial_overview_fully_frames_every_sphere() {
        let snapshot = CameraPrototype::default().snapshot();
        let aspect_ratio = INITIAL_WIDTH as f64 / INITIAL_HEIGHT as f64;
        let vertical_tangent = (snapshot.camera.vertical_field_of_view_radians * 0.5).tan();
        let horizontal_tangent = vertical_tangent * aspect_ratio;

        for body in snapshot.celestial_bodies {
            let offset = body.center.offset_from(snapshot.camera.position);
            let forward_depth = -offset[2];
            let nearest_face_depth = forward_depth - body.radius_meters;
            assert!(
                nearest_face_depth > snapshot.camera.physical_near_plane_meters,
                "{}'s nearest face must be in front of the overview camera",
                body.name
            );
            assert!(
                offset[0].abs() + body.radius_meters <= nearest_face_depth * horizontal_tangent,
                "{} must fit horizontally in the initial overview",
                body.name
            );
            assert!(
                offset[1].abs() + body.radius_meters <= nearest_face_depth * vertical_tangent,
                "{} must fit vertically in the initial overview",
                body.name
            );
        }
    }
}

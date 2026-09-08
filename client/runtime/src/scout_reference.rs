//! Named capture cameras shared with offline authoring. E2E presentation only.
use salimon_renderer::CameraFrame;
use salimon_ship::ShipPose;

#[derive(Clone, Debug, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct View {
    pub(crate) name: String,
    #[serde(default)]
    absolute: bool,
    eye: [f64; 3],
    target: [f64; 3],
}

pub(crate) fn find(name: &str) -> Option<View> {
    // Explicit engineering cameras for the historical representative terrain fixtures.
    if let Some(rest) = name.strip_prefix("geology-") {
        let (body_name, range) = rest.split_once('-')?;
        let body = salimon_world::CELESTIAL_BODIES
            .iter()
            .find(|b| b.name.eq_ignore_ascii_case(body_name))?;
        let altitude = match range {
            "near" => 40.0,
            "mid" => 400.0,
            "far" => 4000.0,
            _ => return None,
        };
        let definition = salimon_world::terrain::TerrainMode::Representative.definition(body.id)?;
        let surface_radius = body.radius_meters + definition.height([0.0, 1.0, 0.0]).ok()?;
        let center = body.center.meters();
        return Some(View {
            name: name.into(),
            absolute: true,
            eye: [
                center[0],
                center[1] + surface_radius + altitude,
                center[2] + altitude * 1.2,
            ],
            target: [center[0], center[1] + surface_radius, center[2]],
        });
    }
    // Engineering scanner view is separate from the approved four-camera
    // authored Scout lighting corpus; adding it does not promote a baseline.
    if name == "scanner" {
        return Some(View {
            name: name.into(),
            absolute: false,
            eye: [3.7, 1.95, -1.6],
            target: [5.2, 1.65, -0.9],
        });
    }
    let radius = salimon_world::canonical::START
        .manifest()
        .physical
        .radius_meters;
    let eye = match name {
        "universe-orbit" => Some([0., radius * 2.5, 0.]),
        "universe-surface" => Some([45., radius + 3., 70.]),
        "universe-night" => {
            let height = salimon_world::terrain::TerrainDefinition::canonical_start()
                .height([0., -1., 0.])
                .expect("published canonical night terrain");
            Some([0., -(radius + height + 40.), 0.])
        }
        "universe-edge" => {
            let direction = [
                std::f64::consts::FRAC_1_SQRT_2,
                std::f64::consts::FRAC_1_SQRT_2,
                0.,
            ];
            let height = salimon_world::terrain::TerrainDefinition::canonical_start()
                .height(direction)
                .expect("published canonical terrain direction");
            Some(direction.map(|v| v * (radius + height + 500.)))
        }
        _ => None,
    };
    if let Some(eye) = eye {
        return Some(View {
            name: name.into(),
            absolute: true,
            eye,
            target: if name == "universe-night" {
                [0., eye[1], 1000.]
            } else {
                [0., radius, 0.]
            },
        });
    }
    let views: Vec<View> =
        serde_json::from_str(include_str!("../../../scenarios/visual/scout-cameras.json"))
            .expect("checked-in reference camera contract");
    let machinery: Vec<View> = serde_json::from_str(include_str!(
        "../../../scenarios/visual/scout-machinery-cameras.json"
    ))
    .expect("checked-in machinery camera contract");
    let finishes: Vec<View> = serde_json::from_str(include_str!(
        "../../../scenarios/visual/scout-finish-cameras.json"
    ))
    .expect("finish cameras");
    views
        .into_iter()
        .chain(machinery)
        .chain(finishes)
        .find(|v| v.name == name)
}

impl View {
    /// Evidence camera follows the actual anti-stellar direction; it changes no
    /// body, player position, lighting, terrain recipe or physical source.
    pub(crate) fn at_epoch(&self, epoch: Option<salimon_universe::Epoch>) -> Self {
        let mut view = self.clone();
        if self.name == "universe-night"
            && let Some(epoch) = epoch
            && let Ok(planet) = salimon_universe::starting_motion::planet(epoch)
        {
            let distance = salimon_math::length(planet.position_meters);
            let direction = planet.position_meters.map(|v| v / distance);
            let height = salimon_world::terrain::TerrainDefinition::canonical_start()
                .height(direction)
                .expect("published canonical terrain");
            let radius = salimon_world::canonical::START
                .manifest()
                .physical
                .radius_meters;
            view.eye = direction.map(|v| v * (radius + height + 40.));
            view.target = salimon_math::add(view.eye, [0., 1000., 0.]);
        }
        view
    }

    /// Evidence-only clearance against the physical canonical evaluator.
    pub(crate) fn canonical_ground_clearance(&self) -> Option<f64> {
        if !matches!(
            self.name.as_str(),
            "universe-orbit" | "universe-surface" | "universe-edge" | "universe-night"
        ) {
            return None;
        }
        let distance = salimon_math::length(self.eye);
        let height = salimon_world::terrain::TerrainDefinition::canonical_start()
            .height(self.eye.map(|v| v / distance))
            .ok()?;
        Some(
            distance
                - salimon_world::canonical::START
                    .manifest()
                    .physical
                    .radius_meters
                - height,
        )
    }

    pub(crate) fn camera(&self, pose: ShipPose) -> CameraFrame {
        if self.absolute {
            return CameraFrame {
                position_meters: self.eye,
                target_meters: self.target,
                up: if self.name == "universe-night" {
                    salimon_math::scale(self.eye, 1. / salimon_math::length(self.eye))
                        .map(|v| v as f32)
                } else {
                    [0., 0., 1.]
                },
                vertical_fov_radians: 70.0_f32.to_radians(),
                near_plane_meters: 0.05,
            };
        }
        let frame = crate::session::frames::character_ship_frame(pose);
        CameraFrame {
            position_meters: frame.local_to_world(self.eye),
            target_meters: frame.local_to_world(self.target),
            up: pose.axes()[1].map(|v| v as f32),
            vertical_fov_radians: 70.0_f32.to_radians(),
            near_plane_meters: 0.05,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn night_reference_remains_above_actual_terrain_on_the_anti_stellar_side() {
        for unix in [0, 1_767_225_600, 1_793_404_800] {
            let epoch = salimon_universe::starting_motion::Clock::at(unix)
                .unwrap()
                .epoch;
            let view = find("universe-night").unwrap().at_epoch(Some(epoch));
            let planet = salimon_universe::starting_motion::planet(epoch).unwrap();
            assert!(salimon_math::dot(view.eye, planet.position_meters) > 0.);
            assert!((view.canonical_ground_clearance().unwrap() - 40.).abs() < 1e-6);
        }
    }
    #[test]
    fn shared_cameras_are_finite_and_follow_ship_pose_without_changing_it() {
        let pose = salimon_ship::ShipController::default().snapshot().pose;
        for name in [
            "finishes",
            "seat-finish",
            "cockpit",
            "core",
            "fabricator",
            "aft-door",
            "pedals-close",
            "fabricator-close",
        ] {
            let v = find(name).unwrap();
            let c = v.camera(pose);
            assert_ne!(c.position_meters, c.target_meters);
            assert!(c.position_meters.iter().all(|v| v.is_finite()));
            let mut moved = pose;
            moved.position_meters[0] += 1_000_000.0;
            let next = v.camera(moved);
            assert!((next.position_meters[0] - c.position_meters[0] - 1_000_000.0).abs() < 1e-6);
            assert_eq!(next.up, c.up);
        }
        assert!(find("unknown").is_none());
    }
}

#[cfg(test)]
mod geology_camera_tests {
    use super::*;
    #[test]
    fn geological_evidence_cameras_clear_actual_ground_and_reject_nonterrain_bodies() {
        for body_name in ["earth", "moon", "mars"] {
            let body = salimon_world::CELESTIAL_BODIES
                .iter()
                .find(|b| b.name.eq_ignore_ascii_case(body_name))
                .unwrap();
            let definition = salimon_world::terrain::TerrainMode::Representative
                .definition(body.id)
                .unwrap();
            let ground = body.center.meters()[1]
                + body.radius_meters
                + definition.height([0.0, 1.0, 0.0]).unwrap();
            for (range, altitude) in [("near", 40.0), ("mid", 400.0), ("far", 4000.0)] {
                let view = find(&format!("geology-{body_name}-{range}")).unwrap();
                assert!((view.target[1] - ground).abs() < 0.001);
                assert!((view.eye[1] - ground - altitude).abs() < 0.001);
                assert!(view.absolute);
            }
        }
        assert!(find("geology-sun-near").is_none());
        assert!(find("geology-moon-invalid").is_none());
    }
}

#[cfg(test)]
mod canonical_camera_tests {
    use super::*;
    #[test]
    fn canonical_reference_cameras_stay_above_actual_terrain() {
        let definition = salimon_world::terrain::TerrainDefinition::canonical_start();
        let radius = salimon_world::canonical::START
            .manifest()
            .physical
            .radius_meters;
        for name in [
            "universe-orbit",
            "universe-surface",
            "universe-edge",
            "universe-night",
        ] {
            let view = find(name).unwrap();
            let distance = salimon_math::length(view.eye);
            let ground = definition.height(view.eye.map(|v| v / distance)).unwrap();
            let clearance = distance - radius - ground;
            assert_eq!(view.canonical_ground_clearance(), Some(clearance));
            assert!(view.absolute);
            assert!(
                clearance > 2.0,
                "{name}: camera intersects actual terrain ({clearance} m)"
            );
            if name == "universe-edge" {
                assert!((clearance - 500.0).abs() < 1e-6);
            }
        }
    }
}

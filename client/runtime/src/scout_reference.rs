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
        "universe-edge" => {
            let d = radius / 2.0_f64.sqrt();
            Some([d + 500., d + 500., 0.])
        }
        _ => None,
    };
    if let Some(eye) = eye {
        return Some(View {
            name: name.into(),
            absolute: true,
            eye,
            target: [0., radius, 0.],
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
    pub(crate) fn camera(&self, pose: ShipPose) -> CameraFrame {
        if self.absolute {
            return CameraFrame {
                position_meters: self.eye,
                target_meters: self.target,
                up: [0., 0., 1.],
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

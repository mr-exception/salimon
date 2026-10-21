//! Named capture cameras shared with offline authoring. E2E presentation only.
use salimon_renderer::CameraFrame;
use salimon_ship::ShipPose;

#[derive(Clone, Debug, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct View {
    pub(crate) name: String,
    eye: [f64; 3],
    target: [f64; 3],
}

pub(crate) fn find(name: &str) -> Option<View> {
    let views: Vec<View> =
        serde_json::from_str(include_str!("../../../scenarios/visual/scout-cameras.json"))
            .expect("checked-in reference camera contract");
    views.into_iter().find(|v| v.name == name)
}

impl View {
    pub(crate) fn camera(&self, pose: ShipPose) -> CameraFrame {
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
        for name in ["cockpit", "core", "fabricator", "aft-door"] {
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

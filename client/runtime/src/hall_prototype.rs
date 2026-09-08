//! Explicit --e2e developer fixture. Never saved, unlocked or exposed by menus.
//! Uses the ordinary production meshes, depth pipeline and capture path.
use salimon_renderer::{CameraFrame, PointLight, SceneInstance, SphereInstance, SurfaceMaterial};
use salimon_world::hall;

pub(crate) struct Prototype {
    eye: [f64; 3],
    target: [f64; 3],
    pub(crate) blocked: bool,
}
impl Default for Prototype {
    fn default() -> Self {
        Self {
            eye: [0., hall::EYE_METERS, 24.],
            target: [0., 4., 0.],
            blocked: false,
        }
    }
}
const POPULATION: u64 = 1_000_000_000;
fn block(center: [f64; 3], half: [f32; 3], yaw: f64, color: [f32; 4]) -> SceneInstance {
    SceneInstance {
        center_meters: center,
        half_extents_meters: half,
        orientation: [0., (yaw * 0.5).sin() as f32, 0., (yaw * 0.5).cos() as f32],
        color,
    }
}
impl Prototype {
    pub(crate) fn view(&mut self, name: &str) -> Result<(), &'static str> {
        let (eye, target) = match name {
            "arrival" => ([0., hall::EYE_METERS, 24.], [0., 4., 0.]),
            "overview" => ([110., 40., 145.], [0., 2., 0.]),
            "sign" => ([0., hall::EYE_METERS, 28.], [0., 1.6, 32.]),
            "outer" => {
                let s = hall::sign(POPULATION - 50);
                let n = [s.yaw.sin(), s.yaw.cos()];
                (
                    [
                        s.center[0] - n[0] * 4.,
                        hall::EYE_METERS,
                        s.center[2] - n[1] * 4.,
                    ],
                    [s.center[0], 1.6, s.center[2]],
                )
            }
            _ => return Err("unknown internal Hall camera"),
        };
        self.eye = eye;
        self.target = target;
        self.blocked = false;
        Ok(())
    }
    pub(crate) fn walk(&mut self, delta: [f64; 2]) -> Result<(), &'static str> {
        let next = hall::walk(self.eye, delta, POPULATION)?;
        self.blocked = next == self.eye && delta != [0.; 2];
        for (i, coordinate) in next.iter().enumerate() {
            self.target[i] += coordinate - self.eye[i];
        }
        self.eye = next;
        Ok(())
    }
    pub(crate) fn observe(&self) -> serde_json::Value {
        let signs = hall::visible(self.eye, POPULATION).expect("validated prototype camera");
        serde_json::json!({"fixture":true,"population":POPULATION,"eye":self.eye,"blocked":self.blocked,
            "signs":signs.len(),"detailed":signs.iter().filter(|s|s.detailed).count(),
            "silhouettes":signs.iter().filter(|s|!s.detailed).count(),
            "ranks":signs.iter().map(|s|s.rank).collect::<Vec<_>>(),
            "account":null,"unlock":false,"registry":false})
    }
    pub(crate) fn scene(
        &self,
    ) -> (
        CameraFrame,
        Vec<SceneInstance>,
        Vec<SphereInstance>,
        PointLight,
    ) {
        let camera = CameraFrame {
            position_meters: self.eye,
            target_meters: self.target,
            up: [0., 1., 0.],
            vertical_fov_radians: 70_f32.to_radians(),
            near_plane_meters: 0.05,
        };
        let stone = [0.18, 0.20, 0.22, 1.];
        let edge = [0.31, 0.32, 0.32, 1.];
        // Recenter a large floor patch beyond the bounded visible arc. There is
        // no exterior wall, finite universe edge or floor-collision discrepancy.
        let mut meshes = vec![block(
            [self.eye[0], -0.2, self.eye[2]],
            [2048., 0.2, 2048.],
            0.,
            [0.075, 0.08, 0.085, 1.],
        )];
        for sign in hall::visible(self.eye, POPULATION).expect("validated prototype camera") {
            let [x, _, z] = sign.center;
            if sign.detailed {
                // Authored three-piece monolith kit, in real metres; instance
                // source and collision plinth share these conservative dimensions.
                meshes.push(block([x, 0.18, z], [1., 0.18, 0.6], sign.yaw, edge));
                meshes.push(block([x, 1.5, z], [0.68, 1.14, 0.27], sign.yaw, stone));
                meshes.push(block([x, 2.72, z], [0.8, 0.08, 0.35], sign.yaw, edge));
            } else {
                meshes.push(block([x, 1.4, z], [0.8, 1.4, 0.35], sign.yaw, stone));
            }
        }
        // The central presence is deliberately symbolic, never an astronomical
        // Big Bang coordinate or a real navigable body. Gate remains local fiction.
        for x in [-4.5, 4.5] {
            meshes.push(block([x, 5., 14.], [0.8, 5., 0.8], 0., edge));
            meshes.push(block([x, 10.15, 14.], [1., 0.15, 1.], 0., stone));
        }
        meshes.push(block([0., 10.5, 14.], [5.5, 0.35, 1.], 0., edge));
        let spheres = vec![SphereInstance {
            center_meters: [0., 8., 0.],
            radius_meters: 8.,
            material: SurfaceMaterial::Emissive,
        }];
        (
            camera,
            meshes,
            spheres,
            PointLight {
                position_meters: [0., 8., 0.],
                color: [1., 0.82, 0.58],
            },
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn developer_views_are_bounded_and_invalid_actions_leave_the_camera_unchanged() {
        assert!(
            crate::app::ClientApplication::default()
                .hall_prototype
                .is_none()
        );
        assert!(
            crate::e2e::parse_args(["--scenario", "hall-prototype"].map(str::to_owned)).is_err()
        );
        assert!(
            crate::e2e::parse_args(std::iter::empty::<String>())
                .unwrap()
                .is_none()
        );
        let mut p = Prototype::default();
        for view in ["arrival", "overview", "sign", "outer"] {
            p.view(view).unwrap();
            let a = p.observe();
            assert!(a["signs"].as_u64().unwrap() <= hall::MAX_SIGNS as u64);
            assert!(p.scene().1.len() <= 3 * hall::MAX_SIGNS + 6);
            assert!(p.view("public-menu").is_err());
            assert!(p.walk([f64::NAN, 0.]).is_err());
            assert_eq!(p.observe(), a);
        }
        p.view("sign").unwrap();
        for _ in 0..20 {
            p.walk([0., 0.5]).unwrap();
        }
        assert!(p.blocked);
        assert!(p.eye[2] < 31.1);
    }
}

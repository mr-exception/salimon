//! Environmental frames and character presentation contracts.
use salimon_math::{add, dot, scale, sub};
use std::time::Duration;
#[derive(Clone, Copy, Debug, PartialEq, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ShipFrame {
    pub origin_meters: [f64; 3],
    /// Local ship axes in world space: forward (+X), up (+Y), port (+Z).
    pub axes: [[f64; 3]; 3],
}

impl ShipFrame {
    /// Convert an absolute point to ship-local meters, subtracting in f64 first.
    ///
    /// ```compile_fail
    /// use salimon_character::{ShipFrame, ShipLocalPoint};
    /// let frame = ShipFrame { origin_meters: [0.0; 3], axes: [[1.0,0.0,0.0],[0.0,1.0,0.0],[0.0,0.0,1.0]] };
    /// frame.project_world(ShipLocalPoint::new([1.0; 3]));
    /// ```
    #[must_use]
    pub fn project_world(self, world: crate::WorldPoint) -> crate::ShipLocalPoint {
        crate::ShipLocalPoint::new(self.world_to_local(world.meters()))
    }

    /// Convert a ship-local point to absolute meters using the existing axes.
    ///
    /// ```compile_fail
    /// use salimon_character::{ShipFrame, WorldPoint};
    /// let frame = ShipFrame { origin_meters: [0.0; 3], axes: [[1.0,0.0,0.0],[0.0,1.0,0.0],[0.0,0.0,1.0]] };
    /// frame.unproject_local(WorldPoint::new([1.0; 3]));
    /// ```
    #[must_use]
    pub fn unproject_local(self, local: crate::ShipLocalPoint) -> crate::WorldPoint {
        crate::WorldPoint::new(self.local_to_world(local.meters()))
    }

    #[must_use]
    pub fn local_to_world(self, local: [f64; 3]) -> [f64; 3] {
        add(
            self.origin_meters,
            add(
                scale(self.axes[0], local[0]),
                add(scale(self.axes[1], local[1]), scale(self.axes[2], local[2])),
            ),
        )
    }

    #[must_use]
    pub fn world_to_local(self, world: [f64; 3]) -> [f64; 3] {
        let relative = sub(world, self.origin_meters);
        [
            dot(relative, self.axes[0]),
            dot(relative, self.axes[1]),
            dot(relative, self.axes[2]),
        ]
    }
}
#[derive(Clone, Copy, Debug, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SurfaceFrame {
    pub body_center_meters: [f64; 3],
    pub radius_meters: f64,
    /// Pure caller query in absolute metres; None retains the flat sphere fixture.
    #[serde(skip)]
    pub query: Option<GroundQuery>,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum CharacterLocation {
    Cockpit,
    InsideShip,
    DoorwayBlend,
    Surface,
    Space,
    NearbyBody,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct CharacterSnapshot {
    pub location: CharacterLocation,
    pub eye_position_meters: [f64; 3],
    pub look_target_meters: [f64; 3],
    pub up: [f32; 3],
    pub local_ship_position_meters: Option<[f64; 3]>,
    pub doorway_blend_fraction: Option<f64>,
}

#[derive(Clone, Copy, Debug, PartialEq, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) enum PositionState {
    Cockpit,
    Inside {
        local: [f64; 3],
    },
    Doorway {
        world: [f64; 3],
        elapsed: Duration,
        leaving_ship: bool,
    },
    Surface {
        world: [f64; 3],
    },
    Space {
        world: [f64; 3],
    },
}

/// Ground point and outward unit normal in the caller's absolute frame.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct GroundSample {
    pub position_meters: [f64; 3],
    pub normal: [f64; 3],
    pub slope: f64,
}
pub type GroundQuery = fn(SurfaceFrame, [f64; 3]) -> GroundSample;

impl PartialEq for SurfaceFrame {
    fn eq(&self, other: &Self) -> bool {
        self.body_center_meters == other.body_center_meters
            && self.radius_meters == other.radius_meters
            && match (self.query, other.query) {
                (None, None) => true,
                (Some(a), Some(b)) => std::ptr::fn_addr_eq(a, b),
                _ => false,
            }
    }
}
impl SurfaceFrame {
    pub fn sample(self, world: [f64; 3]) -> GroundSample {
        if let Some(query) = self.query {
            return query(self, world);
        }
        let relative = sub(world, self.body_center_meters);
        let magnitude = dot(relative, relative).sqrt();
        let normal = if magnitude > 1e-12 {
            scale(relative, 1.0 / magnitude)
        } else {
            [0.0, 1.0, 0.0]
        };
        GroundSample {
            position_meters: add(self.body_center_meters, scale(normal, self.radius_meters)),
            normal,
            slope: 0.0,
        }
    }
    pub fn eye_on_ground(self, world: [f64; 3]) -> [f64; 3] {
        let radial = crate::math::normalize(sub(world, self.body_center_meters));
        if self.query.is_none() {
            return add(
                self.body_center_meters,
                scale(radial, self.radius_meters + crate::PLAYER_EYE_HEIGHT_METERS),
            );
        }
        add(
            self.sample(world).position_meters,
            scale(radial, crate::PLAYER_EYE_HEIGHT_METERS),
        )
    }
}

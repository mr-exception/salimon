//! Shared catalog terrain selection; every enabled consumer uses these definitions.
use super::{SurfaceSample, TerrainDefinition};
use crate::{CELESTIAL_BODIES, CelestialBody, CelestialBodyId};
use std::sync::LazyLock;

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub enum TerrainMode {
    #[default]
    Flat,
    Representative,
}

static DEFINITIONS: LazyLock<[TerrainDefinition; 3]> = LazyLock::new(|| {
    [
        CelestialBodyId::Earth,
        CelestialBodyId::Moon,
        CelestialBodyId::Mars,
    ]
    .map(|id| {
        let body = *CELESTIAL_BODIES
            .iter()
            .find(|b| b.id == id)
            .expect("catalog terrain body");
        TerrainDefinition::representative(body).expect("validated built-in terrain")
    })
});
impl TerrainMode {
    /// Nonnegative sampled-surface distance; catalog order resolves exact ties.
    pub fn nearest_body(
        self,
        position: [f64; 3],
        maximum_meters: f64,
        solid_only: bool,
    ) -> Option<(&'static CelestialBody, f64)> {
        if !maximum_meters.is_finite() || maximum_meters < 0.0 {
            return None;
        }
        CELESTIAL_BODIES
            .iter()
            .filter(|b| !solid_only || b.role == crate::BodyRole::Solid)
            .filter_map(|b| {
                let distance = if self == Self::Flat {
                    b.surface_distance_from(crate::WorldPosition::new(
                        position[0],
                        position[1],
                        position[2],
                    ))
                } else {
                    self.clearance(*b, position).max(0.0)
                };
                (distance <= maximum_meters).then_some((b, distance))
            })
            .min_by(|a, b| a.1.total_cmp(&b.1))
    }

    pub fn definition(self, body: CelestialBodyId) -> Option<&'static TerrainDefinition> {
        (self == Self::Representative)
            .then(|| DEFINITIONS.iter().find(|d| d.body() == body))
            .flatten()
    }
    /// Built-in catalog body and caller-normalized direction; custom definitions use their own query.
    pub fn height(self, body: CelestialBody, direction: [f64; 3]) -> f64 {
        self.definition(body.id).map_or(0.0, |d| {
            d.height(direction)
                .expect("normalized body-local direction")
        })
    }
    pub fn sample(self, body: CelestialBody, direction: [f64; 3]) -> SurfaceSample {
        if let Some(definition) = self.definition(body.id) {
            definition
                .sample(direction)
                .expect("caller supplies normalized body-local direction")
        } else {
            SurfaceSample {
                elevation_meters: 0.0,
                body_local_position_meters: direction.map(|v| v * body.radius_meters),
                outward_normal: direction,
                slope: 0.0,
            }
        }
    }
    pub fn elevation_bounds(self, body: CelestialBody) -> [f64; 2] {
        self.definition(body.id)
            .map_or([0.0; 2], TerrainDefinition::elevation_bounds_meters)
    }
    /// Signed radial clearance. Normal contact uses this distance times radial·normal.
    pub fn clearance(self, body: CelestialBody, position: [f64; 3]) -> f64 {
        let local: [f64; 3] = std::array::from_fn(|i| position[i] - body.center.meters()[i]);
        let distance = local[0].hypot(local[1]).hypot(local[2]);
        let direction = if distance > 0.0 {
            local.map(|v| v / distance)
        } else {
            [0.0, 1.0, 0.0]
        };
        distance - body.radius_meters - self.height(body, direction)
    }
}

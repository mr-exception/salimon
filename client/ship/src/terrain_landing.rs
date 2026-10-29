//! Footprint-aware landing policy. Runtime supplies authored support geometry.
use crate::orientation::{normalize_or, surface_aligned_orientation};
use crate::{LANDED_HULL_CLEARANCE_METERS, LANDED_ORIGIN_HEIGHT_METERS, ShipPose};
use salimon_math::{add, dot, scale, sub};
use salimon_world::{CelestialBody, terrain::TerrainMode};

/// Fixed feet and ramp cannot conform arbitrarily; reject sites rather than burying them.
pub const MAX_SUPPORT_GAP_METERS: f64 = 0.10;
pub const MAX_LANDING_SLOPE: f64 = 0.363_970_234_266_202_34; // 20 degrees
#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct LandingFootprint {
    /// Sole corners and intermediate samples in ship-local metres.
    pub feet: Vec<[f64; 3]>,
    pub ramp: Vec<[f64; 3]>,
    /// Conservative lowest hull envelope samples, excluding the gear.
    pub hull: Vec<[f64; 3]>,
}
pub(crate) fn landing_pose(
    body: CelestialBody,
    direction: [f64; 3],
    forward: [f64; 3],
    mode: TerrainMode,
    footprint: &LandingFootprint,
) -> Option<ShipPose> {
    let sample = mode.sample(body, direction);
    if sample.slope > MAX_LANDING_SLOPE
        || footprint.feet.is_empty()
        || footprint.ramp.is_empty()
        || footprint.hull.is_empty()
    {
        return None;
    }
    let normal = sample.outward_normal;
    let orientation = surface_aligned_orientation(normal, forward);
    let axes = ShipPose {
        position_meters: [0.0; 3],
        orientation,
    }
    .axes();
    let radial_dot = dot(normal, direction);
    let mut local_origin = add(
        sample.body_local_position_meters,
        scale(direction, LANDED_ORIGIN_HEIGHT_METERS / radial_dot),
    );
    let transform = |origin, p: [f64; 3]| {
        add(
            origin,
            add(
                scale(axes[0], p[0]),
                add(scale(axes[1], p[1]), scale(axes[2], p[2])),
            ),
        )
    };
    let gap = |p: [f64; 3]| {
        let d = normalize_or(p, direction);
        let ground = mode.sample(body, d);
        dot(sub(p, ground.body_local_position_meters), normal)
    };
    // Move out enough for every fixed sole/ramp point and the full conservative hull.
    let correction = footprint
        .feet
        .iter()
        .chain(&footprint.ramp)
        .map(|p| -gap(transform(local_origin, *p)))
        .chain(
            footprint
                .hull
                .iter()
                .map(|p| LANDED_HULL_CLEARANCE_METERS - gap(transform(local_origin, *p))),
        )
        .fold(0.0_f64, f64::max);
    local_origin = add(
        local_origin,
        scale(direction, (correction + 0.0005) / radial_dot),
    );
    if footprint
        .feet
        .iter()
        .chain(&footprint.ramp)
        .any(|p| gap(transform(local_origin, *p)) > MAX_SUPPORT_GAP_METERS)
        || footprint
            .hull
            .iter()
            .any(|p| gap(transform(local_origin, *p)) < LANDED_HULL_CLEARANCE_METERS - 0.001)
    {
        return None;
    }
    Some(ShipPose {
        position_meters: add(body.center.meters(), local_origin),
        orientation,
    })
}

impl crate::ShipController {
    /// Select terrain before simulation; changing a live definition/session is not supported.
    pub fn configure_terrain(&mut self, mode: TerrainMode, footprint: LandingFootprint) {
        self.terrain = mode;
        self.landing_footprint = Some(footprint);
    }
    /// Bounded deterministic spawn search. Failure is explicit, never an unsafe touchdown.
    pub fn spawn_on_terrain(
        &mut self,
        body: CelestialBody,
        direction: [f64; 3],
    ) -> Result<(), &'static str> {
        let footprint = self
            .landing_footprint
            .as_ref()
            .ok_or("missing landing footprint")?;
        let pose = nearby_landing_pose(body, direction, self.terrain, footprint)
            .ok_or("no safe landing site within the bounded spawn search")?;
        if !self.external_site_clear(pose) {
            return Err("physical organism blocks full hull clearance");
        };
        self.pose = pose;
        self.flight_state = crate::FlightState::Landed { body: body.id };
        self.assist = None;
        self.thruster_percentage = 0;
        Ok(())
    }
}

/// Fixed bounded search shared by spawn and powered crash recovery. No mutation.
pub(crate) fn nearby_landing_pose(
    body: CelestialBody,
    direction: [f64; 3],
    mode: TerrainMode,
    footprint: &LandingFootprint,
) -> Option<ShipPose> {
    let axis = if direction[0].abs() < 0.8 {
        [1.0, 0.0, 0.0]
    } else {
        [0.0, 1.0, 0.0]
    };
    let tangent = normalize_or(salimon_math::cross(direction, axis), [0.0, 0.0, 1.0]);
    let side = salimon_math::cross(direction, tangent);
    for ring in 0_i32..=20 {
        for u in -ring..=ring {
            for v in -ring..=ring {
                if ring > 0 && u.abs().max(v.abs()) != ring {
                    continue;
                }
                let candidate = normalize_or(
                    add(
                        scale(direction, body.radius_meters),
                        add(
                            scale(tangent, f64::from(u) * 25.0),
                            scale(side, f64::from(v) * 25.0),
                        ),
                    ),
                    direction,
                );
                if let Some(pose) = landing_pose(body, candidate, tangent, mode, footprint) {
                    return Some(pose);
                }
            }
        }
    }
    None
}

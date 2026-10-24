//! Continuous segment contact against catalog solids and version-1 heightfields.
use super::{ShipController, assist::body_definition};
use crate::orientation::normalize_or;
use crate::{COLLISION_RADIUS_METERS, FlightState, SteeringInput};
use salimon_math::{add, dot, length, scale, sub};
#[cfg(test)]
use salimon_world::CELESTIAL_BODIES;
use salimon_world::{
    BodyRole, CelestialBody, CelestialBodyId,
    terrain::{LayerKind, TerrainMode},
};
use std::time::Duration;

const CONTACT_TOLERANCE_METERS: f64 = 0.001;
const MAX_SWEEP_INTERVALS: usize = 4096;
const RESTITUTION: f64 = 0.35;
const MAX_REBOUND_SPEED: f64 = 8.0;
// Local contact settling response; this is not gravity during free flight.
const SETTLING_ACCELERATION: f64 = 10.0;
const MIN_REBOUND_SPEED: f64 = 0.1;
const MAX_REBOUNDS: u8 = 4;

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub(super) struct Bounce {
    anchor: [f64; 3],
    normal: [f64; 3],
    speed: f64,
    elapsed: Duration,
    duration: Duration,
    rebounds: u8,
}

/// Stable body-local line/sphere interval, avoiding the b² - ac cancellation
/// of a quadratic formed from a distant high-speed endpoint.
fn sphere_interval(start: [f64; 3], movement: [f64; 3], radius: f64) -> Option<[f64; 2]> {
    let distance = movement[0].hypot(movement[1]).hypot(movement[2]);
    if distance == 0.0 {
        return (length(start) <= radius).then_some([0.0, 0.0]);
    }
    let direction = scale(movement, 1.0 / distance);
    let along = -dot(start, direction);
    let perpendicular = add(start, scale(direction, along));
    let squared = dot(perpendicular, perpendicular);
    if squared > radius * radius {
        return None;
    }
    let half = (radius * radius - squared).max(0.0).sqrt();
    let enter = (along - half).max(0.0) / distance;
    let exit = (along + half).min(distance) / distance;
    (enter <= exit && exit >= 0.0 && enter <= 1.0).then_some([enter, exit])
}

/// Bound the Cartesian height gradient of generator v1. Smooth lattice noise
/// has per-axis derivative <=3A/w; folded ridges double it. Sparse smoothstep
/// features contribute <=1.5|elevation|/radius. No terrain tessellation or scan.
fn clearance_lipschitz(mode: TerrainMode, body: CelestialBody) -> f64 {
    let Some(definition) = mode.definition(body.id) else {
        return 1.0;
    };
    let gradient = definition
        .layers()
        .iter()
        .map(|layer| {
            let fold = if layer.kind == LayerKind::Ridges {
                2.0
            } else {
                1.0
            };
            3.0 * 3.0_f64.sqrt() * fold * layer.amplitude_meters / layer.wavelength_meters
        })
        .sum::<f64>()
        + definition
            .features()
            .iter()
            .map(|feature| 1.5 * feature.elevation_meters.abs() / feature.radius_meters)
            .sum::<f64>();
    let inner =
        body.radius_meters + definition.elevation_bounds_meters()[0] + COLLISION_RADIUS_METERS;
    1.0 + gradient * body.radius_meters / inner
}

fn terrain_contact(
    mode: TerrainMode,
    body: CelestialBody,
    start: [f64; 3],
    end: [f64; 3],
) -> Option<f64> {
    let center = body.center.meters();
    let local = sub(start, center);
    let movement = sub(end, start);
    let bounds = mode.elevation_bounds(body);
    let [enter, mut exit] = sphere_interval(
        local,
        movement,
        body.radius_meters + bounds[1] + COLLISION_RADIUS_METERS,
    )?;
    if mode.definition(body.id).is_none() {
        return Some(enter);
    }
    // Search only the uncertain terrain shell; entry into its inner sphere is
    // guaranteed contact and keeps the gradient bound away from r=0.
    if let Some([inner, _]) = sphere_interval(
        local,
        movement,
        body.radius_meters + bounds[0] + COLLISION_RADIUS_METERS,
    ) {
        exit = exit.min(inner.max(enter));
    }
    let clearance = |t| {
        let p = add(local, scale(movement, t));
        let radius = p[0].hypot(p[1]).hypot(p[2]);
        radius
            - body.radius_meters
            - mode.height(body, normalize_or(p, [0.0, 1.0, 0.0]))
            - COLLISION_RADIUS_METERS
    };
    let travel = movement[0].hypot(movement[1]).hypot(movement[2]);
    let lipschitz = clearance_lipschitz(mode, body);
    let mut intervals = vec![(enter, exit, clearance(enter), clearance(exit))];
    let mut visited = 0;
    while let Some((a, b, fa, fb)) = intervals.pop() {
        let width = (b - a) * travel;
        // Every point is within half the interval of an endpoint. This bound
        // rejects clear intervals even when both endpoints surround a hill.
        if fa.min(fb) - lipschitz * width * 0.5 > CONTACT_TOLERANCE_METERS {
            continue;
        }
        if fa <= CONTACT_TOLERANCE_METERS || width <= CONTACT_TOLERANCE_METERS {
            return Some(a);
        }
        visited += 1;
        if visited == MAX_SWEEP_INTERVALS {
            // Conservative fail-closed contact on the earliest unresolved interval;
            // work stays bounded and a high-speed crossing cannot tunnel through.
            return Some(a);
        }
        let mid = (a + b) * 0.5;
        let fm = clearance(mid);
        intervals.push((mid, b, fm, fb));
        intervals.push((a, mid, fa, fm));
    }
    None
}

impl ShipController {
    pub(super) fn sweep_contact(
        &mut self,
        start: [f64; 3],
        end: [f64; 3],
        initial_velocity: [f64; 3],
        delta: Duration,
    ) {
        let mut first: Option<(f64, CelestialBodyId)> = None;
        for body in self
            .terrain
            .bodies()
            .iter()
            .filter(|body| body.role == BodyRole::Solid)
        {
            if let Some(fraction) = terrain_contact(self.terrain, *body, start, end)
                && first.is_none_or(|(previous, _)| fraction < previous)
            {
                first = Some((fraction, body.id));
            }
        }
        let Some((fraction, body)) = first else {
            return;
        };
        self.pose.position_meters = add(start, scale(sub(end, start), fraction));
        let definition = *body_definition(body);
        // Keep the established conservative sphere's local-normal clearance,
        // including f64 quantization at the catalog's 1e12 m anchor.
        let mut normal = self.pose.axes()[1];
        for _ in 0..8 {
            let local = sub(self.pose.position_meters, definition.center.meters());
            let ground = self.terrain.sample(definition, normalize_or(local, normal));
            normal = ground.outward_normal;
            let gap = dot(sub(local, ground.body_local_position_meters), normal);
            let correction = COLLISION_RADIUS_METERS - gap;
            if correction.abs() <= 0.0005 {
                break;
            }
            self.pose.position_meters = add(
                self.pose.position_meters,
                scale(normal, correction + 0.0002),
            );
        }
        let impact_velocity = add(
            initial_velocity,
            scale(sub(self.velocity, initial_velocity), fraction),
        );
        let speed = (-dot(impact_velocity, normal) * RESTITUTION).clamp(0.0, MAX_REBOUND_SPEED);
        self.thruster_percentage = 0;
        self.clear_throttle_input();
        self.steering_target = SteeringInput::default();
        self.steering = SteeringInput::default();
        self.assist = None;
        self.cockpit_message = None;
        self.bounce = None;
        if speed < MIN_REBOUND_SPEED {
            self.velocity = [0.0; 3];
            self.flight_state = FlightState::GroundedAfterCrash { body };
        } else {
            self.bounce = Some(Bounce {
                anchor: self.pose.position_meters,
                normal,
                speed,
                elapsed: Duration::ZERO,
                duration: Duration::from_secs_f64(2.0 * speed / SETTLING_ACCELERATION),
                rebounds: 1,
            });
            self.flight_state = FlightState::Bouncing { body };
            self.velocity = scale(normal, speed);
            self.advance_bounce(body, delta.mul_f64((1.0 - fraction).clamp(0.0, 1.0)));
        }
    }

    pub(super) fn advance_bounce(&mut self, body: CelestialBodyId, mut remaining: Duration) {
        while let Some(bounce) = &mut self.bounce {
            let consumed = remaining.min(bounce.duration - bounce.elapsed);
            bounce.elapsed += consumed;
            remaining -= consumed;
            let seconds = bounce.elapsed.as_secs_f64();
            let height =
                (bounce.speed * seconds - 0.5 * SETTLING_ACCELERATION * seconds * seconds).max(0.0);
            self.pose.position_meters = add(bounce.anchor, scale(bounce.normal, height));
            self.velocity = scale(
                bounce.normal,
                bounce.speed - SETTLING_ACCELERATION * seconds,
            );
            if bounce.elapsed < bounce.duration {
                break;
            }
            self.pose.position_meters = bounce.anchor;
            bounce.speed *= RESTITUTION;
            if bounce.speed < MIN_REBOUND_SPEED || bounce.rebounds == MAX_REBOUNDS {
                self.velocity = [0.0; 3];
                self.flight_state = FlightState::GroundedAfterCrash { body };
                self.bounce = None;
                break;
            }
            bounce.rebounds += 1;
            bounce.elapsed = Duration::ZERO;
            bounce.duration = Duration::from_secs_f64(2.0 * bounce.speed / SETTLING_ACCELERATION);
            self.velocity = scale(bounce.normal, bounce.speed);
            if remaining.is_zero() {
                break;
            }
        }
    }
}

impl Bounce {
    pub(super) fn validate_save(&self) -> Result<(), salimon_persistence::SaveError> {
        if self
            .anchor
            .iter()
            .chain(self.normal.iter())
            .any(|v| !v.is_finite())
            || !self.speed.is_finite()
            || self.speed < 0.
            || self.elapsed > self.duration
            || self.rebounds > MAX_REBOUNDS
        {
            return Err(salimon_persistence::invalid("bounce"));
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{CockpitMessage, ShipPose};

    fn falling(
        body: CelestialBody,
        clearance: f64,
        speed: f64,
        mode: TerrainMode,
    ) -> ShipController {
        let normal = [0.0, 1.0, 0.0];
        let ground = mode.sample(body, normal);
        let position = add(
            body.center.meters(),
            add(ground.body_local_position_meters, scale(normal, clearance)),
        );
        let mut ship = ShipController::flying_with_velocity(
            ShipPose {
                position_meters: position,
                orientation: [0.0, 0.0, 0.0, 1.0],
            },
            25,
            [0.0, -speed, 0.0],
        )
        .unwrap();
        ship.terrain = mode;
        ship
    }

    #[test]
    fn high_speed_sweeps_every_solid_even_when_endpoint_is_outside_far_side() {
        for body in CELESTIAL_BODIES
            .iter()
            .filter(|body| body.role == BodyRole::Solid)
        {
            let velocity = (2.0 * body.radius_meters + 10_000.0) / 0.016;
            let mut ship = falling(
                *body,
                COLLISION_RADIUS_METERS + 1000.0,
                velocity,
                TerrainMode::Flat,
            );
            ship.advance(Duration::from_millis(16));
            assert_eq!(
                ship.snapshot().flight_state,
                FlightState::Bouncing { body: body.id }
            );
            assert_eq!(ship.snapshot().thruster_percentage, 0);
            let local = sub(ship.pose.position_meters, body.center.meters());
            assert!(local[1] >= body.radius_meters + COLLISION_RADIUS_METERS - 0.001);
            assert!(local[1] < body.radius_meters + COLLISION_RADIUS_METERS + 4.0);
            assert!(ship.velocity[1] > 0.0);
        }
    }

    #[test]
    fn low_speed_contact_decays_and_powerless_crash_stays_grounded() {
        let earth = *body_definition(CelestialBodyId::Earth);
        let mut ship = falling(earth, COLLISION_RADIUS_METERS + 1.0, 2.0, TerrainMode::Flat);
        ship.advance(Duration::from_millis(600));
        assert!(matches!(ship.flight_state, FlightState::Bouncing { .. }));
        assert!(ship.velocity[1].abs() <= 0.7);
        let before = ship.pose;
        ship.trigger_landing_action();
        assert_eq!(ship.pose, before);
        assert!(!ship.adjust_thruster(1));
        assert_eq!(
            ship.contextual_cockpit_message(),
            Some(CockpitMessage::CollisionSettling)
        );
        ship.advance(Duration::from_secs(5));
        assert_eq!(
            ship.flight_state,
            FlightState::GroundedAfterCrash { body: earth.id }
        );
        assert_eq!(ship.velocity_meters_per_second(), [0.0; 3]);
        assert_eq!(ship.speed_meters_per_second(), 0.0);
        let resting = ship.pose;
        ship.set_stored_core_energy_joules(0);
        ship.trigger_landing_action();
        ship.advance(Duration::from_secs(20));
        assert_eq!(ship.pose, resting);
        assert_eq!(
            ship.flight_state,
            FlightState::GroundedAfterCrash { body: earth.id }
        );
        assert_eq!(ship.contextual_cockpit_message(), None);
    }

    #[test]
    fn captured_bounce_partitions_match_and_rebound_count_is_bounded() {
        let earth = *body_definition(CelestialBodyId::Earth);
        let mut whole = falling(
            earth,
            COLLISION_RADIUS_METERS + 1.0,
            1000.0,
            TerrainMode::Flat,
        );
        whole.advance(Duration::from_millis(16));
        let mut split = whole.clone();
        whole.advance(Duration::from_secs(1));
        for _ in 0..100 {
            split.advance(Duration::from_millis(10));
        }
        assert_eq!(whole.snapshot(), split.snapshot());
        let first_speed = whole.bounce.as_ref().unwrap().speed;
        whole.advance(Duration::from_secs(1));
        if let Some(bounce) = &whole.bounce {
            assert!(bounce.speed < first_speed);
            assert!(bounce.rebounds <= MAX_REBOUNDS);
        }
        whole.advance(Duration::from_secs(1));
        assert!(matches!(
            whole.flight_state,
            FlightState::GroundedAfterCrash { .. }
        ));
    }

    #[test]
    fn representative_terrain_contact_preserves_local_normal_clearance() {
        for id in [
            CelestialBodyId::Earth,
            CelestialBodyId::Moon,
            CelestialBodyId::Mars,
        ] {
            let body = *body_definition(id);
            let mut ship = falling(
                body,
                10_000.0,
                2.0 * body.radius_meters / 0.016,
                TerrainMode::Representative,
            );
            ship.advance(Duration::from_millis(16));
            assert!(matches!(ship.flight_state, FlightState::Bouncing { .. }));
            ship.advance(Duration::from_secs(5));
            let local = sub(ship.pose.position_meters, body.center.meters());
            let ground = ship
                .terrain
                .sample(body, normalize_or(local, [0.0, 1.0, 0.0]));
            let gap = dot(
                sub(local, ground.body_local_position_meters),
                ground.outward_normal,
            );
            assert!(
                (gap - COLLISION_RADIUS_METERS).abs() <= 0.001,
                "{id:?}: {gap}"
            );
            assert_eq!(
                ship.flight_state,
                FlightState::GroundedAfterCrash { body: id }
            );
        }
    }

    #[test]
    fn tangent_hill_sweep_does_not_depend_on_endpoint_penetration() {
        let body = *body_definition(CelestialBodyId::Moon);
        let mode = TerrainMode::Representative;
        // Find a deterministic hill above both endpoints of a long tangent segment.
        let mut found = false;
        for z in (0..4000).step_by(100) {
            let line = |x: f64| [x, body.radius_meters, f64::from(z)];
            let radial_surface = |p| {
                let direction = normalize_or(p, [0.0, 1.0, 0.0]);
                mode.height(body, direction) - (length(p) - body.radius_meters)
            };
            let a = line(-2000.0);
            let b = line(2000.0);
            let middle = line(0.0);
            let endpoint_height = radial_surface(a).max(radial_surface(b));
            if radial_surface(middle) <= endpoint_height + 0.1 {
                continue;
            }
            let y = COLLISION_RADIUS_METERS + (endpoint_height + radial_surface(middle)) * 0.5;
            let start = add(body.center.meters(), add(a, [0.0, y, 0.0]));
            let end = add(body.center.meters(), add(b, [0.0, y, 0.0]));
            assert!(mode.clearance(body, start) > COLLISION_RADIUS_METERS);
            assert!(mode.clearance(body, end) > COLLISION_RADIUS_METERS);
            assert!(terrain_contact(mode, body, start, end).is_some());
            found = true;
            break;
        }
        assert!(
            found,
            "representative fixture contains a hill along the sweep"
        );
    }

    #[test]
    fn clear_pass_and_visual_sun_do_not_collide() {
        let earth = *body_definition(CelestialBodyId::Earth);
        let start = add(
            earth.center.meters(),
            [-1000.0, earth.radius_meters + 1000.0, 0.0],
        );
        let end = add(start, [2000.0, 0.0, 0.0]);
        assert!(terrain_contact(TerrainMode::Representative, earth, start, end).is_none());
        let sun = CELESTIAL_BODIES
            .iter()
            .find(|body| body.role != BodyRole::Solid)
            .unwrap();
        let mut ship = ShipController::flying_with_velocity(
            ShipPose {
                position_meters: sun.center.meters(),
                orientation: [0.0, 0.0, 0.0, 1.0],
            },
            0,
            [10.0, 0.0, 0.0],
        )
        .unwrap();
        ship.advance(Duration::from_secs(1));
        assert_eq!(ship.flight_state, FlightState::Flying);
        assert_eq!(ship.velocity, [10.0, 0.0, 0.0]);
    }
}

//! Runtime composition of the shared terrain definition and renderer-neutral patches.
use salimon_character::{GroundSample, SurfaceFrame};
use salimon_math::{add, length, scale, sub};
use salimon_world::{CELESTIAL_BODIES, CelestialBody, terrain::TerrainMode};

pub(crate) fn body_at(center: [f64; 3], radius: f64) -> CelestialBody {
    *CELESTIAL_BODIES
        .iter()
        .chain(salimon_world::canonical::BODIES.iter())
        .find(|b| b.center.meters() == center && b.radius_meters == radius)
        .expect("ground frames originate in the immutable catalog")
}
pub(crate) fn character_ground(frame: SurfaceFrame, world: [f64; 3]) -> GroundSample {
    let body = body_at(frame.body_center_meters, frame.radius_meters);
    let local = sub(world, body.center.meters());
    let direction = scale(local, 1.0 / length(local));
    let sample = (if body.id == salimon_world::CelestialBodyId::Vesper {
        TerrainMode::Canonical
    } else {
        TerrainMode::Representative
    })
    .sample(body, direction);
    GroundSample {
        position_meters: add(body.center.meters(), sample.body_local_position_meters),
        normal: sample.outward_normal,
        slope: sample.slope,
    }
}
// Exact memoization only: keys retain every input bit. Fixed definitions never change
// during a session, and FIFO eviction changes cost without changing a returned sample.
struct GroundCache {
    entries: std::collections::HashMap<[u64; 7], salimon_physics::GroundSample>,
    order: std::collections::VecDeque<[u64; 7]>,
    hits: u64,
    misses: u64,
}
impl Default for GroundCache {
    fn default() -> Self {
        Self {
            entries: std::collections::HashMap::with_capacity(4096),
            order: std::collections::VecDeque::with_capacity(4096),
            hits: 0,
            misses: 0,
        }
    }
}
thread_local! { static GROUND_CACHE: std::cell::RefCell<GroundCache> = std::cell::RefCell::new(GroundCache::default()); }
pub(crate) fn ground_cache_stats() -> (usize, u64, u64) {
    GROUND_CACHE.with(|c| {
        let c = c.borrow();
        (c.entries.len(), c.hits, c.misses)
    })
}
pub(crate) fn physics_ground(
    frame: salimon_physics::SphereSurface,
    world: [f64; 3],
) -> salimon_physics::GroundSample {
    let key = [
        frame.center[0].to_bits(),
        frame.center[1].to_bits(),
        frame.center[2].to_bits(),
        frame.radius.to_bits(),
        world[0].to_bits(),
        world[1].to_bits(),
        world[2].to_bits(),
    ];
    GROUND_CACHE.with(|cache| {
        let mut cache = cache.borrow_mut();
        if let Some(sample) = cache.entries.get(&key).copied() {
            cache.hits += 1;
            return sample;
        }
        let ground = character_ground(
            SurfaceFrame {
                body_center_meters: frame.center,
                radius_meters: frame.radius,
                query: None,
            },
            world,
        );
        let result = salimon_physics::GroundSample {
            point: ground.position_meters,
            normal: ground.normal,
        };
        if cache.entries.len() == 4096
            && let Some(old) = cache.order.pop_front()
        {
            cache.entries.remove(&old);
        }
        cache.entries.insert(key, result);
        cache.order.push_back(key);
        cache.misses += 1;
        result
    })
}
pub(crate) fn physics_ground_point(
    frame: salimon_physics::SphereSurface,
    world: [f64; 3],
) -> [f64; 3] {
    let body = body_at(frame.center, frame.radius);
    let local = sub(world, frame.center);
    let direction = scale(local, 1.0 / length(local));
    add(
        frame.center,
        scale(
            direction,
            frame.radius + TerrainMode::Representative.height(body, direction),
        ),
    )
}
/// Continuous distant-envelope transition finishes before the local cap can load.
pub(crate) fn far_radius(body: CelestialBody, camera: [f64; 3], mode: TerrainMode) -> f64 {
    let altitude = length(sub(camera, body.center.meters())) - body.radius_meters;
    let t = ((50_000.0 - altitude) / 30_000.0).clamp(0.0, 1.0);
    let blend = t * t * (3.0 - 2.0 * t);
    body.radius_meters + mode.elevation_bounds(body)[0] * blend
}
pub(crate) fn landing_footprint() -> salimon_ship::LandingFootprint {
    salimon_ship::LandingFootprint {
        feet: salimon_character::landing_sole_samples(),
        ramp: salimon_character::ramp_tip_support_samples(),
        hull: salimon_character::hull_ground_samples(),
    }
}
impl crate::session::GameplaySession {
    pub(crate) fn terrain_frame(&self, ship: salimon_ship::ShipSnapshot) -> SurfaceFrame {
        let mut frame = crate::session::frames::surface_frame_for_ship(ship);
        if self.mining.terrain == TerrainMode::Representative {
            frame.query = Some(character_ground);
        }
        frame
    }
    pub(crate) fn nearby_terrain(&self, eye: [f64; 3]) -> Option<SurfaceFrame> {
        let mode = self.mining.terrain;
        mode.nearest_body(
            eye,
            salimon_world::NEARBY_BODY_MAX_SURFACE_DISTANCE_METERS,
            true,
        )
        .map(|(body, _)| SurfaceFrame {
            body_center_meters: body.center.meters(),
            radius_meters: body.radius_meters,
            query: (mode == TerrainMode::Representative).then_some(character_ground),
        })
    }
    pub(crate) fn enable_terrain(
        &mut self,
        body: CelestialBody,
        direction: [f64; 3],
    ) -> Result<(), String> {
        let mut ship = self.ship.clone();
        ship.configure_terrain(TerrainMode::Representative, landing_footprint());
        ship.spawn_on_terrain(body, direction)
            .map_err(str::to_owned)?;
        self.ship = ship;
        self.mining.terrain = TerrainMode::Representative;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use salimon_ship::{CockpitMessage, FlightState, ShipController, ShipPose};
    use std::time::Duration;
    fn footprint() -> salimon_ship::LandingFootprint {
        salimon_ship::LandingFootprint {
            feet: salimon_character::landing_sole_samples(),
            ramp: salimon_character::ramp_tip_support_samples(),
            hull: salimon_character::hull_ground_samples(),
        }
    }
    #[test]
    fn every_profile_has_safe_plain_and_crater_spawn_with_authored_support() {
        for id in [
            salimon_world::CelestialBodyId::Earth,
            salimon_world::CelestialBodyId::Moon,
            salimon_world::CelestialBodyId::Mars,
        ] {
            let body = *CELESTIAL_BODIES.iter().find(|b| b.id == id).unwrap();
            for d in [[0.0, 1.0, 0.0], [0.0, 0.0, 1.0]] {
                let mut app = crate::session::GameplaySession::default();
                app.enable_terrain(body, d).unwrap();
                let ship = app.ship.snapshot();
                let frame = crate::session::frames::character_ship_frame(ship.pose);
                let support = footprint();
                let mut max_gap = 0.0_f64;
                for p in support.feet.iter().chain(&support.ramp) {
                    let world = frame.local_to_world(*p);
                    let ground = app.terrain_frame(ship).sample(world);
                    let gap =
                        salimon_math::dot(sub(world, ground.position_meters), ship.pose.axes()[1]);
                    assert!((-0.001..=salimon_ship::MAX_SUPPORT_GAP_METERS + 0.001).contains(&gap));
                    max_gap = max_gap.max(gap);
                }
                for p in support.hull {
                    let world = frame.local_to_world(p);
                    let ground = app.terrain_frame(ship).sample(world);
                    assert!(
                        salimon_math::dot(sub(world, ground.position_meters), ship.pose.axes()[1])
                            >= 1.999
                    );
                }
                eprintln!(
                    "{} {:?} safe spawn: max fixed support gap {max_gap} m",
                    body.name, d
                );
            }
        }
    }
    #[test]
    fn terrain_touchdown_is_partition_independent_and_uncancellable() {
        let body = *CELESTIAL_BODIES
            .iter()
            .find(|b| b.id == salimon_world::CelestialBodyId::Moon)
            .unwrap();
        let mut app = crate::session::GameplaySession::default();
        app.enable_terrain(body, [0.0, 0.0, 1.0]).unwrap();
        let landed = app.ship.snapshot().pose;
        let d = sub(landed.position_meters, body.center.meters());
        let radial = scale(d, 1.0 / length(d));
        let mut ship = ShipController::flying(
            ShipPose {
                position_meters: add(landed.position_meters, scale(radial, 1000.0)),
                orientation: landed.orientation,
            },
            0,
        );
        ship.configure_terrain(TerrainMode::Representative, footprint());
        ship.trigger_landing_action();
        assert!(matches!(
            ship.snapshot().flight_state,
            FlightState::AssistedLanding { .. }
        ));
        let mut split = ship.clone();
        ship.set_cockpit_control(false);
        split.set_cockpit_control(false);
        ship.advance(Duration::from_secs(8));
        for _ in 0..500 {
            split.trigger_landing_action();
            split.advance(Duration::from_millis(16));
        }
        assert_eq!(ship.snapshot(), split.snapshot());
        assert!(
            length(sub(
                ship.snapshot().pose.position_meters,
                landed.position_meters
            )) < 0.001
        );
        ship.set_cockpit_control(true);
        ship.trigger_landing_action();
        ship.advance(Duration::from_secs(6));
        assert_eq!(ship.snapshot().flight_state, FlightState::Flying);
    }
    #[test]
    fn crashed_ship_repositions_from_unsafe_crater_and_lands_on_supported_feet() {
        let body = *CELESTIAL_BODIES
            .iter()
            .find(|b| b.id == salimon_world::CelestialBodyId::Moon)
            .unwrap();
        let direction = scale(
            [100.0, 0.0, body.radius_meters],
            1.0 / length([100.0, 0.0, body.radius_meters]),
        );
        let mode = TerrainMode::Representative;
        let height = mode.sample(body, direction).elevation_meters;
        let mut ship = ShipController::flying_with_velocity(
            ShipPose {
                position_meters: body
                    .center
                    .translated(scale(direction, body.radius_meters + height + 100.0))
                    .meters(),
                orientation: [0.0, 0.0, 0.0, 1.0],
            },
            0,
            scale(direction, -1000.0),
        )
        .unwrap();
        ship.configure_terrain(mode, footprint());
        ship.advance(Duration::from_secs(1));
        ship.advance(Duration::from_secs(5));
        assert!(matches!(
            ship.snapshot().flight_state,
            FlightState::GroundedAfterCrash { .. }
        ));
        let crashed = ship.snapshot().pose;
        ship.trigger_landing_action();
        assert!(matches!(
            ship.snapshot().flight_state,
            FlightState::AssistedLanding { .. }
        ));
        let mut whole = ship.clone();
        whole.advance(Duration::from_secs(12));
        for _ in 0..1200 {
            ship.advance(Duration::from_millis(10));
        }
        assert_eq!(ship.snapshot(), whole.snapshot());
        let landed = ship.snapshot();
        assert_eq!(landed.flight_state, FlightState::Landed { body: body.id });
        let start_direction = scale(
            sub(crashed.position_meters, body.center.meters()),
            1.0 / length(sub(crashed.position_meters, body.center.meters())),
        );
        let end_direction = scale(
            sub(landed.pose.position_meters, body.center.meters()),
            1.0 / length(sub(landed.pose.position_meters, body.center.meters())),
        );
        assert!(length(sub(start_direction, end_direction)) > 1e-6);
        let frame = crate::session::frames::character_ship_frame(landed.pose);
        for local in footprint().feet.iter().chain(&footprint().ramp) {
            let world = frame.local_to_world(*local);
            let relative = sub(world, body.center.meters());
            let sample = mode.sample(body, scale(relative, 1.0 / length(relative)));
            let gap = salimon_math::dot(
                sub(relative, sample.body_local_position_meters),
                landed.pose.axes()[1],
            );
            assert!((-0.001..=salimon_ship::MAX_SUPPORT_GAP_METERS + 0.001).contains(&gap));
        }
    }

    #[test]
    fn steep_crater_footprints_reject_landing_without_starting_an_assist() {
        let body = *CELESTIAL_BODIES
            .iter()
            .find(|b| b.id == salimon_world::CelestialBodyId::Moon)
            .unwrap();
        let d = scale(
            [100.0, 0.0, body.radius_meters],
            1.0 / length([100.0, 0.0, body.radius_meters]),
        );
        let mut ship = ShipController::flying(
            ShipPose {
                position_meters: body
                    .center
                    .translated(scale(d, body.radius_meters + 1000.0))
                    .meters(),
                orientation: [0.0, 0.0, 0.0, 1.0],
            },
            0,
        );
        ship.configure_terrain(TerrainMode::Representative, footprint());
        let before = ship.snapshot().pose;
        ship.trigger_landing_action();
        assert_eq!(ship.snapshot().flight_state, FlightState::Flying);
        assert_eq!(ship.snapshot().pose, before);
        assert_eq!(
            ship.snapshot().cockpit_message,
            Some(CockpitMessage::UnsafeLandingSite)
        );
    }
}

#[cfg(test)]
mod cache_tests {
    use super::*;
    #[test]
    fn cached_normal_and_height_only_queries_agree_after_eviction_and_reordering() {
        let body = *CELESTIAL_BODIES
            .iter()
            .find(|b| b.id == salimon_world::CelestialBodyId::Earth)
            .unwrap();
        let frame = salimon_physics::SphereSurface {
            center: body.center.meters(),
            radius: body.radius_meters,
        };
        let points: Vec<_> = (0..5000)
            .map(|i| {
                body.center
                    .translated([
                        f64::from(i) * 0.3,
                        body.radius_meters + 10.0,
                        0.5 * f64::from(i),
                    ])
                    .meters()
            })
            .collect();
        let first: Vec<_> = points.iter().map(|p| physics_ground(frame, *p)).collect();
        for (p, expected) in points.iter().zip(&first).rev() {
            assert_eq!(physics_ground(frame, *p), *expected);
            assert_eq!(physics_ground_point(frame, *p), expected.point);
        }
        assert!(ground_cache_stats().0 <= 4096);
        let other = std::thread::spawn(move || {
            points
                .iter()
                .map(|p| physics_ground(frame, *p))
                .collect::<Vec<_>>()
        })
        .join()
        .unwrap();
        assert_eq!(first, other);
    }
}

#[cfg(test)]
mod flight_tests {
    use super::*;
    #[test]
    fn direct_flight_contact_uses_actual_negative_terrain_not_the_global_elevation_envelope() {
        let body = *CELESTIAL_BODIES
            .iter()
            .find(|b| b.id == salimon_world::CelestialBodyId::Mars)
            .unwrap();
        let sample = TerrainMode::Representative.sample(body, [0.0, 1.0, 0.0]);
        assert!(sample.elevation_meters < 0.0);
        let mut ship = salimon_ship::ShipController::flying(
            salimon_ship::ShipPose {
                position_meters: body
                    .center
                    .translated(add(sample.body_local_position_meters, [0.0, 1.0, 0.0]))
                    .meters(),
                orientation: [0.0, 0.0, 0.0, 1.0],
            },
            0,
        );
        ship.configure_terrain(TerrainMode::Representative, landing_footprint());
        ship.advance(std::time::Duration::ZERO);
        let local = sub(ship.snapshot().pose.position_meters, body.center.meters());
        let d = scale(local, 1.0 / length(local));
        let ground = TerrainMode::Representative.sample(body, d);
        let gap = salimon_math::dot(
            sub(local, ground.body_local_position_meters),
            ground.outward_normal,
        );
        assert!(
            (salimon_ship::COLLISION_RADIUS_METERS..=salimon_ship::COLLISION_RADIUS_METERS + 0.01)
                .contains(&gap)
        );
    }
}

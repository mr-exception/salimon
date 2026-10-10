//! Opt-in scenario initialization for native end-to-end runs.

use std::time::Duration;

use salimon_character::{CharacterController, CharacterLocation};
use salimon_ship::{FlightState, ShipController, ShipPose};
use salimon_world::{CELESTIAL_BODIES, CelestialBodyId};

use crate::app::ClientApplication;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum Scenario {
    LandedEarth,
    FragmentPile,
    CockpitEarth,
    OrbitEarth,
    ResourceApproach,
    OrbitMoon,
    OpenSpace,
    EvaApproach,
}

impl Scenario {
    fn parse(value: &str) -> Result<Self, String> {
        match value {
            "fragment-pile" => Ok(Self::FragmentPile),
            "landed-earth" => Ok(Self::LandedEarth),
            "cockpit-earth" => Ok(Self::CockpitEarth),
            "orbit-earth" => Ok(Self::OrbitEarth),
            "resource-approach" => Ok(Self::ResourceApproach),
            "orbit-moon" => Ok(Self::OrbitMoon),
            "open-space" => Ok(Self::OpenSpace),
            "eva-approach" => Ok(Self::EvaApproach),
            _ => Err(format!(
                "unknown scenario '{value}'; expected landed-earth, cockpit-earth, resource-approach, orbit-earth, orbit-moon, open-space, or eva-approach"
            )),
        }
    }

    pub(crate) const fn name(self) -> &'static str {
        match self {
            Self::FragmentPile => "fragment-pile",
            Self::LandedEarth => "landed-earth",
            Self::CockpitEarth => "cockpit-earth",
            Self::OrbitEarth => "orbit-earth",
            Self::ResourceApproach => "resource-approach",
            Self::OrbitMoon => "orbit-moon",
            Self::OpenSpace => "open-space",
            Self::EvaApproach => "eva-approach",
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct Config {
    pub(crate) scenario: Scenario,
    pub(crate) seed: u64,
    pub(crate) step: Duration,
}

pub(crate) fn parse_args(args: impl IntoIterator<Item = String>) -> Result<Option<Config>, String> {
    let mut args = args.into_iter();
    let mut enabled = false;
    let mut scenario = None;
    let mut seed = None;
    let mut step = None;
    while let Some(arg) = args.next() {
        if arg == "--e2e" {
            if enabled {
                return Err("--e2e specified twice".into());
            }
            enabled = true;
            continue;
        }
        let value = args
            .next()
            .ok_or_else(|| format!("{arg} requires a value"))?;
        match arg.as_str() {
            "--scenario" if scenario.is_none() => scenario = Some(Scenario::parse(&value)?),
            "--seed" if seed.is_none() => {
                seed = Some(
                    value
                        .parse::<u64>()
                        .map_err(|_| "--seed must be a decimal u64")?,
                );
            }
            "--step-ms" if step.is_none() => {
                let millis = value
                    .parse::<u64>()
                    .map_err(|_| "--step-ms must be an integer from 1 to 100")?;
                if !(1..=100).contains(&millis) {
                    return Err("--step-ms must be an integer from 1 to 100".into());
                }
                step = Some(Duration::from_millis(millis));
            }
            "--scenario" | "--seed" | "--step-ms" => return Err(format!("{arg} specified twice")),
            _ => return Err(format!("unknown argument '{arg}'")),
        }
    }
    if !enabled {
        return if scenario.is_some() || seed.is_some() || step.is_some() {
            Err("scenario options require --e2e".into())
        } else {
            Ok(None)
        };
    }
    Ok(Some(Config {
        scenario: scenario.unwrap_or(Scenario::LandedEarth),
        seed: seed.unwrap_or(0),
        step: step.unwrap_or(Duration::from_millis(16)),
    }))
}

/// Only alters initial conditions. Later movement, interaction and collision use
/// the regular character and ship controllers.
pub(crate) fn initialize(app: &mut ClientApplication, config: Config) -> Result<(), String> {
    app.character = CharacterController::default();
    app.ship = ShipController::default();
    match config.scenario {
        Scenario::LandedEarth => {}
        Scenario::FragmentPile => initialize_fragment_pile(app, config.seed),
        Scenario::CockpitEarth => app.character.enter_cockpit(),
        Scenario::ResourceApproach
        | Scenario::OrbitEarth
        | Scenario::OrbitMoon
        | Scenario::OpenSpace
        | Scenario::EvaApproach => {
            let id = if config.scenario != Scenario::OrbitMoon {
                CelestialBodyId::Earth
            } else {
                CelestialBodyId::Moon
            };
            let body = CELESTIAL_BODIES
                .iter()
                .find(|body| body.id == id)
                .ok_or_else(|| {
                    format!("scenario setup failed: {id:?} missing from world catalog")
                })?;
            let mut position = body.center.meters();
            position[1] += body.radius_meters
                + if config.scenario == Scenario::EvaApproach {
                    salimon_ship::NEARBY_BODY_MAX_SURFACE_DISTANCE_METERS + 200_000.0
                } else if config.scenario == Scenario::OpenSpace {
                    salimon_ship::NEARBY_BODY_MAX_SURFACE_DISTANCE_METERS + 10_000.0
                } else {
                    1_000.0
                };
            // Small reproducible tangent offset exercises distinct seeds without
            // changing the scenario's body or proximity contract.
            if config.scenario != Scenario::ResourceApproach {
                position[2] += (mix(config.seed) % 201) as f64 - 100.0;
            }
            app.ship = ShipController::flying(
                ShipPose {
                    position_meters: position,
                    orientation: if config.scenario == Scenario::EvaApproach {
                        [
                            0.0,
                            0.0,
                            -std::f64::consts::FRAC_1_SQRT_2,
                            std::f64::consts::FRAC_1_SQRT_2,
                        ]
                    } else {
                        [0.0, 0.0, 0.0, 1.0]
                    },
                },
                0,
            );
            app.character.enter_cockpit();
        }
    }
    if app.character.location() == CharacterLocation::Cockpit {
        app.ship.set_cockpit_control(true);
    }
    let snapshot = app.ship.snapshot();
    if matches!(
        config.scenario,
        Scenario::ResourceApproach
            | Scenario::OrbitEarth
            | Scenario::OrbitMoon
            | Scenario::OpenSpace
            | Scenario::EvaApproach
    ) != (snapshot.flight_state == FlightState::Flying)
    {
        return Err("scenario setup failed: unexpected ship flight state".into());
    }
    app.update_clock.set_fixed_step(config.step);
    app.e2e_config = Some(config);
    Ok(())
}

/// Deterministic initial loose objects; subsequent stepping uses production physics.
/// Seed parity selects cabin or planetary support. No gameplay protocol mutation.
fn initialize_fragment_pile(app: &mut ClientApplication, seed: u64) {
    use salimon_character::{MovementInput, ShipFrame, SurfaceFrame};
    use salimon_world::{WorldPosition, resources::{DepositId, RawMaterial, ResourceDeposit, ResourceId, ResourceTransform}};
    let earth = CELESTIAL_BODIES.iter().find(|b| b.id == CelestialBodyId::Earth).expect("Earth fixture");
    let ship = app.ship.snapshot();
    let frame = ShipFrame { origin_meters: ship.pose.position_meters, axes: ship.pose.axes() };
    let surface = SurfaceFrame { body_center_meters: earth.center.meters(), radius_meters: earth.radius_meters };
    let step = Duration::from_millis(16);
    for _ in 0..90 {
        app.character.advance(step, MovementInput { backward: true, ..Default::default() }, frame, surface, false, true);
    }
    for _ in 0..36 {
        app.character.advance(step, MovementInput { right: true, ..Default::default() }, frame, surface, false, true);
    }
    let outside = seed % 2 == 1;
    if outside {
        app.ship.toggle_door();
        app.ship.advance(Duration::from_secs(1));
        for _ in 0..160 {
            app.character.advance(step, MovementInput { backward: true, ..Default::default() }, frame, surface, true, true);
        }
        app.character.apply_mouse_delta(std::f64::consts::PI / 0.0022, 0.0);
    }
    app.character.apply_mouse_delta(0.0, 260.0);
    let eye = app.character.snapshot(frame, surface).eye_position_meters;
    let mut center = frame.world_to_local(eye);
    center[0] += if outside { -1.8 } else { 1.8 };
    center[1] = if outside {
        let world = frame.local_to_world(center);
        let radial = world.iter().zip(earth.center.meters()).map(|(a,b)| (a-b)*(a-b)).sum::<f64>().sqrt();
        center[1] - (radial - earth.radius_meters)
    } else { salimon_character::SHIP_FLOOR_HEIGHT_METERS };
    for (index, resource) in [ResourceId::IronOre, ResourceId::SilicateRock, ResourceId::WaterIce].into_iter().enumerate() {
        let world = frame.local_to_world(center);
        let mut deposit = ResourceDeposit::new(DepositId { body: earth.id, local: 9000 + index as u64 },
            RawMaterial::new(resource, 4.0).expect("positive fixture mass"), WorldPosition::new(world[0], world[1], world[2]), 4.0).expect("finite fixture deposit");
        app.mining.session.extract(&mut deposit, Duration::from_secs(2));
    }
    let pieces = app.mining.session.fragments().to_vec();
    for (index, piece) in pieces.into_iter().enumerate() {
        let local = [center[0] + (index % 2) as f64 * 0.08, center[1] + 0.5 + index as f64 * 0.45, center[2] + (index % 3) as f64 * 0.06];
        let world = frame.local_to_world(local);
        let pose = ResourceTransform::new(WorldPosition::new(world[0], world[1], world[2]), [0.0, 0.0, 0.0, 1.0]).expect("fixture pose");
        app.mining.session.move_loose(piece.id(), pose);
        if !outside { app.mining.ship_fragments.insert(piece.id(), local); }
        app.mining.fragment_motion.insert(piece.id(), crate::fragment_physics::FragmentMotion::default());
    }
}

fn mix(mut value: u64) -> u64 {
    value ^= value >> 30;
    value = value.wrapping_mul(0xbf58_476d_1ce4_e5b9);
    value ^= value >> 27;
    value = value.wrapping_mul(0x94d0_49bb_1331_11eb);
    value ^ (value >> 31)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::Instant;

    #[test]
    fn same_seed_produces_same_initial_state_and_fixed_updates() {
        let config = Config {
            scenario: Scenario::OrbitMoon,
            seed: 42,
            step: Duration::from_millis(20),
        };
        let mut a = ClientApplication::default();
        let mut b = ClientApplication::default();
        initialize(&mut a, config).unwrap();
        initialize(&mut b, config).unwrap();
        assert_eq!(a.ship.snapshot(), b.ship.snapshot());
        assert_eq!(a.character.location(), CharacterLocation::Cockpit);
        assert_ne!(
            a.ship.snapshot().pose.position_meters,
            ShipController::default().snapshot().pose.position_meters
        );
        let start = Instant::now();
        assert_eq!(a.update_clock.step(start), config.step);
        assert_eq!(
            a.update_clock.step(start + Duration::from_secs(2)),
            config.step
        );
        a.ship.advance(config.step);
        b.ship.advance(config.step);
        assert_eq!(a.ship.snapshot(), b.ship.snapshot());
    }

    #[test]
    fn scenarios_use_production_interactions_and_distinct_seeds() {
        let mut landed = ClientApplication::default();
        initialize(
            &mut landed,
            Config {
                scenario: Scenario::LandedEarth,
                seed: 0,
                step: Duration::from_millis(16),
            },
        )
        .unwrap();
        assert_eq!(landed.character.location(), CharacterLocation::InsideShip);
        landed.ship.toggle_door();
        assert_eq!(
            landed.ship.snapshot().door_state,
            salimon_ship::DoorState::Open
        );
        let mut cockpit = ClientApplication::default();
        initialize(
            &mut cockpit,
            Config {
                scenario: Scenario::CockpitEarth,
                seed: 0,
                step: Duration::from_millis(16),
            },
        )
        .unwrap();
        assert_eq!(cockpit.character.location(), CharacterLocation::Cockpit);
        let mut orbit = ClientApplication::default();
        initialize(
            &mut orbit,
            Config {
                scenario: Scenario::OrbitEarth,
                seed: 1,
                step: Duration::from_millis(16),
            },
        )
        .unwrap();
        let first = orbit.ship.snapshot().pose.position_meters;
        initialize(
            &mut orbit,
            Config {
                scenario: Scenario::OrbitEarth,
                seed: 2,
                step: Duration::from_millis(16),
            },
        )
        .unwrap();
        assert_ne!(first, orbit.ship.snapshot().pose.position_meters);
        orbit.ship.toggle_door();
        assert_eq!(
            orbit.ship.snapshot().door_state,
            salimon_ship::DoorState::Closed
        );
    }

    #[test]
    fn normal_launch_does_not_enable_hooks() {
        assert_eq!(parse_args([]).unwrap(), None);
        assert!(parse_args(["--scenario".into(), "orbit-earth".into()]).is_err());
        assert!(parse_args(["--e2e".into(), "--step-ms".into(), "0".into()]).is_err());
        let mut app = ClientApplication::default();
        assert_eq!(
            app.ship.snapshot().flight_state,
            FlightState::Landed {
                body: CelestialBodyId::Earth
            }
        );
        assert_eq!(app.update_clock.step(Instant::now()), Duration::ZERO);
    }
}

//! Opt-in scenario initialization for native end-to-end runs.

use std::time::Duration;

use salimon_character::{CharacterController, CharacterLocation};
use salimon_ship::{FlightState, ShipController, ShipPose};
use salimon_world::{CELESTIAL_BODIES, CelestialBodyId};

use crate::app::ClientApplication;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum Scenario {
    LandedEarth,
    CockpitEarth,
    OrbitEarth,
    OrbitMoon,
}

impl Scenario {
    fn parse(value: &str) -> Result<Self, String> {
        match value {
            "landed-earth" => Ok(Self::LandedEarth),
            "cockpit-earth" => Ok(Self::CockpitEarth),
            "orbit-earth" => Ok(Self::OrbitEarth),
            "orbit-moon" => Ok(Self::OrbitMoon),
            _ => Err(format!(
                "unknown scenario '{value}'; expected landed-earth, cockpit-earth, orbit-earth, or orbit-moon"
            )),
        }
    }

    pub(crate) const fn name(self) -> &'static str {
        match self {
            Self::LandedEarth => "landed-earth",
            Self::CockpitEarth => "cockpit-earth",
            Self::OrbitEarth => "orbit-earth",
            Self::OrbitMoon => "orbit-moon",
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
        let value = args.next().ok_or_else(|| format!("{arg} requires a value"))?;
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
        Scenario::CockpitEarth => app.character.enter_cockpit(),
        Scenario::OrbitEarth | Scenario::OrbitMoon => {
            let id = if config.scenario == Scenario::OrbitEarth {
                CelestialBodyId::Earth
            } else {
                CelestialBodyId::Moon
            };
            let body = CELESTIAL_BODIES
                .iter()
                .find(|body| body.id == id)
                .ok_or_else(|| format!("scenario setup failed: {id:?} missing from world catalog"))?;
            let mut position = body.center.meters();
            position[1] += body.radius_meters + 1_000.0;
            // Small reproducible tangent offset exercises distinct seeds without
            // changing the scenario's body or proximity contract.
            position[2] += (mix(config.seed) % 201) as f64 - 100.0;
            app.ship = ShipController::flying(
                ShipPose {
                    position_meters: position,
                    orientation: [0.0, 0.0, 0.0, 1.0],
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
    if matches!(config.scenario, Scenario::OrbitEarth | Scenario::OrbitMoon)
        != (snapshot.flight_state == FlightState::Flying)
    {
        return Err("scenario setup failed: unexpected ship flight state".into());
    }
    app.update_clock.set_fixed_step(config.step);
    Ok(())
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
        assert_eq!(a.update_clock.step(start + Duration::from_secs(2)), config.step);
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
        assert_eq!(landed.ship.snapshot().door_state, salimon_ship::DoorState::Open);
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
        assert_eq!(orbit.ship.snapshot().door_state, salimon_ship::DoorState::Closed);
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

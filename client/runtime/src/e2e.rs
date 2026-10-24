//! Opt-in scenario initialization for native end-to-end runs.

use std::time::Duration;

use salimon_character::{CharacterController, CharacterLocation};
use salimon_ship::{FlightState, ShipController, ShipPose};
use salimon_world::{CELESTIAL_BODIES, CelestialBodyId};

use crate::session::GameplaySession;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum Scenario {
    LandedEarth,
    ConstructionMaterials,
    ScoutLighting,
    ScoutLightingShutter,
    ScoutLightingBlackout,
    ScoutLightingBroken,
    ScoutLightingDark,
    ScoutMounts,
    ScoutCabin,
    ScoutFinishes,
    ScoutCoreUpgrade,
    ScoutCoreUpgradePowered,
    ScoutFinishesBlackout,
    ScoutCabinBunk,
    ScoutCabinLocker,
    ScoutCabinTable,
    ScoutFabricator,
    ScoutWorkBlackout,
    ScoutArchive,
    ScoutRecovery,
    FlightRegression,
    CargoFlight,
    CargoBlackout,
    CoreRestart,
    CrashRecovery,
    TerrainEarth,
    TerrainEarthApproach,
    TerrainMoonApproach,
    TerrainMarsApproach,
    TerrainMoonSteep,
    TerrainMoon,
    TerrainMars,
    TerrainEarthCrater,
    TerrainMoonCrater,
    TerrainMarsCrater,
    FragmentPile,
    CockpitEarth,
    OrbitEarth,
    ResourceApproach,
    OrbitMoon,
    OpenSpace,
    CoastingSpace,
    CoastingOrbit,
    EvaApproach,
}

impl Scenario {
    fn parse(value: &str) -> Result<Self, String> {
        match value {
            "scout-lighting" => Ok(Self::ScoutLighting),
            "scout-lighting-shutter" => Ok(Self::ScoutLightingShutter),
            "scout-lighting-blackout" => Ok(Self::ScoutLightingBlackout),
            "scout-lighting-broken" => Ok(Self::ScoutLightingBroken),
            "scout-lighting-dark" => Ok(Self::ScoutLightingDark),
            "construction-materials" => Ok(Self::ConstructionMaterials),
            "scout-cabin-bunk" => Ok(Self::ScoutCabinBunk),
            "scout-cabin-locker" => Ok(Self::ScoutCabinLocker),
            "scout-cabin-table" => Ok(Self::ScoutCabinTable),
            "scout-core-upgrade" => Ok(Self::ScoutCoreUpgrade),
            "scout-core-upgrade-powered" => Ok(Self::ScoutCoreUpgradePowered),
            "scout-finishes" => Ok(Self::ScoutFinishes),
            "scout-finishes-blackout" => Ok(Self::ScoutFinishesBlackout),
            "scout-cabin" => Ok(Self::ScoutCabin),
            "scout-mounts" => Ok(Self::ScoutMounts),
            "scout-fabricator" => Ok(Self::ScoutFabricator),
            "scout-work-blackout" => Ok(Self::ScoutWorkBlackout),
            "scout-archive" => Ok(Self::ScoutArchive),
            "scout-recovery" => Ok(Self::ScoutRecovery),
            "flight-regression" => Ok(Self::FlightRegression),
            "cargo-flight" => Ok(Self::CargoFlight),
            "cargo-blackout" => Ok(Self::CargoBlackout),
            "core-restart" => Ok(Self::CoreRestart),
            "crash-recovery" => Ok(Self::CrashRecovery),
            "terrain-earth-approach" => Ok(Self::TerrainEarthApproach),
            "terrain-moon-approach" => Ok(Self::TerrainMoonApproach),
            "terrain-mars-approach" => Ok(Self::TerrainMarsApproach),
            "terrain-moon-steep" => Ok(Self::TerrainMoonSteep),
            "terrain-earth" => Ok(Self::TerrainEarth),
            "terrain-moon" => Ok(Self::TerrainMoon),
            "terrain-mars" => Ok(Self::TerrainMars),
            "terrain-earth-crater" => Ok(Self::TerrainEarthCrater),
            "terrain-moon-crater" => Ok(Self::TerrainMoonCrater),
            "terrain-mars-crater" => Ok(Self::TerrainMarsCrater),
            "fragment-pile" => Ok(Self::FragmentPile),
            "landed-earth" => Ok(Self::LandedEarth),
            "cockpit-earth" => Ok(Self::CockpitEarth),
            "orbit-earth" => Ok(Self::OrbitEarth),
            "resource-approach" => Ok(Self::ResourceApproach),
            "orbit-moon" => Ok(Self::OrbitMoon),
            "open-space" => Ok(Self::OpenSpace),
            "coasting-space" => Ok(Self::CoastingSpace),
            "coasting-orbit" => Ok(Self::CoastingOrbit),
            "eva-approach" => Ok(Self::EvaApproach),
            _ => Err(format!(
                "unknown scenario '{value}'; expected fragment-pile, landed-earth, cockpit-earth, resource-approach, orbit-earth, orbit-moon, open-space, coasting-space, or eva-approach"
            )),
        }
    }

    pub(crate) const fn name(self) -> &'static str {
        match self {
            Self::ScoutLighting => "scout-lighting",
            Self::ScoutLightingShutter => "scout-lighting-shutter",
            Self::ScoutLightingBlackout => "scout-lighting-blackout",
            Self::ScoutLightingBroken => "scout-lighting-broken",
            Self::ScoutLightingDark => "scout-lighting-dark",
            Self::ConstructionMaterials => "construction-materials",
            Self::ScoutCabinBunk => "scout-cabin-bunk",
            Self::ScoutCabinLocker => "scout-cabin-locker",
            Self::ScoutCabinTable => "scout-cabin-table",
            Self::ScoutCoreUpgrade => "scout-core-upgrade",
            Self::ScoutCoreUpgradePowered => "scout-core-upgrade-powered",
            Self::ScoutFinishes => "scout-finishes",
            Self::ScoutFinishesBlackout => "scout-finishes-blackout",
            Self::ScoutCabin => "scout-cabin",
            Self::ScoutMounts => "scout-mounts",
            Self::ScoutFabricator => "scout-fabricator",
            Self::ScoutWorkBlackout => "scout-work-blackout",
            Self::ScoutArchive => "scout-archive",
            Self::ScoutRecovery => "scout-recovery",
            Self::FlightRegression => "flight-regression",
            Self::CargoFlight => "cargo-flight",
            Self::CargoBlackout => "cargo-blackout",
            Self::CoreRestart => "core-restart",
            Self::CrashRecovery => "crash-recovery",
            Self::TerrainEarthApproach => "terrain-earth-approach",
            Self::TerrainMoonApproach => "terrain-moon-approach",
            Self::TerrainMarsApproach => "terrain-mars-approach",
            Self::TerrainMoonSteep => "terrain-moon-steep",
            Self::TerrainEarth => "terrain-earth",
            Self::TerrainMoon => "terrain-moon",
            Self::TerrainMars => "terrain-mars",
            Self::TerrainEarthCrater => "terrain-earth-crater",
            Self::TerrainMoonCrater => "terrain-moon-crater",
            Self::TerrainMarsCrater => "terrain-mars-crater",
            Self::FragmentPile => "fragment-pile",
            Self::LandedEarth => "landed-earth",
            Self::CockpitEarth => "cockpit-earth",
            Self::OrbitEarth => "orbit-earth",
            Self::ResourceApproach => "resource-approach",
            Self::OrbitMoon => "orbit-moon",
            Self::OpenSpace => "open-space",
            Self::CoastingSpace => "coasting-space",
            Self::CoastingOrbit => "coasting-orbit",
            Self::EvaApproach => "eva-approach",
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct Config {
    pub(crate) scenario: Scenario,
    pub(crate) seed: u64,
    pub(crate) step: Duration,
    pub(crate) fragment_load: Option<(usize, crate::benchmark_fixture::Layout, u64)>,
}

pub(crate) fn parse_args(args: impl IntoIterator<Item = String>) -> Result<Option<Config>, String> {
    let mut args = args.into_iter();
    let mut enabled = false;
    let mut scenario = None;
    let mut seed = None;
    let mut step = None;
    let mut fragment_count = None;
    let mut fragment_layout = None;
    let mut fragment_settle_steps = None;
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
            "--fragment-count" if fragment_count.is_none() => {
                let count = value
                    .parse::<usize>()
                    .map_err(|_| "--fragment-count must be 0..1000")?;
                if count > 1000 {
                    return Err("--fragment-count must be 0..1000".into());
                }
                fragment_count = Some(count);
            }
            "--fragment-layout" if fragment_layout.is_none() => {
                fragment_layout = Some(crate::benchmark_fixture::Layout::parse(&value)?)
            }
            "--fragment-settle-steps" if fragment_settle_steps.is_none() => {
                let steps = value
                    .parse::<u64>()
                    .map_err(|_| "--fragment-settle-steps must be 0..37500")?;
                if steps > 37500 {
                    return Err("--fragment-settle-steps must be 0..37500".into());
                }
                fragment_settle_steps = Some(steps);
            }
            "--scenario"
            | "--seed"
            | "--step-ms"
            | "--fragment-count"
            | "--fragment-layout"
            | "--fragment-settle-steps" => {
                return Err(format!("{arg} specified twice"));
            }
            _ => return Err(format!("unknown argument '{arg}'")),
        }
    }
    if !enabled {
        return if scenario.is_some()
            || seed.is_some()
            || step.is_some()
            || fragment_count.is_some()
            || fragment_layout.is_some()
            || fragment_settle_steps.is_some()
        {
            Err("scenario options require --e2e".into())
        } else {
            Ok(None)
        };
    }
    if (fragment_layout.is_some() || fragment_settle_steps.is_some()) && fragment_count.is_none() {
        return Err("fragment layout/preparation requires --fragment-count".into());
    }
    Ok(Some(Config {
        scenario: scenario.unwrap_or(Scenario::LandedEarth),
        seed: seed.unwrap_or(0),
        step: step.unwrap_or(Duration::from_millis(16)),
        fragment_load: fragment_count.map(|n| {
            (
                n,
                fragment_layout.unwrap_or_default(),
                fragment_settle_steps.unwrap_or(0),
            )
        }),
    }))
}

/// Only alters initial conditions. Later movement, interaction and collision use
/// the regular character and ship controllers.
pub(crate) fn initialize(app: &mut GameplaySession, config: Config) -> Result<(), String> {
    app.character = CharacterController::default();
    app.ship = ShipController::default();
    match config.scenario {
        Scenario::ScoutLighting
        | Scenario::ScoutLightingShutter
        | Scenario::ScoutLightingBlackout
        | Scenario::ScoutLightingBroken
        | Scenario::ScoutLightingDark => {
            let shutter = matches!(
                config.scenario,
                Scenario::ScoutLightingShutter | Scenario::ScoutLightingBlackout
            );
            crate::session::cabin::evidence_initial_conditions(
                app,
                if shutter {
                    "scout-lighting-shutter"
                } else {
                    "scout-finishes"
                },
            )?;
            if matches!(
                config.scenario,
                Scenario::ScoutLightingBlackout | Scenario::ScoutLightingDark
            ) {
                app.ship.set_stored_core_energy_joules(0);
            }
            if config.scenario == Scenario::ScoutLightingBroken {
                app.lighting.rooms.get_mut(&1).unwrap().operable[2] = false;
            }
            if config.scenario == Scenario::ScoutLightingDark {
                let mut pose = app.ship.snapshot().pose;
                pose.position_meters[2] += 100_000_000.;
                app.ship = ShipController::flying(pose, 0);
                app.ship.set_stored_core_energy_joules(0);
                app.habitation.cabin.as_mut().unwrap().motions[3].fraction = 0.;
                app.habitation.cabin.as_mut().unwrap().motions[3].opening = false;
            }
        }

        Scenario::ConstructionMaterials => {
            crate::session::construction::evidence_initial_conditions(app)?
        }
        Scenario::ScoutCoreUpgrade | Scenario::ScoutCoreUpgradePowered => {
            crate::session::core_upgrades::evidence_initial_conditions(
                app,
                config.scenario == Scenario::ScoutCoreUpgradePowered,
            )?;
        }
        Scenario::ScoutFinishes | Scenario::ScoutFinishesBlackout => {
            crate::session::cabin::evidence_initial_conditions(app, "scout-finishes")?;
            if config.scenario == Scenario::ScoutFinishesBlackout {
                app.ship.set_stored_core_energy_joules(0);
            }
        }
        Scenario::ScoutCabin
        | Scenario::ScoutCabinBunk
        | Scenario::ScoutCabinLocker
        | Scenario::ScoutCabinTable => {
            crate::session::cabin::evidence_initial_conditions(app, config.scenario.name())?;
        }
        Scenario::ScoutMounts => {
            crate::session::mounts::evidence_initial_conditions(app)?;
        }
        Scenario::ScoutRecovery => {
            crate::scout_walkthrough::recovery_initial_conditions(app)?;
        }
        Scenario::ScoutArchive => {
            crate::scout_walkthrough::archive_initial_conditions(app)?;
        }
        Scenario::ScoutFabricator | Scenario::ScoutWorkBlackout => {
            crate::scout_walkthrough::initialize(app, config.scenario)?;
        }
        Scenario::FlightRegression
        | Scenario::CargoFlight
        | Scenario::CargoBlackout
        | Scenario::CoreRestart
        | Scenario::CrashRecovery => crate::flight_regression::initialize(app, config.scenario)?,
        Scenario::LandedEarth => {}
        Scenario::TerrainEarthApproach
        | Scenario::TerrainMoonApproach
        | Scenario::TerrainMarsApproach
        | Scenario::TerrainMoonSteep => {
            let id = match config.scenario {
                Scenario::TerrainMoonApproach | Scenario::TerrainMoonSteep => CelestialBodyId::Moon,
                Scenario::TerrainMarsApproach => CelestialBodyId::Mars,
                _ => CelestialBodyId::Earth,
            };
            let body = *CELESTIAL_BODIES
                .iter()
                .find(|b| b.id == id)
                .expect("terrain approach body");
            app.enable_terrain(body, [0.0, 1.0, 0.0])?;
            let pose = app.ship.snapshot().pose;
            let relative = salimon_math::sub(pose.position_meters, body.center.meters());
            let radial = salimon_math::scale(relative, 1.0 / salimon_math::length(relative));
            let position_meters = if config.scenario == Scenario::TerrainMoonSteep {
                let d = [100.0, 0.0, body.radius_meters];
                let n = salimon_math::length(d);
                body.center
                    .translated(d.map(|v| v / n * (body.radius_meters + 1000.0)))
                    .meters()
            } else {
                salimon_math::add(pose.position_meters, salimon_math::scale(radial, 1000.0))
            };
            app.ship = ShipController::flying(
                ShipPose {
                    position_meters,
                    orientation: pose.orientation,
                },
                0,
            );
            app.ship.configure_terrain(
                salimon_world::terrain::TerrainMode::Representative,
                crate::terrain::landing_footprint(),
            );
            app.character.enter_cockpit();
        }

        Scenario::TerrainEarth
        | Scenario::TerrainMoon
        | Scenario::TerrainMars
        | Scenario::TerrainEarthCrater
        | Scenario::TerrainMoonCrater
        | Scenario::TerrainMarsCrater => {
            let id = match config.scenario {
                Scenario::TerrainMoon | Scenario::TerrainMoonCrater => CelestialBodyId::Moon,
                Scenario::TerrainMars | Scenario::TerrainMarsCrater => CelestialBodyId::Mars,
                _ => CelestialBodyId::Earth,
            };
            let body = *CELESTIAL_BODIES
                .iter()
                .find(|b| b.id == id)
                .expect("terrain fixture");
            let direction = if matches!(
                config.scenario,
                Scenario::TerrainEarthCrater
                    | Scenario::TerrainMoonCrater
                    | Scenario::TerrainMarsCrater
            ) {
                [0.0, 0.0, 1.0]
            } else {
                [0.0, 1.0, 0.0]
            };
            app.enable_terrain(body, direction)?;
        }
        Scenario::FragmentPile => initialize_fragment_pile(app, config.seed),
        Scenario::CockpitEarth => app.character.enter_cockpit(),
        Scenario::ResourceApproach
        | Scenario::OrbitEarth
        | Scenario::OrbitMoon
        | Scenario::CoastingOrbit
        | Scenario::CoastingSpace
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
                } else if matches!(
                    config.scenario,
                    Scenario::OpenSpace | Scenario::CoastingOrbit | Scenario::CoastingSpace
                ) {
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
            if matches!(
                config.scenario,
                Scenario::CoastingOrbit | Scenario::CoastingSpace | Scenario::EvaApproach
            ) {
                app.ship = ShipController::flying_with_velocity(
                    app.ship.snapshot().pose,
                    0,
                    if config.scenario == Scenario::EvaApproach {
                        [0.0, -25_000.0, 0.0]
                    } else {
                        [25_000.0, 0.0, 0.0]
                    },
                )
                .map_err(|error| error.to_string())?;
            }
            app.character.enter_cockpit();
        }
    }
    if app.character.location() == CharacterLocation::Cockpit {
        app.ship.set_cockpit_control(true);
    }
    let snapshot = app.ship.snapshot();
    if matches!(
        config.scenario,
        Scenario::FlightRegression
            | Scenario::ScoutLightingDark
            | Scenario::CargoFlight
            | Scenario::CargoBlackout
            | Scenario::CoreRestart
            | Scenario::ScoutWorkBlackout
            | Scenario::CrashRecovery
            | Scenario::TerrainEarthApproach
            | Scenario::TerrainMoonApproach
            | Scenario::TerrainMarsApproach
            | Scenario::TerrainMoonSteep
            | Scenario::ResourceApproach
            | Scenario::OrbitEarth
            | Scenario::OrbitMoon
            | Scenario::CoastingOrbit
            | Scenario::CoastingSpace
            | Scenario::OpenSpace
            | Scenario::EvaApproach
    ) != (snapshot.flight_state == FlightState::Flying)
    {
        return Err("scenario setup failed: unexpected ship flight state".into());
    }
    if let Some((count, layout, settle_steps)) = config.fragment_load {
        if config.scenario == Scenario::FragmentPile {
            return Err("fragment-pile already owns its fixture".into());
        }
        if layout == crate::benchmark_fixture::Layout::Surface
            && config.scenario != Scenario::LandedEarth
        {
            return Err("surface populations require landed-earth".into());
        }
        crate::benchmark_fixture::initialize(app, count, layout, config.seed, false)?;
        crate::benchmark_fixture::settle(app, count, settle_steps)?;
    }
    app.update_clock.set_fixed_step(config.step);
    app.seed = config.seed;
    app.automation_step = Some(config.step);
    app.interpolate_presentation = false;
    Ok(())
}

/// Deterministic initial loose objects; subsequent stepping uses production physics.
/// Seed parity selects cabin or planetary support. No gameplay protocol mutation.
fn initialize_fragment_pile(app: &mut GameplaySession, seed: u64) {
    use salimon_character::{MovementInput, ShipFrame, SurfaceFrame};
    use salimon_world::{
        WorldPosition,
        resources::{DepositId, RawMaterial, ResourceDeposit, ResourceId, ResourceTransform},
    };
    let earth = CELESTIAL_BODIES
        .iter()
        .find(|b| b.id == CelestialBodyId::Earth)
        .expect("Earth fixture");
    let ship = app.ship.snapshot();
    let frame = ShipFrame {
        origin_meters: ship.pose.position_meters,
        axes: ship.pose.axes(),
    };
    let surface = SurfaceFrame {
        query: None,
        body_center_meters: earth.center.meters(),
        radius_meters: earth.radius_meters,
    };
    let step = Duration::from_millis(16);
    for _ in 0..90 {
        app.character.advance(
            step,
            MovementInput {
                backward: true,
                ..Default::default()
            },
            frame,
            surface,
            false,
            true,
        );
    }
    for _ in 0..36 {
        app.character.advance(
            step,
            MovementInput {
                right: true,
                ..Default::default()
            },
            frame,
            surface,
            false,
            true,
        );
    }
    let outside = seed % 2 == 1;
    if outside {
        app.ship.toggle_door();
        app.ship.advance(Duration::from_secs(1));
        for _ in 0..160 {
            app.character.advance(
                step,
                MovementInput {
                    backward: true,
                    ..Default::default()
                },
                frame,
                surface,
                true,
                true,
            );
        }
        app.character
            .apply_mouse_delta(std::f64::consts::PI / 0.0022, 0.0);
    }
    app.character.apply_mouse_delta(0.0, 260.0);
    let eye = app.character.snapshot(frame, surface).eye_position_meters;
    let mut center = frame.world_to_local(eye);
    center[0] += if outside { -1.8 } else { 1.8 };
    center[1] = if outside {
        let world = frame.local_to_world(center);
        let radial = world
            .iter()
            .zip(earth.center.meters())
            .map(|(a, b)| (a - b) * (a - b))
            .sum::<f64>()
            .sqrt();
        center[1] - (radial - earth.radius_meters)
    } else {
        salimon_character::SHIP_FLOOR_HEIGHT_METERS
    };
    for (index, resource) in [
        ResourceId::IronOre,
        ResourceId::SilicateRock,
        ResourceId::WaterIce,
    ]
    .into_iter()
    .enumerate()
    {
        let world = frame.local_to_world(center);
        let mut deposit = ResourceDeposit::new(
            DepositId {
                body: earth.id,
                local: 9000 + index as u64,
            },
            RawMaterial::new(resource, 4.0).expect("positive fixture mass"),
            WorldPosition::new(world[0], world[1], world[2]),
            4.0,
        )
        .expect("finite fixture deposit");
        app.mining
            .session
            .extract(&mut deposit, Duration::from_secs(2));
    }
    let pieces = app.mining.session.fragments().to_vec();
    for (index, piece) in pieces.into_iter().enumerate() {
        let local = [
            center[0] + (index % 2) as f64 * 0.08,
            center[1] + 0.5 + index as f64 * 0.45,
            center[2] + (index % 3) as f64 * 0.06,
        ];
        let world = frame.local_to_world(local);
        let pose = ResourceTransform::new(
            WorldPosition::new(world[0], world[1], world[2]),
            [0.0, 0.0, 0.0, 1.0],
        )
        .expect("fixture pose");
        app.mining.session.move_loose(piece.id(), pose);
        if !outside {
            app.mining
                .ship_fragments
                .insert(piece.id(), salimon_character::ShipLocalPoint::new(local));
        }
        app.mining.fragment_motion.insert(
            piece.id(),
            crate::fragment_physics::FragmentMotion::default(),
        );
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
    fn coasting_fixtures_seed_momentum_without_selected_thrust() {
        for (scenario, velocity) in [
            (Scenario::CoastingSpace, [25_000.0, 0.0, 0.0]),
            (Scenario::CoastingOrbit, [25_000.0, 0.0, 0.0]),
            (Scenario::EvaApproach, [0.0, -25_000.0, 0.0]),
        ] {
            let mut app = GameplaySession::default();
            initialize(
                &mut app,
                Config {
                    scenario,
                    seed: 0,
                    step: Duration::from_millis(16),
                    fragment_load: None,
                },
            )
            .unwrap();
            let initial = app.ship.snapshot();
            assert_eq!(initial.thruster_percentage, 0);
            assert_eq!(initial.velocity_meters_per_second, velocity);
            app.advance_automation(Duration::from_millis(160));
            assert_eq!(app.ship.snapshot().velocity_meters_per_second, velocity);
        }
    }

    #[test]
    fn same_seed_produces_same_initial_state_and_fixed_updates() {
        let config = Config {
            fragment_load: None,
            scenario: Scenario::OrbitMoon,
            seed: 42,
            step: Duration::from_millis(20),
        };
        let mut a = GameplaySession::default();
        let mut b = GameplaySession::default();
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
        let mut landed = GameplaySession::default();
        initialize(
            &mut landed,
            Config {
                fragment_load: None,
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
        let mut cockpit = GameplaySession::default();
        initialize(
            &mut cockpit,
            Config {
                fragment_load: None,
                scenario: Scenario::CockpitEarth,
                seed: 0,
                step: Duration::from_millis(16),
            },
        )
        .unwrap();
        assert_eq!(cockpit.character.location(), CharacterLocation::Cockpit);
        let mut orbit = GameplaySession::default();
        initialize(
            &mut orbit,
            Config {
                fragment_load: None,
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
                fragment_load: None,
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
        for args in [
            vec!["--e2e", "--fragment-settle-steps", "10"],
            vec![
                "--e2e",
                "--fragment-count",
                "10",
                "--fragment-settle-steps",
                "37501",
            ],
        ] {
            assert!(parse_args(args.into_iter().map(str::to_owned)).is_err());
        }
        let mut app = GameplaySession::default();
        assert_eq!(
            app.ship.snapshot().flight_state,
            FlightState::Landed {
                body: CelestialBodyId::Earth
            }
        );
        assert_eq!(app.update_clock.step(Instant::now()), Duration::ZERO);
    }
}

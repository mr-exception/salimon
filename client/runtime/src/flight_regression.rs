//! Reproducible initial conditions for portable/native flight regression routes.
//! No test-only mutation commands: subsequent actions use production controllers.
use crate::{e2e::Scenario, session::GameplaySession, session::frames::character_ship_frame};
use salimon_ship::{ShipController, ShipPose};
use salimon_world::resources::{
    DepositId, RawMaterial, ResourceDeposit, ResourceId, ResourceTransform,
};
use salimon_world::{CelestialBodyId, WorldPosition};
use std::time::Duration;

pub(crate) fn initialize(app: &mut GameplaySession, scenario: Scenario) -> Result<(), String> {
    app.ship = ShipController::flying_with_velocity(
        ShipPose {
            position_meters: [0.0; 3],
            orientation: [0.0, 0.0, 0.0, 1.0],
        },
        0,
        [96.0, 0.0, 0.0],
    )
    .map_err(|e| e.to_string())?;
    match scenario {
        Scenario::FlightRegression => app.character.enter_cockpit(),
        Scenario::CargoFlight => {
            app.character.enter_cockpit();
            initialize_cargo(app, 500)?;
        }
        Scenario::CoreRestart | Scenario::CargoBlackout => {
            app.ship = ShipController::flying_with_velocity(
                app.ship.snapshot().pose,
                0,
                [100.0, 0.0, 0.0],
            )
            .map_err(|e| e.to_string())?;
            app.ship.adjust_thruster(50);
            app.ship.set_cockpit_control(false);
            app.ship.set_stored_core_energy_joules(0);
            app.character.set_ship_gravity_enabled(false);
            app.mining.ship_gravity_disabled = true;
            // A held fragment at normal player start, aimed at the physical Core.
            let frame = character_ship_frame(app.ship.snapshot().pose);
            let eye = app
                .character
                .snapshot(
                    frame,
                    crate::session::frames::surface_frame_for_ship(app.ship.snapshot()),
                )
                .eye_position_meters;
            let target = frame.local_to_world(salimon_character::CORE_INTERACTION_POSITION_METERS);
            let d = salimon_math::sub(target, eye);
            app.character.apply_mouse_delta(
                d[2].atan2(d[0]) / 0.0022,
                -d[1].atan2(d[0].hypot(d[2])) / 0.0022,
            );
            let material = RawMaterial::new(ResourceId::IronOre, 2.0)
                .map_err(|e| format!("refuel material: {e:?}"))?;
            let mut deposit = ResourceDeposit::new(
                DepositId {
                    body: CelestialBodyId::Earth,
                    local: 900_195,
                },
                material,
                WorldPosition::new(0.0, 0.0, 0.0),
                2.0,
            )
            .map_err(|e| format!("refuel source: {e:?}"))?;
            app.mining.session.extract_with_spawn(
                &mut deposit,
                Duration::from_secs(1),
                |_, _, _| {
                    Some(
                        ResourceTransform::new(
                            WorldPosition::new(eye[0], eye[1], eye[2]),
                            [0.0, 0.0, 0.0, 1.0],
                        )
                        .unwrap(),
                    )
                },
            );
            // Retained #195 bootstrap fixture: exactly one legacy 2 kg piece.
            if app.mining.session.fragments().len() != 1
                || app.mining.session.fragments()[0].material() != material
            {
                return Err("legacy refuel fixture mass/population mismatch".into());
            }
            let id = app.mining.session.fragments()[0].id();
            if !app.mining.session.pick_up(id) {
                return Err("refuel fixture pickup failed".into());
            }
            app.sync_carried();
            if scenario == Scenario::CargoBlackout {
                // One held bootstrap fragment plus 499 contained loose pieces.
                initialize_cargo(app, 499)?;
            }
        }
        Scenario::CrashRecovery => {
            let pose = ShipController::default().snapshot().pose;
            let mut start = pose;
            start.position_meters[1] += 100.0;
            app.ship = ShipController::flying_with_velocity(start, 0, [0.0, -1000.0, 0.0])
                .map_err(|e| e.to_string())?;
            app.character.enter_cockpit();
        }
        _ => return Err("unsupported flight regression fixture".into()),
    }
    Ok(())
}

/// Historical legacy-onboard-032kg-v1: 500 × 0.032 kg = 16 kg, never 10 kg spheres.
/// Small fragments fit 500 actual centers inside the aft cabin without
/// an overfull fixture silently excluding cargo from mass accounting.
pub(crate) fn initialize_cargo(app: &mut GameplaySession, count: usize) -> Result<(), String> {
    if count > 500 {
        return Err("onboard regression population must be <=500".into());
    }
    let frame = character_ship_frame(app.ship.snapshot().pose);
    let initial_count = app.mining.session.fragments().len();
    for i in 0..count {
        let material = RawMaterial::new(ResourceId::IronOre, 0.032)
            .map_err(|e| format!("cargo material: {e:?}"))?;
        let mut deposit = ResourceDeposit::new(
            DepositId {
                body: CelestialBodyId::Earth,
                local: 910_000 + i as u64,
            },
            material,
            WorldPosition::new(0.0, 0.0, 0.0),
            0.032,
        )
        .map_err(|e| format!("cargo source: {e:?}"))?;
        let local = [
            -6.4 + (i % 20) as f64 * 0.21,
            0.5,
            -2.6 + (i / 20) as f64 * 0.21,
        ];
        if !salimon_character::ship_contains_object(local) {
            return Err(format!("cargo {i} outside ship"));
        }
        let world = frame.local_to_world(local);
        let pose = ResourceTransform::new(
            WorldPosition::new(world[0], world[1], world[2]),
            [0.0, 0.0, 0.0, 1.0],
        )
        .map_err(|e| format!("cargo pose: {e:?}"))?;
        app.mining.session.extract_with_spawn(
            &mut deposit,
            Duration::from_millis(16),
            |_, _, _| Some(pose),
        );
        let piece = app
            .mining
            .session
            .fragments()
            .last()
            .ok_or("cargo extraction failed")?;
        if piece.material() != material
            || app.mining.session.fragments().len() != initial_count + i + 1
        {
            return Err("legacy 0.032 kg cargo fixture mass/population mismatch".into());
        }
        app.mining
            .ship_fragments
            .insert(piece.id(), salimon_character::ShipLocalPoint::new(local));
    }
    app.sync_carried();
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{automation, e2e};
    use serde_json::{Value, json};

    fn replay(scenario: Scenario, source: &str) {
        let route: Value = serde_json::from_str(source).unwrap();
        let mut app = GameplaySession::default();
        e2e::initialize(
            &mut app,
            e2e::Config {
                scenario,
                seed: 0,
                step: Duration::from_millis(route["setup"]["step_ms"].as_u64().unwrap()),
                fragment_load: None,
            },
        )
        .unwrap();
        for (index, step) in route["steps"].as_array().unwrap().iter().enumerate() {
            if let Some(action) = step.get("action") {
                let mut command = action.clone();
                command["protocol"] = json!(1);
                command["id"] = json!(index);
                let response = automation::execute_session(&mut app, &command.to_string());
                assert_eq!(response["ok"], true, "step {index}: {response}");
            } else if let Some(check) = step.get("assert") {
                let state = automation::execute_session(
                    &mut app,
                    &json!({"protocol":1,"id":index,"op":"inspect"}).to_string(),
                );
                let mut v = &state["result"];
                for part in check["path"].as_str().unwrap().split('.') {
                    v = if let Ok(i) = part.parse::<usize>() {
                        &v[i]
                    } else {
                        &v[part]
                    };
                }
                let message = format!("step {index}: {check}, got {v}; state {state}");
                if let Some(expected) = check.get("equals") {
                    if let (Some(actual), Some(want)) = (v.as_f64(), expected.as_f64()) {
                        assert_eq!(actual, want, "{message}");
                    } else {
                        assert_eq!(v, expected, "{message}");
                    }
                }
                if let Some(expected) = check.get("approx") {
                    assert!(
                        (v.as_f64().unwrap() - expected.as_f64().unwrap()).abs()
                            <= check["tolerance"].as_f64().unwrap(),
                        "{message}"
                    );
                }
                if let Some(expected) = check.get("gt") {
                    assert!(
                        v.as_f64().unwrap() > expected.as_f64().unwrap(),
                        "{message}"
                    );
                }
                if let Some(expected) = check.get("lt") {
                    assert!(
                        v.as_f64().unwrap() < expected.as_f64().unwrap(),
                        "{message}"
                    );
                }
            } else if step.get("screenshot").is_some() {
                // Replay gameplay only; native periodic runs verify actual capture/pixels.
            } else {
                panic!("unhandled route step {index}");
            }
        }
    }

    #[test]
    fn starter_layout_replays_bulkhead_ramp_and_archive_route() {
        replay(
            Scenario::LandedEarth,
            include_str!("../../../scenarios/evidence/starter-layout.json"),
        );
    }
    #[test]
    fn inertial_route() {
        replay(
            Scenario::FlightRegression,
            include_str!("../../../scenarios/inertial-flight.json"),
        );
    }
    #[test]
    fn mechanical_refuel_and_restart_route() {
        replay(
            Scenario::CoreRestart,
            include_str!("../../../scenarios/core-restart.json"),
        );
    }
    #[test]
    fn crash_recovery_reservation_and_takeoff_route() {
        replay(
            Scenario::CrashRecovery,
            include_str!("../../../scenarios/crash-recovery.json"),
        );
    }

    #[test]
    fn cargo_route() {
        replay(
            Scenario::CargoFlight,
            include_str!("../../../scenarios/cargo-flight.json"),
        );
    }

    #[test]
    fn loaded_blackout_refuel_and_depletion_route() {
        replay(
            Scenario::CargoBlackout,
            include_str!("../../../scenarios/cargo-blackout.json"),
        );
    }

    #[test]
    fn landing_rejects_fast_tangential_motion_and_accepts_below_cutoff() {
        replay(
            Scenario::OrbitEarth,
            include_str!("../../../scenarios/landing-speed-cutoff.json"),
        );
    }

    #[test]
    fn landing_cutoff_evidence_matches_current_engine() {
        replay(
            Scenario::OrbitEarth,
            include_str!("../../../scenarios/evidence/landing-speed-cutoff.json"),
        );
    }

    #[test]
    fn cockpit_monitor_evidence_enters_assist_and_completes_landing() {
        replay(
            Scenario::OrbitEarth,
            include_str!("../../../scenarios/evidence/cockpit-wide-monitor.json"),
        );
    }

    #[test]
    fn cabin_blackout_coasts_with_held_fragment_and_restores_in_place() {
        let mut app = GameplaySession::default();
        e2e::initialize(
            &mut app,
            e2e::Config {
                scenario: Scenario::CoreRestart,
                seed: 0,
                step: Duration::from_millis(16),
                fragment_load: None,
            },
        )
        .unwrap();
        let held = app.mining.session.carried_id();
        let start = app.character.local_ship_position().unwrap();
        assert_eq!(
            automation::execute_session(
                &mut app,
                &json!({"protocol":1,"id":1,"op":"key","key":"ascend","pressed":true}).to_string()
            )["ok"],
            true
        );
        for _ in 0..4 {
            app.advance_game(Duration::from_millis(16));
        }
        assert_eq!(
            automation::execute_session(
                &mut app,
                &json!({"protocol":1,"id":2,"op":"key","key":"ascend","pressed":false}).to_string()
            )["ok"],
            true
        );
        let moving = app.character.local_ship_position().unwrap();
        app.advance_game(Duration::from_millis(16));
        let coasting = app.character.local_ship_position().unwrap();
        assert!(moving[1] > start[1] && coasting[1] > moving[1]);
        assert_eq!(app.mining.session.carried_id(), held);
        app.ship.exchange_core_energy_joules(1e9, 0.0).unwrap();
        let before = app.character.local_ship_position().unwrap();
        app.advance_game(Duration::from_millis(16));
        let after = app.character.local_ship_position().unwrap();
        assert!(!app.mining.ship_gravity_disabled);
        assert!(salimon_math::length(salimon_math::sub(after, before)) < 0.02);
        assert_eq!(app.mining.session.carried_id(), held);
    }

    #[test]
    fn five_hundred_fragments_mass_power_transition_and_consumption() {
        let mut app = GameplaySession::default();
        e2e::initialize(
            &mut app,
            e2e::Config {
                scenario: Scenario::FlightRegression,
                seed: 195,
                step: Duration::from_millis(16),
                fragment_load: None,
            },
        )
        .unwrap();
        initialize_cargo(&mut app, 500).unwrap();
        app.sync_carried();
        let cargo: f64 = app
            .mining
            .session
            .fragments()
            .iter()
            .map(|p| p.material().mass_kg())
            .sum();
        assert!((app.ship.snapshot().total_mass_kg - (100_000.0 + cargo)).abs() < 1e-6);
        let original: Vec<_> = app
            .mining
            .session
            .fragments()
            .iter()
            .map(|p| p.id())
            .collect();
        app.ship.set_stored_core_energy_joules(0);
        app.advance_game(Duration::from_millis(16));
        assert!(app.mining.ship_gravity_disabled);
        assert_eq!(
            app.ship.snapshot().velocity_meters_per_second,
            [96.0, 0.0, 0.0]
        );
        let piece = app.mining.session.fragments()[0];
        assert!(app.mining.session.pick_up(piece.id()));
        assert_eq!(
            app.ship.intake_carried_fragment(&mut app.mining.session),
            salimon_ship::IntakeResult::Accepted
        );
        app.sync_carried();
        assert!(
            (app.ship.snapshot().total_mass_kg - (100_000.0 + cargo - piece.material().mass_kg()))
                .abs()
                < 1e-6
        );
        app.advance_game(Duration::from_millis(16));
        assert!(!app.mining.ship_gravity_disabled);
        assert_eq!(app.mining.session.fragments().len(), 499);
        assert_eq!(
            app.ship.snapshot().total_mass_kg,
            app.mining.ship_mass.total_kg()
        );
        assert_eq!(
            original[1..],
            app.mining
                .session
                .fragments()
                .iter()
                .map(|p| p.id())
                .collect::<Vec<_>>()
        );
    }
}

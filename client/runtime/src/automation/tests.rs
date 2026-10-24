use super::*;

#[test]
fn domain_argument_failures_preserve_envelope_and_authoritative_save() {
    let mut session = app(Scenario::LandedEarth);
    let before = session.save_bytes(100).unwrap();
    for (id, fields) in [
        (
            json!("impact"),
            json!({"op":"construction_impact","impact":null}),
        ),
        (json!(42), json!({"op":"offline_resume","seconds":86401})),
        (
            json!("consume"),
            json!({"op":"consume","slot":0,"job":1,"unit":1}),
        ),
        (json!("craft"), json!({"op":"crafting","action":"unknown"})),
        (json!("look"), json!({"op":"look","dx":null,"dy":0})),
        (
            json!("key"),
            json!({"op":"key","key":"unknown","pressed":true}),
        ),
    ] {
        let mut request = fields;
        request["protocol"] = json!(1);
        request["id"] = id.clone();
        let result = execute_session(&mut session, &request.to_string());
        assert_eq!(result["id"], id);
        assert_eq!(result["error"]["code"], "invalid_argument");
        assert_eq!(session.save_bytes(100).unwrap(), before);
    }
}
use super::{execute_session as execute, inspect_session as inspect};

#[test]
fn loaded_step_deadlines_are_bounded_and_ordinary_requests_stay_short() {
    for frames in [1, 299, 300, 600] {
        assert_eq!(
            command_timeout(&json!({"protocol":1,"id":1,"op":"step","frames":frames}).to_string()),
            Duration::from_secs(120)
        );
    }
    for request in [
        json!({"protocol":1,"id":1,"op":"inspect"}),
        json!({"protocol":1,"id":1,"op":"step","frames":0}),
        json!({"protocol":1,"id":1,"op":"step","frames":601}),
        json!({"protocol":2,"id":1,"op":"step","frames":600}),
        json!({"protocol":1,"id":1,"op":"step","frames":"600"}),
        json!({"protocol":1,"op":"step","frames":1}),
        json!({"protocol":1,"id":false,"op":"step","frames":1}),
    ] {
        assert_eq!(
            command_timeout(&request.to_string()),
            Duration::from_secs(5)
        );
    }
    assert_eq!(command_timeout("{"), Duration::from_secs(5));
}

use crate::e2e::{self, Config, Scenario};

fn app(scenario: Scenario) -> GameplaySession {
    let mut app = GameplaySession::default();
    e2e::initialize(
        &mut app,
        Config {
            fragment_load: None,
            scenario,
            seed: 7,
            step: Duration::from_millis(20),
        },
    )
    .unwrap();
    app
}

#[test]
fn scout_archive_fixture_and_observation_preserve_unread_source_until_real_e() {
    let mut s = app(Scenario::ScoutArchive);
    let before = s.save_bytes(100).unwrap();
    let initial = inspect(&s);
    assert_eq!(initial["interaction"], "archive");
    assert_eq!(initial["archive"]["source"]["read"], json!([]));
    assert_eq!(initial["archive"]["open"], false);
    assert_eq!(s.save_bytes(100).unwrap(), before);
    let result = execute(
        &mut s,
        r#"{"protocol":1,"id":1,"op":"key","key":"interact","pressed":true}"#,
    );
    assert_eq!(result["ok"], true);
    assert_eq!(inspect(&s)["archive"]["open"], true);
    assert_eq!(
        inspect(&s)["archive"]["source"]["read"]
            .as_array()
            .unwrap()
            .len(),
        1
    );
}

#[test]
fn deposit_walkthrough_uses_real_movement_and_generated_state() {
    let mut test = app(Scenario::LandedEarth);
    e2e::initialize(
        &mut test,
        Config {
            fragment_load: None,
            scenario: Scenario::LandedEarth,
            seed: 0,
            step: Duration::from_millis(16),
        },
    )
    .unwrap();
    let scenario: Value =
        serde_json::from_str(include_str!("../../../../scenarios/resource-deposits.json")).unwrap();
    for step in scenario["steps"].as_array().unwrap() {
        if let Some(action) = step.get("action") {
            let mut command = action.clone();
            command["protocol"] = json!(1);
            command["id"] = json!(1);
            assert_eq!(execute(&mut test, &command.to_string())["ok"], true);
        }
    }
    let state = inspect(&test);
    assert_eq!(state["player"]["location"], "Surface");
    assert_eq!(state["world"]["deposit_query_error"], Value::Null);
    assert_eq!(
        state["world"]["nearest_deposit"]["id"],
        "Earth:16443359315117775089"
    );
    assert!(
        state["world"]["nearest_deposit"]["distance_to_player_meters"]
            .as_f64()
            .unwrap()
            < 4.0
    );
    assert_eq!(
        state["world"]["nearest_deposit"]["resource"],
        "silicate-rock"
    );
    assert!(state["world"]["nearest_deposit"]["visual"].is_null());
    assert_eq!(
        state["world"]["nearest_deposit"]["authored_visual"]["mesh"],
        "SilicateDepositSlab"
    );
    let iron: Vec<_> = state["world"]["deposits"]
        .as_array()
        .unwrap()
        .iter()
        .filter(|entry| entry["resource"] == "iron-ore")
        .collect();
    assert!(!iron.is_empty());
    for entry in iron {
        assert!(entry["visual"].is_null());
        assert!(
            entry["authored_visual"]["mesh"]
                .as_str()
                .unwrap()
                .starts_with("IronDeposit")
        );
        assert_eq!(
            entry["authored_visual"]["center_meters"],
            entry["position_meters"]
        );
    }
}

#[test]
fn resource_prompt_anchors_match_deposit_and_carried_pose() {
    let mut test = app(Scenario::LandedEarth);
    e2e::initialize(
        &mut test,
        Config {
            fragment_load: None,
            scenario: Scenario::LandedEarth,
            seed: 0,
            step: Duration::from_millis(16),
        },
    )
    .unwrap();
    let scenario: Value =
        serde_json::from_str(include_str!("../../../../scenarios/evidence/carrying.json")).unwrap();
    let mut saw_deposit = false;
    let mut saw_carried = false;
    let mut saw_fragment = false;
    for (index, step) in scenario["steps"].as_array().unwrap().iter().enumerate() {
        if let Some(action) = step.get("action") {
            let mut command = action.clone();
            command["protocol"] = json!(1);
            command["id"] = json!(index);
            assert_eq!(execute(&mut test, &command.to_string())["ok"], true);
        }
        let state = inspect(&test);
        let anchor = &state["resource_ui"]["anchor_meters"];
        if anchor.is_null() {
            continue;
        }
        if let Some(id) = test.mining.session.carried_id() {
            let piece = test
                .mining
                .session
                .fragments()
                .iter()
                .find(|piece| piece.id() == id)
                .unwrap();
            assert_eq!(*anchor, json!(piece.transform().position().meters()));
            saw_carried = true;
        } else if state["carrying"]["target_id"].is_number() {
            let id = state["carrying"]["target_id"].as_u64().unwrap();
            let piece = test
                .mining
                .session
                .fragments()
                .iter()
                .find(|piece| piece.id().0 == id)
                .unwrap();
            assert_eq!(*anchor, json!(piece.transform().position().meters()));
            saw_fragment = true;
        } else {
            assert!(
                state["world"]["deposits"]
                    .as_array()
                    .unwrap()
                    .iter()
                    .any(|entry| entry["position_meters"] == *anchor)
            );
            saw_deposit = true;
        }
    }
    assert!(saw_deposit && saw_fragment && saw_carried);
}

#[test]
fn toolbar_stows_cancel_both_mining_inputs_and_never_resume_on_reselect() {
    let mut test = app(Scenario::LandedEarth);
    e2e::initialize(
        &mut test,
        Config {
            fragment_load: None,
            scenario: Scenario::LandedEarth,
            seed: 0,
            step: Duration::from_millis(16),
        },
    )
    .unwrap();
    let scenario: Value =
        serde_json::from_str(include_str!("../../../../scenarios/mining.json")).unwrap();
    for step in scenario["steps"].as_array().unwrap() {
        if step.get("assert").and_then(|check| check["path"].as_str()) == Some("mining.active")
            && step["assert"]["equals"] == json!(true)
        {
            break;
        }
        if let Some(action) = step.get("action") {
            let mut command = action.clone();
            command["protocol"] = json!(1);
            command["id"] = json!(1);
            assert_eq!(execute(&mut test, &command.to_string())["ok"], true);
        }
    }
    assert_eq!(inspect(&test)["mining"]["active"], true);
    let ship = test.ship.snapshot();
    let frame = character_ship_frame(ship.pose);
    let player = test.character.snapshot(frame, test.terrain_frame(ship));
    for key in [
        KeyCode::Digit2,
        KeyCode::Digit3,
        KeyCode::Digit4,
        KeyCode::Digit5,
    ] {
        test.mining.set_mouse_held(true);
        crate::app::input::dispatch_session_key(&mut test, PhysicalKey::Code(key), true);
        assert!(!test.mining.input.held);
        assert_eq!(inspect(&test)["mining"]["equipped"], false);
        assert_eq!(inspect(&test)["mining"]["target"], Value::Null);
        assert!(
            test.mining
                .held_item(&test.equipment, player, true)
                .is_none()
        );
        let dot = crate::reticle::image(true, &test.equipment).unwrap();
        let mass = test.mining.session.extracted_mass_kg();
        test.advance_game(Duration::from_millis(16));
        assert_eq!(test.mining.session.extracted_mass_kg(), mass);
        crate::app::input::dispatch_session_key(
            &mut test,
            PhysicalKey::Code(KeyCode::Digit1),
            true,
        );
        assert!(
            test.mining
                .held_item(&test.equipment, player, true)
                .is_some()
        );
        assert_ne!(
            crate::reticle::image(true, &test.equipment)
                .unwrap()
                .revision,
            dot.revision
        );
        assert!(
            !test.mining.input.held,
            "selection must not restart held mining"
        );
        assert!(test.mining.press_f());
        test.mining_mouse_input(true);
    }
    assert_eq!(key_code("equip_mining_tool"), None);
    crate::app::input::dispatch_session_key(&mut test, PhysicalKey::Code(KeyCode::KeyM), true);
    assert!(
        test.equipment.mining_equipped(),
        "M cannot toggle selection"
    );
}

#[test]
fn f_isolation_and_mouse_reset_use_shared_production_routes() {
    let mut test = app(Scenario::LandedEarth);
    e2e::initialize(
        &mut test,
        Config {
            fragment_load: None,
            scenario: Scenario::LandedEarth,
            seed: 0,
            step: Duration::from_millis(16),
        },
    )
    .unwrap();
    let scenario: Value =
        serde_json::from_str(include_str!("../../../../scenarios/mining.json")).unwrap();
    for step in scenario["steps"].as_array().unwrap() {
        if step
            .get("assert")
            .and_then(|check| check["contains"].as_str())
            == Some("Hold left mouse to mine")
        {
            break;
        }
        if let Some(action) = step.get("action") {
            let mut command = action.clone();
            command["protocol"] = json!(1);
            command["id"] = json!(1);
            assert_eq!(execute(&mut test, &command.to_string())["ok"], true);
        }
    }
    assert!(inspect(&test)["mining"]["target"].is_object());
    assert_eq!(key_code("mine"), None);
    let old_mine = json!({"protocol": 1, "id": 1, "op": "key", "key": "mine", "pressed": true});
    assert_eq!(execute(&mut test, &old_mine.to_string())["ok"], false);
    let f = PhysicalKey::Code(KeyCode::KeyF);
    for _ in 0..3 {
        crate::app::input::dispatch_session_key(&mut test, f, true);
        test.advance_game(Duration::from_millis(16));
        assert!(!test.mining.input.held);
        assert_eq!(test.mining.session.extracted_mass_kg(), 0.0);
    }
    test.mining_mouse_input(true);
    crate::app::input::dispatch_session_key(&mut test, f, false);
    assert!(test.mining.input.held, "F release preserves mouse mining");
    test.advance_game(Duration::from_millis(16));
    assert!(test.mining.session.extracted_mass_kg() > 0.0);
    crate::app::input::dispatch_session_key(&mut test, f, true);
    test.mining_mouse_input(false);
    assert!(
        !test.mining.input.held,
        "held F cannot sustain mouse mining"
    );
    let mass = test.mining.session.extracted_mass_kg();
    test.advance_game(Duration::from_millis(16));
    assert_eq!(test.mining.session.extracted_mass_kg(), mass);
    test.mining_mouse_input(true);
    test.mining.clear_input(); // Native focus loss/cursor release use this reset.
    crate::app::input::dispatch_session_key(&mut test, f, true);
    assert!(!test.mining.input.held);
    test.mining_mouse_input(true);
    crate::app::input::dispatch_session_key(&mut test, PhysicalKey::Code(KeyCode::Digit2), true);
    assert!(!test.mining.input.held);
    test.mining_mouse_input(true);
    assert!(
        !test.mining.input.held,
        "empty selection rejects mouse input"
    );
    crate::app::input::dispatch_session_key(&mut test, PhysicalKey::Code(KeyCode::Digit1), true);
    assert!(!test.mining.input.held, "re-equip never resumes old input");
}

#[test]
fn mining_real_inputs_gate_surface_equipment_aim_and_release() {
    let mut test = app(Scenario::LandedEarth);
    crate::app::input::dispatch_session_key(&mut test, PhysicalKey::Code(KeyCode::Digit1), true);
    test.mining.input.held = true;
    test.advance_game(Duration::from_secs(1));
    assert_eq!(
        test.mining.session.extracted_mass_kg(),
        0.0,
        "inside cannot mine"
    );
    e2e::initialize(
        &mut test,
        Config {
            fragment_load: None,
            scenario: Scenario::LandedEarth,
            seed: 0,
            step: Duration::from_millis(16),
        },
    )
    .unwrap();
    // The fixture does not alter mining state; reset only test's initial input.
    test.mining = crate::mining::ResourceGameplay::default();
    test.equipment = crate::equipment::EquipmentToolbar::default();
    let scenario: Value =
        serde_json::from_str(include_str!("../../../../scenarios/mining.json")).unwrap();
    for (index, step) in scenario["steps"].as_array().unwrap().iter().enumerate() {
        if let Some(action) = step.get("action") {
            let mut command = action.clone();
            command["protocol"] = json!(1);
            command["id"] = json!(index);
            assert_eq!(execute(&mut test, &command.to_string())["ok"], true);
        } else if let Some(check) = step.get("assert") {
            // Mining/deposit checks independently verify the real route and hold/release controls.
            let path = check["path"].as_str().unwrap();
            if !path.starts_with("mining.")
                && !path.starts_with("world.")
                && !path.starts_with("resource_ui.")
            {
                continue;
            }
            let state = inspect(&test);
            let value = path.split('.').fold(&state, |value, key| {
                if value.is_array() {
                    &value[key.parse::<usize>().unwrap()]
                } else {
                    &value[key]
                }
            });
            if let Some(expected) = check.get("equals") {
                if value.is_number() && expected.is_number() {
                    assert_eq!(value.as_f64(), expected.as_f64(), "step {index}: {path}");
                } else {
                    assert_eq!(value, expected, "step {index}: {path}");
                }
            }
            if let Some(expected) = check.get("contains") {
                assert!(
                    match value {
                        Value::Array(values) => values.contains(expected),
                        Value::String(text) => text.contains(expected.as_str().unwrap()),
                        _ => panic!("unsupported contains at step {index}: {path}"),
                    },
                    "step {index}: {path}: {value}"
                );
            }
            if let Some(expected) = check.get("approx") {
                assert!(
                    (value.as_f64().unwrap() - expected.as_f64().unwrap()).abs()
                        <= check["tolerance"].as_f64().unwrap(),
                    "step {index}: {path}: {value}"
                );
            }
        }
    }
}

#[test]
fn streaming_walkthrough_reloads_partial_and_depleted_state() {
    let mut test = app(Scenario::LandedEarth);
    e2e::initialize(
        &mut test,
        Config {
            fragment_load: None,
            scenario: Scenario::LandedEarth,
            seed: 0,
            step: Duration::from_millis(16),
        },
    )
    .unwrap();
    let scenario: Value = serde_json::from_str(include_str!(
        "../../../../scenarios/resource-streaming.json"
    ))
    .unwrap();
    for (index, step) in scenario["steps"].as_array().unwrap().iter().enumerate() {
        if let Some(action) = step.get("action") {
            let mut command = action.clone();
            command["protocol"] = json!(1);
            command["id"] = json!(index);
            assert_eq!(execute(&mut test, &command.to_string())["ok"], true);
        } else if let Some(check) = step.get("assert") {
            let state = inspect(&test);
            let path = check["path"].as_str().unwrap();
            let value = path.split('.').try_fold(&state, |value, key| {
                if value.is_array() {
                    value.get(key.parse::<usize>().unwrap())
                } else {
                    value.get(key)
                }
            });
            if let Some(expected) = check.get("exists") {
                assert_eq!(
                    value.is_some(),
                    expected.as_bool().unwrap(),
                    "step {index}: {path}"
                );
                continue;
            }
            let value = value.unwrap_or_else(|| panic!("missing step {index}: {path}: {state}"));
            if let Some(expected) = check.get("equals") {
                if value.is_number() && expected.is_number() {
                    assert_eq!(value.as_f64(), expected.as_f64(), "step {index}: {path}");
                } else {
                    assert_eq!(value, expected, "step {index}: {path}");
                }
            } else if let Some(expected) = check.get("approx") {
                assert!(
                    (value.as_f64().unwrap() - expected.as_f64().unwrap()).abs()
                        <= check["tolerance"].as_f64().unwrap(),
                    "step {index}: {path}: {value}"
                );
            } else if let Some(expected) = check.get("gt") {
                assert!(
                    value.as_f64().unwrap() > expected.as_f64().unwrap(),
                    "step {index}: {path}: {value}"
                );
            } else if let Some(expected) = check.get("lt") {
                assert!(
                    value.as_f64().unwrap() < expected.as_f64().unwrap(),
                    "step {index}: {path}: {value}"
                );
            } else {
                panic!("unhandled check at step {index}");
            }
        }
    }
}

#[test]
fn physical_carrying_walkthrough_uses_real_gameplay_actions() {
    assert_gameplay_scenario(
        Scenario::LandedEarth,
        include_str!("../../../../scenarios/carrying.json"),
    );
}

#[test]
fn carrying_evidence_uses_the_same_f_contract() {
    assert_gameplay_scenario(
        Scenario::LandedEarth,
        include_str!("../../../../scenarios/evidence/carrying.json"),
    );
}

#[test]
fn equipped_f_pickup_consumes_the_press_and_e_does_not_release() {
    let mut test = app(Scenario::LandedEarth);
    let scenario: Value =
        serde_json::from_str(include_str!("../../../../scenarios/carrying.json")).unwrap();
    e2e::initialize(
        &mut test,
        Config {
            fragment_load: None,
            scenario: Scenario::LandedEarth,
            seed: 0,
            step: Duration::from_millis(16),
        },
    )
    .unwrap();
    for step in scenario["steps"].as_array().unwrap() {
        if step.get("assert").and_then(|check| check["path"].as_str()) == Some("carrying.target_id")
            && step["assert"]["equals"] == json!(4)
        {
            break;
        }
        if let Some(action) = step.get("action") {
            let mut command = action.clone();
            command["protocol"] = json!(1);
            command["id"] = json!(1);
            assert_eq!(execute(&mut test, &command.to_string())["ok"], true);
        }
    }
    crate::app::input::dispatch_session_key(&mut test, PhysicalKey::Code(KeyCode::Digit1), true);
    assert!(test.equipment.mining_equipped());
    crate::app::input::dispatch_session_key(&mut test, PhysicalKey::Code(KeyCode::Digit1), true);
    assert_eq!(inspect(&test)["equipment"]["selected_slot"], 1);
    let mass = test.mining.session.extracted_mass_kg();
    test.mining.set_mouse_held(true);
    test.interact();
    assert!(test.mining.session.carried_id().is_none());
    crate::app::input::dispatch_session_key(&mut test, PhysicalKey::Code(KeyCode::KeyF), true);
    assert!(test.mining.session.carried_object().is_some());
    assert_eq!(inspect(&test)["equipment"]["selected_slot"], Value::Null);
    assert!(!test.equipment.mining_equipped());
    assert_eq!(inspect(&test)["mining"]["held_item_visible"], false);
    assert_eq!(test.equipment.presentation().selected, None);
    for key in ["slot_1", "slot_2", "slot_3", "slot_4", "slot_5"] {
        let request = json!({"protocol": 1, "id": 1, "op": "key", "key": key, "pressed": true});
        assert_eq!(execute(&mut test, &request.to_string())["ok"], true);
        assert_eq!(inspect(&test)["equipment"]["selected_slot"], Value::Null);
    }
    assert!(!test.mining.input.held);
    crate::app::input::dispatch_session_key(&mut test, PhysicalKey::Code(KeyCode::KeyF), true);
    test.advance_game(Duration::from_secs(1));
    assert_eq!(test.mining.session.extracted_mass_kg(), mass);
    assert!(test.mining.session.carried_object().is_some());
    test.interact();
    assert!(test.mining.session.carried_object().is_some());
    crate::app::input::dispatch_session_key(&mut test, PhysicalKey::Code(KeyCode::KeyF), false);
    crate::app::input::dispatch_session_key(&mut test, PhysicalKey::Code(KeyCode::KeyF), true);
    assert!(test.mining.session.carried_id().is_none());
    assert!(
        !test.mining.input.held,
        "drop must not start F mining either"
    );
    assert_eq!(inspect(&test)["equipment"]["selected_slot"], Value::Null);
    crate::app::input::dispatch_session_key(&mut test, PhysicalKey::Code(KeyCode::Digit5), true);
    assert_eq!(inspect(&test)["equipment"]["selected_slot"], 5);
    assert_eq!(inspect(&test)["equipment"]["slots"][4], Value::Null);
    crate::app::input::dispatch_session_key(&mut test, PhysicalKey::Code(KeyCode::Digit1), true);
    assert_eq!(inspect(&test)["equipment"]["selected_slot"], 1);
    assert!(test.equipment.mining_equipped());
    assert_eq!(inspect(&test)["mining"]["held_item_visible"], true);
    assert!(
        !test.mining.input.held,
        "re-selection must not resume held mining"
    );
}

#[test]
fn fragment_transfer_walkthrough_uses_real_gameplay_actions() {
    assert_gameplay_scenario(
        Scenario::LandedEarth,
        include_str!("../../../../scenarios/fragment-transfer.json"),
    );
}

#[test]
fn space_airlock_walkthrough_uses_real_gameplay_actions() {
    assert_gameplay_scenario(
        Scenario::OpenSpace,
        include_str!("../../../../scenarios/space-airlock.json"),
    );
}

#[test]
fn crash_states_map_to_powered_recovery_through_normal_landing_action() {
    let mut app = app(Scenario::OrbitEarth);
    let earth = salimon_world::CELESTIAL_BODIES
        .iter()
        .find(|body| body.id == salimon_world::CelestialBodyId::Earth)
        .unwrap();
    let mut pose = app.ship.snapshot().pose;
    pose.position_meters = salimon_math::add(
        earth.center.meters(),
        [
            0.0,
            earth.radius_meters + salimon_ship::COLLISION_RADIUS_METERS + 1.0,
            0.0,
        ],
    );
    app.ship =
        salimon_ship::ShipController::flying_with_velocity(pose, 50, [0.0, -1000.0, 0.0]).unwrap();
    execute(&mut app, r#"{"protocol":1,"id":1,"op":"step","frames":1}"#);
    let state = inspect(&app);
    assert_eq!(state["ship"]["flight_state"], "Bouncing { body: Earth }");
    assert_eq!(state["ship"]["thruster_percentage"], 0);
    execute(&mut app, r#"{"protocol":1,"id":2,"op":"landing"}"#);
    assert!(matches!(
        app.ship.snapshot().flight_state,
        salimon_ship::FlightState::Bouncing { .. }
    ));
    execute(
        &mut app,
        r#"{"protocol":1,"id":3,"op":"step","frames":200}"#,
    );
    assert_eq!(
        inspect(&app)["ship"]["flight_state"],
        "GroundedAfterCrash { body: Earth }"
    );
    assert_eq!(inspect(&app)["ship"]["speed_meters_per_second"], 0.0);
    let resting = app.ship.snapshot().pose;
    execute(&mut app, r#"{"protocol":1,"id":4,"op":"landing"}"#);
    app.advance_automation(Duration::from_secs(1));
    assert!(app.ship.snapshot().pose.position_meters[1] > resting.position_meters[1]);
    assert_eq!(
        inspect(&app)["ship"]["flight_state"],
        "AssistedLanding { body: Earth }"
    );
    app.advance_automation(Duration::from_secs(11));
    assert_eq!(
        inspect(&app)["ship"]["flight_state"],
        "Landed { body: Earth }"
    );
    assert_eq!(inspect(&app)["ship"]["speed_meters_per_second"], 0.0);
}

#[test]
fn signed_held_throttle_uses_gameplay_keys_and_fixed_simulation_time() {
    let mut app = app(Scenario::OpenSpace);
    let key = |app: &mut GameplaySession, pressed| {
        execute(
            app,
            &json!({"protocol":1,"id":1,"op":"key","key":"thruster_down","pressed":pressed})
                .to_string(),
        )
    };
    assert_eq!(key(&mut app, true)["ok"], true);
    key(&mut app, true);
    assert_eq!(app.ship.snapshot().thruster_percentage, -1);
    execute(
        &mut app,
        r#"{"protocol":1,"id":2,"op":"step","frames":100}"#,
    );
    assert_eq!(app.ship.snapshot().thruster_percentage, -21);
    assert!(app.ship.snapshot().velocity_meters_per_second[0] < 0.0);
    key(&mut app, false);
    execute(
        &mut app,
        r#"{"protocol":1,"id":3,"op":"step","frames":100}"#,
    );
    assert_eq!(app.ship.snapshot().thruster_percentage, -21);
    execute(&mut app, r#"{"protocol":1,"id":4,"op":"interact"}"#);
    let response = execute(
        &mut app,
        r#"{"protocol":1,"id":5,"op":"thruster","direction":1}"#,
    );
    assert_eq!(response["result"]["applied"], false);
    key(&mut app, true);
    app.advance_automation(Duration::from_secs(1));
    assert_eq!(app.ship.snapshot().thruster_percentage, -21);
}

#[test]
fn moving_eva_walkthrough_uses_real_gameplay_actions() {
    assert_gameplay_scenario(
        Scenario::CoastingSpace,
        include_str!("../../../../scenarios/moving-eva.json"),
    );
}

#[test]
fn nearby_eva_walkthrough_uses_real_gameplay_actions() {
    assert_gameplay_scenario(
        Scenario::EvaApproach,
        include_str!("../../../../scenarios/nearby-eva.json"),
    );
}

#[test]
fn boarding_ramp_walkthrough_preserves_contact_and_rejects_occupied_closure() {
    assert_gameplay_scenario(
        Scenario::LandedEarth,
        include_str!("../../../../scenarios/boarding-ramp.json"),
    );
}

#[test]
fn cockpit_nose_walkthrough_uses_real_gameplay_actions() {
    assert_gameplay_scenario(
        Scenario::LandedEarth,
        include_str!("../../../../scenarios/cockpit-nose.json"),
    );
}

#[test]
fn resource_loop_uses_real_landing_mining_and_cabin_delivery_actions() {
    assert_gameplay_scenario(
        Scenario::ResourceApproach,
        include_str!("../../../../scenarios/resource-loop.json"),
    );
}

#[test]
fn mining_emission_walkthrough_preserves_rate_and_repeated_output() {
    assert_gameplay_scenario(
        Scenario::LandedEarth,
        include_str!("../../../../scenarios/mining-emission.json"),
    );
}

#[test]
fn assisted_baselines_preserve_landing_and_handoff_to_inertial_takeoff() {
    assert_gameplay_scenario(
        Scenario::OrbitEarth,
        include_str!("../../../../scenarios/orbit-earth.json"),
    );
    assert_gameplay_scenario(
        Scenario::LandedEarth,
        include_str!("../../../../scenarios/landed-earth.json"),
    );
}

#[test]
fn cancellation_boundary_routes_use_exact_live_ticks_and_physical_refunds() {
    for source in [
        include_str!("../../../../scenarios/evidence/construction-cancel-0.json"),
        include_str!("../../../../scenarios/evidence/construction-cancel-50.json"),
        include_str!("../../../../scenarios/evidence/construction-cancel-99.json"),
    ] {
        assert_gameplay_scenario(Scenario::ConstructionCancellation, source);
    }
}

#[test]
fn construction_interlock_scenarios_share_native_command_admission() {
    assert_gameplay_scenario(
        Scenario::ConstructionRoom,
        include_str!("../../../../scenarios/evidence/construction-interlocks.json"),
    );
    assert_gameplay_scenario(
        Scenario::ConstructionInterlocks,
        include_str!("../../../../scenarios/evidence/construction-interlocks-cockpit.json"),
    );
}

fn assert_gameplay_scenario(initial: Scenario, source: &str) {
    let mut test = GameplaySession::default();
    let scenario: Value = serde_json::from_str(source).unwrap();
    e2e::initialize(
        &mut test,
        Config {
            fragment_load: None,
            scenario: initial,
            seed: scenario["setup"]["seed"].as_u64().unwrap(),
            step: Duration::from_millis(scenario["setup"]["step_ms"].as_u64().unwrap()),
        },
    )
    .unwrap();
    for (index, step) in scenario["steps"].as_array().unwrap().iter().enumerate() {
        if let Some(wait) = step.get("wait") {
            let path = wait["path"].as_str().unwrap();
            let interval_ms =
                scenario["setup"]["step_ms"].as_u64().unwrap() * step["frames"].as_u64().unwrap();
            let timeout_ms = step["timeout_ms"].as_u64().unwrap();
            let mut elapsed = 0;
            loop {
                let state = inspect(&test);
                let value = path.split('.').fold(&state, |value, key| {
                    if value.is_array() {
                        &value[key.parse::<usize>().unwrap()]
                    } else {
                        &value[key]
                    }
                });
                if value == &wait["equals"] {
                    break;
                }
                assert!(
                    elapsed < timeout_ms,
                    "wait at step {index}: {path}: {value}"
                );
                test.advance_automation(Duration::from_millis(interval_ms));
                elapsed += interval_ms;
            }
        } else if let Some(action) = step.get("action") {
            let mut command = action.clone();
            command["protocol"] = json!(1);
            command["id"] = json!(index);
            assert_eq!(execute(&mut test, &command.to_string())["ok"], true);
            if let Some(id) = test.mining.session.carried_id() {
                let state = inspect(&test);
                let piece = state["world"]["fragments"]
                    .as_array()
                    .unwrap()
                    .iter()
                    .find(|p| p["id"].as_u64() == Some(id.0))
                    .expect("carried identity stays visible");
                assert!(
                    {
                        let radius = piece["side_meters"].as_f64().unwrap() * 0.5 * 3.0_f64.sqrt();
                        let eye = &state["player"]["eye_position_meters"];
                        let up = &state["player"]["up"];
                        let delta: [f64; 3] = std::array::from_fn(|i| {
                            piece["position_meters"][i].as_f64().unwrap() - eye[i].as_f64().unwrap()
                        });
                        let up: [f64; 3] = std::array::from_fn(|i| up[i].as_f64().unwrap());
                        let height = salimon_math::dot(delta, up);
                        let planar = std::array::from_fn(|i| delta[i] - up[i] * height);
                        let distance = salimon_math::length(planar);
                        distance > radius + 0.6 && distance < radius + 0.7
                    },
                    "carried pose at step {index}"
                );
            }
        } else if let Some(check) = step.get("assert") {
            let state = inspect(&test);
            let path = check["path"].as_str().unwrap();
            let value = path.split('.').try_fold(&state, |value, key| {
                if value.is_array() {
                    value.get(key.parse::<usize>().unwrap())
                } else {
                    value.get(key)
                }
            });
            if let Some(expected) = check.get("exists") {
                assert_eq!(
                    value.is_some(),
                    expected.as_bool().unwrap(),
                    "step {index}: {path}"
                );
                continue;
            }
            let value = value.unwrap_or_else(|| panic!("missing step {index}: {path}"));
            if let Some(expected) = check.get("equals") {
                if value.is_number() && expected.is_number() {
                    assert_eq!(value.as_f64(), expected.as_f64(), "step {index}: {path}");
                } else if let (Some(actual), Some(expected)) =
                    (value.as_array(), expected.as_array())
                {
                    assert_eq!(actual.len(), expected.len(), "step {index}: {path}");
                    for (actual, expected) in actual.iter().zip(expected) {
                        if actual.is_number() && expected.is_number() {
                            assert_eq!(actual.as_f64(), expected.as_f64(), "step {index}: {path}");
                        } else {
                            assert_eq!(actual, expected, "step {index}: {path}");
                        }
                    }
                } else {
                    assert_eq!(value, expected, "step {index}: {path}");
                }
            } else if let Some(expected) = check.get("contains") {
                assert!(
                    match value {
                        Value::Array(values) => values.contains(expected),
                        Value::String(text) => text.contains(expected.as_str().unwrap()),
                        _ => panic!("unsupported contains at step {index}: {path}"),
                    },
                    "step {index}: {path}: {value}"
                );
            } else if let Some(expected) = check.get("approx") {
                assert!(
                    (value.as_f64().unwrap() - expected.as_f64().unwrap()).abs()
                        <= check["tolerance"].as_f64().unwrap(),
                    "step {index}: {path}: {value}"
                );
            } else if let Some(expected) = check.get("gt") {
                assert!(
                    value.as_f64().unwrap() > expected.as_f64().unwrap(),
                    "step {index}: {path}: {value}"
                );
            } else if let Some(expected) = check.get("lt") {
                assert!(
                    value.as_f64().unwrap() < expected.as_f64().unwrap(),
                    "step {index}: {path}: {value}"
                );
            } else {
                panic!("unhandled check at step {index}");
            }
        }
    }
}

#[test]
fn protocol_rejects_malformed_and_unavailable_commands() {
    let mut normal = GameplaySession::default();
    assert_eq!(
        execute(&mut normal, r#"{"protocol":1,"id":1,"op":"inspect"}"#)["error"]["code"],
        "disabled"
    );
    let mut test = app(Scenario::LandedEarth);
    assert_eq!(execute(&mut test, "{")["error"]["code"], "invalid_json");
    assert_eq!(
        execute(&mut test, r#"{"protocol":2,"id":"a","op":"inspect"}"#)["error"]["code"],
        "protocol_mismatch"
    );
    assert_eq!(
        execute(
            &mut test,
            r#"{"protocol":1,"id":2,"op":"step","frames":601}"#
        )["error"]["code"],
        "invalid_argument"
    );
    assert_eq!(
        execute(&mut test, r#"{"protocol":1,"id":3,"op":"warp"}"#)["error"]["code"],
        "unknown_op"
    );
}

#[test]
fn capture_polling_observes_only_status_without_mutating_authority() {
    let mut test = ClientApplication::default();
    test.session = app(Scenario::LandedEarth);
    test.e2e_config = Some(Config {
        fragment_load: None,
        scenario: Scenario::LandedEarth,
        seed: 7,
        step: Duration::from_millis(20),
    });
    test.capture_status = "captured".into();
    let before = test.session.save_bytes(100).unwrap();
    let response = super::execute(
        &mut test,
        r#"{"protocol":1,"id":1,"op":"inspect","scope":"capture"}"#,
    );
    assert_eq!(response["result"], json!({"capture_status":"captured"}));
    assert_eq!(before, test.session.save_bytes(100).unwrap());
    let mut normal = ClientApplication::default();
    assert_eq!(
        super::execute(
            &mut normal,
            r#"{"protocol":1,"id":1,"op":"inspect","scope":"capture"}"#
        )["error"]["code"],
        "disabled"
    );
}

#[test]
fn commands_use_live_controller_paths_and_inspection() {
    let mut test = app(Scenario::LandedEarth);
    let before = execute(&mut test, r#"{"protocol":1,"id":1,"op":"inspect"}"#);
    assert_eq!(before["result"]["player"]["location"], "InsideShip");
    assert_eq!(
        before["result"]["world"]["bodies"]
            .as_array()
            .unwrap()
            .len(),
        6
    );
    assert_eq!(
        execute(
            &mut test,
            r#"{"protocol":1,"id":2,"op":"key","key":"forward","pressed":true}"#
        )["ok"],
        true
    );
    assert_eq!(
        execute(&mut test, r#"{"protocol":1,"id":3,"op":"step","frames":5}"#)["ok"],
        true
    );
    let after = execute(&mut test, r#"{"protocol":1,"id":4,"op":"inspect"}"#);
    assert_ne!(
        before["result"]["player"]["eye_position_meters"],
        after["result"]["player"]["eye_position_meters"]
    );
    execute(
        &mut test,
        r#"{"protocol":1,"id":5,"op":"key","key":"forward","pressed":false}"#,
    );
    let stable = execute(&mut test, r#"{"protocol":1,"id":6,"op":"inspect"}"#);
    execute(&mut test, r#"{"protocol":1,"id":7,"op":"step","frames":1}"#);
    assert_eq!(
        stable["result"]["player"]["eye_position_meters"],
        execute(&mut test, r#"{"protocol":1,"id":8,"op":"inspect"}"#)["result"]["player"]["eye_position_meters"]
    );
}

#[test]
fn cockpit_commands_respect_authority() {
    let mut test = app(Scenario::OrbitEarth);
    let before = test.ship.snapshot().thruster_percentage;
    execute(
        &mut test,
        r#"{"protocol":1,"id":1,"op":"thruster","direction":1}"#,
    );
    assert_eq!(test.ship.snapshot().thruster_percentage, before + 1);
    execute(&mut test, r#"{"protocol":1,"id":2,"op":"interact"}"#);
    assert_eq!(test.character.location(), CharacterLocation::InsideShip);
    assert_eq!(
        execute(
            &mut test,
            r#"{"protocol":1,"id":3,"op":"thruster","direction":1}"#
        )["result"]["applied"],
        false
    );
    assert_eq!(test.ship.snapshot().thruster_percentage, before + 1);
}

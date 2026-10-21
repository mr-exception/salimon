//! Portable cross-domain scenarios; fixtures supply deposits, never free queue outputs.
use crate::{automation, e2e, session::GameplaySession};
use salimon_fabrication::{FabricationCommand as C, RecipeId};
use salimon_world::{
    CelestialBodyId, WorldPosition,
    resources::{DepositId, RawMaterial, ResourceDeposit, ResourceId, ResourceTransform},
    spherical_mining::{MiningConfig, SPHERE_TICK},
};
use serde_json::{Value, json};
use std::time::Duration;

fn request(session: &mut GameplaySession, mut value: Value) -> Value {
    value["protocol"] = 1.into();
    value["id"] = 1.into();
    let response = automation::execute_session(session, &value.to_string());
    assert_eq!(response["ok"], true, "{response}");
    response["result"].clone()
}
fn session() -> GameplaySession {
    let mut s = GameplaySession::default();
    e2e::initialize(
        &mut s,
        e2e::Config {
            scenario: e2e::Scenario::LandedEarth,
            seed: 0,
            step: Duration::from_millis(20),
            fragment_load: None,
        },
    )
    .unwrap();
    s
}
fn mine_carry_load(s: &mut GameplaySession, resource: ResourceId, kg: f64, local: u64) {
    let pose =
        ResourceTransform::new(WorldPosition::new(0.0, 0.0, 0.0), [0.0, 0.0, 0.0, 1.0]).unwrap();
    let mut deposit = ResourceDeposit::new(
        DepositId {
            body: CelestialBodyId::Earth,
            local,
        },
        RawMaterial::new(resource, kg).unwrap(),
        pose.position(),
        kg,
    )
    .unwrap();
    let mut id = None;
    for _ in 0..500 {
        id = s.mining.session.tick_spherical(
            true,
            Some(&mut deposit),
            SPHERE_TICK,
            MiningConfig::default(),
            pose,
        );
        if id.is_some() {
            break;
        }
    }
    let id = id.expect("finite fixture deposit releases a sphere");
    assert!(s.mining.session.pick_up(id));
    s.sync_carried();
    s.insert_fabricator_fragment();
    assert_eq!(s.mining.session.carried_id(), None);
    assert!(s.mining.session.fragments().iter().all(|f| f.id() != id));
    assert!(deposit.remaining_mass_kg() < 1e-8);
}
#[test]
fn mined_materials_mixed_fifo_cancel_blackout_tray_inventory_and_mass() {
    let mut s = session();
    for (i, resource) in [
        ResourceId::OrganicBiomass,
        ResourceId::MedicinalCompounds,
        ResourceId::WaterIce,
    ]
    .into_iter()
    .enumerate()
    {
        mine_carry_load(&mut s, resource, 2.0, i as u64 + 9000);
    }
    let before = s.fabrication_snapshot();
    let mass = s.ship.snapshot().total_mass_kg;
    s.crafting_ui.open = true;
    request(&mut s, json!({"op":"crafting","action":"more"}));
    request(&mut s, json!({"op":"crafting","action":"add"}));
    let ration = s.fabrication_snapshot();
    assert_eq!(ration.jobs[0].remaining_quantity, 2);
    assert!(
        ration
            .stored
            .iter()
            .zip(&before.stored)
            .any(|(a, b)| a.milligrams < b.milligrams)
    );
    assert!(s.ship.snapshot().total_mass_kg < mass);
    s.execute_fabricator(C::Enqueue {
        recipe: RecipeId::new("medkit").unwrap(),
        quantity: 1,
    })
    .unwrap();
    let jobs = s.fabrication_snapshot().jobs;
    assert_eq!(jobs.len(), 2);
    let balances = s.fabrication_snapshot().stored;
    s.execute_fabricator(C::Cancel(jobs[1].id)).unwrap();
    assert_eq!(s.fabrication_snapshot().stored, balances);
    s.execute_fabricator(C::Enqueue {
        recipe: RecipeId::new("medkit").unwrap(),
        quantity: 1,
    })
    .unwrap();
    s.crafting_ui.open = false;
    s.ship.set_stored_core_energy_joules(0);
    request(&mut s, json!({"op":"step","frames":100}));
    assert_eq!(s.fabrication_snapshot().jobs[0].progress, Duration::ZERO);
    assert_eq!(
        s.fabrication_snapshot().status,
        salimon_fabrication::FabricatorStatus::NoPower
    );
    s.ship.set_stored_core_energy_joules(100_000_000);
    for (recipe, seconds) in [("food-ration", 11), ("food-ration", 11), ("medkit", 16)] {
        for _ in 0..seconds {
            request(&mut s, json!({"op":"step","frames":50}));
        }
        let snapshot = s.fabrication_snapshot();
        assert_eq!(snapshot.tray.as_ref().unwrap().recipe.key(), recipe);
        let ship_mass = s.ship.snapshot().total_mass_kg;
        request(&mut s, json!({"op":"step","frames":100}));
        assert_eq!(s.fabrication_snapshot(), snapshot); // No queued work sneaks past occupied tray.
        s.collect_fabricator_output();
        assert!(s.fabrication_snapshot().tray.is_none());
        assert!(s.ship.snapshot().total_mass_kg < ship_mass);
        let inventory = s.consumables.clone();
        s.collect_fabricator_output();
        assert_eq!(inventory, s.consumables);
    }
    let snapshot = request(&mut s, json!({"op":"inspect"}));
    assert_eq!(snapshot["consumables"]["food"][0]["recipe"], "food-ration");
    assert_eq!(snapshot["consumables"]["food"][1]["recipe"], "food-ration");
    assert_eq!(snapshot["consumables"]["medical"][0]["recipe"], "medkit");
    assert_eq!(
        s.equipment_presentation()
            .consumables
            .iter()
            .flatten()
            .count(),
        3
    );
    assert!(s.fabrication_snapshot().jobs.is_empty());
    let mass = s.ship.snapshot().total_mass_kg;
    request(
        &mut s,
        json!({"op":"key","key":"consumable_wheel","pressed":true}),
    );
    request(&mut s, json!({"op":"wheel_hover","slot":1}));
    for _ in 0..3 {
        request(&mut s, json!({"op":"mouse","button":"left","pressed":true}));
    }
    assert!(!s.wheel.open);
    assert!(
        !s.mining.input.held,
        "consumption press cannot click through to mining"
    );
    assert!(s.consumables.food()[1].is_none());
    assert!(s.consumables.food()[0].is_some());
    assert_eq!(s.ship.snapshot().total_mass_kg, mass);
    request(
        &mut s,
        json!({"op":"mouse","button":"left","pressed":false}),
    );
    request(
        &mut s,
        json!({"op":"key","key":"consumable_wheel","pressed":false}),
    );
    request(
        &mut s,
        json!({"op":"key","key":"consumable_wheel","pressed":true}),
    );
    request(&mut s, json!({"op":"wheel_hover","slot":3}));
    request(
        &mut s,
        json!({"op":"key","key":"wheel_confirm","pressed":true}),
    );
    assert!(s.consumables.medical()[0].is_none());
    request(
        &mut s,
        json!({"op":"key","key":"consumable_wheel","pressed":false}),
    );
}
#[test]
fn declarative_initial_contract_replays_through_portable_protocol() {
    let route: Value =
        serde_json::from_str(include_str!("../../../scenarios/fabricator-contract.json")).unwrap();
    let mut s = session();
    for step in route["steps"].as_array().unwrap() {
        if let Some(action) = step.get("action") {
            request(&mut s, action.clone());
        } else {
            let state = request(&mut s, json!({"op":"inspect"}));
            let check = &step["assert"];
            let mut value = &state;
            for part in check["path"].as_str().unwrap().split('.') {
                value = if let Ok(i) = part.parse::<usize>() {
                    &value[i]
                } else {
                    &value[part]
                };
            }
            if let (Some(actual), Some(expected)) = (value.as_f64(), check["equals"].as_f64()) {
                assert_eq!(actual, expected, "{check}");
            } else {
                assert_eq!(*value, check["equals"], "{check}");
            }
        }
    }
    let old = s.fabrication_snapshot();
    let response = automation::execute_session(
        &mut s,
        r#"{"protocol":1,"id":1,"op":"crafting","action":"invent"}"#,
    );
    assert_eq!(response["ok"], false);
    assert_eq!(s.fabrication_snapshot(), old);
}

#[test]
fn saved_midcraft_session_resumes_once_with_exact_clocks_energy_and_inventory() {
    let mut s = session();
    s.equipment
        .select(crate::equipment::ToolbarSlot::Three, false);
    for (i, resource) in [
        ResourceId::OrganicBiomass,
        ResourceId::MedicinalCompounds,
        ResourceId::WaterIce,
    ]
    .into_iter()
    .enumerate()
    {
        mine_carry_load(&mut s, resource, 2., 20_000 + i as u64);
    }
    for (recipe, quantity) in [("food-ration", 2), ("medkit", 1)] {
        s.execute_fabricator(C::Enqueue {
            recipe: RecipeId::new(recipe).unwrap(),
            quantity,
        })
        .unwrap();
    }
    s.advance_automation(Duration::from_millis(4003));
    let bytes = s.save_bytes(12345).unwrap();
    let mut restored = session();
    restored.restore_bytes(&bytes).unwrap();
    assert_eq!(bytes, restored.save_bytes(12345).unwrap());
    assert_eq!(s.fabrication_snapshot(), restored.fabrication_snapshot());
    assert_eq!(s.snapshot().backlog, Duration::from_millis(3));
    assert_eq!(s.elapsed, restored.elapsed);
    // Complete two rations then a medkit. Save/reload occupied slots and tray on
    // every boundary; neither ingredients nor energy may be committed twice.
    for (index, ms) in [6016, 10000, 15008].into_iter().enumerate() {
        s.advance_automation(Duration::from_millis(ms));
        restored.advance_automation(Duration::from_millis(ms));
        assert_eq!(s.fabrication_snapshot(), restored.fabrication_snapshot());
        assert_eq!(s.snapshot().ship, restored.snapshot().ship);
        assert!(s.fabrication_snapshot().tray.is_some());
        let occupied = restored.save_bytes(12345).unwrap();
        let mut duplicate: Value = serde_json::from_slice(&occupied).unwrap();
        duplicate["state"]["fabrication"]["inventory"]["food"][0] =
            duplicate["state"]["fabrication"]["machine"]["tray"].clone();
        assert!(
            restored
                .restore_bytes(&serde_json::to_vec(&duplicate).unwrap())
                .is_err()
        );
        assert_eq!(occupied, restored.save_bytes(12345).unwrap());
        restored.restore_bytes(&occupied).unwrap();
        assert_eq!(occupied, restored.save_bytes(12345).unwrap());
        s.collect_fabricator_output();
        restored.collect_fabricator_output();
        restored.collect_fabricator_output();
        assert_eq!(s.consumables, restored.consumables);
        assert_eq!(
            restored
                .consumables
                .food()
                .iter()
                .filter(|v| v.is_some())
                .count(),
            (index + 1).min(2)
        );
    }
    let saved = restored.save_bytes(12345).unwrap();
    restored.restore_bytes(&saved).unwrap();
    // Wheel/modal/hover/held mouse are transient. Reload the real inventory while
    // Q is open, then consume through the production exact-slot Q route.
    use crate::session::{Command, consumable_wheel::WheelAction};
    use salimon_fabrication::ConsumableSlot;
    restored.command(Command::Wheel(WheelAction::Hold(true)));
    restored.command(Command::Wheel(WheelAction::Hover(Some(
        ConsumableSlot::FoodThree,
    ))));
    restored.command(Command::Wheel(WheelAction::Primary(true)));
    assert!(restored.wheel.open && restored.wheel.primary_down);
    let open = restored.save_bytes(12345).unwrap();
    restored.restore_bytes(&open).unwrap();
    assert!(!restored.wheel.open && !restored.wheel.held && !restored.wheel.primary_down);
    assert!(restored.wheel.hovered.is_none());
    assert_eq!(saved, restored.save_bytes(12345).unwrap());
    for slot in [ConsumableSlot::FoodOne, ConsumableSlot::MedicalOne] {
        assert!(restored.consumables.item(slot).is_some());
        restored.command(Command::Wheel(WheelAction::Hold(true)));
        restored.command(Command::Wheel(WheelAction::Hover(Some(slot))));
        restored.command(Command::Wheel(WheelAction::Primary(true)));
        assert!(restored.consumables.item(slot).is_none());
        assert!(!restored.wheel.open);
        restored.command(Command::Mining(false));
        restored.command(Command::Wheel(WheelAction::Hold(false)));
    }
    assert_eq!(restored.consumables.food().iter().flatten().count(), 1);
    assert!(restored.consumables.medical().iter().all(Option::is_none));
}

#[test]
fn save_restore_blackout_is_atomic_and_never_restores_held_input_or_modal() {
    let mut s = session();
    mine_carry_load(&mut s, ResourceId::OrganicBiomass, 1., 30_000);
    mine_carry_load(&mut s, ResourceId::WaterIce, 1., 30_001);
    s.execute_fabricator(C::Enqueue {
        recipe: RecipeId::new("food-ration").unwrap(),
        quantity: 1,
    })
    .unwrap();
    s.advance_automation(Duration::from_millis(1008));
    s.ship.set_stored_core_energy_joules(0);
    s.advance_automation(Duration::from_millis(16));
    s.movement_input.forward = true;
    s.crafting_ui.open = true;
    let bytes = s.save_bytes(10).unwrap();
    let before = s.fabrication_snapshot();
    s.restore_bytes(&bytes).unwrap();
    assert!(!s.movement_input.forward);
    assert!(!s.crafting_ui.open);
    assert_eq!(before, s.fabrication_snapshot());
    s.advance_automation(Duration::from_millis(1008));
    assert_eq!(before.jobs, s.fabrication_snapshot().jobs);
    let mut bad: Value = serde_json::from_slice(&bytes).unwrap();
    bad["state"]["fabrication"]["machine"]["jobs"][0]["progress"] = json!({"secs":999,"nanos":0});
    let current = s.save_bytes(11).unwrap();
    assert!(s.restore_bytes(&serde_json::to_vec(&bad).unwrap()).is_err());
    assert_eq!(current, s.save_bytes(11).unwrap());
    s.ship.exchange_core_energy_joules(1e8, 0.).unwrap();
    s.advance_automation(Duration::from_secs(10));
    assert!(s.fabrication_snapshot().tray.is_some());
}

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

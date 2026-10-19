use super::*;
use salimon_fabrication::{FabricationCommand as C, Mass, RecipeId};
use salimon_persistence::Validate;
use salimon_survival::{
    Command, Points, Survival,
    consumables::{Benefit, Catalog},
};
use serde_json::json;
use std::time::Duration;
fn item(recipe: &str, job: u64, category: OutputCategory) -> OutputItem {
    OutputItem {
        recipe: RecipeId::new(recipe).unwrap(),
        job: JobId(job),
        unit: 1,
        mass: Mass::from_milligrams(1000).unwrap(),
        category,
    }
}
fn select(slot: ConsumableSlot, job: u64) -> Selection {
    Selection {
        slot,
        job: JobId(job),
        unit: 1,
    }
}
fn session() -> GameplaySession {
    GameplaySession {
        automation_step: Some(Duration::from_millis(16)),
        ..Default::default()
    }
}
fn set_meters(s: &mut GameplaySession) {
    let mut v = serde_json::to_value(&s.survival).unwrap();
    v["hunger"] = serde_json::to_value(Points::whole(10).unwrap()).unwrap();
    v["fatigue"] = serde_json::to_value(Points::ZERO).unwrap();
    s.survival = serde_json::from_value(v).unwrap();
    s.survival.validate().unwrap();
}
#[test]
fn exact_slot_consumption_works_when_exhausted_never_compacts_or_changes_ship_mass() {
    let mut s = session();
    set_meters(&mut s);
    for job in 1..=3 {
        s.consumables
            .insert(item("food-ration", job, OutputCategory::Food))
            .unwrap();
    }
    let ship = s.snapshot().ship;
    let result = s
        .consume_selection(select(ConsumableSlot::FoodTwo, 2))
        .unwrap();
    assert_eq!(result.job, JobId(2));
    assert_eq!(s.survival.snapshot().hunger, Points::whole(50).unwrap());
    assert_eq!(s.consumables.food()[0].as_ref().unwrap().job, JobId(1));
    assert!(s.consumables.food()[1].is_none());
    assert_eq!(s.consumables.food()[2].as_ref().unwrap().job, JobId(3));
    assert_eq!(s.snapshot().ship, ship);
    assert_eq!(
        s.consume_selection(select(ConsumableSlot::FoodTwo, 2)),
        Err(ConsumeError::Empty)
    );
    s.consumables
        .insert(item("food-ration", 4, OutputCategory::Food))
        .unwrap();
    let original = s.consumables.clone();
    let vitals = s.survival.clone();
    assert_eq!(
        s.consume_selection(select(ConsumableSlot::FoodTwo, 2)),
        Err(ConsumeError::StaleSelection)
    );
    assert_eq!(s.consumables, original);
    assert_eq!(s.survival, vitals);
}
#[test]
fn full_meter_waste_unknown_category_and_effect_failures_preserve_transaction_contract() {
    let mut s = session();
    s.consumables
        .insert(item("food-ration", 1, OutputCategory::Food))
        .unwrap();
    s.consume_selection(select(ConsumableSlot::FoodOne, 1))
        .unwrap();
    assert_eq!(s.survival.snapshot().hunger, Points::FULL);
    for (recipe, category, error) in [
        (
            "unknown-food",
            OutputCategory::Food,
            ConsumeError::UnknownEffect,
        ),
        (
            "medkit",
            OutputCategory::Food,
            ConsumeError::CategoryMismatch,
        ),
    ] {
        s.consumables = Default::default();
        s.consumables.insert(item(recipe, 2, category)).unwrap();
        let inventory = s.consumables.clone();
        let state = s.survival.clone();
        assert_eq!(
            s.consume_selection(select(ConsumableSlot::FoodOne, 2)),
            Err(error)
        );
        assert_eq!(s.consumables, inventory);
        assert_eq!(s.survival, state);
    }
    s.consumables = Default::default();
    s.consumables
        .insert(item("medkit", 3, OutputCategory::Medical))
        .unwrap();
    let kit = Catalog::starter().get("medkit").unwrap().clone();
    s.survival.apply_consumable(&kit).unwrap();
    let mut changed = kit;
    if let Benefit::OverTime { milli_points, .. } = &mut changed.benefits[0] {
        *milli_points = 3000;
    }
    s.consumable_effects = Catalog::new(vec![changed]).unwrap();
    let original = s.consumables.clone();
    let state = s.survival.clone();
    assert_eq!(
        s.consume_selection(select(ConsumableSlot::MedicalOne, 3)),
        Err(ConsumeError::InvalidEffect)
    );
    assert_eq!(s.consumables, original);
    assert_eq!(s.survival, state);
}
#[test]
fn guards_preserve_inventory_during_crafting_tour_and_terminal_state() {
    let mut s = session();
    s.consumables
        .insert(item("food-ration", 1, OutputCategory::Food))
        .unwrap();
    let original = s.consumables.clone();
    s.crafting_ui.open = true;
    assert_eq!(
        s.consume_selection(select(ConsumableSlot::FoodOne, 1)),
        Err(ConsumeError::Unavailable)
    );
    s.crafting_ui.open = false;
    s.toggle_view();
    assert_eq!(
        s.consume_selection(select(ConsumableSlot::FoodOne, 1)),
        Err(ConsumeError::Unavailable)
    );
    s.toggle_view();
    s.survival.command(Command::Damage(Points::FULL)).unwrap();
    assert_eq!(
        s.consume_selection(select(ConsumableSlot::FoodOne, 1)),
        Err(ConsumeError::Unavailable)
    );
    assert_eq!(s.consumables, original);
}
#[test]
fn collected_crafted_item_uses_protocol_exact_identity_and_restored_effect_timers_do_not_duplicate()
{
    use salimon_world::resources::{RawMaterial, ResourceId};
    let mut s = session();
    for id in [ResourceId::MedicinalCompounds, ResourceId::WaterIce] {
        s.execute_fabricator(C::Insert(RawMaterial::new(id, 1.0).unwrap()))
            .unwrap();
    }
    s.execute_fabricator(C::Enqueue {
        recipe: RecipeId::new("medkit").unwrap(),
        quantity: 1,
    })
    .unwrap();
    s.advance_fabrication(Duration::from_secs(15));
    s.collect_fabricator_output();
    s.survival
        .command(Command::Damage(Points::whole(60).unwrap()))
        .unwrap();
    let selected = s.consumables.medical()[0].as_ref().unwrap();
    let request = json!({"protocol":1,"id":1,"op":"consume","slot":4,"job":selected.job.0,"unit":selected.unit});
    let mass = s.snapshot().ship.total_mass_kg;
    let result = crate::automation::execute_session(&mut s, &request.to_string());
    assert_eq!(result["result"]["applied"], true);
    let again = crate::automation::execute_session(&mut s, &request.to_string());
    assert_eq!(again["result"]["applied"], false);
    assert!(s.consumables.medical()[0].is_none());
    assert_eq!(s.snapshot().ship.total_mass_kg, mass);
    s.survival.advance(Duration::from_millis(1250));
    // #371 composes these existing serializable owners into the full-game envelope.
    let encoded = serde_json::to_vec(&(s.survival.clone(), s.consumables.clone())).unwrap();
    let (mut restored, inventory): (Survival, salimon_fabrication::ConsumableInventory) =
        serde_json::from_slice(&encoded).unwrap();
    restored.validate().unwrap();
    assert!(inventory.medical()[0].is_none());
    let before = restored.active_effects().collect::<Vec<_>>();
    restored.recover_offline(0, 3600);
    assert_eq!(restored.active_effects().collect::<Vec<_>>(), before);
    restored.advance(Duration::from_millis(8750));
    assert_eq!(restored.snapshot().health, Points::whole(65).unwrap());
    assert_eq!(restored.active_effects().count(), 0);
}
#[test]
fn malformed_and_stale_protocol_requests_cannot_consume_replacement_items() {
    let mut s = session();
    s.consumables
        .insert(item("food-ration", 1, OutputCategory::Food))
        .unwrap();
    for slot in [0, 6] {
        let result = crate::automation::execute_session(
            &mut s,
            &json!({"protocol":1,"id":1,"op":"consume","slot":slot,"job":1,"unit":1}).to_string(),
        );
        assert_eq!(result["ok"], false);
    }
    let result = crate::automation::execute_session(
        &mut s,
        &json!({"protocol":1,"id":1,"op":"consume","slot":1,"job":99,"unit":1}).to_string(),
    );
    assert_eq!(result["result"]["applied"], false);
    assert!(s.consumables.food()[0].is_some());
}

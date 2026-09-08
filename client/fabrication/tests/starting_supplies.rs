use salimon_fabrication::{ConsumableSlot, FabricationSaveState, Fabricator, JobId, RecipeId};
use std::time::Duration;

#[test]
fn starting_grant_uses_recipe_values_exact_bins_and_the_shared_identity_allocator() {
    let (mut machine, mut inventory) = Fabricator::starter_with_supplies();
    let (again, same_inventory) = Fabricator::starter_with_supplies();
    assert_eq!(machine, again);
    assert_eq!(inventory, same_inventory);
    let ration = machine
        .recipes()
        .get(&RecipeId::new("food-ration").unwrap())
        .unwrap()
        .clone();
    let medical = machine
        .recipes()
        .get(&RecipeId::new("medkit").unwrap())
        .unwrap()
        .clone();
    for (slot, recipe, job, unit) in [
        (ConsumableSlot::FoodOne, &ration, JobId(1), 1),
        (ConsumableSlot::FoodTwo, &ration, JobId(1), 2),
        (ConsumableSlot::MedicalOne, &medical, JobId(2), 1),
    ] {
        let item = inventory.item(slot).unwrap();
        assert_eq!(item.recipe, recipe.id);
        assert_eq!(item.mass, recipe.output_mass);
        assert_eq!(item.category, recipe.output_category);
        assert_eq!((item.job, item.unit), (job, unit));
    }
    assert!(inventory.item(ConsumableSlot::FoodThree).is_none());
    assert!(inventory.item(ConsumableSlot::MedicalTwo).is_none());
    let snapshot = machine.snapshot(true);
    assert!(snapshot.jobs.is_empty());
    assert!(snapshot.tray.is_none());
    assert_eq!(snapshot.requested_watts, 0.0);
    for bin in &snapshot.stored {
        let expected = ration
            .ingredients
            .iter()
            .find(|i| i.resource == bin.resource)
            .map_or(0, |i| i.mass.milligrams());
        assert_eq!(bin.milligrams, expected);
    }
    assert_eq!(
        machine.mass_contribution().stored_milligrams,
        ration
            .ingredients
            .iter()
            .map(|i| i.mass.milligrams())
            .sum::<u64>()
    );
    let (restored, items) = FabricationSaveState::capture(&machine, &inventory)
        .restore()
        .unwrap();
    assert_eq!(restored, machine);
    assert_eq!(items, inventory);
    assert!(machine.enqueue(&ration.id, 2).is_err());
    assert_eq!(machine, again); // Insufficient batch never partly commits supplies.
    assert_eq!(machine.enqueue(&ration.id, 1).unwrap(), JobId(3));
    assert_eq!(machine.mass_contribution().stored_milligrams, 0);
    assert!(machine.enqueue(&ration.id, 1).is_err());
    assert!(machine.step(ration.duration, true).produced_output);
    let output = machine.collect().unwrap();
    assert_eq!((output.job, output.unit), (JobId(3), 1));
    inventory.insert(output).unwrap();
    assert!(machine.collect().is_err());
    let (mut restored, inventory) = FabricationSaveState::capture(&machine, &inventory)
        .restore()
        .unwrap();
    assert_eq!(inventory.food().iter().flatten().count(), 3);
    assert_eq!(
        restored
            .step(Duration::from_secs(100), true)
            .consumed_joules,
        0.0
    );
}

use salimon_fabrication::*;
use salimon_world::resources::{RESOURCE_CATALOG, RawMaterial, ResourceId};
use std::time::Duration;

fn id(key: &str) -> RecipeId {
    RecipeId::new(key).unwrap()
}
fn mg(value: u64) -> Mass {
    Mass::from_milligrams(value).unwrap()
}
fn seconds(n: u64) -> Duration {
    Duration::from_secs(n)
}
fn insert(f: &mut Fabricator, resource: ResourceId, kg: f64) -> Insertion {
    f.insert(RawMaterial::new(resource, kg).unwrap()).unwrap()
}
fn stocked() -> Fabricator {
    let mut f = Fabricator::starter();
    for resource in RESOURCE_CATALOG {
        insert(&mut f, resource.id, 20.0);
    }
    f
}
fn balance(f: &Fabricator, resource: ResourceId) -> u64 {
    f.snapshot(true)
        .stored
        .iter()
        .find(|s| s.resource == resource)
        .unwrap()
        .milligrams
}
fn recipe() -> RecipeDefinition {
    RecipeBook::starter().recipes()[0].clone()
}
fn custom(recipe: RecipeDefinition, capacity: usize) -> Fabricator {
    Fabricator::new(RecipeBook::new(vec![recipe]).unwrap(), capacity).unwrap()
}

#[test]
fn starter_data_uses_world_feedstocks_and_two_unlocked_products() {
    let book = RecipeBook::starter();
    assert_eq!(book.recipes().len(), 6);
    let ration = book.get(&id("food-ration")).unwrap();
    assert_eq!(ration.output_category, OutputCategory::Food);
    assert_eq!(ration.duration, seconds(10));
    assert!(
        ration
            .ingredients
            .iter()
            .any(|i| i.resource == ResourceId::OrganicBiomass)
    );
    assert_eq!(
        book.get(&id("medkit")).unwrap().output_category,
        OutputCategory::Medical
    );
    for r in book.recipes() {
        assert_eq!(
            r.unlock.unlocked_by_default,
            matches!(
                r.id.key(),
                "food-ration" | "medkit" | "data-recovery-module"
            )
        );
        assert!(r.unlock.prerequisites.is_empty());
    }
}

#[test]
fn mass_quantization_is_once_and_rejects_invalid_or_unrepresentable_input() {
    for kg in [
        0.0,
        -1.0,
        f64::NAN,
        f64::INFINITY,
        f64::NEG_INFINITY,
        f64::MAX,
        0.000_000_49,
    ] {
        assert_eq!(Mass::from_kg(kg), Err(FabricationError::InvalidMass));
    }
    assert_eq!(Mass::from_milligrams(0), Err(FabricationError::InvalidMass));
    assert_eq!(Mass::from_kg(0.000_000_5).unwrap(), mg(1));
    assert_eq!(Mass::from_kg(0.123_456_7).unwrap(), mg(123_457));
    assert_eq!(mg(1_000_000).kg(), 1.0);
    assert!(Mass::from_kg(u64::MAX as f64 / 1_000_000.0).is_err());
}

#[test]
fn every_bin_is_independently_bounded_and_full_insertion_is_atomic_rejection() {
    let mut f = Fabricator::starter();
    for resource in RESOURCE_CATALOG {
        let first = insert(&mut f, resource.id, 19.999_999);
        assert_eq!(first.accepted_milligrams, 19_999_999);
        let overflow = insert(&mut f, resource.id, 0.000_003);
        assert_eq!(overflow.accepted_milligrams, 1);
        assert_eq!(overflow.discarded_milligrams, 2);
        let before = f.clone();
        assert_eq!(
            f.insert(RawMaterial::new(resource.id, 50.0).unwrap()),
            Err(FabricationError::StorageFull(resource.id))
        );
        assert_eq!(f, before);
        assert_eq!(balance(&f, resource.id), STORAGE_CAPACITY_MILLIGRAMS);
    }
    assert_eq!(f.mass_contribution().stored_milligrams, 100_000_000);
}

#[test]
fn invalid_insert_preserves_all_authoritative_state() {
    let mut f = stocked();
    let original = f.clone();
    for kg in [1e-9, f64::MAX] {
        let material = RawMaterial::new(ResourceId::IronOre, kg).unwrap();
        assert_eq!(f.insert(material), Err(FabricationError::InvalidMass));
        assert_eq!(f, original);
    }
}

#[test]
fn repeated_small_insertions_and_batch_reservation_have_exact_integer_balances() {
    let mut f = Fabricator::starter();
    for _ in 0..1000 {
        insert(&mut f, ResourceId::OrganicBiomass, 0.001);
    }
    insert(&mut f, ResourceId::WaterIce, 0.2);
    f.enqueue(&id("food-ration"), 2).unwrap();
    assert_eq!(balance(&f, ResourceId::OrganicBiomass), 200_000);
    assert_eq!(balance(&f, ResourceId::WaterIce), 0);
    assert_eq!(f.snapshot(true).jobs[0].remaining_quantity, 2);
}

#[test]
fn enqueue_reserves_entire_quantity_and_failures_do_not_partially_debit() {
    let mut f = Fabricator::starter();
    insert(&mut f, ResourceId::OrganicBiomass, 5.0);
    let original = f.clone();
    assert_eq!(
        f.enqueue(&id("food-ration"), 2),
        Err(FabricationError::InsufficientMaterial(ResourceId::WaterIce))
    );
    assert_eq!(f, original);
    assert_eq!(
        f.enqueue(&id("food-ration"), 0),
        Err(FabricationError::InvalidQuantity)
    );
    assert_eq!(
        f.enqueue(&id("missing"), 1),
        Err(FabricationError::UnknownRecipe)
    );
    assert_eq!(f, original);
    insert(&mut f, ResourceId::WaterIce, 0.2);
    let job = f.enqueue(&id("food-ration"), 2).unwrap();
    assert_eq!(job, JobId(1));
    assert_eq!(balance(&f, ResourceId::OrganicBiomass), 4_200_000);
    assert_eq!(balance(&f, ResourceId::WaterIce), 0);
}

#[test]
fn multiplication_overflow_is_atomic_even_after_an_earlier_sufficient_ingredient() {
    let mut r = recipe();
    r.ingredients = vec![
        Ingredient {
            resource: ResourceId::WaterIce,
            mass: mg(1),
        },
        Ingredient {
            resource: ResourceId::IronOre,
            mass: mg(u64::MAX),
        },
    ];
    let mut f = custom(r, 2);
    insert(&mut f, ResourceId::WaterIce, 1.0);
    let original = f.clone();
    assert_eq!(
        f.enqueue(&id("food-ration"), 2),
        Err(FabricationError::ArithmeticOverflow)
    );
    assert_eq!(f, original);
}

#[test]
fn queue_capacity_counts_jobs_and_rejection_keeps_balances_and_ids() {
    let mut f = custom(recipe(), 1);
    insert(&mut f, ResourceId::OrganicBiomass, 20.0);
    insert(&mut f, ResourceId::WaterIce, 20.0);
    let a = f.enqueue(&id("food-ration"), 2).unwrap();
    let original = f.clone();
    assert_eq!(
        f.enqueue(&id("food-ration"), 1),
        Err(FabricationError::QueueFull)
    );
    assert_eq!(f, original);
    f.cancel(a).unwrap();
    assert_eq!(f.enqueue(&id("food-ration"), 1).unwrap(), JobId(2));
    assert_eq!(
        Fabricator::new(RecipeBook::starter(), 0),
        Err(FabricationError::InvalidQueueCapacity)
    );
}

#[test]
fn strict_fifo_batches_do_not_interleave_and_only_head_has_progress() {
    let mut f = stocked();
    let first = f.enqueue(&id("food-ration"), 2).unwrap();
    let second = f.enqueue(&id("medkit"), 1).unwrap();
    f.step(seconds(5), true);
    let snap = f.snapshot(true);
    assert_eq!(snap.jobs[0].progress, seconds(5));
    assert_eq!(snap.jobs[1].progress, Duration::ZERO);
    for unit in 1..=2 {
        let result = f.step(seconds(20), true);
        assert!(result.produced_output);
        let item = f.collect().unwrap();
        assert_eq!((item.job, item.unit), (first, unit));
    }
    assert_eq!(f.snapshot(true).jobs[0].id, second);
    assert_eq!(f.snapshot(true).jobs[0].progress, Duration::ZERO);
    f.step(seconds(15), true);
    assert_eq!(f.collect().unwrap().job, second);
    assert!(f.snapshot(true).jobs.is_empty());
}

#[test]
fn huge_delta_stops_at_one_output_and_never_banks_blocked_time() {
    let mut f = stocked();
    f.enqueue(&id("food-ration"), 3).unwrap();
    let step = f.step(Duration::MAX, true);
    assert_eq!(step.worked, seconds(10));
    assert_eq!(step.unused, Duration::MAX - seconds(10));
    assert_eq!(step.consumed_joules, 100_000.0);
    let blocked = f.clone();
    assert_eq!(f.step(Duration::MAX, true).worked, Duration::ZERO);
    assert_eq!(f, blocked);
    assert_eq!(f.power_demand(), None);
    assert_eq!(f.snapshot(true).status, FabricatorStatus::TrayBlocked);
    f.collect().unwrap();
    assert_eq!(f.step(seconds(9), true).worked, seconds(9));
    assert!(f.snapshot(true).tray.is_none());
    assert!(f.step(seconds(1), true).produced_output);
}

#[test]
fn power_interruption_preserves_progress_and_never_consumes_blackout_time() {
    let mut f = stocked();
    f.enqueue(&id("medkit"), 1).unwrap();
    assert_eq!(f.power_demand().unwrap().watts, 20_000.0);
    assert_eq!(f.step(seconds(4), true).consumed_joules, 80_000.0);
    let before = f.clone();
    let unpowered = f.step(seconds(500), false);
    assert_eq!(unpowered.worked, Duration::ZERO);
    assert_eq!(unpowered.consumed_joules, 0.0);
    assert_eq!(f, before);
    assert_eq!(f.snapshot(false).status, FabricatorStatus::NoPower);
    assert_eq!(f.power_demand().unwrap().until_output, seconds(11));
    assert!(f.step(seconds(11), true).produced_output);
    assert_eq!(f.step(seconds(5), false).consumed_joules, 0.0);
}

#[test]
fn exact_duration_partition_and_partial_power_funding_are_deterministic() {
    let mut a = stocked();
    a.enqueue(&id("food-ration"), 2).unwrap();
    let mut b = a.clone();
    assert_eq!(a.step(seconds(3), true).consumed_joules, 30_000.0);
    for _ in 0..300 {
        b.step(Duration::from_millis(10), true);
    }
    assert_eq!(a, b);
    a.step(seconds(7), false);
    for _ in 0..700 {
        b.step(Duration::from_millis(10), false);
    }
    assert_eq!(a, b);
    a.step(seconds(7), true);
    for _ in 0..700 {
        b.step(Duration::from_millis(10), true);
    }
    assert_eq!(a, b);
    assert_eq!(a.collect(), b.collect());
    a.step(seconds(10), true);
    b.step(seconds(1000), true);
    assert_eq!(a, b);
}

#[test]
fn nanosecond_progress_and_zero_delta_have_no_rounding_residue() {
    let mut r = recipe();
    r.duration = Duration::from_nanos(3);
    let mut f = custom(r, 1);
    insert(&mut f, ResourceId::OrganicBiomass, 1.0);
    insert(&mut f, ResourceId::WaterIce, 1.0);
    f.enqueue(&id("food-ration"), 1).unwrap();
    let original = f.clone();
    assert_eq!(f.step(Duration::ZERO, true).worked, Duration::ZERO);
    assert_eq!(f, original);
    for _ in 0..2 {
        assert!(!f.step(Duration::from_nanos(1), true).produced_output);
    }
    assert!(f.step(Duration::from_nanos(1), true).produced_output);
}

#[test]
fn cancelling_active_and_waiting_jobs_never_refunds_and_preserves_fifo_survivors() {
    let mut f = stocked();
    let a = f.enqueue(&id("food-ration"), 2).unwrap();
    let b = f.enqueue(&id("medkit"), 1).unwrap();
    let c = f.enqueue(&id("food-ration"), 1).unwrap();
    f.step(seconds(5), true);
    let mass = f.mass_contribution();
    f.cancel(b).unwrap();
    assert_eq!(
        f.snapshot(true)
            .jobs
            .iter()
            .map(|j| j.id)
            .collect::<Vec<_>>(),
        vec![a, c]
    );
    f.cancel(a).unwrap();
    assert_eq!(f.mass_contribution(), mass);
    assert_eq!(f.snapshot(true).jobs[0].id, c);
    assert_eq!(f.snapshot(true).jobs[0].progress, Duration::ZERO);
    let original = f.clone();
    assert_eq!(f.cancel(a), Err(FabricationError::UnknownJob));
    assert_eq!(f, original);
}

#[test]
fn cancelling_partial_batch_keeps_completed_tray_item_and_does_not_refund_remaining_units() {
    let mut f = stocked();
    let job = f.enqueue(&id("food-ration"), 2).unwrap();
    f.step(seconds(10), true);
    let mass = f.mass_contribution();
    let tray = f.snapshot(true).tray;
    f.cancel(job).unwrap();
    assert_eq!(f.mass_contribution(), mass);
    assert_eq!(f.snapshot(true).tray, tray);
    assert_eq!(f.collect().unwrap().unit, 1);
    assert_eq!(f.snapshot(true).status, FabricatorStatus::Idle);
}

#[test]
fn collection_is_exactly_once_with_stable_item_ids_and_does_not_advance_next_job() {
    let mut f = stocked();
    let a = f.enqueue(&id("food-ration"), 1).unwrap();
    let b = f.enqueue(&id("medkit"), 1).unwrap();
    f.step(seconds(10), true);
    let item = f.collect().unwrap();
    assert_eq!((item.job, item.unit), (a, 1));
    let original = f.clone();
    assert_eq!(f.collect(), Err(FabricationError::EmptyTray));
    assert_eq!(f, original);
    assert_eq!(f.snapshot(true).jobs[0].id, b);
    assert_eq!(f.snapshot(true).jobs[0].progress, Duration::ZERO);
}

#[test]
fn deliberate_ship_mass_exception_is_exact_at_every_domain_transition() {
    let mut f = Fabricator::starter();
    assert_eq!(f.mass_contribution().kg(), 0.0);
    insert(&mut f, ResourceId::OrganicBiomass, 21.0); // one kg discarded
    insert(&mut f, ResourceId::WaterIce, 1.0);
    assert_eq!(f.mass_contribution().stored_milligrams, 21_000_000);
    let job = f.enqueue(&id("food-ration"), 2).unwrap();
    assert_eq!(f.mass_contribution().stored_milligrams, 20_000_000);
    f.step(seconds(5), true);
    assert_eq!(f.mass_contribution().tray_milligrams, 0);
    f.step(seconds(5), true);
    assert_eq!(f.mass_contribution().tray_milligrams, 300_000);
    f.cancel(job).unwrap();
    assert_eq!(f.mass_contribution().stored_milligrams, 20_000_000);
    f.collect().unwrap(); // carried consumable inventory deliberately adds no mass
    assert_eq!(f.mass_contribution().kg(), 20.0);
    assert_eq!(f.mass_contribution().tray_milligrams, 0);
}

#[test]
fn snapshots_are_observations_and_idle_never_requests_or_consumes_power() {
    let mut f = Fabricator::starter();
    let original = f.clone();
    assert_eq!(f.snapshot(false).status, FabricatorStatus::Idle);
    assert_eq!(f.snapshot(true).requested_watts, 0.0);
    assert_eq!(f.step(Duration::MAX, true).consumed_joules, 0.0);
    let mut snapshot = f.snapshot(true);
    snapshot.stored[0].milligrams = u64::MAX;
    assert_eq!(f, original);
}

#[test]
fn advanced_unlocks_are_schema_only_and_enqueue_is_rejected_atomically() {
    let mut r = recipe();
    r.unlock = UnlockRequirements {
        unlocked_by_default: false,
        prerequisites: vec!["research-botany".into()],
    };
    r.output_category = OutputCategory::Component;
    let mut f = custom(r, 2);
    let original = f.clone();
    assert_eq!(
        f.enqueue(&id("food-ration"), 1),
        Err(FabricationError::LockedRecipe)
    );
    assert_eq!(f, original);
}

#[test]
fn invalid_recipe_records_and_ambiguous_catalogs_are_rejected() {
    for key in ["", "Food Ration", "x/y", "-x", "x-", "med_kit"] {
        assert_eq!(RecipeId::new(key), Err(FabricationError::InvalidRecipeId));
    }
    assert_eq!(
        RecipeBook::new(vec![recipe(), recipe()]),
        Err(FabricationError::DuplicateRecipe)
    );
    let mut r = recipe();
    r.ingredients.clear();
    assert_eq!(
        RecipeBook::new(vec![r]),
        Err(FabricationError::InvalidRecipe)
    );
    let mut r = recipe();
    r.ingredients.push(r.ingredients[0].clone());
    assert_eq!(
        RecipeBook::new(vec![r]),
        Err(FabricationError::DuplicateIngredient)
    );
    let mut underflow = recipe();
    underflow.watts = f64::from_bits(1);
    underflow.duration = Duration::from_nanos(1);
    assert_eq!(
        RecipeBook::new(vec![underflow]),
        Err(FabricationError::InvalidRecipe)
    );
    let mut r = recipe();
    r.duration = Duration::ZERO;
    assert_eq!(
        RecipeBook::new(vec![r]),
        Err(FabricationError::InvalidRecipe)
    );
    for watts in [0.0, -1.0, f64::NAN, f64::INFINITY, f64::MAX] {
        let mut r = recipe();
        r.watts = watts;
        assert_eq!(
            RecipeBook::new(vec![r]),
            Err(FabricationError::InvalidRecipe)
        );
    }
    for prerequisites in [vec!["bad key".into()], vec!["x".into(), "x".into()]] {
        let mut r = recipe();
        r.unlock = UnlockRequirements {
            unlocked_by_default: false,
            prerequisites,
        };
        assert_eq!(
            RecipeBook::new(vec![r]),
            Err(FabricationError::InvalidUnlockRequirement)
        );
    }
    let mut r = recipe();
    r.unlock.prerequisites.push("research".into());
    assert_eq!(
        RecipeBook::new(vec![r]),
        Err(FabricationError::InvalidUnlockRequirement)
    );
}

#[test]
fn typed_commands_follow_the_same_transaction_paths() {
    let mut f = Fabricator::starter();
    assert!(matches!(
        f.execute(FabricationCommand::Insert(
            RawMaterial::new(ResourceId::OrganicBiomass, 1.0).unwrap()
        ))
        .unwrap(),
        CommandResult::Inserted(_)
    ));
    f.execute(FabricationCommand::Insert(
        RawMaterial::new(ResourceId::WaterIce, 1.0).unwrap(),
    ))
    .unwrap();
    let result = f
        .execute(FabricationCommand::Enqueue {
            recipe: id("food-ration"),
            quantity: 1,
        })
        .unwrap();
    let CommandResult::Enqueued(job) = result else {
        panic!("expected job")
    };
    assert_eq!(
        f.execute(FabricationCommand::Cancel(job)),
        Ok(CommandResult::Cancelled)
    );
    assert_eq!(
        f.execute(FabricationCommand::Collect),
        Err(FabricationError::EmptyTray)
    );
    f.execute(FabricationCommand::Enqueue {
        recipe: id("food-ration"),
        quantity: 1,
    })
    .unwrap();
    f.step(seconds(10), true);
    assert!(matches!(
        f.execute(FabricationCommand::Collect),
        Ok(CommandResult::Collected(_))
    ));
}

#[test]
fn data_driven_component_recipe_rejects_invalid_wire_mass_and_identity() {
    let original = serde_json::to_value(RecipeBook::recovery_module()).unwrap();
    for (field, value) in [
        ("output_mass", serde_json::json!(0)),
        ("id", serde_json::json!("INVALID KEY")),
    ] {
        let mut bad = original.clone();
        bad[field] = value;
        let recipe = serde_json::from_value(bad).unwrap();
        assert!(RecipeBook::new(vec![recipe]).is_err());
    }
    let mut bad = original;
    bad["ingredients"][0]["mass"] = serde_json::json!(0);
    assert!(RecipeBook::new(vec![serde_json::from_value(bad).unwrap()]).is_err());
}

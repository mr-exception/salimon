//! Deterministic balance scenarios. Travel budgets are explicit fixture inputs,
//! not claims about player pathfinding or subjective native difficulty.
use crate::session::{Command, GameplaySession, consumable_wheel::WheelAction};
use salimon_fabrication::{ConsumableSlot, FabricationCommand, RecipeId};
use salimon_world::{WorldPosition, resources::ResourceId};
use serde_json::{Value, json};
use std::time::Duration;

const TICK: Duration = Duration::from_millis(16);
fn advance(s: &mut GameplaySession, seconds: f64) {
    let mut remaining = Duration::from_secs_f64(seconds);
    // The production session's exact survival/power/physical update; travel is a
    // declared waiting budget between fixture placements, not simulated navigation.
    while !remaining.is_zero() {
        let dt = remaining.min(Duration::from_secs(1));
        s.advance_game(dt);
        remaining -= dt;
    }
}
fn eat(s: &mut GameplaySession, slot: ConsumableSlot) {
    for a in [
        WheelAction::Hold(false),
        WheelAction::Hold(true),
        WheelAction::Hover(Some(slot)),
        WheelAction::Primary(false),
        WheelAction::Primary(true),
        WheelAction::Hold(false),
    ] {
        s.command(Command::Wheel(a));
    }
    assert!(s.consumables.item(slot).is_none());
}
fn meters(s: &mut GameplaySession, hunger: u32, health: u32, fatigue: u32) {
    let mut value = serde_json::to_value(&s.survival).unwrap();
    for (key, amount) in [("hunger", hunger), ("health", health), ("fatigue", fatigue)] {
        value[key] =
            serde_json::to_value(salimon_survival::Points::whole(amount).unwrap()).unwrap();
    }
    s.survival = serde_json::from_value(value).unwrap();
}
fn fixture_pose(s: &mut GameplaySession, position: Value, eye: [f64; 3], target: [f64; 3]) {
    let d: [f64; 3] = std::array::from_fn(|i| target[i] - eye[i]);
    let mut c = serde_json::to_value(s.character.save_state()).unwrap();
    c["position"] = position;
    c["yaw_radians"] = d[2].atan2(d[0]).into();
    c["pitch_radians"] = d[1].atan2(d[0].hypot(d[2])).into();
    s.character =
        salimon_character::CharacterController::from_save(serde_json::from_value(c).unwrap(), None)
            .unwrap();
}
fn inside(s: &mut GameplaySession, target: [f64; 3]) {
    let eye = [-3.0, 1.65, 2.0];
    fixture_pose(s, json!({"Inside":{"local":eye}}), eye, target);
}
fn gather(s: &mut GameplaySession, resource: ResourceId, required_kg: f64, seed: u64) -> f64 {
    let origin = s.ship.snapshot().pose.position_meters;
    let deposits = s.mining.nearby(origin, seed).unwrap();
    let deposit = deposits
        .iter()
        .filter(|d| d.deposit.material().resource() == resource)
        .min_by(|a, b| {
            let distance = |d: &salimon_world::resource_generation::SurfaceDeposit| {
                d.deposit
                    .position()
                    .offset_from(WorldPosition::new(origin[0], origin[1], origin[2]))
                    .iter()
                    .map(|v| v * v)
                    .sum::<f64>()
            };
            distance(a).total_cmp(&distance(b))
        })
        .expect("selected Earth fixture contains both food feedstocks");
    let p = deposit.deposit.position().meters();
    let eye = [p[0] - 2.0, p[1] + 1.65, p[2]];
    fixture_pose(s, json!({"Surface":{"world":eye}}), eye, p);
    s.seed = seed;
    s.command(Command::Select(crate::equipment::ToolbarSlot::One));
    s.command(Command::Mining(false));
    s.command(Command::Mining(true));
    let started = s.elapsed;
    let mut released = None;
    for _ in 0..2000 {
        s.advance_automation(TICK);
        if let Some(id) = s.mining.session.attached_sphere_id()
            && s.mining.session.fragment(id).unwrap().material().mass_kg() >= required_kg
        {
            released = Some(id);
            break;
        }
    }
    s.command(Command::Mining(false));
    s.advance_automation(TICK);
    let id = released.expect("production aim and mining reach required ration feedstock");
    s.policy.set_automation_step(Some(TICK));
    let request = json!({"protocol":1,"id":1,"op":"aim_fragment","fragment_id":id.0});
    let response = crate::automation::execute_session(s, &request.to_string());
    assert_eq!(response["ok"], true, "{response}");
    s.command(Command::GrabDrop(true));
    assert_eq!(s.mining.session.carried_id(), Some(id));
    s.command(Command::GrabDrop(false));
    s.sync_carried();
    advance(s, 0.25);
    inside(s, salimon_character::FABRICATOR_INPUT_MARKER_METERS);
    s.sync_carried();
    s.command(Command::Interact);
    assert!(
        s.mining.session.fragment(id).is_none(),
        "{resource:?} {seed}: {:?}",
        s.feedback
    );
    (s.elapsed - started).as_secs_f64()
}
fn enqueue(s: &mut GameplaySession) {
    advance(s, 0.25);
    inside(s, salimon_character::FABRICATOR_MONITOR_MARKER_METERS);
    s.command(Command::Interact);
    assert!(s.crafting_ui.open, "{:?}", s.feedback);
    advance(s, 0.25);
    s.crafting_action(crate::session::crafting_ui::CraftingAction::Add);
    assert_eq!(s.fabrication_snapshot().jobs.len(), 1);
    s.clear_held_controls();
}
fn collect(s: &mut GameplaySession) {
    inside(s, salimon_character::FABRICATOR_TRAY_MARKER_METERS);
    s.command(Command::Interact);
}
#[test]
fn seeded_food_loop_sensitivity_report() {
    let mut rows = Vec::new();
    for (name, seed, travel, setup, hunger, health, fatigue) in [
        ("baseline", 0, 60.0, 15.0, 50, 100, 100),
        ("slow_setup", 1, 300.0, 120.0, 50, 100, 100),
        ("low_health_fatigue", 0, 180.0, 30.0, 50, 10, 10),
        ("net_negative_route", 1, 900.0, 30.0, 80, 100, 100),
    ] {
        let mut s = GameplaySession::new_run();
        assert_eq!(s.consumables.food().iter().flatten().count(), 2);
        assert_eq!(s.consumables.medical().iter().flatten().count(), 1);
        // Spend one starter ration before measurement; no consumption at full hunger.
        meters(&mut s, 10, health, fatigue);
        eat(&mut s, ConsumableSlot::FoodOne);
        meters(&mut s, hunger, health, fatigue);
        let initial = s.survival.snapshot().hunger.value();
        let energy = s.ship.snapshot().energy_core.stored_joules;
        advance(&mut s, setup + travel);
        let mining = gather(&mut s, ResourceId::OrganicBiomass, 0.4, seed)
            + gather(&mut s, ResourceId::WaterIce, 0.1, seed);
        // First ration uses initial reserves; mine real replacement feedstock and
        // require the second ration to use it, proving repeatable provisioning.
        let mut first = (0.0, 0.0);
        for iteration in 0..2 {
            enqueue(&mut s);
            s.advance_automation(Duration::from_secs(10));
            assert!(s.fabrication_snapshot().tray.is_some());
            collect(&mut s);
            assert!(s.fabrication_snapshot().tray.is_none());
            eat(&mut s, ConsumableSlot::FoodOne);
            if iteration == 0 {
                first = (
                    s.elapsed.as_secs_f64(),
                    s.survival.snapshot().hunger.value() - initial,
                );
            }
        }
        let elapsed = s.elapsed.as_secs_f64();
        let hunger_after = s.survival.snapshot().hunger.value();
        // Report both the uncapped one-ration loop margin and actual clamped
        // two-ration inventory transaction. No approved meter tuning is changed.
        let margin = first.1;
        rows.push(json!({"name":name,"seed":seed,"travel_budget_seconds":travel,
            "setup_budget_seconds":setup,"mining_seconds":mining,"active_seconds":elapsed,
            "hunger_before":initial,"hunger_after":hunger_after,"actual_net_hunger":hunger_after-initial,
            "single_ration_margin":margin,"first_ration_seconds":first.0,"net_negative":margin<0.0,
            "core_joules_used":energy-s.ship.snapshot().energy_core.stored_joules,
            "health_after":s.survival.snapshot().health.value(),"terminal":s.survival.snapshot().terminal}));
        assert!(!s.survival.snapshot().terminal);
        assert!(s.fabrication_snapshot().jobs.is_empty());
    }
    assert!(rows.iter().any(|r| r["net_negative"] == true));
    let report = json!({"schema":1,"fixture":"survival-balance-384-v1",
        "travel_is_assumed_not_pathfinding":true,"native_signoff":false,
        "approved_hunger_seconds":1800,"ration_hunger":40,"break_even_seconds_per_ration":720,
        "recipe":{"biomass_kg":0.4,"ice_kg":0.1,"work_seconds":10,"watts":10000},
        "rows":rows});
    if let Ok(path) = std::env::var("SALIMON_BALANCE_REPORT") {
        std::fs::write(path, serde_json::to_string_pretty(&report).unwrap() + "\n").unwrap();
    }
}
#[test]
fn pause_wheel_reload_blackout_slots_and_exact_starvation_boundaries() {
    let mut s = GameplaySession::new_run();
    enqueue(&mut s);
    s.ship.set_stored_core_energy_joules(0);
    s.advance_automation(Duration::from_secs(20));
    assert_eq!(s.fabrication_snapshot().jobs[0].progress, Duration::ZERO);
    s.ship.set_stored_core_energy_joules(1_000_000_000);
    s.advance_automation(Duration::from_secs(10));
    collect(&mut s); // Three food slots now full.
    let inventory = s.consumables.clone();
    let bytes = s.save_bytes(100).unwrap();
    s.restore_bytes(&bytes).unwrap();
    assert_eq!(inventory, s.consumables);
    s.command(Command::TogglePause);
    let frozen = s.save_bytes(100).unwrap();
    s.advance_automation(Duration::from_secs(600));
    assert_eq!(frozen, s.save_bytes(100).unwrap());
    s.command(Command::TogglePause);
    s.command(Command::Wheel(WheelAction::Hold(true)));
    let before = s.elapsed;
    s.advance_automation(Duration::from_secs(1));
    assert!(s.elapsed > before);
    s.clear_held_controls();
    meters(&mut s, 0, 10, 10);
    s.advance_automation(Duration::from_millis(9984));
    assert_eq!(s.survival.snapshot().health.value(), 10.0);
    s.advance_automation(TICK);
    assert_eq!(s.survival.snapshot().health.value(), 9.0);
    let saved = s.save_bytes(100).unwrap();
    s.restore_for_launch(&saved, 21700).unwrap();
    assert_eq!(s.survival.snapshot().hunger.value(), 0.0);
    assert_eq!(s.survival.snapshot().health.value(), 9.0);
    assert_eq!(s.survival.snapshot().fatigue.value(), 100.0);
    for id in ["nutrient-gel", "complete-meal", "energy-drink"] {
        assert!(
            s.execute_fabricator(FabricationCommand::Enqueue {
                recipe: RecipeId::new(id).unwrap(),
                quantity: 1
            })
            .is_err()
        );
    }
}
#[test]
fn sampled_availability_reports_missing_feedstocks_instead_of_inventing_food() {
    let mut rows = Vec::new();
    for body in salimon_world::CELESTIAL_BODIES
        .iter()
        .filter(|b| b.role == salimon_world::BodyRole::Solid)
    {
        let profile = salimon_world::resource_distribution::default_resource_distribution(body.id);
        for seed in 0..16 {
            let origin = body.center.translated([0.0, body.radius_meters, 0.0]);
            let deposits = salimon_world::resource_generation::materialize_nearby_deposits(
                *body, &profile, seed, None, origin, 200.0,
            )
            .unwrap();
            let nearest = |resource| {
                deposits
                    .iter()
                    .filter(|d| d.deposit.material().resource() == resource)
                    .map(|d| {
                        d.deposit
                            .position()
                            .offset_from(origin)
                            .iter()
                            .map(|v| v * v)
                            .sum::<f64>()
                            .sqrt()
                    })
                    .min_by(f64::total_cmp)
            };
            rows.push(json!({"body":format!("{:?}",body.id),"seed":seed,
                "biomass_meters":nearest(ResourceId::OrganicBiomass),"ice_meters":nearest(ResourceId::WaterIce)}));
        }
    }
    assert!(
        rows.iter()
            .filter(|r| r["body"] == "Earth")
            .all(|r| !r["biomass_meters"].is_null())
    );
    assert!(
        rows.iter()
            .filter(|r| r["body"] == "Moon")
            .all(|r| r["biomass_meters"].is_null())
    );
    if let Ok(path) = std::env::var("SALIMON_AVAILABILITY_REPORT") {
        std::fs::write(
            path,
            serde_json::to_string_pretty(&json!({"query_radius_meters":200,
            "sample_is_not_global_guarantee":true,"rows":rows}))
            .unwrap()
                + "\n",
        )
        .unwrap();
    }
}

#[test]
fn full_slots_block_tray_and_more_work_without_free_energy() {
    let mut s = GameplaySession::new_run();
    // Capacity-only fixture: real raw insertion, no output/recipe/unlock injection.
    for (r, kg) in [
        (ResourceId::OrganicBiomass, 0.8),
        (ResourceId::WaterIce, 0.2),
    ] {
        s.execute_fabricator(FabricationCommand::Insert(
            salimon_world::resources::RawMaterial::new(r, kg).unwrap(),
        ))
        .unwrap();
    }
    s.execute_fabricator(FabricationCommand::Enqueue {
        recipe: RecipeId::new("food-ration").unwrap(),
        quantity: 3,
    })
    .unwrap();
    s.advance_automation(Duration::from_secs(10));
    collect(&mut s); // Third slot fills.
    s.advance_automation(Duration::from_secs(10));
    let tray = s.fabrication_snapshot().tray.unwrap();
    collect(&mut s); // Full inventory cannot accept or destroy it.
    assert_eq!(s.fabrication_snapshot().tray, Some(tray));
    let blocked = s.fabrication_snapshot();
    s.advance_automation(Duration::from_secs(30));
    assert_eq!(s.fabrication_snapshot(), blocked);
    eat(&mut s, ConsumableSlot::FoodOne);
    collect(&mut s);
    assert!(s.fabrication_snapshot().tray.is_none());
    s.advance_automation(Duration::from_secs(10));
    assert!(s.fabrication_snapshot().tray.is_some());
}
#[test]
fn zero_fatigue_denies_work_but_emergency_food_and_eva_return_still_use_production_gates() {
    let mut s = GameplaySession::new_run();
    meters(&mut s, 0, 5, 0);
    assert!(
        !s.restrictions()
            .permits(salimon_survival::restrictions::Action::Mine)
    );
    assert!(
        !s.restrictions()
            .permits(salimon_survival::restrictions::Action::Fabricate)
    );
    eat(&mut s, ConsumableSlot::FoodOne);
    assert_eq!(s.survival.snapshot().hunger.value(), 40.0);
    let route: Value =
        serde_json::from_str(include_str!("../../../scenarios/moving-eva.json")).unwrap();
    crate::e2e::initialize(
        &mut s,
        crate::e2e::Config {
            scenario: crate::e2e::Scenario::CoastingSpace,
            seed: 0,
            step: TICK,
            fragment_load: None,
        },
    )
    .unwrap();
    // Route fixture begins in flight; vitals must be fresh for the pilot commands.
    s.survival = Default::default();
    s.policy.set_automation_step(Some(TICK));
    for step in route["steps"].as_array().unwrap() {
        if let Some(action) = step.get("action") {
            let mut request = action.clone();
            request["protocol"] = 1.into();
            request["id"] = 1.into();
            let response = crate::automation::execute_session(&mut s, &request.to_string());
            assert_eq!(response["ok"], true, "{response}");
        }
    }
    assert_eq!(
        s.character.location(),
        salimon_character::CharacterLocation::InsideShip
    );
    assert!(!s.survival.snapshot().terminal);
}
#[test]
fn unchanged_meter_and_startup_famine_budgets_are_explicit() {
    let mut s = GameplaySession::new_run();
    // No travel safety claim: each ration yields exactly 720 seconds of hunger
    // at the accepted 100/1800 per-second drain. Consume at 60 to avoid waste.
    enqueue(&mut s);
    s.advance_automation(Duration::from_secs(10));
    collect(&mut s);
    meters(&mut s, 100, 100, 100);
    for slot in [
        ConsumableSlot::FoodOne,
        ConsumableSlot::FoodTwo,
        ConsumableSlot::FoodThree,
    ] {
        advance(&mut s, 720.0);
        assert_eq!(s.survival.snapshot().hunger.value(), 60.0);
        eat(&mut s, slot);
        assert_eq!(s.survival.snapshot().hunger.value(), 100.0);
    }
    let saved = s.save_bytes(100).unwrap();
    s.restore_bytes(&saved).unwrap();
    assert!(s.consumables.food().iter().all(Option::is_none));
    advance(&mut s, 1800.0);
    assert_eq!(s.survival.snapshot().hunger.value(), 0.0);
    advance(&mut s, 999.0);
    assert!(!s.survival.snapshot().terminal);
    advance(&mut s, 1.0);
    assert!(s.survival.snapshot().terminal);
}

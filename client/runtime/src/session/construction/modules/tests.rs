use super::*;
use crate::session::{Command, construction::tests::piece};
use salimon_world::resources::ResourceId::IronOre;

fn planned() -> (GameplaySession, PlaceholderId) {
    let mut s = GameplaySession::default();
    // Trusted host still supplies actual installed/player/cargo obstacles.
    for x in -14..=12 {
        for z in -8..=8 {
            let target = [
                f64::from(x) * 0.5,
                SHIP_FLOOR_HEIGHT_METERS,
                f64::from(z) * 0.5,
            ];
            let obstacles = s.construction_obstacles(Footprint::Interior);
            let c = PlacementContext {
                actor: OWNER,
                owner: OWNER,
                flight: FlightPermission::Landed,
                eye_meters: target,
                reach_meters: 5.,
                reachable: true,
                physical_support: true,
                obstacles: &obstacles,
            };
            if let Ok(id) = s.building.source.projects.place(
                registered_catalog(),
                "utility-module",
                target,
                Rotation::Forward,
                &s.building.structure,
                &c,
            ) {
                if has_physical_support(
                    Footprint::Interior,
                    s.building.source.projects.inspect(id).unwrap().space,
                ) {
                    return (s, id);
                }
                s.building.source.projects.cancel(id, OWNER).unwrap();
            }
        }
    }
    panic!("no valid module position")
}
fn feed(s: &mut GameplaySession, id: PlaceholderId, kg: f64) {
    piece(s, IronOre, kg);
    if !s.feed_module(id) {
        let p = s.building.source.projects.funding_mut(id, OWNER).unwrap();
        p.funding
            .contribute(&mut s.mining.session, &p.blueprint.requirements, true)
            .unwrap();
        s.sync_carried();
        s.sync_fabricator_mass();
    }
}
fn installed() -> (GameplaySession, PlaceholderId) {
    let (mut s, id) = planned();
    feed(&mut s, id, 3.);
    s.advance_construction(Duration::from_secs(60));
    assert_eq!(s.utility_recovery_multiplier(), 1.25);
    (s, id)
}
fn plan(s: &mut GameplaySession, id: PlaceholderId, kind: Operation) {
    let m = s.building.source.modules.get_mut(&id).unwrap();
    m.maintenance = Some(Maintenance {
        kind,
        funding: Default::default(),
        work: Default::default(),
    });
}
#[test]
fn install_upgrade_downtime_mass_and_verified_reload() {
    let (mut s, id) = planned();
    feed(&mut s, id, 3.);
    let mass = s.ship.snapshot().total_mass_kg;
    let charge = s.ship.snapshot().energy_core.stored_joules;
    assert_eq!(s.utility_recovery_multiplier(), 1.);
    s.advance_construction(Duration::from_secs(59));
    assert!(s.construction_solids().is_empty());
    assert_eq!(s.utility_recovery_multiplier(), 1.);
    s.advance_construction(Duration::from_secs(1));
    assert_eq!(
        s.ship.snapshot().energy_core.stored_joules,
        charge - 180_000
    );
    assert_eq!(s.ship.snapshot().total_mass_kg, mass);
    assert_eq!(s.utility_recovery_multiplier(), 1.25);
    plan(&mut s, id, Operation::Upgrade);
    assert_eq!(
        s.utility_recovery_multiplier(),
        1.25,
        "planning does not disable source"
    );
    feed(&mut s, id, 1.);
    assert_eq!(
        s.utility_recovery_multiplier(),
        1.25,
        "partial supply keeps source online"
    );
    feed(&mut s, id, 1.);
    assert_eq!(
        s.utility_recovery_multiplier(),
        1.,
        "full supply immediately disables source"
    );
    let ship = s.ship.snapshot();
    s.advance_construction(Duration::from_secs(45));
    let mid = s.save_bytes(100).unwrap();
    let mut r = GameplaySession::default();
    r.restore_bytes(&mid).unwrap();
    assert_eq!(r.building.source, s.building.source);
    assert_eq!(r.utility_recovery_multiplier(), 1.);
    r.advance_construction(Duration::from_secs(45));
    assert_eq!(r.utility_recovery_multiplier(), 1.5);
    assert_eq!(r.ship.snapshot().total_mass_kg, ship.total_mass_kg);
    assert_eq!(
        r.ship.snapshot().energy_core.stored_joules,
        ship.energy_core.stored_joules - 270_000
    );
    assert_eq!(r.ship.snapshot().engines_enabled, ship.engines_enabled);
    let bytes = r.save_bytes(100).unwrap();
    s.restore_bytes(&bytes).unwrap();
    assert_eq!(s.save_bytes(100).unwrap(), bytes);
}
#[test]
fn activated_cancel_refunds_half_without_restoring_old_module_and_restoration_costs_work() {
    let (mut s, id) = installed();
    plan(&mut s, id, Operation::Upgrade);
    feed(&mut s, id, 2.);
    s.advance_construction(Duration::from_secs(45));
    let receipt = s.building.source.modules[&id]
        .maintenance
        .as_ref()
        .unwrap()
        .funding
        .receipts[0];
    s.cancel_module(id).unwrap();
    assert_eq!(
        s.mining
            .session
            .fragment(receipt.id())
            .unwrap()
            .material()
            .mass_kg(),
        1.
    );
    assert_eq!(s.utility_recovery_multiplier(), 1.);
    let bytes = s.save_bytes(100).unwrap();
    assert!(s.cancel_module(id).is_err());
    assert_eq!(s.save_bytes(100).unwrap(), bytes);
    plan(&mut s, id, Operation::Restore);
    feed(&mut s, id, 1.);
    s.advance_construction(Duration::from_secs(29));
    assert_eq!(s.utility_recovery_multiplier(), 1.);
    s.advance_construction(Duration::from_secs(1));
    assert_eq!(s.utility_recovery_multiplier(), 1.25);
    assert_eq!(
        s.building.source.modules[&id]
            .installed
            .iter()
            .map(|f| salimon_ship::structure::funding::milligrams(*f).unwrap())
            .sum::<u64>(),
        1_000_000
    );
    let mut r = GameplaySession::default();
    r.restore_bytes(&s.save_bytes(100).unwrap()).unwrap();
    assert_eq!(r.utility_recovery_multiplier(), 1.25);
}
#[test]
fn pause_power_damage_stop_and_commission_fault_never_advance_or_grant_capability() {
    let (mut s, id) = installed();
    plan(&mut s, id, Operation::Upgrade);
    feed(&mut s, id, 2.);
    let original = s.building.source.clone();
    s.paused = true;
    s.advance_construction(Duration::from_secs(90));
    s.paused = false;
    assert_eq!(s.building.source, original);
    let charge = s.ship.snapshot().energy_core.stored_joules;
    s.ship.set_stored_core_energy_joules(0);
    s.advance_construction(Duration::from_secs(90));
    assert_eq!(s.building.source, original);
    s.ship.set_stored_core_energy_joules(charge);
    s.building
        .source
        .modules
        .get_mut(&id)
        .unwrap()
        .maintenance
        .as_mut()
        .unwrap()
        .work
        .damaged = true;
    let damage = s.building.source.clone();
    s.advance_construction(Duration::from_secs(90));
    assert_eq!(s.building.source, damage);
    let bytes = s.save_bytes(100).unwrap();
    let mut r = GameplaySession::default();
    r.restore_bytes(&bytes).unwrap();
    assert_eq!(r.building.source, damage);
    s.building
        .source
        .modules
        .get_mut(&id)
        .unwrap()
        .maintenance
        .as_mut()
        .unwrap()
        .work
        .damaged = false;
    assert!(s.module_stop_resume(id));
    s.advance_construction(Duration::from_secs(90));
    assert_eq!(s.utility_recovery_multiplier(), 1.);
    assert!(s.module_stop_resume(id));
    // Commission a fully paid candidate against a real disconnected hull.
    let op = s
        .building
        .source
        .modules
        .get_mut(&id)
        .unwrap()
        .maintenance
        .as_mut()
        .unwrap();
    op.work
        .advance(
            &mut op.funding,
            &blueprint(op.kind),
            Duration::from_secs(90),
        )
        .unwrap();
    s.building.structure.damage_part(StructureId(2)).unwrap();
    s.commission_module(id);
    assert_eq!(
        s.building.source.modules[&id].fault,
        Some(Fault::Connection)
    );
    assert_eq!(s.utility_recovery_multiplier(), 1.);
    s.building.structure = s.building.source.structure(None).unwrap();
    s.commission_module(id);
    assert_eq!(s.utility_recovery_multiplier(), 1.5);
    assert!(s.building.source.modules[&id].maintenance.is_none());
}
#[test]
fn malformed_source_duplicate_receipts_and_old_schema_reject_atomically() {
    let (mut s, id) = installed();
    let good = s.save_bytes(100).unwrap();
    let mut json: serde_json::Value = serde_json::from_slice(&good).unwrap();
    json["schema_version"] = 19.into();
    assert!(
        s.restore_bytes(&serde_json::to_vec(&json).unwrap())
            .is_err()
    );
    assert_eq!(s.save_bytes(100).unwrap(), good);
    for tier in [0, 3] {
        let mut json: serde_json::Value = serde_json::from_slice(&good).unwrap();
        json["state"]["construction"]["modules"][id.0.to_string()]["tier"] = tier.into();
        assert!(
            s.restore_bytes(&serde_json::to_vec(&json).unwrap())
                .is_err()
        );
        assert_eq!(s.save_bytes(100).unwrap(), good);
    }
    plan(&mut s, id, Operation::Upgrade);
    feed(&mut s, id, 2.);
    s.advance_construction(Duration::from_secs(90));
    let mut json: serde_json::Value = serde_json::from_slice(&s.save_bytes(100).unwrap()).unwrap();
    json["state"]["construction"]["modules"][id.0.to_string()]["installed"][0] =
        serde_json::to_value(
            s.building
                .source
                .projects
                .inspect(id)
                .unwrap()
                .funding
                .receipts[0],
        )
        .unwrap();
    assert!(
        s.restore_bytes(&serde_json::to_vec(&json).unwrap())
            .is_err()
    );
}
#[test]
fn optional_module_failure_does_not_ground_ship_but_critical_structural_damage_does() {
    let (mut s, id) = installed();
    plan(&mut s, id, Operation::Upgrade);
    feed(&mut s, id, 2.);
    s.ship.set_cockpit_control(true);
    s.command(Command::Landing);
    assert!(matches!(
        s.ship.snapshot().flight_state,
        salimon_ship::FlightState::AssistedTakeoff { .. }
    ));
    // Maintenance cannot change inertia, engine switch or throttle.
    let state = s.ship.snapshot();
    s.advance_construction(Duration::from_secs(90));
    assert_eq!(s.ship.snapshot().pose, state.pose);
    assert_eq!(
        s.ship.snapshot().velocity_meters_per_second,
        state.velocity_meters_per_second
    );
    assert_eq!(s.utility_recovery_multiplier(), 1.5);
}

#[test]
fn native_evidence_setup_uses_reachable_production_feed_and_commissioning() {
    for phase in 0..=2 {
        let mut s = GameplaySession::default();
        crate::session::construction::evidence_module(&mut s, phase).unwrap();
        assert!(s.aimed_construction().is_some());
        if phase > 0 {
            s.construction_action(BuildAction::Module);
            let hint = s.construction_hint().unwrap();
            assert!(
                hint.contains(if phase == 1 { "2.000kg" } else { "1.000kg" }),
                "{hint}"
            );
            assert!(
                hint.contains(if phase == 1 { "90s" } else { "30s" }),
                "{hint}"
            );
        }
        s.command(Command::Interact);
        s.advance_construction(Duration::from_secs(if phase == 1 { 90 } else { 60 }));
        assert_eq!(
            s.utility_recovery_multiplier(),
            if phase == 1 { 1.5 } else { 1.25 },
            "phase {phase} observation {} feedback {:?}",
            s.construction_inspect(),
            s.feedback
        );
        let good = s.save_bytes(100).unwrap();
        let mut r = GameplaySession::default();
        r.restore_bytes(&good).unwrap();
        assert_eq!(r.building.source, s.building.source);
    }
}
#[test]
fn recovery_throughput_uses_real_core_joules_and_downtime_removes_whole_bonus() {
    use salimon_fabrication::{FabricationCommand as C, RecipeId};
    use salimon_world::resources::{RawMaterial, ResourceId};
    let (mut s, id) = installed();
    for r in [ResourceId::IronOre, ResourceId::SilicateRock] {
        s.execute_fabricator(C::Insert(RawMaterial::new(r, 2.).unwrap()))
            .unwrap();
    }
    s.execute_fabricator(C::Enqueue {
        recipe: RecipeId::new("data-recovery-module").unwrap(),
        quantity: 1,
    })
    .unwrap();
    s.advance_fabrication(Duration::from_secs(60));
    s.collect_fabricator_output();
    s.toggle_recovery("received.damaged-01");
    let initial = s.recovery_observation()["spent_joules"][0]
        .as_f64()
        .unwrap();
    let before = s.ship.snapshot().energy_core.stored_joules;
    s.advance_recovery(Duration::from_secs(1));
    assert_eq!(
        s.recovery_observation()["spent_joules"][0]
            .as_f64()
            .unwrap()
            - initial,
        6_250_000.
    );
    assert_eq!(
        before - s.ship.snapshot().energy_core.stored_joules,
        6_250_000
    );
    plan(&mut s, id, Operation::Upgrade);
    feed(&mut s, id, 2.);
    let before = s.ship.snapshot().energy_core.stored_joules;
    s.advance_recovery(Duration::from_secs(1));
    assert_eq!(
        before - s.ship.snapshot().energy_core.stored_joules,
        5_000_000
    );
}

#[test]
fn scarce_surplus_credits_only_paid_nanoseconds_and_discrete_events_exclude_continuous_work() {
    let (mut s, id) = installed();
    plan(&mut s, id, Operation::Upgrade);
    feed(&mut s, id, 2.);
    let events = s.checkpoint_events();
    s.ship.exchange_core_energy_joules(0., f64::MAX).unwrap();
    s.ship
        .exchange_core_energy_joules(s.recovery_reserve() + 1_500., 0.)
        .unwrap();
    let events_after_power = s.checkpoint_events();
    s.advance_construction(Duration::from_secs(1));
    assert_eq!(
        s.building.source.modules[&id]
            .maintenance
            .as_ref()
            .unwrap()
            .work
            .elapsed_nanos,
        Some(500_000_000)
    );
    assert_eq!(
        s.ship.snapshot().energy_core.stored_joules,
        s.recovery_reserve() as u64
    );
    assert_eq!(
        s.checkpoint_events(),
        events_after_power,
        "work is covered by periodic checkpoints"
    );
    assert_eq!(
        events, events_after_power,
        "power stays available at protected reserve"
    );
    let before = s.building.source.clone();
    s.advance_construction(Duration::from_secs(90));
    assert_eq!(s.building.source, before);
}

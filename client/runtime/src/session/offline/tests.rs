use super::*;
use crate::session::{
    Command,
    construction::{self, BuildAction},
};
use salimon_ship::{FlightState, ShipController};
fn stool() -> GameplaySession {
    let mut s = GameplaySession::default();
    construction::evidence_initial_conditions(&mut s, true).unwrap();
    s.command(Command::Interact);
    s
}
fn source(s: &GameplaySession) -> Vec<u8> {
    serde_json::to_vec(&s.building.source).unwrap()
}
fn live_work(s: &mut GameplaySession, delta: Duration) {
    let mut remaining = delta;
    while !remaining.is_zero() {
        let d = remaining.min(Duration::from_nanos(TICK_NANOS));
        s.advance_fabrication(d);
        s.advance_recovery(d);
        s.advance_construction(d);
        remaining -= d;
    }
}
fn copy(s: &GameplaySession) -> GameplaySession {
    let mut r = GameplaySession::default();
    r.restore_bytes(&s.save_bytes(100).unwrap()).unwrap();
    r
}
#[test]
fn offline_paid_completion_matches_live_schedule_and_restore_never_replays_it() {
    let mut online = stool();
    let mut closed = copy(&online);
    let elapsed = closed.elapsed;
    let survival = serde_json::to_vec(&closed.survival).unwrap();
    live_work(&mut online, Duration::from_secs(12));
    closed
        .reconcile_construction_offline(Duration::from_secs(12), &[])
        .unwrap();
    assert_eq!(source(&closed), source(&online));
    assert_eq!(
        closed.ship.snapshot().energy_core,
        online.ship.snapshot().energy_core
    );
    assert_eq!(closed.construction_solids(), online.construction_solids());
    assert_eq!(
        closed.ship.snapshot().total_mass_kg,
        online.ship.snapshot().total_mass_kg
    );
    assert_eq!(closed.elapsed, elapsed);
    assert_eq!(serde_json::to_vec(&closed.survival).unwrap(), survival);
    let restored = copy(&closed);
    assert_eq!(source(&restored), source(&closed));
    assert_eq!(restored.construction_solids(), closed.construction_solids());
}
#[test]
fn elapsed_partition_remainders_and_core_depletion_are_exact() {
    let mut whole = stool();
    let reserve = whole.recovery_reserve();
    whole
        .ship
        .exchange_core_energy_joules(0., f64::MAX)
        .unwrap();
    whole
        .ship
        .exchange_core_energy_joules(reserve + 300_000., 0.)
        .unwrap();
    let mut split = copy(&whole);
    let mut online = copy(&whole);
    whole
        .reconcile_construction_offline(Duration::from_secs(6), &[])
        .unwrap();
    for seconds in [1, 2, 1, 1, 1] {
        split
            .reconcile_construction_offline(Duration::from_secs(seconds), &[])
            .unwrap();
    }
    live_work(&mut online, Duration::from_secs(6));
    assert_eq!(source(&whole), source(&split));
    assert_eq!(source(&whole), source(&online));
    assert_eq!(
        whole.ship.snapshot().energy_core,
        split.ship.snapshot().energy_core
    );
    assert_eq!(
        whole.ship.snapshot().energy_core,
        online.ship.snapshot().energy_core
    );
    assert!(
        !whole
            .building
            .source
            .projects
            .jobs()
            .next()
            .unwrap()
            .work
            .completed
    );
    assert!(
        whole
            .building
            .source
            .projects
            .jobs()
            .next()
            .unwrap()
            .funding
            .progress_ppm
            < 1_000_000
    );
}
#[test]
fn damage_boundary_precedes_future_work_and_late_invalid_event_is_atomic() {
    use construction::damage::{Impact, Target};
    use salimon_ship::structure::placement::PlaceholderId;
    let mut s = stool();
    let mut online = copy(&s);
    let hit = Impact {
        target: Target::Project(PlaceholderId(1)),
        destroy: false,
        survival_ppm: 1_000_000,
    };
    s.reconcile_construction_offline(
        Duration::from_secs(8),
        &[TimedEvent {
            after: Duration::from_secs(3),
            event: Event::Impact(hit),
        }],
    )
    .unwrap();
    live_work(&mut online, Duration::from_secs(3));
    online.construction_impact(hit).unwrap();
    live_work(&mut online, Duration::from_secs(5));
    assert_eq!(source(&s), source(&online));
    s.finish_offline_remainder();
    assert_eq!(
        s.ship.snapshot().energy_core,
        online.ship.snapshot().energy_core
    );
    assert_eq!(
        s.building
            .source
            .projects
            .jobs()
            .next()
            .unwrap()
            .funding
            .progress_ppm,
        300_000
    );
    let mut s = stool();
    let before = s.save_bytes(100).unwrap();
    let bad = Impact {
        target: Target::Project(PlaceholderId(999)),
        ..hit
    };
    assert!(
        s.reconcile_construction_offline(
            Duration::from_secs(20),
            &[TimedEvent {
                after: Duration::from_secs(19),
                event: Event::Impact(bad)
            }]
        )
        .is_err()
    );
    assert_eq!(s.save_bytes(100).unwrap(), before);
}
#[test]
fn exterior_departure_latches_even_when_landing_returns_and_interior_runs_in_flight() {
    let (mut room, id) = construction::room_tests::room();
    let snap = room.ship.snapshot();
    let boundary = |after, flight| TimedEvent {
        after: Duration::from_secs(after),
        event: Event::Motion {
            pose: snap.pose,
            velocity: [0.; 3],
            flight,
        },
    };
    room.reconcile_construction_offline(
        Duration::from_secs(40),
        &[
            boundary(10, FlightState::Flying),
            boundary(20, snap.flight_state),
        ],
    )
    .unwrap();
    let p = room.building.source.projects.inspect(id).unwrap();
    assert!(p.work.stopped);
    assert_eq!(p.work.elapsed_nanos, Some(10_000_000_000));
    let mut s = stool();
    let pose = s.ship.snapshot().pose;
    s.ship = ShipController::flying(pose, 100);
    let velocity = s.ship.snapshot().velocity_meters_per_second;
    let enabled = s.ship.snapshot().engines_enabled;
    s.reconcile_construction_offline(Duration::from_secs(12), &[])
        .unwrap();
    assert!(
        s.building
            .source
            .projects
            .jobs()
            .next()
            .unwrap()
            .work
            .completed
    );
    assert_eq!(s.ship.snapshot().velocity_meters_per_second, velocity);
    assert_eq!(s.ship.snapshot().pose, pose);
    assert_eq!(s.ship.snapshot().engines_enabled, enabled);
}
#[test]
fn clock_migration_malformed_clock_and_rollback_cannot_grant_time_twice() {
    let s = stool();
    let bytes = s.save_bytes(100).unwrap();
    let mut loaded = GameplaySession::default();
    loaded.restore_for_launch(&bytes, 105).unwrap();
    assert_eq!(
        loaded
            .building
            .source
            .projects
            .jobs()
            .next()
            .unwrap()
            .funding
            .progress_ppm,
        499_200
    );
    let credited = loaded.save_bytes(105).unwrap();
    let work = source(&loaded);
    loaded.restore_for_launch(&credited, 105).unwrap();
    assert_eq!(source(&loaded), work);
    let rollback = loaded.save_bytes(1).unwrap();
    loaded.restore_for_launch(&rollback, 2).unwrap();
    assert_eq!(source(&loaded), work);
    let mut old: serde_json::Value = serde_json::from_slice(&bytes).unwrap();
    old["schema_version"] = 21.into();
    old["state"].as_object_mut().unwrap().remove("surface_age");
    old["state"].as_object_mut().unwrap().remove("offline");
    loaded
        .restore_for_launch(&serde_json::to_vec(&old).unwrap(), 1000)
        .unwrap();
    assert_eq!(
        loaded
            .building
            .source
            .projects
            .jobs()
            .next()
            .unwrap()
            .funding
            .progress_ppm,
        0
    );
    let good = loaded.save_bytes(1000).unwrap();
    let mut bad: serde_json::Value = serde_json::from_slice(&good).unwrap();
    bad["state"]["offline"]["pending_nanos"] = TICK_NANOS.into();
    assert!(
        loaded
            .restore_bytes(&serde_json::to_vec(&bad).unwrap())
            .is_err()
    );
    assert_eq!(loaded.save_bytes(1000).unwrap(), good);
}
#[test]
fn live_pause_freezes_work_but_closing_from_pause_allows_eligible_work() {
    let mut s = stool();
    s.command(Command::TogglePause);
    let before = source(&s);
    s.advance_automation(Duration::from_secs(30));
    assert_eq!(source(&s), before);
    let bytes = s.save_bytes(100).unwrap();
    let mut r = GameplaySession::default();
    r.restore_for_launch(&bytes, 112).unwrap();
    assert!(
        r.building
            .source
            .projects
            .jobs()
            .next()
            .unwrap()
            .work
            .completed
    );
    assert!(!r.is_paused());
}
#[test]
fn upgrade_offline_uses_existing_downtime_commissioning_and_exact_joules() {
    let mut online = GameplaySession::default();
    construction::evidence_module(&mut online, 1).unwrap();
    online.command(Command::Build(BuildAction::Module));
    online.command(Command::Interact);
    let mut offline = copy(&online);
    live_work(&mut online, Duration::from_secs(96));
    offline
        .reconcile_construction_offline(Duration::from_secs(96), &[])
        .unwrap();
    assert_eq!(source(&offline), source(&online));
    assert_eq!(offline.utility_recovery_multiplier(), 1.5);
    assert_eq!(
        offline.ship.snapshot().energy_core,
        online.ship.snapshot().energy_core
    );
    assert_eq!(
        offline.ship.snapshot().total_mass_kg,
        online.ship.snapshot().total_mass_kg
    );
    assert_eq!(
        offline.snapshot().character.local_ship_position_meters,
        online.snapshot().character.local_ship_position_meters
    );
}

#[test]
fn competing_fabricator_and_captured_assist_reserves_never_fund_free_work() {
    let mut online = stool();
    for r in salimon_world::resources::RESOURCE_CATALOG {
        online
            .fabricator
            .insert(salimon_world::resources::RawMaterial::new(r.id, 1.).unwrap())
            .unwrap();
    }
    online
        .fabricator
        .enqueue(
            &salimon_fabrication::RecipeId::new("food-ration").unwrap(),
            1,
        )
        .unwrap();
    online.sync_fabricator_mass();
    let reserve = online.recovery_reserve();
    online
        .ship
        .exchange_core_energy_joules(0., f64::MAX)
        .unwrap();
    online
        .ship
        .exchange_core_energy_joules(reserve + 200_000., 0.)
        .unwrap();
    let mut offline = copy(&online);
    live_work(&mut online, Duration::from_secs(4));
    offline
        .reconcile_construction_offline(Duration::from_secs(4), &[])
        .unwrap();
    assert_eq!(source(&online), source(&offline));
    assert_eq!(
        online.ship.snapshot().energy_core,
        offline.ship.snapshot().energy_core
    );
    assert_eq!(
        serde_json::to_value(&online.fabricator).unwrap(),
        serde_json::to_value(&offline.fabricator).unwrap()
    );
    let mut s = stool();
    s.ship.set_cockpit_control(true);
    s.ship.trigger_landing_action();
    assert!(matches!(
        s.ship.snapshot().flight_state,
        FlightState::AssistedTakeoff { .. }
    ));
    s.ship.exchange_core_energy_joules(0., f64::MAX).unwrap();
    let before = serde_json::to_value(s.ship.save_state()).unwrap();
    let work = source(&s);
    s.reconcile_construction_offline(Duration::from_secs(2), &[])
        .unwrap();
    assert_eq!(source(&s), work);
    let after = serde_json::to_value(s.ship.save_state()).unwrap();
    assert_eq!(before["assist"], after["assist"]);
    assert_eq!(before["engines_enabled"], after["engines_enabled"]);
}
#[test]
fn failed_native_publication_exposes_no_completed_project_and_retry_commits_once() {
    use crate::checkpoints::Checkpoints;
    use salimon_persistence::storage::{NativeStore, SaveManager};
    let path = std::env::temp_dir().join(format!(
        "salimon-550-{}-{}",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos()
    ));
    let original = stool();
    let bytes = original.save_bytes(100).unwrap();
    let validate = |b: &[u8]| GameplaySession::default().restore_bytes(b);
    let mut manager = SaveManager::new(NativeStore::open(path.clone()).unwrap());
    manager.checkpoint(&bytes, validate).unwrap();
    drop(manager);
    std::fs::create_dir(path.join("checkpoint-1.tmp")).unwrap();
    let mut visible = GameplaySession::default();
    let before = visible.save_bytes(100).unwrap();
    assert!(Checkpoints::open_at(path.clone(), &mut visible, 112).is_err());
    assert_eq!(visible.save_bytes(100).unwrap(), before);
    let mut manager = SaveManager::new(NativeStore::open(path.clone()).unwrap());
    assert_eq!(manager.recover(validate).unwrap().bytes.unwrap(), bytes);
    drop(manager);
    std::fs::remove_dir(path.join("checkpoint-1.tmp")).unwrap();
    let host = Checkpoints::open_at(path.clone(), &mut visible, 112).unwrap();
    drop(host);
    assert!(
        visible
            .building
            .source
            .projects
            .jobs()
            .next()
            .unwrap()
            .work
            .completed
    );
    let credited = visible.save_bytes(112).unwrap();
    let host = Checkpoints::open_at(path.clone(), &mut visible, 112).unwrap();
    drop(host);
    assert_eq!(visible.save_bytes(112).unwrap(), credited);
    std::fs::remove_dir_all(path).unwrap();
}

#[test]
fn conversion_restarts_starved_work_without_ever_granting_unfunded_progress() {
    let mut online = stool();
    online
        .ship
        .exchange_core_energy_joules(0., f64::MAX)
        .unwrap();
    let pose = online.mining.session.save_state();
    assert!(pose.fragments.is_empty());
    let transform = salimon_world::resources::ResourceTransform::new(
        salimon_world::WorldPosition::new(0., 0., 0.),
        [0., 0., 0., 1.],
    )
    .unwrap();
    crate::sphere_fixtures::sphere(
        &mut online.mining.session,
        salimon_world::resources::ResourceId::IronOre,
        5.,
        transform,
    );
    let id = online.mining.session.fragments()[0].id();
    online.mining.session.pick_up(id);
    assert_eq!(
        online
            .ship
            .intake_carried_fragment(&mut online.mining.session),
        salimon_ship::IntakeResult::Accepted
    );
    online.sync_carried();
    let mut offline = copy(&online);
    offline
        .reconcile_construction_offline(Duration::from_secs(4), &[])
        .unwrap();
    assert_eq!(
        offline
            .building
            .source
            .projects
            .jobs()
            .next()
            .unwrap()
            .funding
            .progress_ppm,
        0
    );
    offline
        .reconcile_construction_offline(Duration::from_secs(36), &[])
        .unwrap();
    live_work(&mut online, Duration::from_secs(40));
    assert_eq!(source(&offline), source(&online));
    assert!(
        offline
            .building
            .source
            .projects
            .jobs()
            .next()
            .unwrap()
            .work
            .completed
    );
    assert_eq!(
        offline.ship.snapshot().energy_core,
        online.ship.snapshot().energy_core
    );
}
#[test]
fn current_clock_requires_checkpoint_watermark_and_exact_site_records() {
    let mut s = stool();
    let good = s.save_bytes(100).unwrap();
    for mode in 0..3 {
        let mut bad: serde_json::Value = serde_json::from_slice(&good).unwrap();
        match mode {
            0 => {
                bad["state"]["offline"]["watermark"] = 99.into();
            }
            1 => {
                bad["state"]["offline"]["safety_reasons"] = serde_json::json!({"999":"BUILDING"});
            }
            _ => {
                bad["state"].as_object_mut().unwrap().remove("offline");
            }
        }
        assert!(s.restore_bytes(&serde_json::to_vec(&bad).unwrap()).is_err());
        assert_eq!(s.save_bytes(100).unwrap(), good);
    }
}

#[test]
fn full_offline_room_publishes_four_independent_cells_once_with_exact_power_and_mass() {
    let (mut s, id) = construction::room_tests::room();
    let energy = s.ship.snapshot().energy_core.stored_joules;
    let mass = s.ship.snapshot().total_mass_kg;
    s.reconcile_construction_offline(Duration::from_secs(300), &[])
        .unwrap();
    assert!(
        s.building
            .source
            .projects
            .inspect(id)
            .unwrap()
            .work
            .completed
    );
    assert_eq!(s.building.structure.parts().count(), 6);
    assert!(!s.structural_solids().is_empty());
    assert!(s.mining.session.fragments().is_empty());
    assert_eq!(s.ship.snapshot().total_mass_kg, mass);
    assert_eq!(
        energy - s.ship.snapshot().energy_core.stored_joules,
        36_000_000
    );
    let restored = copy(&s);
    assert_eq!(source(&restored), source(&s));
    assert_eq!(restored.building.structure, s.building.structure);
    assert_eq!(restored.construction_solids(), s.construction_solids());
}
#[test]
fn a_long_blocked_absence_has_no_progress_cap_or_free_work() {
    let mut s = stool();
    let id = s.building.source.projects.jobs().next().unwrap().id;
    s.construction_impact(construction::damage::Impact {
        target: construction::damage::Target::Project(id),
        destroy: false,
        survival_ppm: 1_000_000,
    })
    .unwrap();
    s.reconcile_construction_offline(Duration::from_secs(365 * 86400), &[])
        .unwrap();
    let p = s.building.source.projects.inspect(id).unwrap();
    assert_eq!(p.funding.progress_ppm, 0);
    assert!(p.work.damaged);
    assert_eq!(s.ship.snapshot().energy_core.stored_joules, 0);
    assert_eq!(s.offline.pending_nanos, 0);
    assert!(s.mining.ship_gravity_disabled);
    assert!(s.building.running.is_none());
}

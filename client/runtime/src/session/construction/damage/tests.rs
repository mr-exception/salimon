use super::super::{
    room_tests::room,
    tests::{move_player, piece},
};
use super::*;
use std::time::Duration;
fn hit(s: &mut GameplaySession, target: Target, destroy: bool, survival_ppm: u32) {
    s.construction_impact(Impact {
        target,
        destroy,
        survival_ppm,
    })
    .unwrap();
}
fn stool() -> (GameplaySession, PlaceholderId) {
    let mut s = GameplaySession::default();
    evidence_initial_conditions(&mut s, true).unwrap();
    let id = s.building.source.projects.jobs().next().unwrap().id;
    s.contribute_construction();
    (s, id)
}
fn roundtrip(s: &GameplaySession) -> GameplaySession {
    let bytes = s.save_bytes(100).unwrap();
    let mut r = GameplaySession::default();
    r.restore_bytes(&bytes).unwrap();
    assert_eq!(r.building.source, s.building.source);
    assert_eq!(r.building.structure, s.building.structure);
    assert_eq!(r.construction_solids(), s.construction_solids());
    assert_eq!(
        r.ship.snapshot().total_mass_kg,
        s.ship.snapshot().total_mass_kg
    );
    r
}
#[test]
fn partial_damage_survives_without_consumption_and_cancel_is_still_proportional() {
    let (mut s, id) = stool();
    s.advance_construction(Duration::from_secs(5));
    hit(&mut s, Target::Project(id), false, COMPLETE);
    let energy = s.ship.snapshot().energy_core.stored_joules;
    let source = s.building.source.clone();
    s.advance_construction(Duration::from_secs(9));
    assert_eq!(s.building.source, source);
    assert_eq!(s.ship.snapshot().energy_core.stored_joules, energy);
    let mut r = roundtrip(&s);
    r.cancel_construction(id).unwrap();
    assert_eq!(
        r.mining
            .session
            .fragments()
            .iter()
            .map(|f| f.material().mass_kg())
            .sum::<f64>(),
        0.5
    );
    assert!(r.cancel_construction(id).is_err());
}
#[test]
fn destroyed_placeholder_uses_actual_survival_not_cancel_progress_and_can_be_replaced() {
    let (mut s, id) = stool();
    s.advance_construction(Duration::from_secs(5));
    hit(&mut s, Target::Project(id), true, 750_000);
    assert_eq!(
        s.mining
            .session
            .fragments()
            .iter()
            .map(|f| f.material().mass_kg())
            .sum::<f64>(),
        0.75
    );
    let mut r = roundtrip(&s);
    let before = r.save_bytes(100).unwrap();
    assert!(
        r.construction_impact(Impact {
            target: Target::Project(id),
            destroy: true,
            survival_ppm: COMPLETE
        })
        .is_err()
    );
    assert_eq!(r.save_bytes(100).unwrap(), before);
    // A retired reservation no longer blocks an exact physical replacement.
    let old = r.building.source.projects.inspect(id).unwrap().clone();
    let fresh = r
        .building
        .source
        .projects
        .place(
            registered_catalog(),
            &old.blueprint.id,
            old.anchor_meters,
            old.rotation,
            &r.building.structure,
            &PlacementContext {
                actor: OWNER,
                owner: OWNER,
                flight: FlightPermission::Landed,
                eye_meters: old.anchor_meters,
                reach_meters: 5.,
                reachable: true,
                physical_support: true,
                obstacles: &[],
            },
        )
        .unwrap();
    assert!(fresh.0 > id.0);
    roundtrip(&r);
}
#[test]
fn cell_destruction_detaches_once_preserves_core_and_exact_mass_and_reload() {
    let (mut s, id) = room();
    s.advance_construction(Duration::from_secs(300));
    let core = s
        .building
        .structure
        .parts()
        .find(|p| p.id == StructureId(1))
        .unwrap()
        .clone();
    let port = s
        .building
        .structure
        .occupied_cells()
        .find(|(c, _)| *c == Cell::new(-3, 0, 1))
        .unwrap()
        .1;
    let energy = s.ship.snapshot().energy_core.stored_joules;
    hit(&mut s, Target::Part(port), true, 500_000);
    assert_eq!(s.building.structure.parts().count(), 2);
    assert_eq!(
        s.building
            .structure
            .parts()
            .find(|p| p.id == StructureId(1)),
        Some(&core)
    );
    assert!(
        s.building
            .source
            .projects
            .inspect(id)
            .unwrap()
            .work
            .destroyed
    );
    assert_eq!(s.mining.session.fragments().len(), 4);
    assert_eq!(
        s.mining
            .session
            .fragments()
            .iter()
            .map(|f| f.material().mass_kg())
            .sum::<f64>(),
        35.
    );
    assert_eq!(
        s.ship.snapshot().total_mass_kg,
        salimon_ship::STARTER_DRY_MASS_KG
    );
    assert_eq!(s.ship.snapshot().energy_core.stored_joules, energy);
    let mut r = roundtrip(&s);
    for protected in [1, 2, port.0] {
        assert!(
            r.construction_impact(Impact {
                target: Target::Part(StructureId(protected)),
                destroy: true,
                survival_ppm: COMPLETE
            })
            .is_err()
        );
    }
    r.advance_construction(Duration::from_secs(400));
    assert_eq!(r.building.structure, s.building.structure);
    assert_eq!(r.mining.session.fragments().len(), 4);
}
#[test]
fn independent_cell_condition_safe_dismantle_and_malformed_save_rejection() {
    let (mut s, _) = room();
    s.advance_construction(Duration::from_secs(300));
    let port = s
        .building
        .structure
        .occupied_cells()
        .find(|(c, _)| *c == Cell::new(-3, 0, 1))
        .unwrap()
        .1;
    hit(&mut s, Target::Part(port), false, COMPLETE);
    assert_eq!(s.building.source.damage.damaged.len(), 1);
    assert!(s.construction_takeoff_blocker().is_some());
    let mut r = roundtrip(&s);
    assert!(r.dismantle_construction(Target::Part(port)).is_err());
    let leaf = r
        .building
        .structure
        .occupied_cells()
        .find(|(c, _)| *c == Cell::new(-4, 1, 1))
        .unwrap()
        .1;
    // Clear player/cargo before deliberate removal of this upper cell.
    move_player(&mut r, [0., SHIP_FLOOR_HEIGHT_METERS + 1.75, 0.]);
    r.dismantle_construction(Target::Part(leaf)).unwrap();
    assert_eq!(r.building.structure.parts().count(), 5);
    assert_eq!(r.mining.session.fragments().len(), 1);
    roundtrip(&r);
    let good = r.save_bytes(100).unwrap();
    let mut bad: serde_json::Value = serde_json::from_slice(&good).unwrap();
    bad["schema_version"] = 20.into();
    assert!(r.restore_bytes(&serde_json::to_vec(&bad).unwrap()).is_err());
    assert_eq!(r.save_bytes(100).unwrap(), good);
    bad["schema_version"] = 21.into();
    bad["state"]["construction"]["damage"]["lost"] = serde_json::json!([1]);
    assert!(r.restore_bytes(&serde_json::to_vec(&bad).unwrap()).is_err());
    assert_eq!(r.save_bytes(100).unwrap(), good);
}
#[test]
fn salvage_allocator_failure_is_atomic() {
    let mut s = GameplaySession::default();
    piece(&mut s, ResourceId::IronOre, 1.);
    let mut state = s.mining.session.save_state();
    state.next_id = u64::MAX;
    let mut world = salimon_world::mining::MiningSession::from_save(state).unwrap();
    let f = world.fragments()[0];
    assert!(
        world
            .import_salvage(&[(f.source(), f.material(), f.transform())])
            .is_none()
    );
    assert_eq!(world.fragments().len(), 1);
}

#[test]
fn damaged_module_requires_physical_restoration_and_destroyed_module_never_commissions() {
    let mut s = GameplaySession::default();
    evidence_module(&mut s, 1).unwrap();
    let id = s.building.source.projects.jobs().next().unwrap().id;
    hit(&mut s, Target::Project(id), false, COMPLETE);
    assert_eq!(s.utility_recovery_multiplier(), 1.);
    roundtrip(&s);
    s.construction_physical_action(BuildAction::Repair);
    assert!(!s.building.source.projects.inspect(id).unwrap().work.damaged);
    assert_eq!(s.utility_recovery_multiplier(), 1.);
    s.construction_action(BuildAction::Module);
    assert_eq!(
        s.building.source.modules[&id]
            .maintenance
            .as_ref()
            .unwrap()
            .kind,
        modules::Operation::Restore
    );
    hit(&mut s, Target::Project(id), true, 500_000);
    s.advance_construction(Duration::from_secs(500));
    assert_eq!(s.utility_recovery_multiplier(), 1.);
    roundtrip(&s);
}

#[test]
fn crash_marks_recoverable_cell_conditions_without_a_hidden_project_hull_lock() {
    let (mut s, id) = room();
    s.advance_construction(Duration::from_secs(300));
    let energy = s.ship.snapshot().energy_core.stored_joules;
    s.interrupt_construction_after_crash();
    assert_eq!(s.building.source.damage.damaged.len(), 4);
    assert!(!s.building.source.projects.inspect(id).unwrap().work.damaged);
    assert!(!s.construction_hull_ready());
    roundtrip(&s);
    s.building.source.damage.damaged.clear();
    assert!(s.construction_hull_ready());
    assert_eq!(s.ship.snapshot().energy_core.stored_joules, energy);
}

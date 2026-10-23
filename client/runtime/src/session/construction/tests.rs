use super::*;
use crate::session::{Command, ControlKey};
use std::time::Duration;
fn prepare() -> GameplaySession {
    let mut s = GameplaySession::default();
    s.command(Command::Control(ControlKey::Right, true));
    s.advance_automation(Duration::from_millis(72 * 16));
    s.command(Command::Control(ControlKey::Right, false));
    let snapshot = s.snapshot();
    let frame = character_ship_frame(snapshot.ship.pose);
    let eye = frame.world_to_local(snapshot.character.eye_position_meters);
    let d = salimon_math::sub([-1.4, SHIP_FLOOR_HEIGHT_METERS, 3.2], eye);
    s.command(Command::Look(
        d[2].atan2(d[0]) / 0.0022,
        -d[1].atan2(d[0].hypot(d[2])) / 0.0022,
    ));
    s.command(Command::Build(BuildAction::Toggle));
    s
}
#[test]
fn production_preview_place_inspect_cancel_preserves_physical_mass_and_materials() {
    let mut s = prepare();
    let before_mass = s.ship.snapshot().total_mass_kg;
    assert!(s.building.active);
    let preview = s.construction_preview().unwrap();
    assert_eq!(preview.rejection, None, "{preview:?}");
    s.command(Command::Mining(true));
    s.command(Command::Mining(false));
    assert_eq!(s.building.source.projects.jobs().count(), 1);
    assert_eq!(s.ship.snapshot().total_mass_kg, before_mass);
    assert!(s.mining.session.fragments().is_empty());
    assert!(s.construction_inspect()["aimed_project"].is_u64());
    s.advance_automation(Duration::from_millis(300));
    s.command(Command::Mining(true));
    s.command(Command::Mining(false));
    assert_eq!(s.building.source.projects.jobs().count(), 1);
    s.command(Command::Interact);
    assert!(!s.archive_reader.open);
    assert!(!s.crafting_ui.open);
    s.command(Command::Build(BuildAction::Cancel));
    assert_eq!(s.building.source.projects.jobs().count(), 0);
    assert_eq!(s.ship.snapshot().total_mass_kg, before_mass);
}
#[test]
fn placeholder_restore_is_exact_and_ui_is_transient_without_advancing_progress() {
    let mut s = prepare();
    s.command(Command::Mining(true));
    s.command(Command::Mining(false));
    let bytes = s.save_bytes(100).unwrap();
    let source = s.building.source.clone();
    let elapsed = s.elapsed;
    let mut restored = GameplaySession::default();
    restored.restore_bytes(&bytes).unwrap();
    assert_eq!(restored.building.source, source);
    assert_eq!(restored.elapsed, elapsed);
    assert!(!restored.building.active);
    assert_eq!(restored.construction_boxes().len(), 1);
    restored.building.active = true;
    restored.command(Command::Build(BuildAction::Cancel));
    assert_eq!(restored.building.source.projects.jobs().count(), 0);
}
#[test]
fn schema_eight_requires_source_and_legacy_seven_migrates_empty_atomically() {
    let mut s = prepare();
    s.command(Command::Mining(true));
    s.command(Command::Mining(false));
    let good = s.save_bytes(100).unwrap();
    let mut current: serde_json::Value = serde_json::from_slice(&good).unwrap();
    assert_eq!(current["schema_version"], 8);
    current["state"]
        .as_object_mut()
        .unwrap()
        .remove("construction");
    assert!(
        s.restore_bytes(&serde_json::to_vec(&current).unwrap())
            .is_err()
    );
    assert_eq!(s.save_bytes(100).unwrap(), good);
    current["schema_version"] = 7.into();
    s.restore_bytes(&serde_json::to_vec(&current).unwrap())
        .unwrap();
    assert_eq!(s.building.source.projects.jobs().count(), 0);
}
#[test]
fn wrong_owner_catalog_or_cells_never_replace_a_live_session() {
    let mut s = prepare();
    s.command(Command::Mining(true));
    s.command(Command::Mining(false));
    let good = s.save_bytes(100).unwrap();
    for (key, value) in [
        ("owner", serde_json::json!(2)),
        ("cells", serde_json::json!([{ "x":100,"y":0,"z":0}])),
    ] {
        let mut source: serde_json::Value = serde_json::from_slice(&good).unwrap();
        source["state"]["construction"]["projects"]["jobs"][0][key] = value;
        assert!(
            s.restore_bytes(&serde_json::to_vec(&source).unwrap())
                .is_err()
        );
        assert_eq!(s.save_bytes(100).unwrap(), good);
    }
}
#[test]
fn normal_ship_and_placeholders_do_not_install_structure_or_consume_resources() {
    let mut s = prepare();
    let parts = s.building.structure.parts().count();
    s.command(Command::Mining(true));
    s.command(Command::Mining(false));
    s.advance_automation(Duration::from_secs(10));
    assert_eq!(s.building.structure.parts().count(), parts);
    assert_eq!(s.building.source.projects.jobs().count(), 1);
    assert!(!s.building.catalog.approved_economy);
    assert_eq!(
        s.ship.snapshot().total_mass_kg,
        salimon_ship::STARTER_DRY_MASS_KG
    );
}

#[test]
fn saved_projects_cannot_bypass_the_registered_catalog_or_protected_routes() {
    use salimon_persistence::Validate;
    let structure = starter_structure();
    let mut source = ConstructionSave::default();
    let context = PlacementContext {
        actor: OWNER,
        owner: OWNER,
        flight: FlightPermission::Landed,
        eye_meters: [0.5, 1.9, -2.2],
        reach_meters: REACH_METERS,
        reachable: true,
        physical_support: true,
        obstacles: &[],
    };
    source
        .projects
        .place(
            registered_catalog(),
            "stool",
            [0.5, SHIP_FLOOR_HEIGHT_METERS, -2.2],
            Rotation::Forward,
            &structure,
            &context,
        )
        .unwrap();
    assert!(
        source.validate().is_err(),
        "a forged trusted-host context must not survive restore"
    );
    let mut s = prepare();
    s.command(Command::Mining(true));
    s.command(Command::Mining(false));
    let good = s.save_bytes(100).unwrap();
    let mut bad: serde_json::Value = serde_json::from_slice(&good).unwrap();
    bad["state"]["construction"]["projects"]["jobs"][0]["blueprint"]["requirements"][0]["milligrams"] =
        1.into();
    assert!(s.restore_bytes(&serde_json::to_vec(&bad).unwrap()).is_err());
    assert_eq!(s.save_bytes(100).unwrap(), good);
}

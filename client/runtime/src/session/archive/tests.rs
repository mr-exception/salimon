use super::*;
use crate::{
    e2e::{self, Config, Scenario},
    session::{
        Command, ControlKey,
        interaction::{InteractionTarget, available_interaction_target},
    },
};
use salimon_character::CharacterLocation;
use std::time::Duration;
fn place(s: &mut GameplaySession, local: [f64; 3]) {
    let mut v = serde_json::to_value(s.character.save_state()).unwrap();
    v["position"] = serde_json::json!({"Inside":{"local":local}});
    v["yaw_radians"] = serde_json::json!(std::f64::consts::PI * 1.5);
    s.character =
        salimon_character::CharacterController::from_save(serde_json::from_value(v).unwrap(), None)
            .unwrap();
}
fn at_terminal() -> GameplaySession {
    let mut s = GameplaySession::default();
    e2e::initialize(
        &mut s,
        Config {
            scenario: Scenario::LandedEarth,
            seed: 467,
            step: Duration::from_millis(16),
            fragment_load: None,
        },
    )
    .unwrap();
    place(&mut s, [-3.0, 1.8, -2.8]);
    s
}
#[test]
fn physical_target_has_no_exterior_remote_or_behind_panel_access() {
    let eye = [-3., 1.7, -2.8];
    assert_eq!(
        available_interaction_target(CharacterLocation::InsideShip, eye, TERMINAL),
        Some(InteractionTarget::Archive)
    );
    for location in [
        CharacterLocation::Space,
        CharacterLocation::Surface,
        CharacterLocation::Cockpit,
    ] {
        assert_eq!(available_interaction_target(location, eye, TERMINAL), None);
    }
    for (eye, look) in [
        ([-3., 1.7, 0.], TERMINAL),
        ([-3., 1.7, -4.5], TERMINAL),
        (eye, [-3., 1.7, 0.]),
        (eye, [100., 1.7, -4.13]),
    ] {
        assert!(!target(eye, look));
    }
}
#[test]
fn archive_roundtrip_preserves_exact_records_and_read_progress_but_not_modal_input() {
    let mut s = at_terminal();
    s.open_archive();
    assert!(s.archive_reader.open);
    s.archive_action(ReaderAction::Next);
    assert_eq!(s.archive.read.len(), 2);
    let bytes = s.save_bytes(100).unwrap();
    let mut restored = GameplaySession::default();
    restored.restore_bytes(&bytes).unwrap();
    assert!(!restored.archive_reader.open);
    assert_eq!(restored.save_bytes(100).unwrap(), bytes);
    restored.open_archive();
    assert!(restored.archive_text().unwrap().contains("Mars transit"));
    for _ in 0..20 {
        restored.archive_action(ReaderAction::Next);
    }
    assert!(restored.archive_text().unwrap().contains("CONTENT UNKNOWN"));
    assert_eq!(restored.archive.read.len(), RECORDS.len());
    restored.archive_action(ReaderAction::Close);
    restored.open_archive();
    assert!(restored.archive_text().unwrap().contains("record 02"));
    assert!(GameplaySession::new_run().archive.read.is_empty());
}
#[test]
fn missing_duplicate_unknown_and_future_record_state_reject_transactionally_and_legacy_initializes()
{
    let mut s = at_terminal();
    let bytes = s.save_bytes(100).unwrap();
    for mutation in [
        serde_json::Value::Null,
        serde_json::json!({"version":2,"selected":"voyage.earth-departure","read":[]}),
        serde_json::json!({"version":1,"selected":"not-a-record","read":[]}),
        serde_json::json!({"version":1,"selected":"voyage.earth-departure","read":["voyage.mars-transit","voyage.mars-transit"]}),
    ] {
        let mut v: serde_json::Value = serde_json::from_slice(&bytes).unwrap();
        v["state"]["archive"] = mutation;
        assert!(s.restore_bytes(&serde_json::to_vec(&v).unwrap()).is_err());
        assert_eq!(s.save_bytes(100).unwrap(), bytes);
    }
    let mut old: serde_json::Value = serde_json::from_slice(&bytes).unwrap();
    old["schema_version"] = 5.into();
    old["state"].as_object_mut().unwrap().remove("archive");
    s.restore_bytes(&serde_json::to_vec(&old).unwrap()).unwrap();
    assert!(s.archive.read.is_empty());
}
#[test]
fn reader_blocks_world_inputs_and_closes_on_pause_focus_power_loss_and_range_loss() {
    let mut s = at_terminal();
    s.open_archive();
    s.command(Command::Control(ControlKey::Forward, true));
    s.command(Command::Mining(true));
    assert!(!s.movement_input.forward);
    s.command(Command::Wheel(
        super::super::consumable_wheel::WheelAction::Hold(true),
    ));
    assert!(!s.wheel.open);
    s.command(Command::TogglePause);
    assert!(s.is_paused());
    assert!(!s.archive_reader.open);
    s.command(Command::TogglePause);
    s.open_archive();
    s.clear_held_controls();
    assert!(!s.archive_reader.open);
    s.open_archive();
    place(&mut s, [4., 1.8, 0.]);
    s.validate_archive_focus();
    assert!(!s.archive_reader.open);
}
#[test]
fn initial_records_have_bounded_accessible_lines_and_no_destination_or_spoiler() {
    let mut s = at_terminal();
    s.open_archive();
    for _ in 0..RECORDS.len() {
        let t = s.archive_text().unwrap();
        assert!(t.lines().all(|l| l.len() <= 58));
        for forbidden in ["Hall", "wormhole", "Absenat", "coordinates:"] {
            assert!(!t.contains(forbidden));
        }
        assert!(t.contains("TIME: UNKNOWN"));
        s.archive_action(ReaderAction::Next);
    }
}

#[test]
fn production_e_native_keys_disk_checkpoint_and_blackout_use_same_reader() {
    let mut s = at_terminal();
    let events = s.checkpoint_events();
    s.command(Command::Interact);
    assert!(s.archive_reader.open);
    assert_ne!(s.checkpoint_events(), events);
    crate::app::input::dispatch_session_key(
        &mut s,
        winit::keyboard::PhysicalKey::Code(winit::keyboard::KeyCode::ArrowDown),
        true,
    );
    assert!(s.archive_text().unwrap().contains("Mars transit"));
    let (w, h, _) = crate::action_bar::rasterize_action_bar(&s.archive_text().unwrap(), 3);
    assert!(w <= 1248 && h <= 768, "{w}x{h}");
    s.advance_automation(Duration::from_millis(16));
    assert_eq!(s.elapsed, Duration::from_millis(16));
    assert!(s.archive_reader.open);
    let path = std::env::temp_dir().join(format!("salimon-467-{}", std::process::id()));
    std::fs::create_dir(&path).unwrap();
    let validate = |b: &[u8]| GameplaySession::default().restore_bytes(b);
    {
        let mut store = salimon_persistence::storage::SaveManager::new(
            salimon_persistence::storage::NativeStore::open(path.clone()).unwrap(),
        );
        store.recover(validate).unwrap();
        store
            .checkpoint(&s.save_bytes(100).unwrap(), validate)
            .unwrap();
    }
    let mut restored = GameplaySession::default();
    let host = crate::checkpoints::Checkpoints::open_at(path.clone(), &mut restored, 100).unwrap();
    assert_eq!(restored.archive.selected, s.archive.selected);
    assert_eq!(restored.archive.read, s.archive.read);
    assert!(!restored.archive_reader.open);
    drop(host);
    std::fs::remove_dir_all(path).unwrap();
    s.ship.set_stored_core_energy_joules(0);
    s.validate_archive_focus();
    assert!(!s.archive_reader.open);
    s.command(Command::Interact);
    assert!(!s.archive_reader.open);
    s.ship.set_stored_core_energy_joules(1_000_000);
    s.command(Command::Interact);
    assert!(s.archive_reader.open);
    crate::app::input::dispatch_session_key(
        &mut s,
        winit::keyboard::PhysicalKey::Code(winit::keyboard::KeyCode::KeyE),
        true,
    );
    assert!(!s.archive_reader.open);
}

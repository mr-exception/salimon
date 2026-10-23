use super::*;
use serde_json::{Value, json};
fn flying() -> GameplaySession {
    let mut s = GameplaySession::new_run();
    s.character.enter_cockpit();
    s.ship =
        ShipController::flying_with_velocity(s.ship.snapshot().pose, 70, [20.0, 0.0, 0.0]).unwrap();
    s
}
#[test]
fn engine_switch_survives_whole_save_and_legacy_forces_migrate_without_velocity_or_energy_changes()
{
    let mut s = flying();
    s.command(Command::ToggleEngines);
    assert!(!s.ship.snapshot().engines_enabled);
    let before = s.save_bytes(100).unwrap();
    s.restore_bytes(&before).unwrap();
    assert_eq!(s.save_bytes(100).unwrap(), before);
    assert!(!s.ship.snapshot().engines_enabled);
    let duplicate = String::from_utf8(before.clone()).unwrap().replace(
        "\"engines_enabled\":false",
        "\"engines_enabled\":false,\"engines_enabled\":true",
    );
    assert!(s.restore_bytes(duplicate.as_bytes()).is_err());
    assert_eq!(s.save_bytes(100).unwrap(), before);
    let mut old: Value = serde_json::from_slice(&before).unwrap();
    old["schema_version"] = 4.into();
    let ship = old["state"]["physical"]["ship"].as_object_mut().unwrap();
    ship.remove("engines_enabled");
    ship["propulsion"] =
        json!({"forward_force_newtons":4_000_000.0,"reverse_force_newtons":2_000_000.0});
    let velocity = s.ship.snapshot().velocity_meters_per_second;
    let energy = s.ship.snapshot().energy_core;
    s.restore_bytes(&serde_json::to_vec(&old).unwrap()).unwrap();
    assert!(s.ship.snapshot().engines_enabled);
    assert_eq!(s.ship.snapshot().velocity_meters_per_second, velocity);
    assert_eq!(s.ship.snapshot().energy_core, energy);
    assert_eq!(
        s.ship
            .snapshot()
            .navigation
            .acceleration_meters_per_second_squared,
        Some(700.0 / (s.ship.snapshot().total_mass_kg / 100_000.0))
    );
    // New-schema files cannot smuggle retired force capacity or omit the switch.
    old["schema_version"] = 5.into();
    let current = s.save_bytes(100).unwrap();
    assert!(s.restore_bytes(&serde_json::to_vec(&old).unwrap()).is_err());
    assert_eq!(s.save_bytes(100).unwrap(), current);
}

#[test]
fn engine_off_disk_relaunch_keeps_momentum_selection_and_requires_fresh_enable() {
    use salimon_persistence::storage::{NativeStore, SaveManager};
    use std::time::Duration;
    let mut s = flying();
    // Clear-space fixture: the new-run default pose is grounded on Earth.
    s.ship = ShipController::flying_with_velocity(
        salimon_ship::ShipPose {
            position_meters: [0.0; 3],
            orientation: [0.0, 0.0, 0.0, 1.0],
        },
        70,
        [20.0, 0.0, 0.0],
    )
    .unwrap();
    s.command(Command::ToggleEngines);
    let before = s.ship.snapshot();
    let path = std::env::temp_dir().join(format!("salimon-498-{}", std::process::id()));
    if path.exists() {
        std::fs::remove_dir_all(&path).unwrap();
    }
    std::fs::create_dir(&path).unwrap();
    let validate = |bytes: &[u8]| GameplaySession::default().restore_bytes(bytes);
    {
        let mut store = SaveManager::new(NativeStore::open(path.clone()).unwrap());
        store.recover(validate).unwrap();
        store
            .checkpoint(&s.save_bytes(100).unwrap(), validate)
            .unwrap();
    }
    let mut restored = GameplaySession::default();
    let host = crate::checkpoints::Checkpoints::open_at(path.clone(), &mut restored, 100).unwrap();
    let after = restored.ship.snapshot();
    assert!(!after.engines_enabled);
    assert_eq!(after.thruster_percentage, before.thruster_percentage);
    assert_eq!(
        after.velocity_meters_per_second,
        before.velocity_meters_per_second
    );
    assert_eq!(after.energy_core, before.energy_core);
    restored.ship.set_stored_core_energy_joules(0);
    restored.ship.set_stored_core_energy_joules(1_000_000_000);
    restored.advance_game(Duration::from_millis(96));
    assert!(!restored.ship.snapshot().engines_enabled);
    assert_eq!(
        restored.ship.snapshot().velocity_meters_per_second,
        before.velocity_meters_per_second
    );
    restored.command(Command::ToggleEngines);
    restored.advance_game(Duration::from_millis(96));
    assert!(restored.ship.snapshot().engines_enabled);
    assert!(restored.ship.snapshot().speed_meters_per_second > before.speed_meters_per_second);
    drop(host);
    std::fs::remove_dir_all(path).unwrap();
}

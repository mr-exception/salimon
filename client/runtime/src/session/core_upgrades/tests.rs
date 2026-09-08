use super::*;
use crate::session::Command;
use salimon_world::resources::{FragmentId, ResourceId, ResourceTransform};
fn setup() -> GameplaySession {
    let mut s = GameplaySession::new_run();
    evidence_initial_conditions(&mut s, false).unwrap();
    s
}
fn upgrade(s: &mut GameplaySession, a: CoreUpgradeAction) {
    s.command(Command::CoreUpgrade(a));
}
fn json_ship(s: &GameplaySession) -> serde_json::Value {
    serde_json::to_value(s.ship.save_state()).unwrap()
}
fn next_piece(s: &mut GameplaySession, material: ResourceId, mass: f64) -> FragmentId {
    let frame = character_ship_frame(s.ship.snapshot().pose);
    let p = frame.local_to_world([-1., 1.3, -1.5]);
    use salimon_world::resources::{DepositId, RawMaterial, ResourceDeposit};
    use salimon_world::spherical_mining::{MiningConfig, SPHERE_TICK};
    let pose = ResourceTransform::new(
        salimon_world::WorldPosition::new(p[0], p[1], p[2]),
        [0., 0., 0., 1.],
    )
    .unwrap();
    let mut deposit = ResourceDeposit::new(
        DepositId {
            body: salimon_world::CelestialBodyId::Earth,
            local: 548100 + s.mining.session.save_state().next_id,
        },
        RawMaterial::new(material, mass).unwrap(),
        pose.position(),
        mass,
    )
    .unwrap();
    let id = (0..400)
        .find_map(|_| {
            s.mining.session.tick_spherical(
                true,
                Some(&mut deposit),
                SPHERE_TICK,
                MiningConfig::default(),
                pose,
            )
        })
        .unwrap();
    assert!(s.mining.session.pick_up(id));
    s.equipment.stow();
    s.sync_carried();
    id
}
#[test]
fn zero_charge_exact_material_instant_capacity_mass_and_repeated_activation() {
    let mut s = setup();
    let mass = s.ship.snapshot().total_mass_kg;
    assert_eq!(s.core_upgrades.pending.len(), 2);
    upgrade(&mut s, CoreUpgradeAction::ToggleInput);
    let before = s.save_bytes(100).unwrap();
    upgrade(&mut s, CoreUpgradeAction::Apply);
    assert_eq!(s.save_bytes(100).unwrap(), before, "insufficient is atomic");
    s.command(Command::Interact);
    assert_eq!(s.core_upgrades.pending.len(), 3);
    assert_eq!(s.mining.session.carried_id(), None);
    assert_eq!(s.ship.snapshot().total_mass_kg, mass);
    let mut expected = json_ship(&s);
    let elapsed = s.elapsed;
    upgrade(&mut s, CoreUpgradeAction::Apply);
    expected["energy_core"]["config"]["capacity_joules"] = 90_000_000_000u64.into();
    assert_eq!(
        json_ship(&s),
        expected,
        "only capacity changes in the existing Core"
    );
    assert_eq!(s.elapsed, elapsed);
    assert_eq!(s.core_upgrades.applied.len(), 1);
    assert_eq!(s.ship.snapshot().total_mass_kg, mass);
    let bytes = s.save_bytes(100).unwrap();
    upgrade(&mut s, CoreUpgradeAction::Apply);
    assert_eq!(
        s.save_bytes(100).unwrap(),
        bytes,
        "repeat has no cost or parameter change"
    );
    let mut r = GameplaySession::default();
    r.restore_bytes(&bytes).unwrap();
    assert_eq!(r.save_bytes(100).unwrap(), bytes);
}
#[test]
fn refund_keeps_original_identity_mass_and_duplicate_returns_are_rejected() {
    let mut s = setup();
    upgrade(&mut s, CoreUpgradeAction::ToggleInput);
    s.command(Command::Interact);
    let original = *s.core_upgrades.pending.last().unwrap();
    let mut bad = serde_json::to_value(original).unwrap();
    bad["material"]["volume_m3"] = 999.0.into();
    let bad: salimon_world::resources::ResourceFragment = serde_json::from_value(bad).unwrap();
    let world_before = serde_json::to_value(s.mining.session.save_state()).unwrap();
    assert!(!s.mining.session.return_reserved_fragment(bad));
    assert_eq!(
        serde_json::to_value(s.mining.session.save_state()).unwrap(),
        world_before
    );
    let mass = s.ship.snapshot().total_mass_kg;
    upgrade(&mut s, CoreUpgradeAction::Recover);
    let id = s.mining.session.carried_id().unwrap();
    assert_eq!(id, original.id());
    let f = *s.mining.session.fragment(id).unwrap();
    assert_eq!(f.source(), original.source());
    assert_eq!(f.material(), original.material());
    assert_eq!(s.ship.snapshot().total_mass_kg, mass);
    let before = s.save_bytes(100).unwrap();
    upgrade(&mut s, CoreUpgradeAction::Recover);
    assert_eq!(s.save_bytes(100).unwrap(), before);
    assert!(!s.mining.session.return_reserved_fragment(original));
    s.command(Command::Interact);
    assert_eq!(s.core_upgrades.pending.last().unwrap().id(), id);
}
#[test]
fn explicit_fuel_mode_burns_only_carried_material_and_upgrade_mode_survives_reload() {
    let mut s = setup();
    let pending = serde_json::to_value(&s.core_upgrades.pending).unwrap();
    s.command(Command::Interact);
    assert!(s.ship.conversion_remaining_joules() > 0.);
    assert_eq!(
        serde_json::to_value(&s.core_upgrades.pending).unwrap(),
        pending
    );
    assert_eq!(
        s.ship.snapshot().energy_core.stored_joules,
        0,
        "no conversion timer elapsed"
    );
    upgrade(&mut s, CoreUpgradeAction::ToggleInput);
    let bytes = s.save_bytes(100).unwrap();
    let mut r = GameplaySession::default();
    r.restore_bytes(&bytes).unwrap();
    assert_eq!(r.core_upgrades.mode, InputMode::Upgrade);
    let id = next_piece(&mut r, ResourceId::WaterIce, 10.);
    let before = r.save_bytes(100).unwrap();
    r.command(Command::Interact);
    assert_eq!(r.save_bytes(100).unwrap(), before);
    assert_eq!(r.mining.session.carried_id(), Some(id));
}
#[test]
fn oversized_and_corrupt_receipts_leave_source_unchanged() {
    let mut s = setup();
    upgrade(&mut s, CoreUpgradeAction::ToggleInput);
    s.command(Command::Interact);
    next_piece(&mut s, ResourceId::IronOre, 10.);
    let before = s.save_bytes(100).unwrap();
    s.command(Command::Interact);
    assert_eq!(s.save_bytes(100).unwrap(), before);
    let mut v: serde_json::Value = serde_json::from_slice(&before).unwrap();
    v["state"]["core_upgrades"]["pending"][0] = v["state"]["core_upgrades"]["pending"][1].clone();
    assert!(s.restore_bytes(&serde_json::to_vec(&v).unwrap()).is_err());
    assert_eq!(s.save_bytes(100).unwrap(), before);
}
#[test]
fn fractional_joules_and_flight_source_are_preserved() {
    let mut s = setup();
    upgrade(&mut s, CoreUpgradeAction::ToggleInput);
    s.command(Command::Interact);
    next_piece(&mut s, ResourceId::WaterIce, 10.);
    assert_eq!(
        s.ship.intake_carried_fragment(&mut s.mining.session),
        salimon_ship::IntakeResult::Accepted
    );
    s.sync_carried();
    s.ship
        .configure_energy_core(s.ship.energy_config(), 12345678.456)
        .unwrap();
    s.ship.set_cockpit_control(true);
    s.ship.trigger_landing_action();
    assert!(
        matches!(
            s.ship.snapshot().flight_state,
            salimon_ship::FlightState::Landed { .. }
        ),
        "insufficient assist leaves state intact"
    );
    // Independently establish a real, legal live takeoff reservation.
    s.ship
        .configure_energy_core(s.ship.energy_config(), 6_000_000_000.456)
        .unwrap();
    s.ship.trigger_landing_action();
    assert!(matches!(
        s.ship.snapshot().flight_state,
        salimon_ship::FlightState::AssistedTakeoff { .. }
    ));
    let mut expected = json_ship(&s);
    assert_eq!(
        s.core_upgrades
            .apply(&mut s.ship, &s.mining.session, catalog()),
        UpgradeResult::Applied
    );
    expected["energy_core"]["config"]["capacity_joules"] = 90_000_000_000u64.into();
    assert_eq!(
        json_ship(&s),
        expected,
        "reservation, fractional energy, motion and engine switch untouched"
    );
}
#[test]
fn blocked_refund_and_paused_controls_are_atomic() {
    let mut s = setup();
    upgrade(&mut s, CoreUpgradeAction::ToggleInput);
    s.command(Command::Interact);
    s.mining.interior_solids = vec![[-100., 100., -100., 100., -100., 100.]];
    let before = s.save_bytes(100).unwrap();
    upgrade(&mut s, CoreUpgradeAction::Recover);
    assert_eq!(s.save_bytes(100).unwrap(), before);
    s.command(Command::TogglePause);
    let before = s.save_bytes(100).unwrap();
    upgrade(&mut s, CoreUpgradeAction::ToggleInput);
    upgrade(&mut s, CoreUpgradeAction::Apply);
    assert_eq!(s.save_bytes(100).unwrap(), before);
}

#[test]
fn all_three_catalog_tiers_use_exact_paid_receipts_and_keep_the_same_core() {
    let mut s = setup();
    upgrade(&mut s, CoreUpgradeAction::ToggleInput);
    s.command(Command::Interact);
    upgrade(&mut s, CoreUpgradeAction::Apply);
    for level in [2usize, 3] {
        let tier = &catalog()[level - 1];
        for (material, mg) in [
            (ResourceId::IronOre, tier.iron_milligrams),
            (ResourceId::SilicateRock, tier.silicate_milligrams),
        ] {
            for _ in 0..mg / 10_000_000 {
                next_piece(&mut s, material, 10.);
                s.command(Command::Interact);
            }
        }
        assert_eq!(s.core_upgrades.remaining(catalog()), Some([0, 0]));
        let mass = s.ship.snapshot().total_mass_kg;
        upgrade(&mut s, CoreUpgradeAction::Apply);
        assert_eq!(s.core_upgrades.applied.len(), level);
        assert_eq!(
            s.ship.snapshot().energy_core.capacity_joules,
            tier.capacity_joules
        );
        assert_eq!(s.ship.snapshot().energy_core.stored_joules, 0);
        assert_eq!(s.ship.snapshot().total_mass_kg, mass);
        let bytes = s.save_bytes(100).unwrap();
        let mut r = GameplaySession::default();
        r.restore_bytes(&bytes).unwrap();
        assert_eq!(r.save_bytes(100).unwrap(), bytes);
    }
    assert_eq!(s.core_upgrades.mass_kg(), 130.);
    assert_eq!(
        s.core_upgrades
            .applied
            .iter()
            .map(|a| a.receipts.len())
            .sum::<usize>(),
        13
    );
    let before = s.save_bytes(100).unwrap();
    upgrade(&mut s, CoreUpgradeAction::Apply);
    assert_eq!(s.save_bytes(100).unwrap(), before);
}
#[test]
fn non_milligram_and_configuration_mismatch_declines_are_atomic() {
    let mut s = setup();
    upgrade(&mut s, CoreUpgradeAction::ToggleInput);
    s.command(Command::Interact);
    s.ship
        .configure_energy_core(
            salimon_ship::EnergyConfig {
                capacity_joules: 1000,
                ..Default::default()
            },
            123.456,
        )
        .unwrap();
    let before = s.save_bytes(100).unwrap();
    upgrade(&mut s, CoreUpgradeAction::Apply);
    assert_eq!(s.save_bytes(100).unwrap(), before);
    // A new tier is never purchased by rounding sub-milligram material upward.
    let character = s.character.save_state();
    let mut s = GameplaySession::new_run();
    s.character = salimon_character::CharacterController::from_save(character, None).unwrap();
    s.core_upgrades.mode = InputMode::Upgrade;
    next_piece(&mut s, ResourceId::IronOre, 0.000_000_3);
    let before = s.save_bytes(100).unwrap();
    s.command(Command::Interact);
    assert_eq!(s.save_bytes(100).unwrap(), before);
}

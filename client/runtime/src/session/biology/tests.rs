use super::*;
use crate::session::{Command, ControlKey};
fn fixture(embedded: bool) -> GameplaySession {
    let mut s = GameplaySession::default();
    initialize_fixture(&mut s, embedded).unwrap();
    s
}
fn grab(s: &mut GameplaySession) {
    s.command(Command::GrabDrop(true));
    s.command(Command::GrabDrop(false));
}
#[test]
fn actual_f_pull_commit_drop_save_reopen_preserve_identity_anatomy_and_missing_source() {
    let mut s = fixture(false);
    let key = s.biological_target().expect("reachable fixture appendage");
    grab(&mut s);
    assert!(s.biology.pending.is_some());
    s.advance_automation(Duration::from_millis(128));
    assert!(s.mining.session.fragments().is_empty());
    s.advance_automation(Duration::from_millis(160));
    let id = s.mining.session.carried_id().expect("same part carried");
    let shape = s
        .mining
        .session
        .fragment(id)
        .unwrap()
        .natural_shape()
        .unwrap();
    assert_eq!(s.biological_fraction(&key.organism, key.part), 0.);
    assert!(s.equipment.selected().is_none());
    assert!(
        s.organism_field().boxes().len()
            < population(true)
                .objects()
                .iter()
                .map(|o| o.parts.len())
                .sum::<usize>()
    );
    let bytes = s.save_bytes(100).unwrap();
    s.restore_bytes(&bytes).unwrap();
    assert_eq!(
        s.mining.biological_obstacles.boxes(),
        s.organism_field().boxes()
    );
    assert_eq!(
        s.mining.session.fragment(id).unwrap().natural_shape(),
        Some(shape)
    );
    assert_eq!(s.biological_fraction(&key.organism, key.part), 0.);
    grab(&mut s);
    assert!(s.mining.session.carried_id().is_none());
    assert_eq!(s.mining.session.fragments().len(), 1);
    s.advance_automation(Duration::from_millis(160));
    assert!(
        s.mining
            .session
            .fragment(id)
            .unwrap()
            .transform()
            .position()
            .is_finite()
    );
    let bytes = s.save_bytes(100).unwrap();
    s.restore_bytes(&bytes).unwrap();
    assert_eq!(s.mining.session.fragments().len(), 1);
}
#[test]
fn pause_focus_loss_aim_loss_and_occupied_slot_do_not_commit_or_regrow() {
    for mode in 0..3 {
        let mut s = fixture(false);
        grab(&mut s);
        assert!(s.biology.pending.is_some());
        s.advance_automation(Duration::from_millis(64));
        match mode {
            0 => s.clear_held_controls(),
            1 => s.command(Command::TogglePause),
            _ => s.command(Command::Look(900., 0.)),
        };
        s.advance_automation(Duration::from_millis(400));
        assert!(s.mining.session.fragments().is_empty());
        assert!(
            s.biology
                .journal
                .as_ref()
                .is_none_or(|j| j.deltas().is_empty())
        );
    }
    let mut s = fixture(false);
    assert!(s.mining.session.carry_fixture(999));
    grab(&mut s);
    s.advance_automation(Duration::from_millis(400));
    assert!(s.mining.session.fragments().is_empty());
    let mut s = fixture(false);
    s.command(Command::Control(ControlKey::Backward, true));
    s.advance_automation(Duration::from_secs(3));
    s.command(Command::Control(ControlKey::Backward, false));
    assert!(s.biological_target().is_none());
}
#[test]
fn actual_embedded_mining_partial_shape_and_checkpoint_conserve_material() {
    let mut s = fixture(true);
    let key = s.biological_target().expect("reachable medicinal reserve");
    s.command(Command::Select(crate::equipment::ToolbarSlot::One));
    s.command(Command::Mining(true));
    s.advance_automation(Duration::from_millis(64));
    s.command(Command::Mining(false));
    let fraction = s.biological_fraction(&key.organism, key.part);
    assert!(fraction > 0. && fraction < 1.);
    assert!(!s.mining.session.fragments().is_empty());
    let bytes = s.save_bytes(100).unwrap();
    s.restore_bytes(&bytes).unwrap();
    assert_eq!(fraction, s.biological_fraction(&key.organism, key.part));
    assert!(s.biology.pending.is_none());
    s.biology_capture().unwrap();
}

#[test]
fn malformed_cross_owner_saves_and_mid_pull_reload_never_publish_or_duplicate() {
    let mut s = fixture(false);
    grab(&mut s);
    s.advance_automation(Duration::from_millis(64));
    let mid = s.save_bytes(100).unwrap();
    s.restore_bytes(&mid).unwrap();
    assert!(s.biology.pending.is_none());
    assert!(s.mining.session.fragments().is_empty());
    grab(&mut s);
    s.advance_automation(Duration::from_millis(288));
    let saved = s.save_bytes(100).unwrap();
    let original: serde_json::Value = serde_json::from_slice(&saved).unwrap();
    for mode in 0..5 {
        let mut bad = original.clone();
        match mode {
            0 => bad["state"]["biology"]["journal"]["identity"] = "wrong-publication".into(),
            1 => bad["state"]["biology"]["journal"]["deltas"][0]["remaining_kg"] = 1.into(),
            2 => bad["state"]["biology"]["journal"]["deltas"][0]["detached"] = 999.into(),
            3 => {
                bad["state"]["physical"]["world"]["fragments"][0]["natural_shape"]["radii"][0] =
                    0.9.into()
            }
            _ => bad["state"]["biology"] = serde_json::Value::Null,
        }
        assert!(
            s.restore_bytes(&serde_json::to_vec(&bad).unwrap()).is_err(),
            "corruption mode {mode}"
        );
        assert_eq!(s.save_bytes(100).unwrap(), saved);
    }
}

#[test]
fn embedded_checkpoint_rejects_forged_material_exceeding_debit() {
    let mut s = fixture(true);
    s.command(Command::Select(crate::equipment::ToolbarSlot::One));
    s.command(Command::Mining(true));
    s.advance_automation(Duration::from_millis(64));
    s.command(Command::Mining(false));
    let before = s.save_bytes(100).unwrap();
    let mut corrupt: serde_json::Value = serde_json::from_slice(&before).unwrap();
    let mass = 100.;
    let material = &mut corrupt["state"]["physical"]["world"]["fragments"][0]["material"];
    material["mass_kg"] = mass.into();
    material["volume_m3"] = (mass
        / salimon_world::resources::ResourceId::MedicinalCompounds
            .definition()
            .density_kg_per_m3)
        .into();
    assert!(
        s.restore_bytes(&serde_json::to_vec(&corrupt).unwrap())
            .is_err()
    );
    assert_eq!(s.save_bytes(100).unwrap(), before);
}

#[test]
fn construction_modal_keeps_its_controls_and_cancels_pending_separation() {
    let mut s = fixture(false);
    grab(&mut s);
    assert!(s.biology.pending.is_some());
    s.command(Command::Build(
        crate::session::construction::BuildAction::Toggle,
    ));
    assert!(s.building.active);
    assert!(s.biological_target().is_none());
    s.advance_automation(Duration::from_millis(400));
    assert!(s.mining.session.fragments().is_empty());
    assert!(s.biology.pending.is_none());
}

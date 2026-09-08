//! Integration acceptance uses actual scanner funding, source and input adapters.
use super::*;
use crate::session::input::{Key, dispatch};

fn cockpit() -> GameplaySession {
    let mut s = GameplaySession::new_canonical_run().unwrap();
    s.character = Default::default();
    s.character.enter_cockpit();
    s.ship.set_cockpit_control(true);
    s
}

#[test]
fn scanner_exact_pulse_boundary_cost_and_unfunded_contact_do_not_invent_history() {
    let mut s = cockpit();
    s.ship.exchange_core_energy_joules(0., 1e30).unwrap();
    s.ship.exchange_core_energy_joules(1_000_000., 0.).unwrap();
    // Isolate the actual production scanner service from other bus consumers.
    s.try_advance_scanner(Duration::from_millis(4999)).unwrap();
    assert_eq!(s.scanner.contacts.history().count(), 0);
    assert_eq!(s.ship.snapshot().energy_core.stored_joules, 995_001);
    s.try_advance_scanner(Duration::from_millis(1)).unwrap();
    assert_eq!(s.scanner.contacts.history().count(), 1);
    assert_eq!(s.ship.snapshot().energy_core.stored_joules, 985_000);
    assert_eq!(
        s.scanner
            .contacts
            .history()
            .next()
            .unwrap()
            .last_seen_utc_millis(),
        5000
    );
    s.try_advance_scanner(Duration::from_secs(5)).unwrap();
    assert_eq!(s.ship.snapshot().energy_core.stored_joules, 970_000);
    assert_eq!(
        s.scanner
            .contacts
            .history()
            .next()
            .unwrap()
            .last_seen_utc_millis(),
        10_000
    );
    s.ship.exchange_core_energy_joules(0., 1e30).unwrap();
    s.ship.exchange_core_energy_joules(500., 0.).unwrap();
    s.try_advance_scanner(Duration::from_secs(5)).unwrap();
    assert!(!s.scanner.powered);
    assert_eq!(s.ship.snapshot().energy_core.stored_joules, 0);
    assert_eq!(
        s.scanner
            .contacts
            .history()
            .next()
            .unwrap()
            .last_seen_utc_millis(),
        10_000
    );
}

#[test]
fn scanner_every_modal_and_view_gate_blocks_selection_and_hides_readout() {
    for gate in 0..7 {
        let mut s = cockpit();
        s.advance_automation(Duration::from_millis(5008));
        match gate {
            0 => s.command(crate::session::Command::TogglePause),
            1 => s.archive_reader.open = true,
            2 => s.crafting_ui.open = true,
            3 => s.wheel.open = true,
            4 => {
                s.character.leave_cockpit();
            }
            5 => s.view_mode = crate::session::ViewMode::PrecisionTour,
            _ => {
                s.survival
                    .command(salimon_survival::Command::Damage(
                        salimon_survival::Points::FULL,
                    ))
                    .unwrap();
            }
        }
        let before = serde_json::to_value(s.scanner.contacts.checkpoint()).unwrap();
        let energy = s.ship.snapshot().energy_core.stored_joules;
        dispatch(&mut s, Key::ScannerNext, true);
        dispatch(&mut s, Key::ScannerNext, false);
        assert_eq!(s.scanner.selected, 0, "gate {gate}");
        assert!(s.scanner_hint().is_none(), "gate {gate}");
        assert_eq!(
            serde_json::to_value(s.scanner.contacts.checkpoint()).unwrap(),
            before
        );
        assert_eq!(s.ship.snapshot().energy_core.stored_joules, energy);
    }
}

//! Stateless snapshot-to-visual adapter. Never advances, consumes or persists.
use crate::session::GameplaySession;
use salimon_fabrication::FabricatorStatus;
use salimon_renderer::{MachineryFeedback, MachineryState};
use salimon_ship::ShipSnapshot;

pub(crate) fn capture(session: &GameplaySession, ship: ShipSnapshot) -> MachineryFeedback {
    let machine = session.fabrication_snapshot();
    let powered = ship.power_available;
    MachineryFeedback {
        rudder: if powered && ship.cockpit_control_active {
            ship.steering.yaw as f32
        } else {
            0.
        },
        thrust: if powered
            && ship
                .navigation
                .acceleration_meters_per_second_squared
                .is_some()
        {
            f32::from(ship.thruster_percentage) / 100.
        } else {
            0.
        },
        converting: powered && ship.core_status.conversion_watts > 0.,
        core_charge: (ship.energy_core.stored_joules as f64
            / ship.energy_core.capacity_joules.max(1) as f64)
            .clamp(0., 1.) as f32,
        fabricator: match machine.status {
            FabricatorStatus::TrayBlocked => MachineryState::Blocked,
            FabricatorStatus::NoPower => MachineryState::Unpowered,
            FabricatorStatus::Working if powered && ship.core_status.fabricator_watts > 0. => {
                MachineryState::Working
            }
            FabricatorStatus::Working => MachineryState::Unpowered,
            FabricatorStatus::Idle => MachineryState::Idle,
        },
        fabrication_progress: machine.jobs.first().map_or(0., |job| {
            (job.progress.as_secs_f64() / job.unit_duration.as_secs_f64()).clamp(0., 1.) as f32
        }),
        // Reduce integer time BEFORE narrowing, even after very long journeys.
        phase: (session.elapsed.as_nanos() % 4_000_000_000) as f32 / 4_000_000_000.,
        cockpit_active: powered && ship.cockpit_control_active,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use salimon_fabrication::{FabricationCommand, RecipeId};
    use std::time::Duration;
    #[test]
    fn snapshot_mapping_is_observational_repeatable_and_frozen_while_paused() {
        let mut s = GameplaySession::new_run();
        s.advance_automation(Duration::from_millis(300));
        let before = s.save_bytes(100).unwrap();
        let a = capture(&s, s.ship.snapshot());
        assert_eq!(a, capture(&s, s.ship.snapshot()));
        assert_eq!(before, s.save_bytes(100).unwrap());
        let mut restored = GameplaySession::default();
        restored.restore_bytes(&before).unwrap();
        assert_eq!(a, capture(&restored, restored.ship.snapshot()));
        s.toggle_pause();
        s.advance_automation(Duration::from_secs(9));
        assert_eq!(a, capture(&s, s.ship.snapshot()));
    }
    #[test]
    fn processing_progress_stops_for_blackout_and_blocked_tray() {
        let mut s = GameplaySession::new_run();
        s.execute_fabricator(FabricationCommand::Enqueue {
            recipe: RecipeId::new("food-ration").unwrap(),
            quantity: 1,
        })
        .unwrap();
        s.advance_automation(Duration::from_secs(1));
        let working = capture(&s, s.ship.snapshot());
        assert_eq!(working.fabricator, MachineryState::Working);
        assert!(working.fabrication_progress > 0.);
        s.ship.set_stored_core_energy_joules(0);
        let off = capture(&s, s.ship.snapshot());
        assert_eq!(off.fabricator, MachineryState::Unpowered);
        assert_eq!(off.thrust, 0.);
        assert!(!off.converting);
        s.ship.set_stored_core_energy_joules(10_000_000_000);
        s.advance_automation(Duration::from_secs(10));
        let blocked = capture(&s, s.ship.snapshot());
        assert_eq!(blocked.fabricator, MachineryState::Blocked);
        assert!(s.fabricator_output_present());
    }
    #[test]
    fn retained_throttle_does_not_look_like_thrust_during_blackout() {
        let mut s = GameplaySession::default();
        s.ship = salimon_ship::ShipController::flying(s.ship.snapshot().pose, -75);
        let powered = capture(&s, s.ship.snapshot());
        assert_eq!(powered.thrust, -0.75);
        s.ship.set_stored_core_energy_joules(0);
        assert_eq!(s.ship.snapshot().thruster_percentage, -75);
        assert_eq!(capture(&s, s.ship.snapshot()).thrust, 0.);
    }
}

#[cfg(test)]
mod steering_tests {
    use super::*;
    use std::time::Duration;
    #[test]
    fn rudder_follows_ramped_authority_and_inspection_is_consistent() {
        let mut s = GameplaySession::default();
        let mut pose = s.ship.snapshot().pose;
        pose.position_meters[1] += 100.;
        s.ship = salimon_ship::ShipController::flying(pose, 0);
        s.ship.set_cockpit_control(true);
        s.ship.set_steering_input(salimon_ship::SteeringInput {
            yaw: 1.,
            ..Default::default()
        });
        s.ship.advance(Duration::from_millis(100));
        let ship = s.ship.snapshot();
        let feedback = capture(&s, ship);
        assert_eq!(feedback.rudder, ship.steering.yaw as f32);
        assert!(feedback.rudder > 0. && feedback.rudder < 1.);
        let inspected = crate::automation::inspect_session(&s);
        assert_eq!(
            inspected["machinery"]["rudder"].as_f64().unwrap() as f32,
            feedback.rudder
        );
        s.ship.set_cockpit_control(false);
        assert_eq!(capture(&s, s.ship.snapshot()).rudder, 0.);
    }
}

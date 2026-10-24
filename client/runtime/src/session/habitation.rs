//! Base habitation fixtures; no new hygiene, water, sleep or survival authority.
use super::GameplaySession;
use salimon_persistence::{SaveError, Validate, invalid};
use std::time::Duration;
#[derive(Clone, Debug, Default, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct Habitation {
    pub(crate) bunk_open_fraction: f64,
    pub(crate) bunk_deploying: bool,
}
impl Validate for Habitation {
    fn validate(&self) -> Result<(), SaveError> {
        if !self.bunk_open_fraction.is_finite() || !(0.0..=1.0).contains(&self.bunk_open_fraction) {
            return Err(invalid("habitation bunk motion"));
        }
        Ok(())
    }
}
impl GameplaySession {
    fn bunk_clear(&self) -> bool {
        let snapshot = self.snapshot();
        let frame = super::frames::character_ship_frame(snapshot.ship.pose);
        // Entire swept volume, including mattress thickness and a small margin.
        let b = salimon_character::habitation_geometry::BUNK_SWEEP_BOUNDS;
        let eye = frame.world_to_local(snapshot.character.eye_position_meters);
        if snapshot.character.location == salimon_character::CharacterLocation::InsideShip
            && eye[0] + salimon_character::PLAYER_RADIUS_METERS > b[0]
            && eye[0] - salimon_character::PLAYER_RADIUS_METERS < b[1]
            && eye[2] + salimon_character::PLAYER_RADIUS_METERS > b[4]
            && eye[2] - salimon_character::PLAYER_RADIUS_METERS < b[5]
            && eye[1] - salimon_character::PLAYER_EYE_HEIGHT_METERS < b[3]
        {
            return false;
        }
        !self.mining.session.fragments().iter().any(|p| {
            salimon_character::cargo::sphere_overlaps_box(
                frame.world_to_local(p.transform().position().meters()),
                salimon_world::resource_size::fragment_contact_radius_meters(p.material()),
                b,
            )
        })
    }
    pub(super) fn toggle_bunk(&mut self) {
        if !self.bunk_clear() {
            self.feedback = Some("Bunk obstructed; clear belongings");
            return;
        }
        self.habitation.bunk_deploying = !self.habitation.bunk_deploying;
        self.feedback = Some(if self.habitation.bunk_deploying {
            "Unfolding bunk"
        } else {
            "Stowing bunk"
        });
    }
    pub(super) fn advance_habitation(&mut self, delta: Duration) {
        let target = if self.habitation.bunk_deploying {
            1.0
        } else {
            0.0
        };
        if self.habitation.bunk_open_fraction == target {
            return;
        }
        if !self.bunk_clear() {
            return;
        }
        let step = delta.as_secs_f64() / 0.8;
        let p = self.habitation.bunk_open_fraction;
        self.habitation.bunk_open_fraction = if self.habitation.bunk_deploying {
            (p + step).min(1.0)
        } else {
            (p - step).max(0.0)
        };
    }
    pub(super) fn use_galley(&mut self) {
        // Reuse exact existing inventory/effect transactions. No free food,
        // water reserve, duplicate fabrication or sleep-time acceleration.
        self.wheel_action(super::consumable_wheel::WheelAction::Hold(true));
        self.feedback = Some("Galley: choose carried food; prepare meals at Fabricator");
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::session::{
        Command,
        interaction::{InteractionTarget, available_interaction_target, habitation_position},
    };
    #[test]
    fn habitation_has_physical_aim_gates_and_never_grants_remote_actions() {
        for (target, eye) in [
            (InteractionTarget::Bunk, [-0.8, 1.9973, -2.2]),
            (InteractionTarget::Galley, [0.65, 1.9973, 2.2]),
            (InteractionTarget::Hygiene, [1.5, 1.9973, -2.2]),
        ] {
            let point = habitation_position(target);
            assert_eq!(
                available_interaction_target(
                    salimon_character::CharacterLocation::InsideShip,
                    eye,
                    point
                ),
                Some(target)
            );
            assert_ne!(
                available_interaction_target(
                    salimon_character::CharacterLocation::Surface,
                    eye,
                    point
                ),
                Some(target)
            );
            assert_ne!(
                available_interaction_target(
                    salimon_character::CharacterLocation::InsideShip,
                    [6., 2., 0.],
                    point
                ),
                Some(target)
            );
        }
    }
    #[test]
    fn bunk_is_manual_paused_exactly_restorable_and_does_not_skip_survival_time() {
        let mut s = GameplaySession::default();
        s.ship.set_stored_core_energy_joules(0);
        s.toggle_bunk();
        s.advance_automation(Duration::from_millis(320));
        assert!((s.habitation.bunk_open_fraction - 0.4).abs() < 1e-10);
        let before = s.save_bytes(100).unwrap();
        let mut r = GameplaySession::default();
        r.restore_bytes(&before).unwrap();
        assert_eq!(
            r.habitation.bunk_open_fraction,
            s.habitation.bunk_open_fraction
        );
        let elapsed = r.elapsed;
        r.command(Command::TogglePause);
        r.advance_automation(Duration::from_secs(10));
        assert_eq!(r.elapsed, elapsed);
        assert_eq!(
            r.habitation.bunk_open_fraction,
            s.habitation.bunk_open_fraction
        );
        r.command(Command::TogglePause);
        r.advance_automation(Duration::from_millis(480));
        assert_eq!(r.habitation.bunk_open_fraction, 1.0);
        assert_eq!(r.elapsed, elapsed + Duration::from_millis(480));
        r.toggle_bunk();
        r.advance_automation(Duration::from_millis(800));
        assert_eq!(r.habitation.bunk_open_fraction, 0.0);
    }
    #[test]
    fn galley_uses_existing_inventory_without_free_food_charge_or_needs() {
        let mut s = GameplaySession::new_run();
        let before: serde_json::Value =
            serde_json::from_slice(&s.save_bytes(100).unwrap()).unwrap();
        s.use_galley();
        assert!(s.wheel.open);
        let after: serde_json::Value = serde_json::from_slice(&s.save_bytes(100).unwrap()).unwrap();
        assert_eq!(
            before["state"]["fabrication"],
            after["state"]["fabrication"]
        );
        assert_eq!(before["state"]["survival"], after["state"]["survival"]);
        assert_eq!(
            before["state"]["physical"]["ship"],
            after["state"]["physical"]["ship"]
        );
    }
    #[test]
    fn blocked_bunk_does_not_teleport_or_delete_belongings() {
        let mut s = GameplaySession::default();
        let frame = super::super::frames::character_ship_frame(s.ship.snapshot().pose);
        let p = frame.local_to_world([-0.8, 0.8, -3.2]);
        let id = crate::sphere_fixtures::sphere(
            &mut s.mining.session,
            salimon_world::resources::ResourceId::IronOre,
            2.0,
            salimon_world::resources::ResourceTransform::new(
                salimon_world::WorldPosition::new(p[0], p[1], p[2]),
                [0., 0., 0., 1.],
            )
            .unwrap(),
        );
        let before = *s.mining.session.fragment(id).unwrap();
        s.toggle_bunk();
        assert!(!s.habitation.bunk_deploying);
        assert_eq!(s.mining.session.fragment(id), Some(&before));
        assert!(s.feedback.unwrap().contains("obstructed"));
    }
}

#[cfg(test)]
mod migration_tests {
    use super::*;
    #[test]
    fn schema_ten_requires_habitation_and_legacy_nine_stows_without_ticking() {
        let mut s = GameplaySession::default();
        let good = s.save_bytes(100).unwrap();
        let mut v: serde_json::Value = serde_json::from_slice(&good).unwrap();
        v["state"]["habitation"]["bunk_open_fraction"] = 1.1.into();
        assert!(s.restore_bytes(&serde_json::to_vec(&v).unwrap()).is_err());
        assert_eq!(s.save_bytes(100).unwrap(), good);
        v["state"].as_object_mut().unwrap().remove("habitation");
        assert!(s.restore_bytes(&serde_json::to_vec(&v).unwrap()).is_err());
        v["schema_version"] = 9.into();
        s.restore_bytes(&serde_json::to_vec(&v).unwrap()).unwrap();
        assert_eq!(s.habitation.bunk_open_fraction, 0.0);
        assert_eq!(s.elapsed, Duration::ZERO);
    }
}

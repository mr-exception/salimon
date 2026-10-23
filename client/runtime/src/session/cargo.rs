//! Internal cargo door composition; resource identity and motion remain world/physics owned.
use super::GameplaySession;
use salimon_character::cargo::{cargo_door_bounds, sphere_overlaps_box};
use salimon_persistence::{SaveError, Validate, invalid};
use std::time::Duration;

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct CargoDoor {
    pub(crate) open_fraction: f64,
    pub(crate) opening: bool,
}
impl Default for CargoDoor {
    fn default() -> Self {
        Self {
            open_fraction: 1.0,
            opening: true,
        }
    }
}
impl Validate for CargoDoor {
    fn validate(&self) -> Result<(), SaveError> {
        if !self.open_fraction.is_finite() || !(0.0..=1.0).contains(&self.open_fraction) {
            return Err(invalid("cargo door fraction"));
        }
        Ok(())
    }
}
impl GameplaySession {
    pub(super) fn cargo_sweep_clear(&self, from: f64, to: f64) -> bool {
        let a = cargo_door_bounds(from);
        let b = cargo_door_bounds(to);
        let bounds = [a[0], a[1], a[2], a[3], a[4].min(b[4]), a[5].max(b[5])];
        let snapshot = self.snapshot();
        let frame = super::frames::character_ship_frame(snapshot.ship.pose);
        let eye = frame.world_to_local(snapshot.character.eye_position_meters);
        if snapshot.character.location == salimon_character::CharacterLocation::InsideShip {
            let bottom = eye[1] - salimon_character::PLAYER_EYE_HEIGHT_METERS;
            if bottom < bounds[3]
                && bottom + salimon_character::PLAYER_BODY_HEIGHT_METERS > bounds[2]
                && eye[0] + salimon_character::PLAYER_RADIUS_METERS > bounds[0]
                && eye[0] - salimon_character::PLAYER_RADIUS_METERS < bounds[1]
                && eye[2] + salimon_character::PLAYER_RADIUS_METERS > bounds[4]
                && eye[2] - salimon_character::PLAYER_RADIUS_METERS < bounds[5]
            {
                return false;
            }
        }
        !self.mining.session.fragments().iter().any(|p| {
            sphere_overlaps_box(
                frame.world_to_local(p.transform().position().meters()),
                salimon_world::resource_size::fragment_contact_radius_meters(p.material()),
                bounds,
            )
        })
    }
    pub(super) fn toggle_cargo_door(&mut self) {
        let opening = !self.cargo_door.opening;
        let end = if opening { 1.0 } else { 0.0 };
        if !self.cargo_sweep_clear(self.cargo_door.open_fraction, end) {
            self.feedback = Some("Cargo door obstructed; clear player and belongings");
            return;
        }
        self.cargo_door.opening = opening;
        self.feedback = Some(if opening {
            "Opening cargo door"
        } else {
            "Closing cargo door"
        });
    }
    pub(super) fn advance_cargo_door(&mut self, delta: Duration) {
        let old = self.cargo_door.open_fraction;
        let step = delta.as_secs_f64() / 0.8;
        let next = if self.cargo_door.opening {
            (old + step).min(1.0)
        } else {
            (old - step).max(0.0)
        };
        if old != next && self.cargo_sweep_clear(old, next) {
            self.cargo_door.open_fraction = next;
        } else if old != next {
            self.feedback = Some("Cargo door obstructed; reverse or clear belongings");
        }
        self.sync_cargo_geometry();
    }
    pub(super) fn sync_cargo_geometry(&mut self) {
        self.character
            .set_cargo_door_blocking(self.cargo_door.open_fraction < 1.0);
        self.mining.cargo_door_blocking = self.cargo_door.open_fraction < 1.0;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use salimon_world::{
        WorldPosition,
        resources::{ResourceId, ResourceTransform},
    };
    fn insert(s: &mut GameplaySession, p: [f64; 3]) -> salimon_world::resources::FragmentId {
        let frame = super::super::frames::character_ship_frame(s.ship.snapshot().pose);
        let p = frame.local_to_world(p);
        crate::sphere_fixtures::sphere(
            &mut s.mining.session,
            ResourceId::IronOre,
            2.0,
            ResourceTransform::new(WorldPosition::new(p[0], p[1], p[2]), [0., 0., 0., 1.]).unwrap(),
        )
    }
    #[test]
    fn cargo_door_checks_loose_and_carried_objects_and_restores_partial_motion() {
        let mut s = GameplaySession::default();
        let ramp = s.ship.snapshot().door_state;
        let id = insert(&mut s, [-4.80, 1.4, 0.0]);
        s.toggle_cargo_door();
        assert!(s.cargo_door.opening);
        assert!(s.feedback.unwrap().contains("obstructed"));
        assert!(s.mining.session.pick_up(id));
        s.toggle_cargo_door();
        assert!(
            s.cargo_door.opening,
            "a held object is not excluded from the sweep"
        );
        let frame = super::super::frames::character_ship_frame(s.ship.snapshot().pose);
        let p = frame.local_to_world([-3.0, 1.0, 0.0]);
        s.mining.session.move_carried(
            ResourceTransform::new(WorldPosition::new(p[0], p[1], p[2]), [0., 0., 0., 1.]).unwrap(),
        );
        s.toggle_cargo_door();
        assert!(!s.cargo_door.opening);
        s.advance_cargo_door(Duration::from_millis(320));
        assert!((s.cargo_door.open_fraction - 0.6).abs() < 1e-10);
        let bytes = s.save_bytes(100).unwrap();
        let mut r = GameplaySession::default();
        r.restore_bytes(&bytes).unwrap();
        assert_eq!(r.cargo_door.open_fraction, s.cargo_door.open_fraction);
        assert!(r.mining.cargo_door_blocking);
        assert_eq!(r.mining.session.carried_id(), Some(id));
        r.ship.set_stored_core_energy_joules(0);
        r.advance_cargo_door(Duration::from_secs(1));
        assert_eq!(
            r.cargo_door.open_fraction, 0.0,
            "mechanical door remains usable in blackout"
        );
        assert_eq!(r.ship.snapshot().door_state, ramp);
    }
    #[test]
    fn schema_nine_requires_door_and_schema_eight_migrates_open() {
        let mut s = GameplaySession::default();
        let good = s.save_bytes(100).unwrap();
        let mut v: serde_json::Value = serde_json::from_slice(&good).unwrap();
        v["state"].as_object_mut().unwrap().remove("cargo_door");
        assert!(s.restore_bytes(&serde_json::to_vec(&v).unwrap()).is_err());
        v["schema_version"] = 8.into();
        s.restore_bytes(&serde_json::to_vec(&v).unwrap()).unwrap();
        assert_eq!(s.cargo_door.open_fraction, 1.0);
    }
    #[test]
    fn physical_bin_retains_identity_mass_and_can_be_retrieved_after_restore() {
        let mut s = GameplaySession::default();
        let id = insert(&mut s, [-6.1, 1.5, 3.45]);
        s.mining.ship_fragments.insert(
            id,
            salimon_character::ShipLocalPoint::new([-6.1, 1.5, 3.45]),
        );
        let frame = super::super::frames::character_ship_frame(s.ship.snapshot().pose);
        crate::carrying::sync_ship_mass(&mut s.mining, frame, &mut s.ship);
        let mass = s.ship.snapshot().total_mass_kg;
        for _ in 0..200 {
            s.advance_automation(Duration::from_millis(16));
        }
        let local = frame.world_to_local(
            s.mining
                .session
                .fragment(id)
                .unwrap()
                .transform()
                .position()
                .meters(),
        );
        assert!(
            (local[0] + 6.1).abs() < 0.01 && (local[2] - 3.45).abs() < 0.01,
            "{local:?}"
        );
        let radius =
            crate::fragment_physics::sphere_radius(*s.mining.session.fragment(id).unwrap());
        assert!(
            (local[1] - (salimon_character::SHIP_FLOOR_HEIGHT_METERS + 0.12 + radius)).abs() < 0.01,
            "{local:?}"
        );
        assert_eq!(s.ship.snapshot().total_mass_kg, mass);
        let bytes = s.save_bytes(100).unwrap();
        let mut r = GameplaySession::default();
        r.restore_bytes(&bytes).unwrap();
        assert_eq!(r.mining.session.fragment(id), s.mining.session.fragment(id));
        assert!(r.mining.session.pick_up(id));
        assert_eq!(r.mining.session.carried_id(), Some(id));
        assert_eq!(r.mining.session.fragments().len(), 1);
    }
}

#[cfg(test)]
mod route_tests {
    #[test]
    fn cargo_handle_uses_explicit_aim_without_stealing_exit_ramp_interactions() {
        use super::super::interaction::{InteractionTarget, available_interaction_target};
        use salimon_character::CharacterLocation;
        let eye = [-3.452, 1.9973118279569892, -0.0112];
        assert_eq!(
            available_interaction_target(
                CharacterLocation::InsideShip,
                eye,
                salimon_character::cargo::CARGO_DOOR_TARGET
            ),
            Some(InteractionTarget::CargoDoor)
        );
        let aft = [-5.8, 1.9973118279569892, 0.0];
        assert_eq!(
            available_interaction_target(CharacterLocation::InsideShip, aft, [-4.8, 1.9973, 0.0]),
            Some(InteractionTarget::ExitDoor)
        );
        assert_ne!(
            available_interaction_target(
                CharacterLocation::Surface,
                eye,
                salimon_character::cargo::CARGO_DOOR_TARGET
            ),
            Some(InteractionTarget::CargoDoor)
        );
    }
}

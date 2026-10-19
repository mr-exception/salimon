//! Physical intake adapter. All validation precedes the infallible single-owner commit.
use super::GameplaySession;
use salimon_fabrication::FabricationError;

impl GameplaySession {
    pub(super) fn insert_fabricator_fragment(&mut self) {
        let Some(id) = self.mining.session.carried_id() else {
            self.feedback = Some("Carry a fragment to feed the Fabricator");
            return;
        };
        let material = self
            .mining
            .session
            .fragment(id)
            .expect("carried identity belongs to the world session")
            .material();
        match self.fabricator.insert(material) {
            Ok(_) => {
                // Both domains are exclusively borrowed by this command; no mutation
                // can invalidate the previously checked world identity between commits.
                assert!(self.mining.session.remove_fragment(id));
                self.equipment.sync_carrying(false);
                self.sync_carried();
                self.feedback = Some("Fabricator accepted fragment");
            }
            Err(FabricationError::StorageFull(_)) => {
                self.feedback = Some("Fabricator reserve is full");
            }
            Err(_) => self.feedback = Some("Invalid Fabricator fragment mass"),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::session::interaction::{
        FABRICATOR_INPUT_METERS, InteractionTarget, available_interaction_target,
    };
    use salimon_character::CharacterLocation;
    use salimon_world::{WorldPosition, resources::ResourceTransform};

    #[test]
    fn intake_consumes_identity_once_clears_physics_and_preserves_full_bin_input() {
        for resource in salimon_world::resources::RESOURCE_CATALOG {
            let mut session = GameplaySession::default();
            let frame = crate::session::frames::character_ship_frame(session.ship.snapshot().pose);
            let mut inputs = Vec::new();
            for kg in [0.123_456_7, 10.0, 10.0, 1.0] {
                let p = frame.local_to_world([-3.0, 1.5, 2.0]);
                let id = crate::sphere_fixtures::sphere(
                    &mut session.mining.session,
                    resource.id,
                    kg,
                    ResourceTransform::new(
                        WorldPosition::new(p[0], p[1], p[2]),
                        [0.0, 0.0, 0.0, 1.0],
                    )
                    .unwrap(),
                );
                inputs.push((id, kg));
            }
            for (id, kg) in inputs {
                assert!(session.mining.session.pick_up(id));
                session.sync_carried();
                let before = session.fabricator.mass_contribution();
                session.insert_fabricator_fragment();
                if kg == 1.0 {
                    assert_eq!(session.mining.session.carried_id(), Some(id));
                    assert_eq!(session.fabricator.mass_contribution(), before);
                    assert_eq!(session.feedback, Some("Fabricator reserve is full"));
                } else {
                    assert!(session.mining.session.fragment(id).is_none());
                    assert!(!session.mining.ship_fragments.contains_key(&id));
                    assert!(!session.mining.fragment_motion.contains_key(&id));
                    assert!(!session.mining.fragment_previous.contains_key(&id));
                    let stored = session.fabricator.clone();
                    session.insert_fabricator_fragment();
                    assert_eq!(session.fabricator, stored);
                }
            }
            assert_eq!(
                session.fabricator.mass_contribution().stored_milligrams,
                20_000_000
            );
        }
    }

    #[test]
    fn input_gate_checks_interior_range_aim_backface_and_occlusion() {
        let port = FABRICATOR_INPUT_METERS;
        let eye = [-3.0, 1.65, 2.0];
        assert_eq!(
            available_interaction_target(CharacterLocation::InsideShip, eye, port),
            Some(InteractionTarget::Fabricator)
        );
        for location in [
            CharacterLocation::Space,
            CharacterLocation::Surface,
            CharacterLocation::DoorwayBlend,
        ] {
            assert_ne!(
                available_interaction_target(location, eye, port),
                Some(InteractionTarget::Fabricator)
            );
        }
        for (eye, look) in [
            ([-3.0, 1.65, 0.0], port),
            ([-3.0, 1.65, 4.0], port),
            (eye, [-3.0, 1.65, -2.0]),
            ([0.0, 1.65, 1.0], port),
        ] {
            assert_ne!(
                available_interaction_target(CharacterLocation::InsideShip, eye, look),
                Some(InteractionTarget::Fabricator)
            );
        }
        // Translation/rotation stays outside the gate; targeting uses current local coordinates.
        let frame = salimon_character::ShipFrame {
            origin_meters: [1e10, 20.0, 30.0],
            axes: [[0.0, 0.0, -1.0], [0.0, 1.0, 0.0], [1.0, 0.0, 0.0]],
        };
        assert_eq!(
            available_interaction_target(
                CharacterLocation::InsideShip,
                frame.world_to_local(frame.local_to_world(eye)),
                frame.world_to_local(frame.local_to_world(port))
            ),
            Some(InteractionTarget::Fabricator)
        );
    }
}

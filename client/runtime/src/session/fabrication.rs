//! Physical intake adapter. All validation precedes the infallible single-owner commit.
use super::GameplaySession;
use salimon_fabrication::FabricationError;

impl GameplaySession {
    /// One shared tick; recipe completion ends its draw before the rest of the tick.
    pub(super) fn advance_fabrication(&mut self, delta: std::time::Duration) {
        let demand = self.fabricator.power_demand();
        let active = demand.map_or(std::time::Duration::ZERO, |d| d.until_output.min(delta));
        self.ship
            .set_fabricator_watts(demand.map_or(0.0, |d| d.watts))
            .expect("validated recipe and starter ship watts have finite sum");
        if !active.is_zero() {
            self.ship.advance(active);
            let funded = self.ship.fabricator_powered_duration(active);
            self.fabricator.step(funded, true);
            self.fabricator.step(active - funded, false);
        }
        self.ship
            .set_fabricator_watts(self.fabricator.power_demand().map_or(0.0, |d| d.watts))
            .expect("validated recipe watts");
        if active < delta {
            self.ship.advance(delta - active);
        }
    }

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

#[cfg(test)]
mod power_tests {
    use super::*;
    use salimon_fabrication::{Fabricator, RecipeBook, RecipeId};
    use salimon_world::resources::{RESOURCE_CATALOG, RawMaterial};
    use std::time::Duration;

    pub(super) fn queued() -> GameplaySession {
        let mut session = GameplaySession::default();
        let mut recipe = RecipeBook::starter().recipes()[0].clone();
        recipe.duration = Duration::from_millis(100);
        recipe.watts = 10_000.0;
        session.fabricator = Fabricator::new(RecipeBook::new(vec![recipe]).unwrap(), 8).unwrap();
        for resource in RESOURCE_CATALOG {
            session
                .fabricator
                .insert(RawMaterial::new(resource.id, 1.0).unwrap())
                .unwrap();
        }
        session
            .fabricator
            .enqueue(&RecipeId::new("food-ration").unwrap(), 2)
            .unwrap();
        session
    }

    #[test]
    fn core_cutoff_pauses_exactly_and_restores_without_banking_or_double_charge() {
        let mut s = queued();
        s.ship.set_stored_core_energy_joules(960); // 8 ms at 110 kW idle + 10 kW recipe.
        s.advance_game(Duration::from_millis(16));
        assert_eq!(
            s.fabricator.snapshot(false).jobs[0].progress,
            Duration::from_millis(8)
        );
        assert!(!s.ship.snapshot().power_available);
        let paused = s.fabricator.clone();
        for _ in 0..10 {
            s.advance_game(Duration::from_millis(16));
        }
        assert_eq!(s.fabricator, paused);
        s.ship
            .exchange_core_energy_joules(1_000_000.0, 0.0)
            .unwrap();
        s.advance_game(Duration::from_millis(16));
        assert_eq!(
            s.fabricator.snapshot(true).jobs[0].progress,
            Duration::from_millis(24)
        );
        assert_eq!(s.ship.snapshot().energy_core.stored_joules, 998_080);
        assert_eq!(s.ship.snapshot().core_status.fabricator_watts, 10_000.0);
    }

    #[test]
    fn completion_stops_draw_mid_tick_and_occupied_tray_only_pays_idle() {
        let mut s = queued();
        s.ship.set_stored_core_energy_joules(1_000_000);
        for _ in 0..7 {
            s.advance_game(Duration::from_millis(16));
        }
        assert!(s.fabricator.snapshot(true).tray.is_some());
        assert_eq!(s.ship.snapshot().energy_core.stored_joules, 986_680); // 112 ms idle + 100 ms craft.
        assert_eq!(s.ship.snapshot().core_status.fabricator_watts, 0.0);
        s.advance_game(Duration::from_secs(1));
        assert_eq!(s.ship.snapshot().energy_core.stored_joules, 876_680);
    }

    #[test]
    fn frame_stalls_and_partitioned_admission_fund_the_same_fixed_ticks_away_from_machine() {
        let mut whole = queued();
        let mut split = queued();
        whole.advance_automation(Duration::from_millis(250));
        for ms in [1, 33, 100, 116] {
            split.advance_automation(Duration::from_millis(ms));
        }
        assert_eq!(whole.fabricator, split.fabricator);
        assert_eq!(whole.ship.snapshot(), split.ship.snapshot());
        assert_eq!(whole.update_clock.backlog(), split.update_clock.backlog());
    }
}

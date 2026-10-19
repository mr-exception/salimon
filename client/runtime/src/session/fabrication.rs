//! Physical intake adapter. All validation precedes the infallible single-owner commit.
use super::GameplaySession;
use salimon_fabrication::FabricationError;

impl GameplaySession {
    pub(crate) fn fabricator_output_present(&self) -> bool {
        self.fabricator.mass_contribution().tray_milligrams > 0
    }

    /// Portable command seam for later machine UI/inventory adapters. Collection
    /// callers must validate a receiving slot before committing this command.
    pub(crate) fn execute_fabricator(
        &mut self,
        command: salimon_fabrication::FabricationCommand,
    ) -> Result<salimon_fabrication::CommandResult, FabricationError> {
        let result = self.fabricator.execute(command)?;
        self.sync_fabricator_mass();
        self.ship
            .set_fabricator_watts(self.fabricator.power_demand().map_or(0.0, |d| d.watts))
            .expect("validated recipe watts");
        Ok(result)
    }

    pub(super) fn sync_fabricator_mass(&mut self) {
        let mass = self.fabricator.mass_contribution();
        let mut changed = false;
        for (id, mg) in [
            (
                salimon_ship::MassObjectId::FabricatorReserve,
                mass.stored_milligrams,
            ),
            (
                salimon_ship::MassObjectId::FabricatorTray,
                mass.tray_milligrams,
            ),
        ] {
            changed |= self
                .mining
                .ship_mass
                .update(id, (mg > 0).then_some(mg as f64 / 1_000_000.0))
                .expect("bounded stored quantities and validated output mass");
        }
        if changed || self.ship.snapshot().total_mass_kg != self.mining.ship_mass.total_kg() {
            self.ship
                .set_total_mass_kg(self.mining.ship_mass.total_kg())
                .expect("finite Fabricator and physical cargo ledger");
        }
    }

    /// One shared tick; recipe completion ends its draw before the rest of the tick.
    pub(super) fn advance_fabrication(&mut self, delta: std::time::Duration) {
        let mut remaining = delta;
        while !remaining.is_zero() {
            let Some(demand) = self.fabricator.power_demand() else {
                self.ship
                    .set_fabricator_watts(0.0)
                    .expect("zero load is valid");
                self.ship.advance(remaining);
                break;
            };
            let active = demand.until_output.min(remaining);
            self.ship
                .set_fabricator_watts(demand.watts)
                .expect("validated recipe watts");
            self.ship.advance(active);
            let funded = self.ship.fabricator_powered_duration(active);
            self.fabricator.step(funded, true);
            self.fabricator.step(active - funded, false);
            self.sync_fabricator_mass();
            remaining -= active;
            if funded.is_zero() && !remaining.is_zero() {
                // Insufficient allocated power: pause until the next shared tick,
                // allowing ordinary loads/conversion to settle without charging work.
                self.ship
                    .set_fabricator_watts(0.0)
                    .expect("zero load is valid");
                self.ship.advance(remaining);
                break;
            }
        }
        self.ship
            .set_fabricator_watts(self.fabricator.power_demand().map_or(0.0, |d| d.watts))
            .expect("validated recipe watts");
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
        match self.execute_fabricator(salimon_fabrication::FabricationCommand::Insert(material)) {
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

impl GameplaySession {
    pub(crate) fn fabricator_output_medical(&self) -> bool {
        self.fabricator
            .snapshot(self.ship.snapshot().power_available)
            .tray
            .is_some_and(|item| item.category == salimon_fabrication::OutputCategory::Medical)
    }

    pub(super) fn collect_fabricator_output(&mut self) {
        let Some(item) = self
            .fabricator
            .snapshot(self.ship.snapshot().power_available)
            .tray
        else {
            self.feedback = Some("Fabricator tray is empty");
            return;
        };
        match self.consumables.insert(item) {
            Ok(_) => {
                // Exclusive session ownership: tray cannot change after slot acceptance.
                self.execute_fabricator(salimon_fabrication::FabricationCommand::Collect)
                    .expect("validated occupied tray in single-owner collection transaction");
                self.feedback = Some("Fabricator item collected");
            }
            Err(salimon_fabrication::InventoryError::Full) => {
                self.feedback = Some("Matching consumable slots are full; item stays on tray")
            }
            Err(_) => self.feedback = Some("Cannot collect this output category"),
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
                let loose: f64 = session
                    .mining
                    .session
                    .fragments()
                    .iter()
                    .map(|f| f.material().mass_kg())
                    .sum();
                assert_eq!(
                    session.ship.snapshot().total_mass_kg,
                    salimon_ship::STARTER_DRY_MASS_KG
                        + loose
                        + session.fabricator.mass_contribution().kg()
                );
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

    fn queued() -> GameplaySession {
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
    fn fabrication_runs_in_inertial_flight_takeoff_and_after_mechanical_core_restart() {
        let mut flight = queued();
        flight.ship = salimon_ship::ShipController::flying(
            salimon_ship::ShipPose {
                position_meters: [1e12, 0.0, 0.0],
                orientation: [0.0, 0.0, 0.0, 1.0],
            },
            50,
        );
        flight.ship.set_cockpit_control(false);
        flight.advance_game(Duration::from_millis(16));
        assert_eq!(
            flight.fabricator.snapshot(true).jobs[0].progress,
            Duration::from_millis(16)
        );
        assert_eq!(
            flight.ship.snapshot().core_status.fabricator_watts,
            10_000.0
        );
        let mut takeoff = queued();
        takeoff.ship.set_cockpit_control(true);
        takeoff.ship.trigger_landing_action();
        assert!(matches!(
            takeoff.ship.snapshot().flight_state,
            salimon_ship::FlightState::AssistedTakeoff { .. }
        ));
        takeoff.ship.set_cockpit_control(false);
        takeoff.advance_game(Duration::from_millis(16));
        assert_eq!(
            takeoff.fabricator.snapshot(true).jobs[0].progress,
            Duration::from_millis(16)
        );
        let mut restart = queued();
        restart.ship.set_stored_core_energy_joules(0);
        let id = crate::sphere_fixtures::sphere(
            &mut restart.mining.session,
            salimon_world::resources::ResourceId::IronOre,
            1.0,
            salimon_world::resources::ResourceTransform::new(
                salimon_world::WorldPosition::new(0.0, 0.0, 0.0),
                [0.0, 0.0, 0.0, 1.0],
            )
            .unwrap(),
        );
        restart.mining.session.pick_up(id);
        assert_eq!(
            restart
                .ship
                .intake_carried_fragment(&mut restart.mining.session),
            salimon_ship::IntakeResult::Accepted
        );
        restart.advance_game(Duration::from_millis(16));
        assert_eq!(
            restart.fabricator.snapshot(true).jobs[0].progress,
            Duration::from_millis(16)
        );
        assert!(restart.ship.snapshot().power_available);
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

#[cfg(test)]
mod mass_tests {
    use super::*;
    use salimon_fabrication::{CommandResult, FabricationCommand as C, RecipeId};
    use salimon_world::resources::{RawMaterial, ResourceId};
    use std::time::Duration;
    fn mass(s: &GameplaySession) -> f64 {
        s.ship.snapshot().total_mass_kg
    }
    #[test]
    fn command_and_tick_boundaries_update_mass_with_deliberate_queue_inventory_exclusions() {
        let mut s = GameplaySession::default();
        let dry = mass(&s);
        for (resource, kg) in [
            (ResourceId::OrganicBiomass, 0.800_000_4),
            (ResourceId::WaterIce, 0.200_000_4),
        ] {
            s.execute_fabricator(C::Insert(RawMaterial::new(resource, kg).unwrap()))
                .unwrap();
        }
        assert_eq!(mass(&s), dry + 1.0);
        let CommandResult::Enqueued(job) = s
            .execute_fabricator(C::Enqueue {
                recipe: RecipeId::new("food-ration").unwrap(),
                quantity: 2,
            })
            .unwrap()
        else {
            panic!()
        };
        assert_eq!(mass(&s), dry); // Committed ingredients intentionally disappear from ship mass.
        for _ in 0..625 {
            s.advance_fabrication(Duration::from_millis(16));
        }
        assert_eq!(mass(&s), dry + 0.3);
        assert!(s.fabricator_output_present());
        s.execute_fabricator(C::Cancel(job)).unwrap();
        assert_eq!(mass(&s), dry + 0.3); // Cancellation preserves completed tray; never refunds.
        s.execute_fabricator(C::Collect).unwrap();
        assert!(!s.fabricator_output_present());
        assert_eq!(mass(&s), dry); // Collected consumable inventory intentionally contributes zero.
        assert!(s.execute_fabricator(C::Collect).is_err());
        assert_eq!(mass(&s), dry);
        assert!(s.mining.ship_mass.reconciles([]));
    }
    #[test]
    fn reserves_follow_ship_motion_and_replacements_keep_loose_core_and_equipment_mass() {
        let mut s = GameplaySession::default();
        let id = salimon_ship::MassObjectId::Equipment(5);
        s.mining.ship_mass.update(id, Some(2.0)).unwrap();
        s.execute_fabricator(C::Insert(
            RawMaterial::new(ResourceId::IronOre, 19.999_999).unwrap(),
        ))
        .unwrap();
        s.execute_fabricator(C::Insert(
            RawMaterial::new(ResourceId::IronOre, 0.000_003).unwrap(),
        ))
        .unwrap();
        assert_eq!(mass(&s), salimon_ship::STARTER_DRY_MASS_KG + 22.0);
        let before = mass(&s);
        let pose = s.ship.snapshot().pose;
        s.ship =
            salimon_ship::ShipController::flying_with_velocity(pose, 0, [20.0, 0.0, 0.0]).unwrap();
        s.sync_carried();
        assert_eq!(mass(&s), before);
        for _ in 0..10 {
            s.advance_game(Duration::from_millis(16));
        }
        assert_eq!(mass(&s), before);
        assert!(
            s.execute_fabricator(C::Insert(
                RawMaterial::new(ResourceId::IronOre, 1.0).unwrap()
            ))
            .is_err()
        );
        assert_eq!(mass(&s), before);
    }
}

#[cfg(test)]
mod tray_tests {
    use super::*;
    use salimon_fabrication::{FabricationCommand as C, OutputCategory, RecipeId};
    use salimon_world::resources::{RESOURCE_CATALOG, RawMaterial};
    use std::time::Duration;

    fn loaded() -> GameplaySession {
        let mut s = GameplaySession::default();
        for r in RESOURCE_CATALOG {
            s.execute_fabricator(C::Insert(RawMaterial::new(r.id, 5.0).unwrap()))
                .unwrap();
        }
        s
    }
    #[test]
    fn anchored_output_blocks_queue_survives_blackout_and_collects_once_into_matching_slots() {
        for (recipe, seconds, capacity, category) in [
            ("food-ration", 10, 3, OutputCategory::Food),
            ("medkit", 15, 2, OutputCategory::Medical),
        ] {
            let mut s = loaded();
            let fragments = s.mining.session.fragments().len();
            let held = s.mining.session.carried_id();
            s.execute_fabricator(C::Enqueue {
                recipe: RecipeId::new(recipe).unwrap(),
                quantity: capacity + 1,
            })
            .unwrap();
            for index in 0..=capacity {
                s.advance_fabrication(Duration::from_secs(seconds));
                let output = s.fabricator.snapshot(true).tray.unwrap();
                assert_eq!(output.category, category);
                assert_eq!(output.unit, index + 1);
                assert_eq!(
                    s.fabricator_output_medical(),
                    category == OutputCategory::Medical
                );
                let blocked = s.fabricator.clone();
                s.ship.set_stored_core_energy_joules(0);
                s.advance_fabrication(Duration::from_secs(100));
                assert_eq!(s.fabricator, blocked);
                assert_eq!(s.ship.snapshot().core_status.fabricator_watts, 0.0);
                let before = s.ship.snapshot().total_mass_kg;
                s.collect_fabricator_output();
                if index == capacity {
                    assert_eq!(s.fabricator, blocked);
                    assert_eq!(s.ship.snapshot().total_mass_kg, before);
                    assert!(s.feedback.unwrap().contains("full"));
                } else {
                    assert!(!s.fabricator_output_present());
                    assert_eq!(s.ship.snapshot().total_mass_kg, before - output.mass.kg());
                    let inventory = s.consumables.clone();
                    s.collect_fabricator_output();
                    assert_eq!(s.consumables, inventory);
                    assert!(!s.fabricator_output_present());
                }
                s.ship.set_stored_core_energy_joules(10_000_000);
            }
            assert_eq!(s.mining.session.fragments().len(), fragments);
            assert_eq!(s.mining.session.carried_id(), held);
            assert_eq!(
                s.consumables.food().iter().flatten().count(),
                if category == OutputCategory::Food {
                    3
                } else {
                    0
                }
            );
            assert_eq!(
                s.consumables.medical().iter().flatten().count(),
                if category == OutputCategory::Medical {
                    2
                } else {
                    0
                }
            );
        }
    }
    #[test]
    fn tray_prompt_tracks_ship_pose_and_obeys_front_range_interior_and_cabin_solids() {
        use crate::session::interaction::{InteractionTarget, available_interaction_target};
        let target = salimon_character::FABRICATOR_TRAY_MARKER_METERS;
        let eye = [-2.55, 1.65, 2.0];
        let frame = salimon_character::ShipFrame {
            origin_meters: [1e12, 30.0, 1e10],
            axes: [[0.0, 0.0, -1.0], [0.0, 1.0, 0.0], [1.0, 0.0, 0.0]],
        };
        assert_eq!(
            available_interaction_target(
                salimon_character::CharacterLocation::InsideShip,
                frame.world_to_local(frame.local_to_world(eye)),
                frame.world_to_local(frame.local_to_world(target))
            ),
            Some(InteractionTarget::FabricatorTray)
        );
        for invalid in [[-2.55, 1.65, 0.0], [-2.55, 1.65, 4.5], [1.0, 1.65, 1.0]] {
            assert_ne!(
                available_interaction_target(
                    salimon_character::CharacterLocation::InsideShip,
                    invalid,
                    target
                ),
                Some(InteractionTarget::FabricatorTray)
            );
        }
        assert_eq!(
            available_interaction_target(salimon_character::CharacterLocation::Space, eye, target),
            None
        );
    }
}

impl GameplaySession {
    pub(crate) fn equipment_presentation(&self) -> salimon_renderer::EquipmentToolbar {
        use salimon_renderer::ConsumableIcon;
        let mut hud = self.equipment.presentation();
        for (target, item) in hud.consumables.iter_mut().zip(
            self.consumables
                .food()
                .iter()
                .chain(self.consumables.medical()),
        ) {
            *target = item.as_ref().map(|item| match item.category {
                salimon_fabrication::OutputCategory::Medical => ConsumableIcon::Medical,
                _ => ConsumableIcon::Food,
            });
        }
        hud
    }
}

impl GameplaySession {
    pub(crate) fn fabrication_snapshot(&self) -> salimon_fabrication::FabricatorSnapshot {
        self.fabricator
            .snapshot(self.ship.snapshot().power_available)
    }
}

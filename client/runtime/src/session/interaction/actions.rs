use super::InteractionTarget;
use crate::session::{
    GameplaySession, ShipControlInput, cabin, frames::character_ship_frame, tutorial,
};
use salimon_character::CharacterLocation;
use salimon_ship::CockpitMessage;

impl GameplaySession {
    pub(crate) fn interact(&mut self) {
        if self.paused || self.survival.snapshot().terminal {
            return;
        }
        self.enforce_survival_restrictions();
        if self.building.active {
            self.contribute_construction();
            return;
        }
        let ship_snapshot = self.ship.snapshot();
        let ship_frame = character_ship_frame(ship_snapshot.pose);
        let surface = self.terrain_frame(ship_snapshot);
        match self.character.location() {
            CharacterLocation::Cockpit => {
                self.character.leave_cockpit();
                self.ship.set_cockpit_control(false);
                self.input.steering = ShipControlInput::default();
                self.ship.clear_throttle_input();
                log::info!("left cockpit control; ship motion remains autonomous");
            }
            CharacterLocation::InsideShip
            | CharacterLocation::DoorwayBlend
            | CharacterLocation::Surface
            | CharacterLocation::Space
            | CharacterLocation::NearbyBody => {
                let character = self.character.snapshot(ship_frame, surface);
                let local_eye = ship_frame.world_to_local(character.eye_position_meters);
                let local_look = ship_frame.world_to_local(character.look_target_meters);
                let interaction =
                    self.available_target(self.character.location(), local_eye, local_look);
                if let Some(target) = interaction
                    && !self.admit_survival_action(Self::interaction_action(target))
                {
                    return;
                }
                if self
                    .habitation
                    .cabin
                    .as_ref()
                    .is_some_and(|c| c.rest.is_some())
                    && !matches!(
                        interaction,
                        Some(
                            InteractionTarget::PrivacyScreen
                                | InteractionTarget::ObservationShutter
                        )
                    )
                {
                    self.stand_from_furniture();
                    return;
                }
                match interaction {
                    Some(InteractionTarget::CabinTable) => {
                        self.toggle_cabin_fixture(cabin::Furniture::Table)
                    }
                    Some(InteractionTarget::CabinSeat) => {
                        self.toggle_cabin_fixture(cabin::Furniture::Seat)
                    }
                    Some(InteractionTarget::PrivacyScreen) => {
                        self.toggle_cabin_fixture(cabin::Furniture::Screen)
                    }
                    Some(InteractionTarget::ObservationShutter) => {
                        self.toggle_cabin_fixture(cabin::Furniture::Shutter)
                    }
                    Some(InteractionTarget::CabinLocker) => {
                        self.toggle_cabin_fixture(cabin::Furniture::Locker)
                    }
                    Some(InteractionTarget::Bunk) => self.toggle_bunk(),
                    Some(InteractionTarget::Galley) => self.use_galley(),
                    Some(InteractionTarget::Hygiene) => {
                        self.feedback = Some("Compact wash station; no water or hygiene needs")
                    }
                    Some(InteractionTarget::CargoDoor) => self.toggle_cargo_door(),
                    Some(InteractionTarget::Archive) => self.open_archive(),
                    Some(InteractionTarget::FabricatorTray) => self.collect_fabricator_output(),
                    Some(InteractionTarget::FabricatorScreen) => {
                        self.clear_held_controls();
                        self.crafting_ui.open = true;
                        self.tutorial.record(tutorial::Objective::Fabricator);
                    }
                    Some(InteractionTarget::Fabricator) => {
                        self.insert_fabricator_fragment();
                    }
                    Some(InteractionTarget::Core) => {
                        if self.mining.carry_blocked {
                            self.feedback =
                                Some("Carried object obstructed - back up or drop before intake");
                            return;
                        }
                        if self.core_upgrades.mode
                            == salimon_ship::core_upgrades::InputMode::Upgrade
                        {
                            self.contribute_core_upgrade();
                            return;
                        }
                        let result = self.ship.intake_carried_fragment(&mut self.mining.session);
                        let text = match result {
                            salimon_ship::IntakeResult::Accepted => "Core FUEL accepted fragment",
                            salimon_ship::IntakeResult::Busy => "Core is processing a fragment",
                            salimon_ship::IntakeResult::NoCarriedFragment => {
                                "CORE FUEL - carry fragment to burn; I selects upgrades"
                            }
                            salimon_ship::IntakeResult::InvalidYield => {
                                "Invalid fragment energy yield"
                            }
                        };
                        self.feedback = Some(text);
                        crate::carrying::sync_ship_mass(
                            &mut self.mining,
                            ship_frame,
                            &mut self.ship,
                        );
                    }
                    Some(InteractionTarget::Cockpit) => {
                        self.character.enter_cockpit();
                        if self.character.location() == CharacterLocation::Cockpit {
                            self.ship.set_cockpit_control(true);
                            log::info!("entered cockpit control");
                        }
                    }
                    Some(InteractionTarget::ExitDoor) => {
                        let clear = self.character.location() != CharacterLocation::DoorwayBlend
                            && !salimon_character::boarding_ramp_occupied(local_eye)
                            && self.modular_gate_clear();
                        self.ship.toggle_door_with_clearance(clear);
                        if let Some(message) = self.ship.snapshot().cockpit_message {
                            if matches!(
                                message,
                                CockpitMessage::DoorLockedWhileInFlight
                                    | CockpitMessage::DoorObstructed
                            ) {
                                self.feedback = Some(message.text());
                            }
                            log::info!("cockpit monitor: {}", message.text());
                        } else {
                            log::info!("exit door is now {:?}", self.ship.snapshot().door_state);
                        }
                    }
                    None => {}
                }
            }
        }
    }
}

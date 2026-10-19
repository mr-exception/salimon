//! Portable production gameplay composition. No window, GPU lifecycle or evidence IO.
pub(crate) mod consumable_wheel;
pub(crate) mod crafting_ui;
mod fabrication;
pub(crate) mod frames;
pub(crate) mod interaction;
mod presentation;
// Portable adapters are consumed by whole-session orchestration in #365/#366.
#[allow(dead_code)]
pub(crate) mod persistence;
#[cfg(test)]
mod tests;
use self::frames::character_ship_frame;
use self::interaction::{InteractionTarget, available_interaction_target};
use crate::update_clock::UpdateClock;
use salimon_character::{CharacterController, CharacterLocation, MovementInput};
use salimon_ship::SteeringInput;
use salimon_ship::{CockpitMessage, DoorState, FlightState, ShipController, ShipSnapshot};
use salimon_world::{CameraPrototype, TransitionPhase};
use std::time::Duration;

pub(crate) struct GameplaySession {
    pub(crate) presentation: presentation::History,
    pub(crate) update_clock: UpdateClock,
    pub(crate) camera_prototype: CameraPrototype,
    pub(crate) character: CharacterController,
    pub(crate) ship: ShipController,
    pub(crate) automation_aim_delta: Option<[f64; 2]>,
    pub(crate) mining: crate::mining::ResourceGameplay,
    fabricator: salimon_fabrication::Fabricator,
    pub(crate) wheel: consumable_wheel::WheelState,
    pub(crate) consumables: salimon_fabrication::ConsumableInventory,
    pub(crate) crafting_ui: crafting_ui::CraftingUi,
    pub(crate) equipment: crate::equipment::EquipmentToolbar,
    pub(crate) movement_input: MovementInput,
    pub(crate) ship_control_input: ShipControlInput,
    pub(crate) view_mode: ViewMode,
    pub(crate) seed: u64,
    pub(crate) automation_step: Option<Duration>,
    pub(crate) interpolate_presentation: bool,
    pub(crate) measure_physics: bool,
    pub(crate) latest_physics: crate::fragment_physics::Measurements,
    pub(crate) elapsed: Duration,
    pub(crate) feedback: Option<&'static str>,
}
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub(crate) struct ShipControlInput {
    pub(crate) pitch_up: bool,
    pub(crate) pitch_down: bool,
    pub(crate) yaw_left: bool,
    pub(crate) yaw_right: bool,
    pub(crate) roll_left: bool,
    pub(crate) roll_right: bool,
}

impl ShipControlInput {
    pub(crate) fn steering(self) -> SteeringInput {
        SteeringInput {
            pitch: f64::from(i8::from(self.pitch_up) - i8::from(self.pitch_down)),
            yaw: f64::from(i8::from(self.yaw_left) - i8::from(self.yaw_right)),
            roll: f64::from(i8::from(self.roll_right) - i8::from(self.roll_left)),
        }
    }
}

impl Default for GameplaySession {
    fn default() -> Self {
        Self {
            presentation: Default::default(),
            update_clock: UpdateClock::default(),
            camera_prototype: CameraPrototype::default(),
            character: CharacterController::default(),
            ship: ShipController::default(),
            automation_aim_delta: None,
            mining: Default::default(),
            fabricator: salimon_fabrication::Fabricator::starter(),
            crafting_ui: Default::default(),
            consumables: Default::default(),
            wheel: Default::default(),
            equipment: Default::default(),
            movement_input: Default::default(),
            ship_control_input: Default::default(),
            view_mode: ViewMode::default(),
            seed: 0,
            automation_step: None,
            interpolate_presentation: true,
            measure_physics: false,
            latest_physics: Default::default(),
            feedback: None,
            elapsed: Duration::ZERO,
        }
    }
}
#[derive(Clone, Copy, Debug)]
pub(crate) enum ControlKey {
    Forward,
    Backward,
    Left,
    Right,
    Jump,
    Descend,
    RollLeft,
    RollRight,
}
#[derive(Clone, Copy, Debug)]
pub(crate) enum Command {
    Wheel(consumable_wheel::WheelAction),
    Control(ControlKey, bool),
    Throttle(salimon_ship::ThrottleDirection, bool),
    Interact,
    Landing,
    Select(crate::equipment::ToolbarSlot),
    GrabDrop(bool),
    Look(f64, f64),
    Mining(bool),
    ToggleView,
    Camera(salimon_world::CameraCommand),
}

/// Owned domain observations; no native or renderer handles.
#[derive(Clone, Copy)]
pub(crate) struct Snapshot {
    pub(crate) ship: ShipSnapshot,
    pub(crate) character: salimon_character::CharacterSnapshot,
    pub(crate) backlog: Duration,
}
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) enum ViewMode {
    #[default]
    Gameplay,
    PrecisionTour,
}

pub(crate) fn door_passable(ship: ShipSnapshot) -> bool {
    ship.door_state == DoorState::Open && ship.door_open_fraction >= 1.0
}

impl GameplaySession {
    pub(crate) fn e2e_step(&self) -> Option<Duration> {
        self.automation_step
    }
    pub(crate) fn world_seed(&self) -> u64 {
        self.seed
    }
    pub(crate) fn interact(&mut self) {
        let ship_snapshot = self.ship.snapshot();
        let ship_frame = character_ship_frame(ship_snapshot.pose);
        let surface = self.terrain_frame(ship_snapshot);
        match self.character.location() {
            CharacterLocation::Cockpit => {
                self.character.leave_cockpit();
                self.ship.set_cockpit_control(false);
                self.ship_control_input = ShipControlInput::default();
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
                    available_interaction_target(self.character.location(), local_eye, local_look);
                match interaction {
                    Some(InteractionTarget::FabricatorTray) => self.collect_fabricator_output(),
                    Some(InteractionTarget::FabricatorScreen) => {
                        self.clear_held_controls();
                        self.crafting_ui.open = true;
                    }
                    Some(InteractionTarget::Fabricator) => {
                        self.insert_fabricator_fragment();
                    }
                    Some(InteractionTarget::Core) => {
                        let result = self.ship.intake_carried_fragment(&mut self.mining.session);
                        let text = match result {
                            salimon_ship::IntakeResult::Accepted => "Core accepted fragment",
                            salimon_ship::IntakeResult::Busy => "Core is processing a fragment",
                            salimon_ship::IntakeResult::NoCarriedFragment => {
                                "Carry a fragment to feed the Core"
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
                            && !salimon_character::boarding_ramp_occupied(local_eye);
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

    pub(crate) fn held_item(
        &self,
        player: salimon_character::CharacterSnapshot,
        valid_target: bool,
    ) -> Option<salimon_renderer::HeldItemInstance> {
        (self.view_mode == ViewMode::Gameplay)
            .then(|| self.mining.held_item(&self.equipment, player, valid_target))
            .flatten()
    }

    pub(crate) fn camera_phase(&self) -> TransitionPhase {
        self.camera_prototype.snapshot().transition_phase
    }

    pub(crate) fn automation_interaction(
        &self,
        local_eye: [f64; 3],
        local_look: [f64; 3],
    ) -> Option<&'static str> {
        match available_interaction_target(self.character.location(), local_eye, local_look) {
            Some(InteractionTarget::FabricatorTray) => Some("fabricator_tray"),
            Some(InteractionTarget::FabricatorScreen) => Some("fabricator_screen"),
            Some(InteractionTarget::Fabricator) => Some("fabricator"),
            Some(InteractionTarget::Core) => Some("core"),
            Some(InteractionTarget::Cockpit) => Some("cockpit"),
            Some(InteractionTarget::ExitDoor) => Some("exit_door"),
            None => None,
        }
    }

    pub(crate) fn toggle_view(&mut self) {
        self.view_mode = match self.view_mode {
            ViewMode::Gameplay => ViewMode::PrecisionTour,
            ViewMode::PrecisionTour => ViewMode::Gameplay,
        };
        self.clear_held_controls();
        self.reset_presentation();
        log::info!("view mode changed to {:?} (F2 toggles)", self.view_mode);
    }

    pub(crate) fn view_name(&self) -> &'static str {
        match self.view_mode {
            ViewMode::Gameplay => "Gameplay",
            ViewMode::PrecisionTour => "PrecisionTour",
        }
    }

    pub(crate) fn apply_look_delta(&mut self, dx: f64, dy: f64) {
        self.character.apply_mouse_delta(dx, dy);
        if self.view_mode == ViewMode::Gameplay {
            let ship = self.ship.snapshot();
            let frame = character_ship_frame(ship.pose);
            let player = self.character.snapshot(frame, self.terrain_frame(ship));
            let seed = self.world_seed();
            self.mining.advance(
                &self.equipment,
                Duration::ZERO,
                player,
                frame,
                door_passable(ship),
                seed,
            );
        }
    }

    pub(crate) fn mining_mouse_input(&mut self, pressed: bool) {
        self.mining.set_mouse_held(
            pressed
                && self.view_mode == ViewMode::Gameplay
                && self.equipment.mining_equipped()
                && self.mining.session.carried_id().is_none(),
        );
    }

    pub(crate) fn carry_action(&mut self, pickup: bool) {
        let ship = self.ship.snapshot();
        let frame = character_ship_frame(ship.pose);
        let player = self.character.snapshot(frame, self.terrain_frame(ship));
        let door_open = door_passable(ship);
        let message = if pickup {
            match crate::carrying::target(&self.mining, player, frame, door_open) {
                Some(id) if self.mining.session.pick_up(id) => {
                    self.equipment.sync_carrying(true);
                    self.mining.stop_mining();
                    self.mining.fragment_motion.remove(&id);
                    self.mining.ship_fragments.remove(&id);
                    "Fragment picked up - F to drop"
                }
                Some(_) => "Only one world object - F to drop first",
                None => "Aim at a fragment within 3 m",
            }
        } else if self.mining.session.carried_id().is_none() {
            "No world object carried"
        } else if let Some(pose) = crate::carrying::release_pose(&self.mining, player, frame) {
            let id = self
                .mining
                .session
                .carried_id()
                .expect("drop has carried identity");
            self.mining.session.drop_carried(pose);
            if player.location == CharacterLocation::InsideShip {
                self.mining.ship_fragments.insert(
                    id,
                    frame.project_world(salimon_character::WorldPoint::new(
                        pose.position().meters(),
                    )),
                );
            } else {
                self.mining.ship_fragments.remove(&id);
            }
            let forward = std::array::from_fn(|i| {
                player.look_target_meters[i] - player.eye_position_meters[i]
            });
            let length = forward.iter().map(|v| v * v).sum::<f64>().sqrt();
            let forward = forward.map(|v| v / length);
            crate::fragment_physics::release(
                &mut self.mining,
                id,
                forward,
                frame,
                player.location == CharacterLocation::InsideShip,
            );
            "Fragment dropped"
        } else {
            "Cannot drop fragment here"
        };
        crate::carrying::sync_ship_fragments(&mut self.mining, frame);
        crate::carrying::follow(&mut self.mining, player);
        crate::carrying::sync_ship_mass(&mut self.mining, frame, &mut self.ship);
        self.mining.carry_feedback = Some(message);
        self.feedback = Some(message);
    }

    pub(crate) fn sync_carried(&mut self) {
        let ship = self.ship.snapshot();
        let frame = character_ship_frame(ship.pose);
        crate::carrying::sync_ship_fragments(&mut self.mining, frame);
        let player = self.character.snapshot(frame, self.terrain_frame(ship));
        crate::carrying::follow(&mut self.mining, player);
        crate::carrying::sync_ship_mass(&mut self.mining, frame, &mut self.ship);
        self.sync_fabricator_mass();
    }

    pub(crate) fn clear_held_controls(&mut self) {
        self.crafting_ui.open = false;
        self.wheel = Default::default();
        self.movement_input = MovementInput::default();
        self.ship_control_input = ShipControlInput::default();
        self.ship.clear_steering_input();
        self.mining.clear_input();
        self.ship.clear_throttle_input();
    }

    pub(crate) fn advance_automation(&mut self, elapsed: Duration) {
        self.update_clock.accumulate(elapsed);
        while let Some(tick) = self.update_clock.take_tick() {
            self.advance_game(tick);
        }
    }

    pub(crate) fn advance_game(&mut self, delta: Duration) {
        self.elapsed += delta;
        if self.interpolate_presentation {
            self.presentation.begin(self.snapshot());
            let ship = self.ship.snapshot();
            let frame = character_ship_frame(ship.pose);
            let player = self
                .character
                .snapshot(frame, self.terrain_frame(ship))
                .eye_position_meters;
            crate::fragment_physics::capture_previous(&mut self.mining, frame, player);
        }
        self.camera_prototype.advance(delta);
        self.ship
            .set_steering_input(self.ship_control_input.steering());
        self.advance_fabrication(delta);
        let ship = self.ship.snapshot();
        if !ship.power_available
            || !ship.cockpit_control_active
            || ship.flight_state != FlightState::Flying
        {
            self.ship_control_input = ShipControlInput::default();
        }
        self.character
            .set_ship_gravity_enabled(ship.power_available);
        self.mining.ship_gravity_disabled = !ship.power_available;
        if self.view_mode == ViewMode::Gameplay {
            let eye = self
                .character
                .snapshot(character_ship_frame(ship.pose), self.terrain_frame(ship))
                .eye_position_meters;
            self.character.set_nearby_surface(self.nearby_terrain(eye));
            self.character
                .set_ramp_moving(ship.door_open_fraction > 0.0 && ship.door_open_fraction < 1.0);
            self.character.advance_with_motion(
                delta,
                self.movement_input,
                (
                    character_ship_frame(ship.pose),
                    ship.velocity_meters_per_second,
                ),
                self.terrain_frame(ship),
                door_passable(ship),
                matches!(ship.flight_state, FlightState::Landed { .. }),
            );
            let eye = self
                .character
                .snapshot(character_ship_frame(ship.pose), self.terrain_frame(ship))
                .eye_position_meters;
            self.character.set_nearby_surface(self.nearby_terrain(eye));
        }
        if self.view_mode == ViewMode::Gameplay {
            let frame = character_ship_frame(ship.pose);
            self.mining.advance(
                &self.equipment,
                delta,
                self.character.snapshot(frame, self.terrain_frame(ship)),
                frame,
                door_passable(ship),
                self.world_seed(),
            );
            let player = self
                .character
                .snapshot(frame, self.terrain_frame(ship))
                .eye_position_meters;
            let measurements = if self.measure_physics {
                crate::fragment_physics::advance_measured(
                    &mut self.mining,
                    frame,
                    delta,
                    player,
                    true,
                )
            } else {
                crate::fragment_physics::advance(&mut self.mining, frame, delta, player);
                Default::default()
            };
            self.latest_physics.accumulate(measurements);
        }
        self.sync_carried();
        self.validate_crafting_focus();
        if self.interpolate_presentation && !self.presentation.complete(self.snapshot()) {
            self.mining.fragment_previous.clear();
        }
    }
}

impl GameplaySession {
    pub(crate) fn snapshot(&self) -> Snapshot {
        let ship = self.ship.snapshot();
        Snapshot {
            ship,
            character: self
                .character
                .snapshot(character_ship_frame(ship.pose), self.terrain_frame(ship)),
            backlog: self.update_clock.backlog(),
        }
    }
    /// Desktop admission is a shell policy. View/domain gates apply to every caller.
    pub(crate) fn command(&mut self, command: Command) {
        use Command::*;
        if let Wheel(action) = command {
            self.wheel_action(action);
            return;
        }
        if self.wheel.open {
            if matches!(command, ToggleView) {
                self.toggle_view();
            }
            return;
        }
        if self.crafting_ui.open {
            return; // Modal UI owns input; simulation and Core funding continue.
        }
        if let ToggleView = command {
            self.toggle_view();
            return;
        }
        if let Camera(camera) = command {
            if self.view_mode == ViewMode::Gameplay {
                self.toggle_view();
            }
            self.camera_prototype.apply_command(camera);
            return;
        }
        let release = matches!(
            command,
            Control(_, false) | Throttle(_, false) | GrabDrop(false) | Mining(false)
        );
        if self.view_mode != ViewMode::Gameplay && !release {
            return;
        }
        match command {
            Control(key, pressed) => self.control(key, pressed),
            Throttle(direction, pressed) => {
                if !pressed || self.character.location() == CharacterLocation::Cockpit {
                    self.ship.set_throttle_key(direction, pressed);
                } else {
                    self.ship.clear_throttle_input();
                }
            }
            Interact => {
                let location = self.character.location();
                self.interact();
                if location != self.character.location() {
                    self.reset_presentation();
                }
            }
            Landing => {
                self.ship.trigger_landing_action();
                self.reset_presentation();
            }
            Select(slot) => {
                let previous = self.equipment.selected();
                self.equipment
                    .select(slot, self.mining.session.carried_id().is_some());
                if previous != self.equipment.selected() {
                    self.mining.clear_input();
                }
            }
            GrabDrop(false) => self.mining.release_f(),
            GrabDrop(true) => {
                if self.mining.press_f() {
                    let snapshot = self.snapshot();
                    let frame = character_ship_frame(snapshot.ship.pose);
                    let pickup = self.mining.session.carried_id().is_none();
                    if !pickup
                        || crate::carrying::target(
                            &self.mining,
                            snapshot.character,
                            frame,
                            door_passable(snapshot.ship),
                        )
                        .is_some()
                    {
                        self.carry_action(pickup);
                    }
                }
            }
            Look(dx, dy) => self.apply_look_delta(dx, dy),
            Mining(pressed) => {
                let was_down = std::mem::replace(&mut self.wheel.primary_down, pressed);
                if !pressed || !was_down {
                    self.mining_mouse_input(pressed);
                }
            }
            Wheel(_) => unreachable!(),
            ToggleView | Camera(_) => unreachable!(),
        }
    }
    fn control(&mut self, key: ControlKey, pressed: bool) {
        use ControlKey::*;
        // Releases clear both meanings even across location/view changes.
        let cockpit = self.character.location() == CharacterLocation::Cockpit;
        if !pressed || !cockpit || matches!(key, Jump | Descend) {
            match key {
                Forward => self.movement_input.forward = pressed,
                Backward => self.movement_input.backward = pressed,
                Left => self.movement_input.left = pressed,
                Right => self.movement_input.right = pressed,
                Jump => self.movement_input.jump = pressed,
                Descend => self.movement_input.descend = pressed,
                _ => {}
            }
        }
        if !pressed || cockpit {
            let ship = self.ship.snapshot();
            if pressed
                && !(ship.power_available
                    && ship.cockpit_control_active
                    && ship.flight_state == FlightState::Flying)
            {
                if !matches!(key, Jump | Descend) {
                    self.ship_control_input = ShipControlInput::default();
                }
                return;
            }
            match key {
                Forward => self.ship_control_input.pitch_up = pressed,
                Backward => self.ship_control_input.pitch_down = pressed,
                Left => self.ship_control_input.yaw_left = pressed,
                Right => self.ship_control_input.yaw_right = pressed,
                RollLeft => self.ship_control_input.roll_left = pressed,
                RollRight => self.ship_control_input.roll_right = pressed,
                _ => {}
            }
        }
    }
}

#[cfg(test)]
#[path = "fabrication_scenarios.rs"]
mod fabrication_scenarios;

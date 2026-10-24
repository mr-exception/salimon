//! Portable production gameplay composition. No window, GPU lifecycle or evidence IO.
pub(crate) mod archive;
mod cargo;
pub(crate) mod construction;
pub(crate) mod consumable_wheel;
pub(crate) mod consumables;
pub(crate) mod crafting_ui;
mod fabrication;
mod fixtures;
pub(crate) mod frames;
mod habitation;
pub(crate) mod interaction;
pub(crate) mod journey;
pub(crate) mod lifecycle;
pub(crate) mod mounts;
mod presentation;
mod recovery;
mod survival;
pub(crate) mod tutorial;
// Portable adapters are consumed by whole-session orchestration in #365/#366.
mod pause;
#[allow(dead_code)]
pub(crate) mod persistence;
#[cfg(test)]
mod persistence_scenarios;
#[cfg(test)]
mod propulsion_tests;
#[cfg(test)]
mod starting_supplies_tests;
#[cfg(test)]
mod tests;
use self::frames::character_ship_frame;
use self::interaction::{InteractionTarget, available_interaction_target};
use crate::update_clock::UpdateClock;
use salimon_character::{CharacterController, CharacterLocation, MovementInput};
use salimon_ship::SteeringInput;
use salimon_ship::{CockpitMessage, DoorState, FlightState, ShipController, ShipSnapshot};
use salimon_survival::restrictions::Action;
use salimon_world::{CameraPrototype, TransitionPhase};
use std::time::Duration;

pub(crate) struct GameplaySession {
    paused: bool,
    pub(crate) mounts: salimon_ship::attachments::Attachments,
    pub(crate) cargo_door: cargo::CargoDoor,
    pub(crate) habitation: habitation::Habitation,
    pub(crate) tutorial: tutorial::Tutorial,
    pub(crate) building: construction::Building,
    pub(crate) archive: archive::Archive,
    recovery: recovery::Recovery,
    pub(crate) archive_reader: archive::Reader,
    pub(crate) run: lifecycle::RunState,
    pub(crate) consumable_effects: salimon_survival::consumables::Catalog,
    pub(crate) survival: salimon_survival::Survival,
    pub(super) restriction_config: salimon_survival::restrictions::RestrictionConfig,
    interaction_ready_at: Duration,
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
            paused: false,
            mounts: Default::default(),
            cargo_door: Default::default(),
            habitation: Default::default(),
            building: Default::default(),
            archive: Default::default(),
            recovery: Default::default(),
            archive_reader: Default::default(),
            tutorial: tutorial::Tutorial::dismissed(),
            run: Default::default(),
            consumable_effects: salimon_survival::consumables::Catalog::starter(),
            survival: Default::default(),
            restriction_config: Default::default(),
            interaction_ready_at: Duration::ZERO,
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
    Build(construction::BuildAction),
    Mount(mounts::MountAction),
    TogglePause,
    InspectVitals,
    SkipTutorial,
    Control(ControlKey, bool),
    Throttle(salimon_ship::ThrottleDirection, bool),
    Interact,
    Landing,
    ToggleEngines,
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
    /// Explicit new-run boundary. Empty Default remains useful for restoration
    /// and isolated fixtures; loading/resuming never invokes this constructor.
    pub(crate) fn new_run() -> Self {
        let (fabricator, consumables) = salimon_fabrication::Fabricator::starter_with_supplies();
        let mut session = Self {
            mounts: salimon_ship::attachments::Attachments::starter(),
            fabricator,
            consumables,
            tutorial: tutorial::Tutorial::default(),
            ..Default::default()
        };
        session.sync_fabricator_mass();
        session.sync_accessory();
        session
    }

    pub(crate) fn e2e_step(&self) -> Option<Duration> {
        self.automation_step
    }
    pub(crate) fn world_seed(&self) -> u64 {
        self.seed
    }
    pub(crate) fn interact(&mut self) {
        if self.paused || self.survival.snapshot().terminal {
            return;
        }
        self.enforce_survival_restrictions();
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
                if let Some(target) = interaction
                    && !self.admit_survival_action(Self::interaction_action(target))
                {
                    return;
                }
                match interaction {
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
        match available_interaction_target(self.character.location(), local_eye, local_look)
            .filter(|target| self.permitted_interaction(*target))
        {
            Some(InteractionTarget::Bunk) => Some("bunk"),
            Some(InteractionTarget::Galley) => Some("galley"),
            Some(InteractionTarget::Hygiene) => Some("hygiene"),
            Some(InteractionTarget::CargoDoor) => Some("cargo_door"),
            Some(InteractionTarget::Archive) => Some("archive"),
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
        if self.paused {
            return;
        }
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
        if self.paused || self.survival.snapshot().terminal {
            return;
        }
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
                && !self.paused
                && self.view_mode == ViewMode::Gameplay
                && self.restrictions().permits(Action::Mine)
                && self.equipment.mining_equipped()
                && self.mining.session.carried_id().is_none(),
        );
    }

    pub(crate) fn carry_action(&mut self, pickup: bool) {
        if self.mounts.carried.is_some() {
            self.feedback = Some("Accessory in hand - G at a compatible rail to attach");
            return;
        }
        if !self.admit_survival_action(Action::Carry) {
            return;
        }
        let ship = self.ship.snapshot();
        let frame = character_ship_frame(ship.pose);
        let player = self.character.snapshot(frame, self.terrain_frame(ship));
        let door_open = door_passable(ship);
        let message = if pickup {
            match crate::carrying::target(&self.mining, player, frame, door_open) {
                Some(id) if self.mining.secured.contains(&id) => {
                    "Release mechanical clamp with K first"
                }
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
        if self.paused {
            return;
        }
        self.mining.mount_geometry = self.mounts.clone();
        let ship = self.ship.snapshot();
        let frame = character_ship_frame(ship.pose);
        crate::carrying::sync_ship_fragments(&mut self.mining, frame);
        let player = self.character.snapshot(frame, self.terrain_frame(ship));
        crate::carrying::follow(&mut self.mining, player);
        crate::carrying::sync_ship_mass(&mut self.mining, frame, &mut self.ship);
        self.sync_fabricator_mass();
        self.sync_accessory();
    }

    pub(crate) fn clear_held_controls(&mut self) {
        self.crafting_ui.open = false;
        self.archive_reader.open = false;
        self.wheel = Default::default();
        self.movement_input = MovementInput::default();
        self.ship_control_input = ShipControlInput::default();
        self.ship.clear_steering_input();
        self.mining.clear_input();
        self.ship.clear_throttle_input();
    }

    pub(crate) fn advance_automation(&mut self, elapsed: Duration) {
        if self.paused {
            return;
        }
        if self.survival.snapshot().terminal {
            self.freeze_terminal();
            return;
        }
        self.update_clock.accumulate(elapsed);
        while let Some(tick) = self.update_clock.take_tick() {
            self.advance_game(tick);
        }
    }

    pub(crate) fn advance_game(&mut self, delta: Duration) {
        if self.paused {
            return;
        }
        if self.survival.snapshot().terminal {
            self.freeze_terminal();
            return;
        }
        let before = self.survival.snapshot().active_time;
        self.survival.advance(delta);
        let delta = self.survival.snapshot().active_time - before;
        self.enforce_survival_restrictions();
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
        let journey_ship_start = self.ship.snapshot().pose.position_meters;
        self.camera_prototype.advance(delta);
        self.ship
            .set_steering_input(self.ship_control_input.steering());
        self.advance_fabrication(delta);
        self.advance_recovery(delta);
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
        // Both character endpoints use the completed ship frame. Transport and
        // rotation of the cabin therefore cannot become extra walking distance.
        let journey_character_start = self.snapshot().character;
        if self.view_mode == ViewMode::Gameplay {
            let eye = self
                .character
                .snapshot(character_ship_frame(ship.pose), self.terrain_frame(ship))
                .eye_position_meters;
            self.character.set_nearby_surface(self.nearby_terrain(eye));
            self.advance_cargo_door(delta);
            self.advance_habitation(delta);
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
                self.mining_delta(delta),
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
        let journey_character_end = self.snapshot().character;
        self.run.journey.record(
            journey_ship_start,
            ship.pose.position_meters,
            journey_character_start,
            journey_character_end,
            !delta.is_zero(),
        );
        self.sync_carried();
        self.validate_crafting_focus();
        self.validate_archive_focus();
        if self.survival.snapshot().terminal {
            self.freeze_terminal();
        }
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
        if self.survival.snapshot().terminal {
            self.freeze_terminal();
            return;
        }
        if let TogglePause = command {
            self.toggle_pause();
            return;
        }
        if self.paused {
            return;
        }
        self.enforce_survival_restrictions();
        if self.view_mode == ViewMode::Gameplay {
            match command {
                SkipTutorial => {
                    self.tutorial.skip();
                    return;
                }
                InspectVitals
                    if !self.wheel.open && !self.crafting_ui.open && !self.archive_reader.open =>
                {
                    self.tutorial.record(tutorial::Objective::Vitals);
                    return;
                }
                _ => {}
            }
        }
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
        if self.crafting_ui.open || self.archive_reader.open {
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
            Build(action) => self.construction_action(action),
            Mount(action) => self.mount_action(action),
            Control(key, pressed) => self.control(key, pressed),
            Throttle(direction, pressed) => {
                if !pressed
                    || (self.character.location() == CharacterLocation::Cockpit
                        && self.restrictions().permits(Action::Pilot))
                {
                    self.ship.set_throttle_key(direction, pressed);
                } else {
                    self.ship.clear_throttle_input();
                }
            }
            Interact if self.building.active => {
                self.feedback = Some("Project inspected");
            }
            Interact => {
                let location = self.character.location();
                self.interact();
                if location != self.character.location() {
                    self.reset_presentation();
                }
            }
            ToggleEngines => {
                if self.admit_survival_action(Action::Pilot) && self.ship.toggle_engines() {
                    self.feedback = Some(if self.ship.snapshot().engines_enabled {
                        "Engines on"
                    } else {
                        "Engines off - coasting"
                    });
                }
            }
            Landing => {
                if !self.admit_survival_action(Action::Pilot) {
                    return;
                }
                self.ship.trigger_landing_action();
                self.reset_presentation();
            }
            Select(slot) => {
                let previous = self.equipment.selected();
                self.equipment
                    .select(slot, self.mining.session.carried_object().is_some());
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
            Mining(pressed) if self.building.active => {
                self.mining.clear_input();
                if pressed {
                    self.construction_action(construction::BuildAction::Place);
                }
            }
            Mining(pressed) => {
                let was_down = std::mem::replace(&mut self.wheel.primary_down, pressed);
                if !pressed || !was_down {
                    self.mining_mouse_input(pressed);
                }
            }
            Wheel(_) => unreachable!(),
            InspectVitals | SkipTutorial => {}
            TogglePause | ToggleView | Camera(_) => unreachable!(),
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
                && !(self.restrictions().permits(Action::Pilot)
                    && ship.power_available
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

//! Portable production gameplay composition. No window, GPU lifecycle or evidence IO.
pub(crate) mod archive;
pub(crate) mod cabin;
pub(crate) mod canonical;
mod cargo;
mod commands;
pub(crate) mod construction;
pub(crate) mod consumable_wheel;
pub(crate) mod consumables;
pub(crate) mod core_upgrades;
pub(crate) mod crafting_ui;
mod fabrication;
pub(crate) mod finishes;
mod fixtures;
pub(crate) mod frames;
mod habitation;
mod init;
pub(crate) mod input;
mod organisms;
pub(crate) mod policy;
mod tick;
pub(crate) use input::ShipControlInput;
pub(crate) mod interaction;
pub(crate) mod journey;
pub(crate) mod lifecycle;
pub(crate) mod lighting;
pub(crate) mod mounts;
mod presentation;
pub(crate) use presentation::Presentation;
mod recovery;
pub(crate) mod surface_age;
mod survival;
pub(crate) mod tutorial;
// Portable adapters are consumed by whole-session orchestration in #365/#366.
pub(crate) mod chart;
pub(crate) mod field_marks;
pub(crate) mod navigation;
pub(crate) mod offline;
mod pause;
#[allow(dead_code)]
pub(crate) mod persistence;
#[cfg(test)]
mod persistence_scenarios;
#[cfg(test)]
mod propulsion_tests;
pub(crate) mod scanner;
#[cfg(test)]
mod starting_supplies_tests;
#[cfg(test)]
mod tests;
use self::frames::character_ship_frame;
use self::interaction::InteractionTarget;
use crate::update_clock::UpdateClock;
use salimon_character::{CharacterController, CharacterLocation};
#[cfg(test)]
use salimon_ship::FlightState;
use salimon_ship::{DoorState, ShipController, ShipSnapshot};
use salimon_survival::restrictions::Action;
use salimon_world::{CameraPrototype, TransitionPhase};
use std::time::Duration;

pub(crate) struct GameplaySession {
    pub(crate) scanner: scanner::Scanner,
    pub(crate) chart: salimon_universe::scanner::chart::Chart,
    pub(crate) field_marks: field_marks::State,
    navigation_cache: std::cell::RefCell<Option<navigation::Cached>>,
    paused: bool,
    pub(crate) offline: offline::OfflineClock,
    offline_power: bool,
    pub(crate) audio_epoch: u64,
    pub(crate) mount_audio: (u64, [f64; 3]),
    pub(crate) lighting: salimon_ship::lighting::Lighting,
    pub(crate) core_upgrades: salimon_ship::core_upgrades::CoreUpgrades,
    pub(crate) surface_age: salimon_ship::surface_age::SurfaceAge,
    pub(crate) finishes: salimon_ship::finishes::Finishes,
    pub(crate) finish_group: usize,
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
    pub(crate) input: input::State,
    pub(crate) view_mode: ViewMode,
    pub(crate) seed: u64,
    pub(crate) policy: policy::Policy,
    pub(crate) latest_physics: crate::fragment_physics::Measurements,
    pub(crate) elapsed: Duration,
    pub(crate) feedback: Option<&'static str>,
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
    Cabin(cabin::CabinAction),
    Finish(finishes::FinishAction),
    CoreUpgrade(core_upgrades::CoreUpgradeAction),
    Light(lighting::LightAction),
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
    pub(crate) fn e2e_step(&self) -> Option<Duration> {
        self.policy.automation_step()
    }
    pub(crate) fn world_seed(&self) -> u64 {
        self.seed
    }

    pub(crate) fn held_item(
        &self,
        player: salimon_character::CharacterSnapshot,
        valid_target: bool,
    ) -> Option<salimon_renderer::HeldItemInstance> {
        (self.view_mode == ViewMode::Gameplay)
            .then(|| {
                self.mining
                    .held_item(&self.equipment, player, valid_target)
                    .map(|mut item| {
                        item.surface_age =
                            self.surface_age.intensity(salimon_ship::surface_age::TOOL);
                        item
                    })
            })
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
        match self
            .available_target(self.character.location(), local_eye, local_look)
            .filter(|target| self.permitted_interaction(*target))
        {
            Some(InteractionTarget::CabinTable) => Some("cabin_table"),
            Some(InteractionTarget::CabinSeat) => Some("cabin_seat"),
            Some(InteractionTarget::PrivacyScreen) => Some("privacy_screen"),
            Some(InteractionTarget::ObservationShutter) => Some("observation_shutter"),
            Some(InteractionTarget::CabinLocker) => Some("cabin_locker"),
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
        if self
            .habitation
            .cabin
            .as_ref()
            .is_some_and(|c| c.rest.is_some())
        {
            self.feedback = Some("Get up before carrying a physical object");
            return;
        }
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
        crate::carrying::follow(&mut self.mining, player, frame);
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
        crate::carrying::follow(&mut self.mining, player, frame);
        crate::carrying::sync_ship_mass(&mut self.mining, frame, &mut self.ship);
        self.sync_fabricator_mass();
        self.sync_accessory();
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
}

#[cfg(test)]
#[path = "fabrication_scenarios.rs"]
mod fabrication_scenarios;

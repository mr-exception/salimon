//! Typed gameplay admission and delegation; domain actions retain their owners.
use super::{
    Command, GameplaySession, ViewMode, construction, door_passable, frames::character_ship_frame,
    tutorial,
};
use salimon_character::CharacterLocation;
use salimon_survival::restrictions::Action;

impl GameplaySession {
    pub(crate) fn command(&mut self, command: Command) {
        use Command::*;
        self.navigation_cache.get_mut().take();
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
        if self.surface_chart.open
            && !matches!(
                command,
                ToggleView
                    | Control(_, false)
                    | Throttle(_, false)
                    | Mining(false)
                    | GrabDrop(false)
            )
        {
            return;
        }
        self.finish_offline_remainder();
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
            Cabin(action) => self.cabin_action(action),
            Finish(action) => self.finish_action(action),
            Light(action) => self.light_action(action),
            CoreUpgrade(action) => self.core_upgrade_action(action),
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
                self.contribute_construction();
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
                if matches!(
                    self.ship.snapshot().flight_state,
                    salimon_ship::FlightState::Landed { .. }
                ) && self.ship.snapshot().cockpit_control_active
                    && let Some(reason) = self.construction_takeoff_blocker()
                {
                    self.feedback = Some(reason.text());
                    return;
                }
                self.ship.trigger_landing_action();
                self.reset_presentation();
            }
            Select(slot) => {
                if self.biology.pending.is_some() {
                    return;
                }
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
                    } else {
                        self.begin_biological_pull();
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
}

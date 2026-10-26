//! Portable domain binding/admission composition; native events end in app/input.rs.
use super::{Command, ControlKey, GameplaySession, ViewMode};
use salimon_character::CharacterLocation;
use salimon_ship::FlightState;
use salimon_survival::restrictions::Action;
mod state;
use keys::{camera_command, control_key, throttle_direction, toolbar_slot};
use salimon_ship::ThrottleDirection;
use salimon_world::{CameraCommand, CelestialBodyId};
pub(crate) use state::{ShipControlInput, State};
mod construction;
mod equipment;
mod interior;
pub(crate) mod keys;
mod lighting;
mod modals;
mod ship;
#[cfg(test)]
mod tests;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum Key {
    Forward,
    Backward,
    Left,
    Right,
    Jump,
    Descend,
    Up,
    Down,
    Previous,
    Next,
    Build,
    Cancel,
    Recovery,
    Status,
    Rest,
    Place,
    Tutorial,
    Delete,
    Confirm,
    ConfirmAlternate,
    Interact,
    Tab,
    Wheel,
    View,
    Diagnostics,
    LightPreset,
    LightMaster,
    LightFixture,
    LightBrightness,
    LightWarmth,
    LightGeneral,
    Engines,
    Landing,
    CoreInput,
    CoreApply,
    CoreRecover,
    FinishGroup,
    FinishNext,
    FinishReset,
    Restraint,
    MoveAccessory,
    Secure,
    GrabDrop,
    TourPause,
    Slot1,
    Slot2,
    Slot3,
    Slot4,
    Slot5,
    Slot6,
    Slot7,
}

pub(crate) fn dispatch(session: &mut GameplaySession, key: Key, pressed: bool) {
    if construction::handle(session, key, pressed) || modals::handle(session, key, pressed) {
        return;
    }
    let command = if !pressed {
        if let Some(control) = control_key(key) {
            Some(Command::Control(control, false))
        } else if let Some(direction) = throttle_direction(key) {
            Some(Command::Throttle(direction, false))
        } else if key == Key::GrabDrop {
            Some(Command::GrabDrop(false))
        } else {
            None
        }
    } else if key == Key::View {
        Some(Command::ToggleView)
    } else if session.view_mode != ViewMode::Gameplay {
        camera_command(key, true).map(Command::Camera)
    } else {
        match key {
            Key::Status => Some(Command::InspectVitals),
            Key::Interact => Some(Command::Interact),
            Key::GrabDrop => Some(Command::GrabDrop(true)),
            _ => lighting::command(key)
                .or_else(|| ship::command(key))
                .or_else(|| equipment::command(key))
                .or_else(|| interior::command(key))
                .or_else(|| control_key(key).map(|key| Command::Control(key, true))),
        }
    };
    if let Some(command) = command {
        session.command(command);
    }
}

impl GameplaySession {
    pub(crate) fn clear_held_controls(&mut self) {
        self.crafting_ui.open = false;
        self.archive_reader.open = false;
        self.wheel = Default::default();
        self.input.clear();
        self.ship.clear_steering_input();
        self.mining.clear_input();
        self.ship.clear_throttle_input();
    }
    pub(super) fn control(&mut self, key: ControlKey, pressed: bool) {
        use ControlKey::*;
        // Releases clear both meanings even across location/view changes.
        let cockpit = self.character.location() == CharacterLocation::Cockpit;
        if !pressed || !cockpit || matches!(key, Jump | Descend) {
            match key {
                Forward => self.input.movement.forward = pressed,
                Backward => self.input.movement.backward = pressed,
                Left => self.input.movement.left = pressed,
                Right => self.input.movement.right = pressed,
                Jump => self.input.movement.jump = pressed,
                Descend => self.input.movement.descend = pressed,
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
                    self.input.steering = ShipControlInput::default();
                }
                return;
            }
            match key {
                Forward => self.input.steering.pitch_up = pressed,
                Backward => self.input.steering.pitch_down = pressed,
                Left => self.input.steering.yaw_left = pressed,
                Right => self.input.steering.yaw_right = pressed,
                RollLeft => self.input.steering.roll_left = pressed,
                RollRight => self.input.steering.roll_right = pressed,
                _ => {}
            }
        }
    }
}

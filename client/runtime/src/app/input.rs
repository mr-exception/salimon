//! Native physical-key translation and held control state.

use salimon_character::MovementInput;
use salimon_ship::{SteeringInput, ThrottleDirection};
use salimon_world::{CameraCommand, CelestialBodyId};
use winit::event::ElementState;
use winit::keyboard::{KeyCode, PhysicalKey};

pub(super) fn release_cursor_pressed(state: ElementState, repeat: bool, key: PhysicalKey) -> bool {
    state == ElementState::Pressed && !repeat && key == PhysicalKey::Code(KeyCode::Escape)
}

pub(super) fn toolbar_slot(key: PhysicalKey) -> Option<crate::equipment::ToolbarSlot> {
    use crate::equipment::ToolbarSlot;
    match key {
        PhysicalKey::Code(KeyCode::Digit1) => Some(ToolbarSlot::One),
        PhysicalKey::Code(KeyCode::Digit2) => Some(ToolbarSlot::Two),
        PhysicalKey::Code(KeyCode::Digit3) => Some(ToolbarSlot::Three),
        PhysicalKey::Code(KeyCode::Digit4) => Some(ToolbarSlot::Four),
        PhysicalKey::Code(KeyCode::Digit5) => Some(ToolbarSlot::Five),
        _ => None,
    }
}

/// Automation deliberately bypasses desktop capture/focus, never gameplay/view
/// or domain authority. Native release edges are admitted even after capture loss.
#[derive(Clone, Copy, Debug)]
pub(super) enum InputSource {
    Native,
    Automation,
}

#[derive(Clone, Copy, Debug)]
pub(super) struct InputPolicy {
    pub gameplay: bool,
    pub captured: bool,
    pub focused: bool,
    pub occluded: bool,
}
impl InputPolicy {
    pub fn admits_press(self, source: InputSource, global: bool) -> bool {
        match source {
            InputSource::Automation => global || self.gameplay,
            InputSource::Native => {
                self.focused && !self.occluded && (global || (self.gameplay && self.captured))
            }
        }
    }
    pub fn admits_key(
        self,
        source: InputSource,
        key: PhysicalKey,
        pressed: bool,
        repeat: bool,
        synthetic: bool,
    ) -> bool {
        if !pressed {
            return true;
        }
        if repeat || synthetic {
            return false;
        }
        let global = matches!(key, PhysicalKey::Code(KeyCode::F2 | KeyCode::F3))
            || camera_command(ElementState::Pressed, false, key, !self.gameplay).is_some();
        self.admits_press(source, global)
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum MovementKey {
    Forward,
    Backward,
    Left,
    Right,
    Jump,
    Descend,
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub(super) struct ShipControlInput {
    pitch_up: bool,
    pitch_down: bool,
    yaw_left: bool,
    yaw_right: bool,
    roll_left: bool,
    roll_right: bool,
}

impl ShipControlInput {
    pub(super) fn steering(self) -> SteeringInput {
        SteeringInput {
            pitch: f64::from(i8::from(self.pitch_up) - i8::from(self.pitch_down)),
            yaw: f64::from(i8::from(self.yaw_left) - i8::from(self.yaw_right)),
            roll: f64::from(i8::from(self.roll_right) - i8::from(self.roll_left)),
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum ShipControlKey {
    PitchUp,
    PitchDown,
    YawLeft,
    YawRight,
    RollLeft,
    RollRight,
}

fn ship_control_key(key: PhysicalKey) -> Option<ShipControlKey> {
    match key {
        PhysicalKey::Code(KeyCode::KeyW) => Some(ShipControlKey::PitchUp),
        PhysicalKey::Code(KeyCode::KeyS) => Some(ShipControlKey::PitchDown),
        PhysicalKey::Code(KeyCode::KeyA) => Some(ShipControlKey::YawLeft),
        PhysicalKey::Code(KeyCode::KeyD) => Some(ShipControlKey::YawRight),
        PhysicalKey::Code(KeyCode::ArrowLeft) => Some(ShipControlKey::RollLeft),
        PhysicalKey::Code(KeyCode::ArrowRight) => Some(ShipControlKey::RollRight),
        _ => None,
    }
}

pub(super) fn is_ship_control_key(key: PhysicalKey) -> bool {
    ship_control_key(key).is_some()
}

/// Denied input is cleared instead of retained for replay after power/assist return.
pub(super) fn update_powered_ship_control_input(
    input: &mut ShipControlInput,
    key: PhysicalKey,
    pressed: bool,
    allowed: bool,
) {
    if allowed {
        update_ship_control_input(input, key, pressed);
    } else {
        *input = ShipControlInput::default();
    }
}

pub(super) fn update_ship_control_input(
    input: &mut ShipControlInput,
    key: PhysicalKey,
    pressed: bool,
) {
    match ship_control_key(key) {
        Some(ShipControlKey::PitchUp) => input.pitch_up = pressed,
        Some(ShipControlKey::PitchDown) => input.pitch_down = pressed,
        Some(ShipControlKey::YawLeft) => input.yaw_left = pressed,
        Some(ShipControlKey::YawRight) => input.yaw_right = pressed,
        Some(ShipControlKey::RollLeft) => input.roll_left = pressed,
        Some(ShipControlKey::RollRight) => input.roll_right = pressed,
        None => {}
    }
}

pub(super) fn throttle_direction(key: PhysicalKey) -> Option<ThrottleDirection> {
    match key {
        PhysicalKey::Code(KeyCode::ArrowUp) => Some(ThrottleDirection::Up),
        PhysicalKey::Code(KeyCode::ArrowDown) => Some(ThrottleDirection::Down),
        _ => None,
    }
}

fn movement_key(key: PhysicalKey) -> Option<MovementKey> {
    match key {
        PhysicalKey::Code(KeyCode::KeyW) => Some(MovementKey::Forward),
        PhysicalKey::Code(KeyCode::KeyS) => Some(MovementKey::Backward),
        PhysicalKey::Code(KeyCode::KeyA) => Some(MovementKey::Left),
        PhysicalKey::Code(KeyCode::KeyD) => Some(MovementKey::Right),
        PhysicalKey::Code(KeyCode::Space) => Some(MovementKey::Jump),
        PhysicalKey::Code(KeyCode::ShiftLeft) => Some(MovementKey::Descend),
        _ => None,
    }
}

pub(super) fn is_movement_key(key: PhysicalKey) -> bool {
    movement_key(key).is_some()
}

pub(super) fn update_movement_input(input: &mut MovementInput, key: PhysicalKey, pressed: bool) {
    match movement_key(key) {
        Some(MovementKey::Forward) => input.forward = pressed,
        Some(MovementKey::Backward) => input.backward = pressed,
        Some(MovementKey::Left) => input.left = pressed,
        Some(MovementKey::Right) => input.right = pressed,
        Some(MovementKey::Jump) => input.jump = pressed,
        Some(MovementKey::Descend) => input.descend = pressed,
        None => {}
    }
}

pub(super) fn camera_command(
    state: ElementState,
    repeat: bool,
    key: PhysicalKey,
    precision_tour: bool,
) -> Option<CameraCommand> {
    if state != ElementState::Pressed || repeat {
        return None;
    }

    if !precision_tour
        && matches!(
            key,
            PhysicalKey::Code(
                KeyCode::Digit1
                    | KeyCode::Digit2
                    | KeyCode::Digit3
                    | KeyCode::Digit4
                    | KeyCode::Digit5
                    | KeyCode::Digit6
            )
        )
    {
        return None;
    }

    match key {
        PhysicalKey::Code(KeyCode::KeyP) => Some(CameraCommand::TogglePause),
        PhysicalKey::Code(KeyCode::KeyR) => Some(CameraCommand::Restart),
        PhysicalKey::Code(KeyCode::KeyN) => Some(CameraCommand::JumpToNear),
        PhysicalKey::Code(KeyCode::Digit1) => {
            Some(CameraCommand::InspectBody(CelestialBodyId::Sun))
        }
        PhysicalKey::Code(KeyCode::Digit2) => {
            Some(CameraCommand::InspectBody(CelestialBodyId::Mercury))
        }
        PhysicalKey::Code(KeyCode::Digit3) => {
            Some(CameraCommand::InspectBody(CelestialBodyId::Venus))
        }
        PhysicalKey::Code(KeyCode::Digit4) => {
            Some(CameraCommand::InspectBody(CelestialBodyId::Earth))
        }
        PhysicalKey::Code(KeyCode::Digit5) => {
            Some(CameraCommand::InspectBody(CelestialBodyId::Moon))
        }
        PhysicalKey::Code(KeyCode::Digit6) => {
            Some(CameraCommand::InspectBody(CelestialBodyId::Mars))
        }
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use salimon_world::CameraCommand;
    #[test]
    fn native_press_policy_rejects_capture_focus_tour_repeats_and_synthetic_replay() {
        let ready = InputPolicy {
            gameplay: true,
            captured: true,
            focused: true,
            occluded: false,
        };
        for key in [
            KeyCode::KeyW,
            KeyCode::KeyE,
            KeyCode::KeyL,
            KeyCode::KeyF,
            KeyCode::Digit1,
            KeyCode::ArrowUp,
            KeyCode::ArrowLeft,
        ] {
            let key = PhysicalKey::Code(key);
            assert!(ready.admits_key(InputSource::Native, key, true, false, false));
            assert!(!ready.admits_key(InputSource::Native, key, true, true, false));
            assert!(!ready.admits_key(InputSource::Native, key, true, false, true));
            for denied in [
                InputPolicy {
                    captured: false,
                    ..ready
                },
                InputPolicy {
                    focused: false,
                    ..ready
                },
                InputPolicy {
                    occluded: true,
                    ..ready
                },
            ] {
                assert!(!denied.admits_key(InputSource::Native, key, true, false, false));
                assert!(denied.admits_key(InputSource::Native, key, false, false, false));
                assert!(denied.admits_key(InputSource::Automation, key, true, false, false));
            }
            let tour = InputPolicy {
                gameplay: false,
                ..ready
            };
            if key != PhysicalKey::Code(KeyCode::Digit1) {
                assert!(!tour.admits_key(InputSource::Native, key, true, false, false));
                assert!(!tour.admits_key(InputSource::Automation, key, true, false, false));
            }
            assert!(tour.admits_key(InputSource::Native, key, false, false, false));
        }
        for key in [KeyCode::F2, KeyCode::F3] {
            assert!(!ready.admits_key(
                InputSource::Native,
                PhysicalKey::Code(key),
                true,
                false,
                true
            ));
        }
    }

    #[test]
    fn denied_steering_is_not_replayed_after_power_returns() {
        let mut input = ShipControlInput::default();
        let key = PhysicalKey::Code(KeyCode::KeyW);
        update_powered_ship_control_input(&mut input, key, true, true);
        assert_ne!(input.steering(), SteeringInput::default());
        update_powered_ship_control_input(&mut input, key, true, false);
        assert_eq!(input.steering(), SteeringInput::default());
        update_powered_ship_control_input(&mut input, key, false, true);
        assert_eq!(input.steering(), SteeringInput::default());
    }

    #[test]
    fn escape_releases_cursor_only_on_the_initial_press() {
        let escape = PhysicalKey::Code(KeyCode::Escape);
        assert!(release_cursor_pressed(ElementState::Pressed, false, escape));
        assert!(!release_cursor_pressed(ElementState::Pressed, true, escape));
        assert!(!release_cursor_pressed(
            ElementState::Released,
            false,
            escape
        ));
    }

    #[test]
    fn camera_keys_map_only_initial_physical_presses() {
        for (key, body) in [
            (KeyCode::Digit1, CelestialBodyId::Sun),
            (KeyCode::Digit2, CelestialBodyId::Mercury),
            (KeyCode::Digit3, CelestialBodyId::Venus),
            (KeyCode::Digit4, CelestialBodyId::Earth),
            (KeyCode::Digit5, CelestialBodyId::Moon),
            (KeyCode::Digit6, CelestialBodyId::Mars),
        ] {
            assert_eq!(
                camera_command(ElementState::Pressed, false, PhysicalKey::Code(key), true),
                Some(CameraCommand::InspectBody(body))
            );
            assert_eq!(
                camera_command(ElementState::Pressed, true, PhysicalKey::Code(key), true),
                None
            );
            assert_eq!(
                camera_command(ElementState::Released, false, PhysicalKey::Code(key), true),
                None
            );
        }
        assert_eq!(
            camera_command(
                ElementState::Pressed,
                false,
                PhysicalKey::Code(KeyCode::KeyP),
                true,
            ),
            Some(CameraCommand::TogglePause)
        );
        assert_eq!(
            camera_command(
                ElementState::Pressed,
                false,
                PhysicalKey::Code(KeyCode::KeyR),
                true,
            ),
            Some(CameraCommand::Restart)
        );
        assert_eq!(
            camera_command(
                ElementState::Pressed,
                false,
                PhysicalKey::Code(KeyCode::KeyN),
                true,
            ),
            Some(CameraCommand::JumpToNear)
        );
        assert_eq!(
            camera_command(
                ElementState::Pressed,
                true,
                PhysicalKey::Code(KeyCode::KeyP),
                true,
            ),
            None
        );
        assert_eq!(
            camera_command(
                ElementState::Released,
                false,
                PhysicalKey::Code(KeyCode::KeyN),
                true,
            ),
            None
        );
        assert_eq!(
            camera_command(
                ElementState::Pressed,
                false,
                PhysicalKey::Code(KeyCode::KeyW),
                true,
            ),
            None
        );
    }

    #[test]
    fn cockpit_keys_map_to_independent_held_axes() {
        let mut input = ShipControlInput::default();
        update_ship_control_input(&mut input, PhysicalKey::Code(KeyCode::KeyW), true);
        update_ship_control_input(&mut input, PhysicalKey::Code(KeyCode::KeyA), true);
        update_ship_control_input(&mut input, PhysicalKey::Code(KeyCode::ArrowRight), true);
        assert_eq!(input.steering().pitch, 1.0);
        assert_eq!(input.steering().yaw, 1.0);
        assert_eq!(input.steering().roll, 1.0);
        update_ship_control_input(&mut input, PhysicalKey::Code(KeyCode::KeyW), false);
        assert_eq!(input.steering().pitch, 0.0);
        assert!(ship_control_key(PhysicalKey::Code(KeyCode::ArrowUp)).is_none());
    }

    #[test]
    fn a_and_d_emit_the_corrected_opposite_yaw_directions() {
        let mut input = ShipControlInput::default();
        update_ship_control_input(&mut input, PhysicalKey::Code(KeyCode::KeyA), true);
        assert_eq!(input.steering().yaw, 1.0);

        update_ship_control_input(&mut input, PhysicalKey::Code(KeyCode::KeyA), false);
        update_ship_control_input(&mut input, PhysicalKey::Code(KeyCode::KeyD), true);
        assert_eq!(input.steering().yaw, -1.0);
    }

    #[test]
    fn throttle_keys_map_both_edges_and_ignore_repeat_and_focus_replay() {
        assert_eq!(
            throttle_direction(PhysicalKey::Code(KeyCode::ArrowUp)),
            Some(ThrottleDirection::Up)
        );
        assert_eq!(
            throttle_direction(PhysicalKey::Code(KeyCode::ArrowDown)),
            Some(ThrottleDirection::Down)
        );
    }
}

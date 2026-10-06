//! Native physical-key translation and held control state.

use salimon_character::MovementInput;
use salimon_ship::SteeringInput;
use salimon_world::{CameraCommand, CelestialBodyId};
use winit::event::ElementState;
use winit::keyboard::{KeyCode, PhysicalKey};

pub(super) fn release_cursor_pressed(state: ElementState, repeat: bool, key: PhysicalKey) -> bool {
    state == ElementState::Pressed && !repeat && key == PhysicalKey::Code(KeyCode::Escape)
}

pub(super) fn interaction_pressed(state: ElementState, repeat: bool, key: PhysicalKey) -> bool {
    state == ElementState::Pressed && !repeat && key == PhysicalKey::Code(KeyCode::KeyE)
}

pub(super) fn landing_action_pressed(state: ElementState, repeat: bool, key: PhysicalKey) -> bool {
    state == ElementState::Pressed && !repeat && key == PhysicalKey::Code(KeyCode::KeyL)
}

pub(super) fn view_toggle_pressed(
    state: ElementState,
    repeat: bool,
    key: PhysicalKey,
    is_synthetic: bool,
) -> bool {
    // Winit replays held keys on focus changes. A replay must not toggle views.
    state == ElementState::Pressed
        && !repeat
        && !is_synthetic
        && key == PhysicalKey::Code(KeyCode::F2)
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

pub(super) fn thruster_step(state: ElementState, repeat: bool, key: PhysicalKey) -> Option<i8> {
    if state != ElementState::Pressed || repeat {
        return None;
    }
    match key {
        PhysicalKey::Code(KeyCode::ArrowUp) => Some(1),
        PhysicalKey::Code(KeyCode::ArrowDown) => Some(-1),
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

pub(super) fn is_diagnostics_toggle(state: ElementState, repeat: bool, key: PhysicalKey) -> bool {
    state == ElementState::Pressed && !repeat && key == PhysicalKey::Code(KeyCode::F3)
}

pub(super) fn camera_command(
    state: ElementState,
    repeat: bool,
    key: PhysicalKey,
) -> Option<CameraCommand> {
    if state != ElementState::Pressed || repeat {
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
    fn focus_replayed_f2_does_not_toggle_the_view_again() {
        let f2 = PhysicalKey::Code(KeyCode::F2);
        assert!(view_toggle_pressed(ElementState::Pressed, false, f2, false));
        assert!(!view_toggle_pressed(ElementState::Pressed, false, f2, true));
        assert!(!view_toggle_pressed(ElementState::Pressed, true, f2, false));
        assert!(!view_toggle_pressed(
            ElementState::Released,
            false,
            f2,
            false
        ));
        assert!(!view_toggle_pressed(
            ElementState::Pressed,
            false,
            PhysicalKey::Code(KeyCode::F3),
            false,
        ));
    }

    #[test]
    fn f3_toggles_only_on_the_initial_press() {
        let f3 = PhysicalKey::Code(KeyCode::F3);

        assert!(is_diagnostics_toggle(ElementState::Pressed, false, f3));
        assert!(!is_diagnostics_toggle(ElementState::Pressed, true, f3));
        assert!(!is_diagnostics_toggle(ElementState::Released, false, f3));
        assert!(!is_diagnostics_toggle(
            ElementState::Pressed,
            false,
            PhysicalKey::Code(KeyCode::F2),
        ));
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
    fn landing_action_accepts_only_initial_l_press() {
        let landing = PhysicalKey::Code(KeyCode::KeyL);
        assert!(landing_action_pressed(
            ElementState::Pressed,
            false,
            landing
        ));
        assert!(!landing_action_pressed(
            ElementState::Pressed,
            true,
            landing
        ));
        assert!(!landing_action_pressed(
            ElementState::Released,
            false,
            landing
        ));
        assert!(!landing_action_pressed(
            ElementState::Pressed,
            false,
            PhysicalKey::Code(KeyCode::KeyK),
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
                camera_command(ElementState::Pressed, false, PhysicalKey::Code(key)),
                Some(CameraCommand::InspectBody(body))
            );
            assert_eq!(
                camera_command(ElementState::Pressed, true, PhysicalKey::Code(key)),
                None
            );
            assert_eq!(
                camera_command(ElementState::Released, false, PhysicalKey::Code(key)),
                None
            );
        }
        assert_eq!(
            camera_command(
                ElementState::Pressed,
                false,
                PhysicalKey::Code(KeyCode::KeyP),
            ),
            Some(CameraCommand::TogglePause)
        );
        assert_eq!(
            camera_command(
                ElementState::Pressed,
                false,
                PhysicalKey::Code(KeyCode::KeyR),
            ),
            Some(CameraCommand::Restart)
        );
        assert_eq!(
            camera_command(
                ElementState::Pressed,
                false,
                PhysicalKey::Code(KeyCode::KeyN),
            ),
            Some(CameraCommand::JumpToNear)
        );
        assert_eq!(
            camera_command(
                ElementState::Pressed,
                true,
                PhysicalKey::Code(KeyCode::KeyP),
            ),
            None
        );
        assert_eq!(
            camera_command(
                ElementState::Released,
                false,
                PhysicalKey::Code(KeyCode::KeyN),
            ),
            None
        );
        assert_eq!(
            camera_command(
                ElementState::Pressed,
                false,
                PhysicalKey::Code(KeyCode::KeyW),
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
    fn thruster_keys_accept_only_initial_arrow_presses() {
        let up = PhysicalKey::Code(KeyCode::ArrowUp);
        let down = PhysicalKey::Code(KeyCode::ArrowDown);
        assert_eq!(thruster_step(ElementState::Pressed, false, up), Some(1));
        assert_eq!(thruster_step(ElementState::Pressed, false, down), Some(-1));
        assert_eq!(thruster_step(ElementState::Pressed, true, up), None);
        assert_eq!(thruster_step(ElementState::Released, false, down), None);
    }
}

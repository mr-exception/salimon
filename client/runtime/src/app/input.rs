//! Native physical-key translation and held control state.

use salimon_character::MovementInput;
use salimon_ship::SteeringInput;
use salimon_world::{CameraCommand, CelestialBodyId};
use winit::event::ElementState;
use winit::keyboard::{KeyCode, PhysicalKey};

pub(super) fn release_cursor_pressed(state: ElementState, repeat: bool, key: PhysicalKey) -> bool {
    state == ElementState::Pressed && !repeat && key == PhysicalKey::Code(KeyCode::Escape)
}

/// Tool/carrying input accepts real edges only; focus replay cannot grab/drop.
pub(super) fn tool_key_event(repeat: bool, is_synthetic: bool, key: PhysicalKey) -> bool {
    !repeat && !is_synthetic && matches!(key, PhysicalKey::Code(KeyCode::KeyF))
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

pub(super) fn toolbar_selection_pressed(
    state: ElementState,
    repeat: bool,
    key: PhysicalKey,
    is_synthetic: bool,
) -> bool {
    state == ElementState::Pressed && !repeat && !is_synthetic && toolbar_slot(key).is_some()
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
    fn toolbar_keys_are_real_initial_edges_and_tour_numbers_are_gated() {
        for (index, key) in [
            KeyCode::Digit1,
            KeyCode::Digit2,
            KeyCode::Digit3,
            KeyCode::Digit4,
            KeyCode::Digit5,
        ]
        .into_iter()
        .enumerate()
        {
            let key = PhysicalKey::Code(key);
            assert_eq!(toolbar_slot(key).unwrap().index(), index);
            assert!(toolbar_selection_pressed(
                ElementState::Pressed,
                false,
                key,
                false
            ));
            assert!(!toolbar_selection_pressed(
                ElementState::Pressed,
                true,
                key,
                false
            ));
            assert!(!toolbar_selection_pressed(
                ElementState::Pressed,
                false,
                key,
                true
            ));
            assert!(!toolbar_selection_pressed(
                ElementState::Released,
                false,
                key,
                false
            ));
            assert_eq!(
                camera_command(ElementState::Pressed, false, key, false),
                None
            );
        }
        let six = PhysicalKey::Code(KeyCode::Digit6);
        assert_eq!(toolbar_slot(six), None);
        assert_eq!(
            camera_command(ElementState::Pressed, false, six, false),
            None
        );
    }

    #[test]
    fn f_carrying_ignores_repeat_focus_replay_and_other_keys() {
        let f = PhysicalKey::Code(KeyCode::KeyF);
        assert!(tool_key_event(false, false, f));
        assert!(!tool_key_event(
            false,
            false,
            PhysicalKey::Code(KeyCode::KeyM)
        ));
        assert!(!tool_key_event(true, false, f));
        assert!(!tool_key_event(false, true, f));
        assert!(!tool_key_event(
            false,
            false,
            PhysicalKey::Code(KeyCode::KeyE)
        ));
    }

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
    fn thruster_keys_accept_only_initial_arrow_presses() {
        let up = PhysicalKey::Code(KeyCode::ArrowUp);
        let down = PhysicalKey::Code(KeyCode::ArrowDown);
        assert_eq!(thruster_step(ElementState::Pressed, false, up), Some(1));
        assert_eq!(thruster_step(ElementState::Pressed, false, down), Some(-1));
        assert_eq!(thruster_step(ElementState::Pressed, true, up), None);
        assert_eq!(thruster_step(ElementState::Released, false, down), None);
    }
}

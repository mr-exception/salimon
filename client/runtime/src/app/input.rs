//! Native physical-key translation and held control state.

use salimon_ship::ThrottleDirection;
use salimon_world::{CameraCommand, CelestialBodyId};
use winit::event::ElementState;
use winit::keyboard::{KeyCode, PhysicalKey};

pub(super) fn release_cursor_pressed(state: ElementState, repeat: bool, key: PhysicalKey) -> bool {
    state == ElementState::Pressed
        && !repeat
        && matches!(key, PhysicalKey::Code(KeyCode::Escape | KeyCode::F1))
}

pub(super) fn terminal_restart_pressed(
    state: ElementState,
    repeat: bool,
    synthetic: bool,
    key: PhysicalKey,
) -> bool {
    state == ElementState::Pressed
        && !repeat
        && !synthetic
        && matches!(
            key,
            PhysicalKey::Code(KeyCode::KeyR | KeyCode::Enter | KeyCode::NumpadEnter)
        )
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

pub(super) fn throttle_direction(key: PhysicalKey) -> Option<ThrottleDirection> {
    match key {
        PhysicalKey::Code(KeyCode::ArrowUp) => Some(ThrottleDirection::Up),
        PhysicalKey::Code(KeyCode::ArrowDown) => Some(ThrottleDirection::Down),
        _ => None,
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

    if !precision_tour {
        return None;
    }

    match key {
        PhysicalKey::Code(KeyCode::KeyP) => Some(CameraCommand::TogglePause),
        PhysicalKey::Code(KeyCode::KeyR) => Some(CameraCommand::Restart),
        PhysicalKey::Code(KeyCode::KeyN) if precision_tour => Some(CameraCommand::JumpToNear),
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

/// Native key translation only; portable session owns command semantics.
pub(super) fn control_key(key: PhysicalKey) -> Option<crate::session::ControlKey> {
    use crate::session::ControlKey::*;
    Some(match key {
        PhysicalKey::Code(KeyCode::KeyW) => Forward,
        PhysicalKey::Code(KeyCode::KeyS) => Backward,
        PhysicalKey::Code(KeyCode::KeyA) => Left,
        PhysicalKey::Code(KeyCode::KeyD) => Right,
        PhysicalKey::Code(KeyCode::Space) => Jump,
        PhysicalKey::Code(KeyCode::ShiftLeft) => Descend,
        PhysicalKey::Code(KeyCode::ArrowLeft) => RollLeft,
        PhysicalKey::Code(KeyCode::ArrowRight) => RollRight,
        _ => return None,
    })
}

pub(crate) fn dispatch_session_key(
    session: &mut crate::session::GameplaySession,
    key: PhysicalKey,
    pressed: bool,
) {
    use crate::session::Command;
    if pressed
        && session.view_mode == crate::session::ViewMode::Gameplay
        && !session.is_paused()
        && !session.archive_reader.open
        && !session.crafting_ui.open
        && !session.wheel.open
    {
        use crate::session::construction::BuildAction as B;
        let action = match key {
            PhysicalKey::Code(KeyCode::KeyC) => Some(B::Toggle),
            PhysicalKey::Code(KeyCode::Escape) if session.building.active => Some(B::Toggle),
            PhysicalKey::Code(KeyCode::KeyR) if session.building.active => Some(B::Rotate),
            PhysicalKey::Code(code) if session.building.active => match code {
                KeyCode::Digit1 => Some(B::Select(0)),
                KeyCode::Digit2 => Some(B::Select(1)),
                KeyCode::Digit3 => Some(B::Select(2)),
                KeyCode::Digit4 => Some(B::Select(3)),
                KeyCode::Digit5 => Some(B::Select(4)),
                KeyCode::Digit6 => Some(B::Select(5)),
                KeyCode::Digit7 => Some(B::Select(6)),
                _ => None,
            },
            _ => None,
        };
        if let Some(action) = action {
            session.command(Command::Build(action));
            return;
        }
    }
    if pressed
        && key == PhysicalKey::Code(KeyCode::Escape)
        && session.view_mode == crate::session::ViewMode::Gameplay
    {
        session.command(Command::TogglePause);
        return;
    }
    if session.is_paused() {
        if pressed
            && matches!(
                key,
                PhysicalKey::Code(KeyCode::Enter | KeyCode::NumpadEnter)
            )
        {
            session.command(Command::TogglePause);
        }
        return;
    }
    if session.archive_reader.open {
        if pressed {
            use crate::session::archive::ReaderAction as A;
            let action = match key {
                PhysicalKey::Code(KeyCode::KeyE) => Some(A::Close),
                PhysicalKey::Code(KeyCode::KeyR) => Some(A::ToggleRecovery),
                PhysicalKey::Code(KeyCode::ArrowUp | KeyCode::ArrowLeft) => Some(A::Previous),
                PhysicalKey::Code(KeyCode::ArrowDown | KeyCode::ArrowRight | KeyCode::Tab) => {
                    Some(A::Next)
                }
                _ => None,
            };
            if let Some(action) = action {
                session.archive_action(action);
            }
        }
        return;
    }
    if key == PhysicalKey::Code(KeyCode::KeyQ) {
        session.command(Command::Wheel(
            crate::session::consumable_wheel::WheelAction::Hold(pressed),
        ));
        return;
    }
    if pressed && key == PhysicalKey::Code(KeyCode::KeyT) {
        session.command(Command::SkipTutorial);
        return;
    }
    if session.wheel.open {
        use crate::session::consumable_wheel::WheelAction as W;
        let action = match (key, pressed) {
            (PhysicalKey::Code(KeyCode::Tab | KeyCode::ArrowRight | KeyCode::ArrowDown), true) => {
                Some(W::Next)
            }
            (PhysicalKey::Code(KeyCode::ArrowLeft | KeyCode::ArrowUp), true) => Some(W::Previous),
            (PhysicalKey::Code(KeyCode::Enter | KeyCode::NumpadEnter), true) => Some(W::Confirm),
            (PhysicalKey::Code(KeyCode::Escape), true) => Some(W::Cancel),
            (PhysicalKey::Code(KeyCode::F2), true) => {
                session.command(Command::ToggleView);
                return;
            }
            _ => None,
        };
        if let Some(action) = action {
            session.command(Command::Wheel(action));
        }
        return;
    }
    if session.crafting_ui.open {
        if pressed {
            use crate::session::crafting_ui::CraftingAction as A;
            let action = match key {
                PhysicalKey::Code(KeyCode::Escape | KeyCode::KeyE) => Some(A::Close),
                PhysicalKey::Code(KeyCode::ArrowUp) => Some(A::PreviousRecipe),
                PhysicalKey::Code(KeyCode::ArrowDown) => Some(A::NextRecipe),
                PhysicalKey::Code(KeyCode::ArrowLeft) => Some(A::Less),
                PhysicalKey::Code(KeyCode::ArrowRight) => Some(A::More),
                PhysicalKey::Code(KeyCode::Tab) => Some(A::NextJob),
                PhysicalKey::Code(KeyCode::Delete) => Some(A::Cancel),
                PhysicalKey::Code(KeyCode::Enter) => Some(A::Add),
                _ => None,
            };
            if let Some(action) = action {
                session.crafting_action(action);
            }
        }
        return;
    }
    let command = if !pressed {
        if let Some(control) = control_key(key) {
            Some(Command::Control(control, false))
        } else if let Some(direction) = throttle_direction(key) {
            Some(Command::Throttle(direction, false))
        } else if key == PhysicalKey::Code(KeyCode::KeyF) {
            Some(Command::GrabDrop(false))
        } else {
            None
        }
    } else if key == PhysicalKey::Code(KeyCode::F2) {
        Some(Command::ToggleView)
    } else if session.view_mode != crate::session::ViewMode::Gameplay {
        camera_command(ElementState::Pressed, false, key, true).map(Command::Camera)
    } else if key == PhysicalKey::Code(KeyCode::KeyH) {
        Some(Command::InspectVitals)
    } else if key == PhysicalKey::Code(KeyCode::KeyE) {
        Some(Command::Interact)
    } else if key == PhysicalKey::Code(KeyCode::KeyB) {
        Some(Command::ToggleEngines)
    } else if key == PhysicalKey::Code(KeyCode::KeyL) {
        Some(Command::Landing)
    } else if let Some(camera) = camera_command(ElementState::Pressed, false, key, false) {
        Some(Command::Camera(camera))
    } else if let Some(direction) = throttle_direction(key) {
        Some(Command::Throttle(direction, true))
    } else if let Some(slot) = toolbar_slot(key) {
        Some(Command::Select(slot))
    } else if key == PhysicalKey::Code(KeyCode::KeyI) {
        Some(Command::CoreUpgrade(
            crate::session::core_upgrades::CoreUpgradeAction::ToggleInput,
        ))
    } else if key == PhysicalKey::Code(KeyCode::KeyY) {
        Some(Command::CoreUpgrade(
            crate::session::core_upgrades::CoreUpgradeAction::Apply,
        ))
    } else if key == PhysicalKey::Code(KeyCode::KeyN) {
        Some(Command::CoreUpgrade(
            crate::session::core_upgrades::CoreUpgradeAction::Recover,
        ))
    } else if key == PhysicalKey::Code(KeyCode::KeyV) {
        Some(Command::Finish(
            crate::session::finishes::FinishAction::SelectGroup,
        ))
    } else if key == PhysicalKey::Code(KeyCode::KeyO) {
        Some(Command::Finish(
            crate::session::finishes::FinishAction::Next,
        ))
    } else if key == PhysicalKey::Code(KeyCode::KeyZ) {
        Some(Command::Finish(
            crate::session::finishes::FinishAction::Reset,
        ))
    } else if key == PhysicalKey::Code(KeyCode::KeyG) {
        Some(Command::Mount(crate::session::mounts::MountAction::Place))
    } else if key == PhysicalKey::Code(KeyCode::KeyX) {
        Some(Command::Cabin(crate::session::cabin::CabinAction::Rest))
    } else if key == PhysicalKey::Code(KeyCode::KeyU) {
        Some(Command::Cabin(
            crate::session::cabin::CabinAction::Restraint,
        ))
    } else if key == PhysicalKey::Code(KeyCode::KeyJ) {
        Some(Command::Mount(
            crate::session::mounts::MountAction::MoveAccessory,
        ))
    } else if key == PhysicalKey::Code(KeyCode::KeyK) {
        Some(Command::Mount(
            crate::session::mounts::MountAction::ToggleLock,
        ))
    } else if key == PhysicalKey::Code(KeyCode::KeyF) {
        Some(Command::GrabDrop(true))
    } else {
        control_key(key).map(|key| Command::Control(key, true))
    };
    if let Some(command) = command {
        session.command(command);
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

#[cfg(test)]
mod crafting_tests {
    use super::*;
    #[test]
    fn initial_ui_edges_dispatch_once_and_losing_focus_closes_modal() {
        let policy = InputPolicy {
            gameplay: true,
            captured: true,
            focused: true,
            occluded: false,
        };
        let mut s = crate::session::GameplaySession::default();
        s.crafting_ui.open = true;
        for r in salimon_world::resources::RESOURCE_CATALOG {
            s.execute_fabricator(salimon_fabrication::FabricationCommand::Insert(
                salimon_world::resources::RawMaterial::new(r.id, 2.0).unwrap(),
            ))
            .unwrap();
        }
        for (repeat, synthetic) in [(false, false), (true, false), (false, true)] {
            let key = PhysicalKey::Code(KeyCode::Enter);
            if policy.admits_key(InputSource::Native, key, true, repeat, synthetic) {
                dispatch_session_key(&mut s, key, true);
            }
        }
        assert!(s.crafting_text().unwrap().contains("QUEUE: 1 /"));
        dispatch_session_key(&mut s, PhysicalKey::Code(KeyCode::KeyW), true);
        assert!(!s.movement_input.forward);
        s.clear_held_controls();
        assert!(!s.crafting_ui.open);
    }
}

#[cfg(test)]
mod consumable_tests {
    use super::*;
    #[test]
    fn mn_no_longer_dispatch_consumption_or_change_gameplay_view() {
        let mut session = crate::session::GameplaySession::default();
        for key in [KeyCode::KeyM, KeyCode::KeyN] {
            dispatch_session_key(&mut session, PhysicalKey::Code(key), true);
            dispatch_session_key(&mut session, PhysicalKey::Code(key), false);
        }
        assert_eq!(session.view_mode, crate::session::ViewMode::Gameplay);
        assert!(session.feedback.is_none());
    }
}

#[cfg(test)]
mod wheel_input_tests {
    use super::*;
    #[test]
    fn native_q_repeat_synthetic_modal_and_release_admission_share_production_dispatch() {
        let mut s = crate::session::GameplaySession::default();
        let ready = InputPolicy {
            gameplay: true,
            captured: true,
            focused: true,
            occluded: false,
        };
        let q = PhysicalKey::Code(KeyCode::KeyQ);
        for (repeat, synthetic) in [(false, false), (true, false), (false, true)] {
            if ready.admits_key(InputSource::Native, q, true, repeat, synthetic) {
                dispatch_session_key(&mut s, q, true);
            }
        }
        assert!(s.wheel.open);
        for key in [KeyCode::KeyW, KeyCode::KeyE, KeyCode::KeyF, KeyCode::Digit1] {
            dispatch_session_key(&mut s, PhysicalKey::Code(key), true);
        }
        assert!(!s.movement_input.forward);
        assert!(s.equipment.selected().is_none());
        assert!(!s.crafting_ui.open);
        dispatch_session_key(&mut s, q, false);
        assert!(!s.wheel.open);
        assert!(!s.wheel.held);
        dispatch_session_key(&mut s, q, true);
        s.clear_held_controls();
        assert!(!s.wheel.open);
        for policy in [
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
            assert!(!policy.admits_key(InputSource::Native, q, true, false, false));
            assert!(policy.admits_key(InputSource::Native, q, false, false, false));
        }
    }
}

#[cfg(test)]
mod engine_tests {
    #[test]
    fn native_press_admission_and_modals_gate_engine_toggles() {
        use super::*;
        use crate::session::{Command, consumable_wheel};
        use winit::keyboard::{KeyCode, PhysicalKey};
        let mut s = crate::session::GameplaySession::new_run();
        s.character.enter_cockpit();
        s.ship = salimon_ship::ShipController::flying(s.ship.snapshot().pose, 70);
        let key = PhysicalKey::Code(KeyCode::KeyB);
        let policy = InputPolicy {
            gameplay: true,
            captured: true,
            focused: true,
            occluded: false,
        };
        for (repeat, synthetic) in [(false, false), (true, false), (false, true)] {
            if policy.admits_key(InputSource::Native, key, true, repeat, synthetic) {
                dispatch_session_key(&mut s, key, true);
            }
        }
        assert!(!s.ship.snapshot().engines_enabled);
        s.command(Command::TogglePause);
        s.command(Command::ToggleEngines);
        assert!(!s.ship.snapshot().engines_enabled);
        s.command(Command::TogglePause);
        s.command(Command::Wheel(consumable_wheel::WheelAction::Hold(true)));
        s.command(Command::ToggleEngines);
        assert!(!s.ship.snapshot().engines_enabled);
        s.clear_held_controls();
        s.command(Command::ToggleEngines);
        assert!(s.ship.snapshot().engines_enabled);
        s.ship.set_cockpit_control(false);
        s.command(Command::ToggleEngines);
        assert!(s.ship.snapshot().engines_enabled);
    }
}

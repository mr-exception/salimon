//! Native physical-key translation and held control state.

#[cfg(test)]
use salimon_ship::ThrottleDirection;
use salimon_world::CameraCommand;
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

#[cfg(test)]
fn throttle_direction(key: PhysicalKey) -> Option<ThrottleDirection> {
    session_key(key).and_then(crate::session::input::keys::throttle_direction)
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
    session_key(key)
        .and_then(|key| crate::session::input::keys::camera_command(key, precision_tour))
}

/// Native key translation only; portable session owns command semantics.
pub(crate) fn dispatch_session_key(
    session: &mut crate::session::GameplaySession,
    key: PhysicalKey,
    pressed: bool,
) {
    if let Some(key) = session_key(key) {
        crate::session::input::dispatch(session, key, pressed);
    }
}

fn session_key(key: PhysicalKey) -> Option<crate::session::input::Key> {
    use crate::session::input::Key;
    Some(match key {
        PhysicalKey::Code(KeyCode::KeyW) => Key::Forward,
        PhysicalKey::Code(KeyCode::KeyS) => Key::Backward,
        PhysicalKey::Code(KeyCode::KeyA) => Key::Left,
        PhysicalKey::Code(KeyCode::KeyD) => Key::Right,
        PhysicalKey::Code(KeyCode::Space) => Key::Jump,
        PhysicalKey::Code(KeyCode::ShiftLeft) => Key::Descend,
        PhysicalKey::Code(KeyCode::ArrowUp) => Key::Up,
        PhysicalKey::Code(KeyCode::ArrowDown) => Key::Down,
        PhysicalKey::Code(KeyCode::ArrowLeft) => Key::Previous,
        PhysicalKey::Code(KeyCode::ArrowRight) => Key::Next,
        PhysicalKey::Code(KeyCode::KeyC) => Key::Build,
        PhysicalKey::Code(KeyCode::Escape) => Key::Cancel,
        PhysicalKey::Code(KeyCode::KeyR) => Key::Recovery,
        PhysicalKey::Code(KeyCode::KeyH) => Key::Status,
        PhysicalKey::Code(KeyCode::KeyX) => Key::Rest,
        PhysicalKey::Code(KeyCode::KeyG) => Key::Place,
        PhysicalKey::Code(KeyCode::KeyT) => Key::Tutorial,
        PhysicalKey::Code(KeyCode::Delete) => Key::Delete,
        PhysicalKey::Code(KeyCode::Enter) => Key::Confirm,
        PhysicalKey::Code(KeyCode::NumpadEnter) => Key::ConfirmAlternate,
        PhysicalKey::Code(KeyCode::KeyE) => Key::Interact,
        PhysicalKey::Code(KeyCode::Tab) => Key::Tab,
        PhysicalKey::Code(KeyCode::KeyQ) => Key::Wheel,
        PhysicalKey::Code(KeyCode::F2) => Key::View,
        PhysicalKey::Code(KeyCode::F3) => Key::Diagnostics,
        PhysicalKey::Code(KeyCode::F4) => Key::LightPreset,
        PhysicalKey::Code(KeyCode::F5) => Key::LightMaster,
        PhysicalKey::Code(KeyCode::F6) => Key::LightFixture,
        PhysicalKey::Code(KeyCode::F7) => Key::LightBrightness,
        PhysicalKey::Code(KeyCode::F8) => Key::LightWarmth,
        PhysicalKey::Code(KeyCode::F9) => Key::LightGeneral,
        PhysicalKey::Code(KeyCode::KeyB) => Key::Engines,
        PhysicalKey::Code(KeyCode::KeyL) => Key::Landing,
        PhysicalKey::Code(KeyCode::KeyI) => Key::CoreInput,
        PhysicalKey::Code(KeyCode::KeyY) => Key::CoreApply,
        PhysicalKey::Code(KeyCode::KeyN) => Key::CoreRecover,
        PhysicalKey::Code(KeyCode::KeyV) => Key::FinishGroup,
        PhysicalKey::Code(KeyCode::KeyO) => Key::FinishNext,
        PhysicalKey::Code(KeyCode::KeyZ) => Key::FinishReset,
        PhysicalKey::Code(KeyCode::KeyU) => Key::Restraint,
        PhysicalKey::Code(KeyCode::KeyJ) => Key::MoveAccessory,
        PhysicalKey::Code(KeyCode::KeyK) => Key::Secure,
        PhysicalKey::Code(KeyCode::KeyF) => Key::GrabDrop,
        PhysicalKey::Code(KeyCode::KeyP) => Key::TourPause,
        PhysicalKey::Code(KeyCode::KeyM) => Key::ScannerNext,
        PhysicalKey::Code(KeyCode::Digit1) => Key::Slot1,
        PhysicalKey::Code(KeyCode::Digit2) => Key::Slot2,
        PhysicalKey::Code(KeyCode::Digit3) => Key::Slot3,
        PhysicalKey::Code(KeyCode::Digit4) => Key::Slot4,
        PhysicalKey::Code(KeyCode::Digit5) => Key::Slot5,
        PhysicalKey::Code(KeyCode::Digit6) => Key::Slot6,
        PhysicalKey::Code(KeyCode::Digit7) => Key::Slot7,
        _ => return None,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use salimon_world::CameraCommand;
    #[cfg(test)]
    use salimon_world::CelestialBodyId;
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
        assert!(!s.input.movement().forward);
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
        assert!(!s.input.movement().forward);
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

//! Ship-local interaction gates and contextual prompt selection.

use super::native::WINDOW_TITLE;
use crate::session::interaction::{
    COCKPIT_INTERACTION_POSITION_METERS, COCKPIT_INTERACTION_PROMPT, InteractionTarget,
};
#[cfg(test)]
use crate::session::interaction::{available_interaction_target, interaction_target};
#[cfg(test)]
use salimon_character::CharacterLocation;
use salimon_ship::CockpitMessage;

pub(crate) fn prompt_placement(
    target: InteractionTarget,
    frame: salimon_character::ShipFrame,
) -> salimon_renderer::OverlayPlacement {
    let (local, radius_meters) = match target {
        InteractionTarget::CabinTable
        | InteractionTarget::CabinSeat
        | InteractionTarget::PrivacyScreen
        | InteractionTarget::ObservationShutter
        | InteractionTarget::CabinLocker => (crate::session::cabin::default_position(target), 0.15),
        InteractionTarget::Bunk | InteractionTarget::Galley | InteractionTarget::Hygiene => (
            crate::session::interaction::habitation_position(target),
            0.25,
        ),
        InteractionTarget::CargoDoor => (salimon_character::cargo::CARGO_DOOR_TARGET, 0.22),
        InteractionTarget::Archive => (crate::session::archive::TERMINAL, 0.15),
        InteractionTarget::FabricatorTray => {
            (salimon_character::FABRICATOR_TRAY_MARKER_METERS, 0.15)
        }
        InteractionTarget::FabricatorScreen => {
            (salimon_character::FABRICATOR_MONITOR_MARKER_METERS, 0.2)
        }
        InteractionTarget::Fabricator => {
            (crate::session::interaction::FABRICATOR_INPUT_METERS, 0.15)
        }
        InteractionTarget::Core => (salimon_character::CORE_INTERACTION_POSITION_METERS, 0.2),
        InteractionTarget::Cockpit => (COCKPIT_INTERACTION_POSITION_METERS, 0.1),
        InteractionTarget::ExitDoor => (
            salimon_character::EXIT_DOOR_MARKER_METERS,
            salimon_character::EXIT_DOOR_MARKER_DEPTH_METERS,
        ),
    };
    salimon_renderer::OverlayPlacement::World {
        anchor_meters: frame.local_to_world(local),
        radius_meters,
    }
}

pub(crate) fn gameplay_window_title(
    monitor_message: Option<CockpitMessage>,
    interaction: Option<InteractionTarget>,
) -> String {
    if let Some(message) = monitor_message {
        format!("{WINDOW_TITLE} — {}", message.text())
    } else if interaction == Some(InteractionTarget::Cockpit) {
        format!("{WINDOW_TITLE} — {COCKPIT_INTERACTION_PROMPT}")
    } else {
        WINDOW_TITLE.to_owned()
    }
}

pub(crate) fn action_bar_context(
    monitor_message: Option<CockpitMessage>,
    interaction: Option<InteractionTarget>,
) -> Option<&'static str> {
    monitor_message
        .filter(|message| *message != CockpitMessage::DoorLockedWhileInFlight)
        .map(CockpitMessage::text)
        .or(match interaction {
            Some(InteractionTarget::CabinTable) => Some("[E] Fold/unfold table"),
            Some(InteractionTarget::CabinSeat) => {
                Some("[E] Fold/unfold seat [X] Sit [U] Restraint")
            }
            Some(InteractionTarget::PrivacyScreen) => Some("[E] Slide privacy screen"),
            Some(InteractionTarget::ObservationShutter) => {
                Some("[E] Slide observation shutter (manual)")
            }
            Some(InteractionTarget::CabinLocker) => Some("[E] Open/close physical locker"),
            Some(InteractionTarget::Bunk) => Some("[E] Unfold/stow bunk [X] Lie [U] Restraint"),
            Some(InteractionTarget::Galley) => Some("[E] Use galley"),
            Some(InteractionTarget::Hygiene) => Some("[E] Inspect wash station"),
            Some(InteractionTarget::CargoDoor) => Some("[E] Open/close cargo door (manual)"),
            Some(InteractionTarget::Archive) => Some("[E] Read Scout Archive"),
            Some(InteractionTarget::FabricatorTray) => Some("[E] Collect Fabricator tray item"),
            Some(InteractionTarget::FabricatorScreen) => Some("[E] Open Fabricator crafting"),
            Some(InteractionTarget::Fabricator) => Some("[E] Insert cargo into Fabricator"),
            Some(InteractionTarget::Core) => Some("[E] Feed cargo to Core"),
            Some(InteractionTarget::Cockpit) => Some(COCKPIT_INTERACTION_PROMPT),
            Some(InteractionTarget::ExitDoor) => Some("[E] Open/close exit door"),
            None => None,
        })
}

#[cfg(test)]
mod tests {
    use super::*;
    use salimon_ship::{FlightState, ShipController};
    use salimon_world::CelestialBodyId;
    #[test]
    fn core_uses_authored_center_and_requires_nearby_interior_aim() {
        let core = salimon_character::CORE_INTERACTION_POSITION_METERS;
        let eye = [-1.0, 2.0, -2.2];
        assert_eq!(
            available_interaction_target(CharacterLocation::InsideShip, eye, core),
            Some(InteractionTarget::Core)
        );
        assert_eq!(
            available_interaction_target(CharacterLocation::Space, eye, core),
            None
        );
        assert_ne!(
            available_interaction_target(CharacterLocation::InsideShip, [-1.0, 2.0, -5.0], core),
            Some(InteractionTarget::Core)
        );
    }

    #[test]
    fn prompt_anchors_follow_authored_markers_through_ship_translation_and_rotation() {
        let frame = salimon_character::ShipFrame {
            origin_meters: [1.0e12, 20.0, 30.0],
            axes: [[0.0, 0.0, -1.0], [0.0, 1.0, 0.0], [1.0, 0.0, 0.0]],
        };
        for (target, local) in [
            (
                InteractionTarget::Cockpit,
                COCKPIT_INTERACTION_POSITION_METERS,
            ),
            (
                InteractionTarget::ExitDoor,
                salimon_character::EXIT_DOOR_MARKER_METERS,
            ),
        ] {
            let salimon_renderer::OverlayPlacement::World { anchor_meters, .. } =
                prompt_placement(target, frame)
            else {
                panic!("object prompt must be world anchored")
            };
            assert_eq!(anchor_meters, frame.local_to_world(local));
        }
    }

    #[test]
    fn interaction_zones_follow_seat_aisle_and_rear_door() {
        let aisle = [0.50, 1.997_311_827_956_989_2, -2.20];
        assert_eq!(
            interaction_target(aisle, [2.76, 1.799_032_258_064_516, 0.0]),
            Some(InteractionTarget::Cockpit)
        );
        assert_eq!(
            interaction_target(aisle, [-1.76, 2.195_591_397_849_462, -4.40]),
            None,
        );
        assert_eq!(
            interaction_target(
                [-7.10, 1.344_086_021_505_376_3, 0.0],
                [-8.10, 1.344_086_021_505_376_3, 0.0],
            ),
            Some(InteractionTarget::ExitDoor)
        );
        assert_eq!(
            interaction_target(
                [-2.0, 1.997_311_827_956_989_2, 3.0],
                [2.76, 1.799_032_258_064_516, 0.0],
            ),
            None
        );
        assert_eq!(
            interaction_target(
                [-7.0, 1.997_311_827_956_989_2, 4.0],
                [-8.0, 1.997_311_827_956_989_2, 4.0],
            ),
            None
        );
    }

    #[test]
    fn cockpit_interaction_requires_ship_interior_but_exterior_door_remains_usable() {
        let aisle = [0.50, 1.997_311_827_956_989_2, -2.20];
        let cockpit = [2.76, 1.799_032_258_064_516, 0.0];
        assert_eq!(
            available_interaction_target(CharacterLocation::InsideShip, aisle, cockpit),
            Some(InteractionTarget::Cockpit)
        );
        for location in [CharacterLocation::DoorwayBlend, CharacterLocation::Surface] {
            assert_eq!(available_interaction_target(location, aisle, cockpit), None);
            assert_eq!(
                available_interaction_target(
                    location,
                    [-7.10, 1.344_086_021_505_376_3, 0.0],
                    [-8.10, 1.344_086_021_505_376_3, 0.0],
                ),
                Some(InteractionTarget::ExitDoor)
            );
        }
    }

    #[test]
    fn cockpit_prompt_is_shown_only_for_the_aimed_nearby_cockpit() {
        let ship = ShipController::default().snapshot();
        assert_eq!(
            ship.flight_state,
            FlightState::Landed {
                body: CelestialBodyId::Earth
            }
        );
        assert_eq!(
            gameplay_window_title(None, Some(InteractionTarget::Cockpit)),
            format!("Salimon — {COCKPIT_INTERACTION_PROMPT}")
        );
        assert_eq!(gameplay_window_title(None, None), "Salimon");
    }

    #[test]
    fn action_bar_context_tracks_ship_prompt_state_transitions() {
        let mut landed = ShipController::default();
        assert_eq!(
            action_bar_context(landed.contextual_cockpit_message(), None),
            None
        );
        assert_eq!(
            action_bar_context(None, Some(InteractionTarget::ExitDoor)),
            Some("[E] Open/close exit door")
        );

        landed.set_cockpit_control(true);
        assert_eq!(
            action_bar_context(landed.contextual_cockpit_message(), None),
            Some("Press L to take off")
        );
        landed.toggle_door();
        landed.trigger_landing_action();
        assert_eq!(
            action_bar_context(landed.contextual_cockpit_message(), None),
            Some("Close door before takeoff")
        );
        landed.toggle_door();
        assert_eq!(
            action_bar_context(landed.contextual_cockpit_message(), None),
            Some("Press L to take off")
        );
        landed.set_cockpit_control(false);
        assert_eq!(
            action_bar_context(landed.contextual_cockpit_message(), None),
            None
        );

        let pose = ShipController::default().snapshot().pose;
        let flying = ShipController::flying(pose, 0);
        assert_eq!(
            action_bar_context(flying.contextual_cockpit_message(), None),
            Some("Press L to land")
        );
        assert_eq!(
            action_bar_context(
                Some(CockpitMessage::DoorLockedWhileInFlight),
                Some(super::InteractionTarget::Cockpit),
            ),
            Some(super::COCKPIT_INTERACTION_PROMPT),
            "timed blocked feedback is owned by ActionBar, not stale ship state"
        );
    }
}

#[cfg(test)]
mod tray_anchor_tests {
    use super::*;
    #[test]
    fn tray_anchor_tracks_current_rotated_ship_pose() {
        let frame = salimon_character::ShipFrame {
            origin_meters: [1e12, 20.0, 30.0],
            axes: [[0.0, 0.0, -1.0], [0.0, 1.0, 0.0], [1.0, 0.0, 0.0]],
        };
        assert_eq!(
            prompt_placement(InteractionTarget::FabricatorTray, frame),
            salimon_renderer::OverlayPlacement::World {
                anchor_meters: frame
                    .local_to_world(salimon_character::FABRICATOR_TRAY_MARKER_METERS),
                radius_meters: 0.15
            }
        );
    }
}

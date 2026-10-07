//! Ship-local interaction gates and contextual prompt selection.

use super::native::WINDOW_TITLE;
use salimon_character::CharacterLocation;
use salimon_ship::CockpitMessage;

const COCKPIT_INTERACTION_POSITION_METERS: [f64; 3] = [
    salimon_character::COCKPIT_SEAT_MARKER_METERS[0],
    salimon_character::COCKPIT_SEAT_MARKER_METERS[1] + 0.67,
    salimon_character::COCKPIT_SEAT_MARKER_METERS[2],
];
const COCKPIT_INTERACTION_RANGE_METERS: f64 = 4.0;
const COCKPIT_INTERACTION_MINIMUM_AIM_DOT: f64 = 0.866_025_403_784_438_6;
const COCKPIT_INTERACTION_PROMPT: &str = "Press E to use";

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(super) enum InteractionTarget {
    Cockpit,
    ExitDoor,
}

pub(super) fn prompt_placement(
    target: InteractionTarget,
    frame: salimon_character::ShipFrame,
) -> salimon_renderer::OverlayPlacement {
    let (local, radius_meters) = match target {
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

fn interaction_target(
    local_eye_position: [f64; 3],
    local_look_target: [f64; 3],
) -> Option<InteractionTarget> {
    let to_cockpit = subtract(COCKPIT_INTERACTION_POSITION_METERS, local_eye_position);
    let look_direction = subtract(local_look_target, local_eye_position);
    let cockpit_distance = vector_length(to_cockpit);
    let cockpit_is_aimed_at = cockpit_distance > f64::EPSILON
        && cockpit_distance <= COCKPIT_INTERACTION_RANGE_METERS
        && dot(to_cockpit, look_direction)
            / (cockpit_distance * vector_length(look_direction)).max(f64::EPSILON)
            >= COCKPIT_INTERACTION_MINIMUM_AIM_DOT;

    if cockpit_is_aimed_at {
        Some(InteractionTarget::Cockpit)
    } else if (local_eye_position[0] + 7.10).abs() < 2.7 && local_eye_position[2].abs() < 2.3 {
        Some(InteractionTarget::ExitDoor)
    } else {
        None
    }
}

pub(super) fn available_interaction_target(
    location: CharacterLocation,
    local_eye_position: [f64; 3],
    local_look_target: [f64; 3],
) -> Option<InteractionTarget> {
    match interaction_target(local_eye_position, local_look_target) {
        Some(InteractionTarget::Cockpit) if location != CharacterLocation::InsideShip => None,
        target => target,
    }
}

fn subtract(left: [f64; 3], right: [f64; 3]) -> [f64; 3] {
    std::array::from_fn(|index| left[index] - right[index])
}

fn dot(left: [f64; 3], right: [f64; 3]) -> f64 {
    left.into_iter().zip(right).map(|(a, b)| a * b).sum()
}

fn vector_length(vector: [f64; 3]) -> f64 {
    dot(vector, vector).sqrt()
}

pub(super) fn gameplay_window_title(
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

pub(super) fn action_bar_context(
    monitor_message: Option<CockpitMessage>,
    interaction: Option<InteractionTarget>,
) -> Option<&'static str> {
    monitor_message
        .filter(|message| *message != CockpitMessage::DoorLockedWhileInFlight)
        .map(CockpitMessage::text)
        .or(match interaction {
            Some(InteractionTarget::Cockpit) => Some(COCKPIT_INTERACTION_PROMPT),
            Some(InteractionTarget::ExitDoor) => Some("E to open/close exit door"),
            None => None,
        })
}

#[cfg(test)]
mod tests {
    use super::*;
    use salimon_ship::{FlightState, ShipController};
    use salimon_world::CelestialBodyId;
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
            format!("Salimon — Compressed Solar System — {COCKPIT_INTERACTION_PROMPT}")
        );
        assert_eq!(
            gameplay_window_title(None, None),
            "Salimon — Compressed Solar System"
        );
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
            Some("E to open/close exit door")
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

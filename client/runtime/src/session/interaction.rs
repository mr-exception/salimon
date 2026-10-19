//! Portable authored interaction gates shared by commands and presentation.
use salimon_character::CharacterLocation;
pub(crate) const COCKPIT_INTERACTION_POSITION_METERS: [f64; 3] = [
    salimon_character::COCKPIT_SEAT_MARKER_METERS[0],
    salimon_character::COCKPIT_SEAT_MARKER_METERS[1] + 0.67,
    salimon_character::COCKPIT_SEAT_MARKER_METERS[2],
];
const COCKPIT_INTERACTION_RANGE_METERS: f64 = 4.0;
const COCKPIT_INTERACTION_MINIMUM_AIM_DOT: f64 = 0.866_025_403_784_438_6;
pub(crate) const COCKPIT_INTERACTION_PROMPT: &str = "Press E to use";

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum InteractionTarget {
    Fabricator,
    FabricatorScreen,
    FabricatorTray,
    Cockpit,
    Core,
    ExitDoor,
}

pub(crate) fn interaction_target(
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

    let to_core = subtract(
        salimon_character::CORE_INTERACTION_POSITION_METERS,
        local_eye_position,
    );
    let core_distance = vector_length(to_core);
    let core_aimed = core_distance > f64::EPSILON
        && core_distance <= 3.0
        && dot(to_core, look_direction)
            / (core_distance * vector_length(look_direction)).max(f64::EPSILON)
            >= COCKPIT_INTERACTION_MINIMUM_AIM_DOT;
    if core_aimed {
        Some(InteractionTarget::Core)
    } else if cockpit_is_aimed_at {
        Some(InteractionTarget::Cockpit)
    } else if ((local_eye_position[0] + 7.10).abs() < 2.7
        || (local_eye_position[0] >= salimon_character::boarding_ramp_tip_meters()[0] - 1.5
            && local_eye_position[0] < -7.10))
        && local_eye_position[2].abs() < 2.3
    {
        Some(InteractionTarget::ExitDoor)
    } else if machine_target(
        local_eye_position,
        local_look_target,
        salimon_character::FABRICATOR_TRAY_MARKER_METERS,
    ) {
        Some(InteractionTarget::FabricatorTray)
    } else if machine_target(
        local_eye_position,
        local_look_target,
        salimon_character::FABRICATOR_MONITOR_MARKER_METERS,
    ) {
        Some(InteractionTarget::FabricatorScreen)
    } else if fabricator_target(local_eye_position, local_look_target) {
        Some(InteractionTarget::Fabricator)
    } else {
        None
    }
}

pub(crate) fn available_interaction_target(
    location: CharacterLocation,
    local_eye_position: [f64; 3],
    local_look_target: [f64; 3],
) -> Option<InteractionTarget> {
    match interaction_target(local_eye_position, local_look_target) {
        Some(
            InteractionTarget::Cockpit
            | InteractionTarget::Core
            | InteractionTarget::Fabricator
            | InteractionTarget::FabricatorScreen
            | InteractionTarget::FabricatorTray,
        ) if location != CharacterLocation::InsideShip => None,
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

/// Canonical authored input face, generated from the Blender source.
pub(crate) const FABRICATOR_INPUT_METERS: [f64; 3] =
    salimon_character::FABRICATOR_INPUT_MARKER_METERS;

fn fabricator_target(eye: [f64; 3], look: [f64; 3]) -> bool {
    machine_target(eye, look, FABRICATOR_INPUT_METERS)
}

pub(super) fn machine_target(eye: [f64; 3], look: [f64; 3], target: [f64; 3]) -> bool {
    let distance = vector_length(subtract(target, eye));
    let direction = subtract(look, eye);
    // Intersect the cabin-facing plane; disjoint authored hit regions prevent
    // a broad aim cone from routing the same E press to both port and monitor.
    let t = (target[2] - eye[2]) / direction[2];
    let hit = std::array::from_fn::<_, 3, _>(|i| eye[i] + t * direction[i]);
    distance > f64::EPSILON
        && distance <= 3.0
        && t > 0.0
        && eye[2] < target[2]
        && (hit[0] - target[0]).abs() <= 0.30
        && (hit[1] - target[1]).abs() <= 0.22
        && salimon_character::fabricator_sight_obstruction(eye, target)
            .is_none_or(|obstruction| obstruction >= distance - 0.02)
}

#[cfg(test)]
mod machine_tests {
    use super::*;
    #[test]
    fn input_and_monitor_are_disjoint_and_obey_aim_distance_and_solid_occlusion() {
        let eye = [-3.0, 1.65, 2.0];
        for (point, expected) in [
            (FABRICATOR_INPUT_METERS, InteractionTarget::Fabricator),
            (
                salimon_character::FABRICATOR_MONITOR_MARKER_METERS,
                InteractionTarget::FabricatorScreen,
            ),
        ] {
            assert_eq!(
                available_interaction_target(CharacterLocation::InsideShip, eye, point),
                Some(expected)
            );
            for invalid in [[-3.0, 1.65, 0.0], [-3.0, 1.65, 4.8], [0.0, 1.65, 1.0]] {
                assert_ne!(
                    available_interaction_target(CharacterLocation::InsideShip, invalid, point),
                    Some(expected)
                );
            }
            assert_eq!(
                available_interaction_target(CharacterLocation::Space, eye, point),
                None
            );
        }
    }
}

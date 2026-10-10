//! Compose physical carrying with character aim and release from hand height.
use crate::mining::MiningTool;
use salimon_character::{CharacterLocation, CharacterSnapshot, ShipFrame};
use salimon_math::{cross, dot, length};
use salimon_world::carrying::{WorldObjectId, aimed_fragment};
use salimon_world::mining::MiningRay;
use salimon_world::resources::{FragmentId, ResourceTransform};
use salimon_world::{BodyRole, CELESTIAL_BODIES, WorldPosition};

fn position(p: [f64; 3]) -> WorldPosition {
    WorldPosition::new(p[0], p[1], p[2])
}

fn ray(player: CharacterSnapshot) -> Option<MiningRay> {
    MiningRay::new(
        position(player.eye_position_meters),
        position(player.look_target_meters),
    )
}

fn hull_obstruction(
    player: CharacterSnapshot,
    frame: ShipFrame,
    door_open: bool,
    target: [f64; 3],
) -> Option<f64> {
    salimon_character::ship_sight_obstruction(
        frame.world_to_local(player.eye_position_meters),
        frame.world_to_local(target),
        door_open,
    )
}

pub(crate) fn target(
    tool: &MiningTool,
    player: CharacterSnapshot,
    frame: ShipFrame,
    door_open: bool,
) -> Option<FragmentId> {
    if !matches!(
        player.location,
        CharacterLocation::Surface | CharacterLocation::InsideShip
    ) {
        return None;
    }
    let ray = ray(player)?;
    let blocked_at = CELESTIAL_BODIES
        .iter()
        .filter(|body| body.role == BodyRole::Solid)
        .filter_map(|body| ray.sphere_distance(body.center, body.radius_meters))
        .chain(hull_obstruction(
            player,
            frame,
            door_open,
            player.look_target_meters,
        ))
        .min_by(f64::total_cmp)
        .unwrap_or(f64::INFINITY);
    aimed_fragment(
        ray,
        tool.session.fragments(),
        tool.session
            .carried_id()
            .map(WorldObjectId::ResourceFragment),
        blocked_at,
    )
}

pub(crate) fn context(
    tool: &MiningTool,
    target: Option<FragmentId>,
    player: CharacterSnapshot,
) -> Option<&'static str> {
    if !matches!(
        player.location,
        CharacterLocation::Surface | CharacterLocation::InsideShip
    ) {
        return None;
    }
    if tool.session.carried_id().is_some() {
        Some(if target.is_some() {
            "Only one world object - F to drop first"
        } else {
            "Carrying fragment - F to drop"
        })
    } else {
        target.map(|_| "F to pick up fragment")
    }
}

pub(crate) fn follow(tool: &mut MiningTool, player: CharacterSnapshot) {
    let Some(id) = tool.session.carried_id() else {
        return;
    };
    let orientation = tool
        .session
        .fragments()
        .iter()
        .find(|p| p.id() == id)
        .expect("carried fragment exists")
        .transform()
        .orientation_xyzw();
    let Some(ray) = ray(player) else {
        return;
    };
    let up = player.up.map(f64::from);
    let vertical = dot(ray.direction, up);
    let planar = std::array::from_fn(|i| ray.direction[i] - up[i] * vertical);
    let planar_length = length(planar);
    if planar_length < 1e-9 {
        return;
    }
    let planar = planar.map(|v| v / planar_length);
    let right = cross(planar, up);
    let piece = tool
        .session
        .fragments()
        .iter()
        .find(|piece| piece.id() == id)
        .expect("carried fragment exists");
    let clearance = salimon_world::resource_size::fragment_contact_radius_meters(piece.material());
    // Keep the entire object ahead of the player's vertical capsule even when
    // looking straight down; eye-only clearance would clip the torso/legs.
    let support = crate::fragment_physics::support(*piece, up);
    let height = (-0.18 + vertical * (0.65 + clearance))
        .max(support + 0.005 - salimon_character::PLAYER_EYE_HEIGHT_METERS);
    let p = std::array::from_fn(|i| {
        player.eye_position_meters[i] + planar[i] * (0.65 + clearance) - right[i] * 0.20
            + up[i] * height
    });
    if let Ok(pose) = ResourceTransform::new(position(p), orientation) {
        tool.session.move_carried(pose);
    }
}

/// Release at the current carried pose. Gravity and contacts then determine where it lands.
pub(crate) fn release_pose(
    tool: &MiningTool,
    player: CharacterSnapshot,
    frame: ShipFrame,
) -> Option<ResourceTransform> {
    if !matches!(
        player.location,
        CharacterLocation::Surface | CharacterLocation::InsideShip
    ) {
        return None;
    }
    let id = tool.session.carried_id()?;
    tool.session
        .fragments()
        .iter()
        .find(|piece| piece.id() == id)
        .filter(|piece| {
            player.location != CharacterLocation::InsideShip
                || salimon_character::ship_floor_placement(
                    frame.world_to_local(piece.transform().position().meters()),
                    salimon_world::resource_size::fragment_contact_radius_meters(piece.material()),
                )
                .is_some()
        })
        .map(|piece| piece.transform())
}

/// Interior gravity abstracts acceleration: loose pieces retain their ship-local
/// coordinates while the ship translates/rotates. Surface pieces remain world-local.
pub(crate) fn sync_ship_fragments(tool: &mut MiningTool, frame: ShipFrame) {
    if let Some(id) = tool.session.carried_id() {
        tool.ship_fragments.remove(&id);
    }
    for (&id, &local) in &tool.ship_fragments {
        if let Ok(pose) = ResourceTransform::new(
            position(frame.local_to_world(local)),
            crate::fragment_physics::to_world_orientation(
                frame,
                tool.fragment_motion
                    .get(&id)
                    .and_then(|m| m.ship_orientation)
                    .unwrap_or([0.0, 0.0, 0.0, 1.0]),
            ),
        ) {
            tool.session.move_loose(id, pose);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use salimon_world::CelestialBodyId;
    use salimon_world::resources::{DepositId, RawMaterial, ResourceDeposit, ResourceId};
    use std::time::Duration;

    fn mined() -> MiningTool {
        let mut tool = MiningTool::default();
        let mut deposit = ResourceDeposit::new(
            DepositId {
                body: CelestialBodyId::Earth,
                local: 1,
            },
            RawMaterial::new(ResourceId::SilicateRock, 2.0).unwrap(),
            position([0.0, 0.0, 0.0]),
            2.0,
        )
        .unwrap();
        tool.session.extract(&mut deposit, Duration::from_secs(1));
        tool
    }

    #[test]
    fn enlarged_carrying_clears_player_and_ground_at_extreme_pitch_and_rejects_unsafe_drops() {
        for resource in [
            ResourceId::IronOre,
            ResourceId::SilicateRock,
            ResourceId::WaterIce,
        ] {
            for mass in [0.032, 1.0, 2.0] {
                let mut tool = MiningTool::default();
                let mut deposit = ResourceDeposit::new(
                    DepositId {
                        body: CelestialBodyId::Earth,
                        local: 1,
                    },
                    RawMaterial::new(resource, mass).unwrap(),
                    position([0.0; 3]),
                    mass,
                )
                .unwrap();
                tool.session.extract(&mut deposit, Duration::from_secs(1));
                let piece = tool.session.fragments()[0];
                assert!(tool.session.pick_up(piece.id()));
                let radius =
                    salimon_world::resource_size::fragment_contact_radius_meters(piece.material());
                for pitch in [-100.0, 0.0, 100.0] {
                    let player = CharacterSnapshot {
                        location: CharacterLocation::Surface,
                        eye_position_meters: [0.0, 1.75, 0.0],
                        look_target_meters: [1.0, 1.75 + pitch, 0.0],
                        up: [0.0, 1.0, 0.0],
                        local_ship_position_meters: None,
                        doorway_blend_fraction: None,
                    };
                    follow(&mut tool, player);
                    let carried = tool.session.fragments()[0];
                    let p = carried.transform().position().meters();
                    assert!(
                        p[0].hypot(p[2]) > radius + 0.3,
                        "whole object clears torso and legs"
                    );
                    let support = crate::resource_presentation::fragment_mesh(carried)
                        .mesh
                        .support_meters([0.0, 1.0, 0.0])
                        * salimon_world::resource_fragments::side_meters(carried);
                    assert!(p[1] - support >= 0.0049);
                    assert_eq!(carried.id(), piece.id());
                    assert_eq!(carried.material(), piece.material());
                }
                let frame = ShipFrame {
                    origin_meters: [0.0; 3],
                    axes: [[1.0, 0.0, 0.0], [0.0, 1.0, 0.0], [0.0, 0.0, 1.0]],
                };
                let eye = [
                    -5.0,
                    salimon_character::SHIP_FLOOR_HEIGHT_METERS + 1.75,
                    0.0,
                ];
                let mut player = CharacterSnapshot {
                    location: CharacterLocation::InsideShip,
                    eye_position_meters: eye,
                    look_target_meters: [eye[0], eye[1] - 100.0, 1.0],
                    up: [0.0, 1.0, 0.0],
                    local_ship_position_meters: Some(eye),
                    doorway_blend_fraction: None,
                };
                follow(&mut tool, player);
                assert!(release_pose(&tool, player, frame).is_some());
                player.eye_position_meters[0] = -7.5;
                player.look_target_meters = [-8.5, eye[1], 0.0];
                follow(&mut tool, player);
                assert!(release_pose(&tool, player, frame).is_none());
                assert_eq!(tool.session.carried_id(), Some(piece.id()));
            }
        }
    }

    #[test]
    fn loose_ship_piece_keeps_local_pose_across_translation_rotation_and_pickup() {
        let mut tool = mined();
        let original = tool.session.fragments()[0];
        let id = original.id();
        let local = [-5.8, 0.4, 0.0];
        tool.ship_fragments.insert(id, local);
        let orientation = [0.0, 0.0, 0.3_f64.sin(), 0.3_f64.cos()];
        tool.fragment_motion.insert(
            id,
            crate::fragment_physics::FragmentMotion {
                ship_orientation: Some(orientation),
                ..Default::default()
            },
        );
        for axes in [
            [[1.0, 0.0, 0.0], [0.0, 1.0, 0.0], [0.0, 0.0, 1.0]],
            [[0.0, 1.0, 0.0], [-1.0, 0.0, 0.0], [0.0, 0.0, 1.0]],
        ] {
            for offset in [0.0, 25_000.0, 1.0e12] {
                let frame = ShipFrame {
                    origin_meters: [offset, offset, offset],
                    axes,
                };
                sync_ship_fragments(&mut tool, frame);
                let piece = tool.session.fragments()[0];
                assert_eq!(piece.id(), id);
                assert_eq!(piece.source(), original.source());
                assert_eq!(piece.material(), original.material());
                let expected = crate::fragment_physics::to_world_orientation(frame, orientation);
                for (actual, expected) in piece.transform().orientation_xyzw().iter().zip(expected) {
                    assert!((actual - expected).abs() < 1e-12);
                }
                for (actual, expected) in frame
                    .world_to_local(piece.transform().position().meters())
                    .iter()
                    .zip(local)
                {
                    assert!((actual - expected).abs() < 0.001);
                }
            }
        }
        assert!(tool.session.pick_up(id));
        let carried =
            ResourceTransform::new(position([10.0, 20.0, 30.0]), [0.0, 0.0, 0.0, 1.0]).unwrap();
        tool.session.move_carried(carried);
        assert!(!tool.session.move_loose(id, original.transform()));
        sync_ship_fragments(
            &mut tool,
            ShipFrame {
                origin_meters: [0.0; 3],
                axes: [[1.0, 0.0, 0.0], [0.0, 1.0, 0.0], [0.0, 0.0, 1.0]],
            },
        );
        assert!(tool.ship_fragments.is_empty());
        assert_eq!(tool.session.fragments()[0].transform(), carried);
    }

    #[test]
    fn cabin_placement_rejects_walls_core_edges_and_oversized_objects() {
        use salimon_character::ship_floor_placement;
        assert!(ship_floor_placement([-5.8, 2.0, 0.0], 0.1).is_some());
        assert!(ship_floor_placement([1.0, 2.0, 3.0], 0.1).is_some());
        for point in [
            [-1.0, 2.0, 0.0],
            [5.0, 2.0, -1.0],
            [2.3, 2.0, 8.0],
            [0.0, 2.0, 8.0],
            [-8.0, 2.0, 0.0],
            [0.0, 2.0, 4.8],
        ] {
            assert!(ship_floor_placement(point, 0.1).is_none(), "{point:?}");
        }
        assert!(ship_floor_placement([1.0, 2.0, 3.0], 10.0).is_none());
        assert!(ship_floor_placement([f64::NAN, 2.0, 3.0], 0.1).is_none());
    }

    #[test]
    fn ship_sight_reaches_loose_items_but_not_through_core_console_or_closed_gate() {
        use salimon_character::ship_sight_obstruction;
        assert!(
            ship_sight_obstruction([-5.8, 2.0, 0.0], [-4.9, 0.4, 0.0], true)
                .is_none_or(|d| d > 1.84)
        );
        assert!(ship_sight_obstruction([-2.5, 2.0, 0.0], [1.0, 0.4, 0.0], true).is_some());
        assert!(ship_sight_obstruction([2.5, 2.0, 1.3], [5.0, 0.4, -1.0], true).is_some());
        assert_eq!(
            ship_sight_obstruction([-6.5, 2.0, 0.0], [-9.0, 0.4, 0.0], true),
            None
        );
        assert!(ship_sight_obstruction([-6.5, 2.0, 0.0], [-9.0, 0.4, 0.0], false).is_some());
        assert!(ship_sight_obstruction([-5.8, 2.0, 0.0], [-5.8, 0.0, 0.0], true).is_some());
    }
}

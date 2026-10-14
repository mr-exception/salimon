//! Compose physical carrying with character aim and release from hand height.
use crate::mining::MiningTool;
use salimon_character::{CharacterLocation, CharacterSnapshot, ShipFrame};
use salimon_math::{cross, dot, length};
use salimon_world::WorldPosition;
use salimon_world::carrying::{WorldObjectId, aimed_fragment};
use salimon_world::mining::MiningRay;
use salimon_world::resources::{FragmentId, ResourceTransform};

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
    let blocked_at = salimon_world::mining::terrain_obstruction(ray, tool.terrain, 4.0)
        .into_iter()
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
    let mut p: [f64; 3] = std::array::from_fn(|i| {
        player.eye_position_meters[i] + planar[i] * (0.65 + clearance) - right[i] * 0.20
            + up[i] * height
    });
    if tool.terrain == salimon_world::terrain::TerrainMode::Representative
        && player.location == CharacterLocation::Surface
        && let Some((body, _)) = tool.terrain.nearest_body(
            p,
            salimon_world::NEARBY_BODY_MAX_SURFACE_DISTANCE_METERS,
            true,
        )
    {
        let mesh = crate::resource_presentation::fragment_mesh(*piece);
        for _ in 0..3 {
            let local = salimon_math::sub(p, body.center.meters());
            let mut lift = 0.0_f64;
            for vertex in mesh.mesh.unit_vertices() {
                let point = salimon_math::add(
                    local,
                    salimon_physics::rotate(
                        orientation,
                        salimon_math::scale(*vertex, mesh.side_meters),
                    ),
                );
                let radius = length(point);
                let radial = salimon_math::scale(point, 1.0 / radius);
                let gap = radius - body.radius_meters - tool.terrain.height(*body, radial);
                lift = lift.max((0.005 - gap) / dot(up, radial).max(0.1));
            }
            if lift < 0.00001 {
                break;
            }
            p = salimon_math::add(p, salimon_math::scale(up, lift));
        }
    }
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

/// Drain mutation events only; physics/frame adapters already visit moved objects.
pub(crate) fn sync_ship_mass(
    tool: &mut MiningTool,
    frame: ShipFrame,
    ship: &mut salimon_ship::ShipController,
) {
    for (id, fragment) in tool.session.drain_fragment_changes() {
        if fragment.is_none() {
            tool.ship_fragments.remove(&id);
            tool.fragment_motion.remove(&id);
            tool.fragment_previous.remove(&id);
        }
        let mass = fragment
            .filter(|p| {
                salimon_character::ship_contains_object(
                    frame.world_to_local(p.transform().position().meters()),
                )
            })
            .map(|p| p.material().mass_kg());
        tool.ship_mass
            .update(salimon_ship::MassObjectId::Fragment(id.0), mass)
            .expect("validated physical fragment mass");
    }
    ship.set_total_mass_kg(tool.ship_mass.total_kg())
        .expect("validated mass ledger");
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
                for (actual, expected) in piece.transform().orientation_xyzw().iter().zip(expected)
                {
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

#[cfg(test)]
mod mass_tests {
    use super::*;
    use salimon_ship::{ShipController, ShipPose};
    use salimon_world::{
        CelestialBodyId,
        resources::{DepositId, RawMaterial, ResourceDeposit, ResourceId},
    };
    use std::time::Duration;

    #[test]
    fn physical_events_containment_moving_ship_carry_drop_and_despawn() {
        let mut tool = MiningTool::default();
        let mut ship = ShipController::flying_with_velocity(
            ShipPose {
                position_meters: [1e12, 0.0, 0.0],
                orientation: [0.0, 0.0, 0.0, 1.0],
            },
            100,
            [12.0, 0.0, 0.0],
        )
        .unwrap();
        let frame = crate::app::character_ship_frame(ship.snapshot().pose);
        let pose = |local| {
            ResourceTransform::new(position(frame.local_to_world(local)), [0.0, 0.0, 0.0, 1.0])
                .unwrap()
        };
        let mut deposit = ResourceDeposit::new(
            DepositId {
                body: CelestialBodyId::Earth,
                local: 1,
            },
            RawMaterial::new(ResourceId::IronOre, 1000.0).unwrap(),
            position([0.0; 3]),
            1000.0,
        )
        .unwrap();
        tool.session
            .extract_with_spawn(&mut deposit, Duration::from_secs(500), |_, _, _| {
                Some(pose([-5.0, 1.0, 0.0]))
            });
        sync_ship_mass(&mut tool, frame, &mut ship);
        assert_eq!(ship.snapshot().total_mass_kg, 101_000.0);
        assert_eq!(ship.snapshot().velocity_meters_per_second, [12.0, 0.0, 0.0]);
        let id = tool.session.fragments()[0].id();
        assert!(tool.session.pick_up(id));
        tool.session.move_carried(pose([-7.24, 1.0, 0.0]));
        sync_ship_mass(&mut tool, frame, &mut ship);
        assert_eq!(ship.snapshot().total_mass_kg, 101_000.0);
        tool.session.move_carried(pose([-7.25, 1.0, 0.0]));
        sync_ship_mass(&mut tool, frame, &mut ship);
        assert_eq!(ship.snapshot().total_mass_kg, 100_998.0);
        tool.session.drop_carried(pose([-5.0, 1.0, 0.0]));
        sync_ship_mass(&mut tool, frame, &mut ship);
        assert_eq!(ship.snapshot().total_mass_kg, 101_000.0);
        for piece in tool.session.fragments() {
            tool.ship_fragments.insert(piece.id(), [-5.0, 1.0, 0.0]);
        }
        ship.advance(Duration::from_secs(1));
        let moved = crate::app::character_ship_frame(ship.snapshot().pose);
        sync_ship_fragments(&mut tool, moved);
        sync_ship_mass(&mut tool, moved, &mut ship);
        assert_eq!(ship.snapshot().total_mass_kg, 101_000.0);
        assert!(
            (ship.snapshot().velocity_meters_per_second[0] - (12.0 + 4e6 / 101000.0)).abs() < 1e-10
        );
        assert!(tool.session.pick_up(id));
        assert_eq!(
            ship.intake_carried_fragment(&mut tool.session),
            salimon_ship::IntakeResult::Accepted
        );
        sync_ship_mass(&mut tool, moved, &mut ship);
        assert_eq!(ship.snapshot().total_mass_kg, 100_998.0);
        assert!(!tool.ship_fragments.contains_key(&id));
    }
}

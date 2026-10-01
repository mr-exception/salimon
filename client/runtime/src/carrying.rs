//! Compose physical carrying with character aim and surface placement.
use crate::mining::MiningTool;
use salimon_character::{CharacterLocation, CharacterSnapshot, ShipFrame};
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
            "Only one world object - G to place/drop first"
        } else {
            "Carrying fragment - G to place/drop"
        })
    } else {
        target.map(|_| "Q to pick up fragment")
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
    let right = cross(ray.direction, up);
    let p = std::array::from_fn(|i| {
        player.eye_position_meters[i] + ray.direction[i] * 0.65 - right[i] * 0.20 - up[i] * 0.18
    });
    if let Ok(pose) = ResourceTransform::new(position(p), orientation) {
        tool.session.move_carried(pose);
    }
}

/// Place on the aimed nearby surface, or just ahead at ground level when looking
/// horizontally. Interior placement uses the shared ship-local deck/collision layout.
pub(crate) fn drop_pose(
    tool: &MiningTool,
    player: CharacterSnapshot,
    frame: ShipFrame,
    door_open: bool,
    seed: u64,
) -> Option<ResourceTransform> {
    if !matches!(
        player.location,
        CharacterLocation::Surface | CharacterLocation::InsideShip
    ) {
        return None;
    }
    let id = tool.session.carried_id()?;
    let piece = *tool
        .session
        .fragments()
        .iter()
        .find(|piece| piece.id() == id)?;
    let ray = ray(player)?;
    if player.location == CharacterLocation::InsideShip {
        let eye = frame.world_to_local(player.eye_position_meters);
        let direction = std::array::from_fn::<_, 3, _>(|i| dot(ray.direction, frame.axes[i]));
        // Bounding sphere clearance keeps world-axis greybox cubes clear even as the ship rotates.
        let half = salimon_world::resource_fragments::side_meters(piece) * 0.5 * 3.0_f64.sqrt();
        let floor = salimon_character::CARGO_ROOM_MIN_METERS[1];
        let t = if direction[1] < -1e-9 {
            ((floor - eye[1]) / direction[1]).min(3.0)
        } else {
            0.9
        };
        let aim = std::array::from_fn(|i| eye[i] + direction[i] * t);
        let local = salimon_character::ship_floor_placement(aim, half)?;
        let center = position(frame.local_to_world(local));
        let distance = dot(
            center.offset_from(ray.origin),
            center.offset_from(ray.origin),
        )
        .sqrt();
        if distance > 3.0
            || hull_obstruction(player, frame, door_open, center.meters())
                .is_some_and(|d| d < distance - half)
        {
            return None;
        }
        if overlaps(tool, id, center, half) {
            return None;
        }
        return ResourceTransform::new(center, piece.transform().orientation_xyzw()).ok();
    }
    let body = CELESTIAL_BODIES
        .iter()
        .filter(|body| body.role == BodyRole::Solid)
        .min_by(|a, b| {
            let distance = |body: &salimon_world::CelestialBody| {
                let offset = ray.origin.offset_from(body.center);
                dot(offset, offset).sqrt() - body.radius_meters
            };
            distance(a).total_cmp(&distance(b))
        })?;
    let radial = ray.origin.offset_from(body.center);
    let radius = dot(radial, radial).sqrt();
    if radius - body.radius_meters > 3.0 {
        return None;
    }
    let up = radial.map(|v| v / radius);
    let tangent = std::array::from_fn(|i| ray.direction[i] - up[i] * dot(ray.direction, up));
    let norm = dot(tangent, tangent).sqrt();
    let ground = if let Some(distance) = ray
        .sphere_distance(body.center, body.radius_meters)
        .filter(|d| *d <= 3.0)
    {
        ray.origin.translated(ray.direction.map(|v| v * distance))
    } else if norm > 1e-9 {
        ray.origin.translated(tangent.map(|v| v / norm * 0.9))
    } else {
        return None;
    };
    let radial = ground.offset_from(body.center);
    let length = dot(radial, radial).sqrt();
    let up = radial.map(|v| v / length);
    let half = salimon_world::resource_fragments::side_meters(piece) * 0.5;
    let support = half * up.iter().map(|v| v.abs()).sum::<f64>() + 0.005;
    let center = body
        .center
        .translated(up.map(|v| v * (body.radius_meters + support)));
    let offset = center.offset_from(ray.origin);
    let distance = dot(offset, offset).sqrt();
    if distance > 3.0
        || hull_obstruction(player, frame, door_open, center.meters())
            .is_some_and(|d| d <= distance + half)
    {
        return None;
    }
    // Keep placed physical pieces distinct, and avoid placing inside deposits.
    if overlaps(tool, id, center, half) {
        return None;
    }
    let deposits = tool.nearby(player.eye_position_meters, seed).ok()?;
    if deposits.iter().any(|entry| {
        entry.deposit.remaining_mass_kg() > 0.0 && {
            let offset = center.offset_from(entry.deposit.position());
            dot(offset, offset).sqrt() < entry.bounds_radius_meters + half * 3.0_f64.sqrt()
        }
    }) {
        return None;
    }
    ResourceTransform::new(center, piece.transform().orientation_xyzw()).ok()
}

fn overlaps(tool: &MiningTool, id: FragmentId, center: WorldPosition, half: f64) -> bool {
    tool.session
        .fragments()
        .iter()
        .filter(|p| p.id() != id)
        .any(|other| {
            let extent =
                half + salimon_world::resource_fragments::side_meters(*other) * 0.5 + 0.005;
            center
                .offset_from(other.transform().position())
                .iter()
                .all(|v| v.abs() < extent)
        })
}

/// Interior gravity abstracts acceleration: loose pieces retain their ship-local
/// coordinates while the ship translates/rotates. Surface pieces remain world-local.
pub(crate) fn sync_ship_fragments(tool: &mut MiningTool, frame: ShipFrame) {
    if let Some(id) = tool.session.carried_id() {
        tool.ship_fragments.remove(&id);
    }
    for (&id, &local) in &tool.ship_fragments {
        if let Ok(pose) =
            ResourceTransform::new(position(frame.local_to_world(local)), [0.0, 0.0, 0.0, 1.0])
        {
            tool.session.move_loose(id, pose);
        }
    }
}

fn dot(a: [f64; 3], b: [f64; 3]) -> f64 {
    a.iter().zip(b).map(|(a, b)| a * b).sum()
}
fn cross(a: [f64; 3], b: [f64; 3]) -> [f64; 3] {
    [
        a[1] * b[2] - a[2] * b[1],
        a[2] * b[0] - a[0] * b[2],
        a[0] * b[1] - a[1] * b[0],
    ]
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
    fn loose_ship_piece_keeps_local_pose_across_translation_rotation_and_pickup() {
        let mut tool = mined();
        let original = tool.session.fragments()[0];
        let id = original.id();
        let local = [-5.8, 0.4, 0.0];
        tool.ship_fragments.insert(id, local);
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
    fn interior_placement_rejects_walls_furniture_edges_and_oversized_objects() {
        use salimon_character::ship_floor_placement;
        assert!(ship_floor_placement([-5.8, 2.0, 0.0], 0.1).is_some());
        assert!(ship_floor_placement([1.0, 2.0, 8.0], 0.1).is_some());
        for point in [
            [-1.0, 2.0, 0.0],
            [-3.2, 2.0, 8.0],
            [2.3, 2.0, 8.0],
            [0.0, 2.0, 10.6],
            [-8.0, 2.0, 0.0],
            [0.0, 2.0, 4.8],
        ] {
            assert!(ship_floor_placement(point, 0.1).is_none(), "{point:?}");
        }
        assert!(ship_floor_placement([1.0, 2.0, 8.0], 2.0).is_none());
        assert!(ship_floor_placement([f64::NAN, 2.0, 8.0], 0.1).is_none());
    }

    #[test]
    fn ship_sight_reaches_loose_items_but_not_through_core_partition_or_closed_gate() {
        use salimon_character::ship_sight_obstruction;
        assert!(
            ship_sight_obstruction([-5.8, 2.0, 0.0], [-4.9, 0.4, 0.0], true)
                .is_none_or(|d| d > 1.84)
        );
        assert!(ship_sight_obstruction([-2.5, 2.0, 0.0], [1.0, 0.4, 0.0], true).is_some());
        assert!(ship_sight_obstruction([-4.0, 2.0, 3.0], [-4.0, 0.4, 8.0], true).is_some());
        assert_eq!(
            ship_sight_obstruction([-6.5, 2.0, 0.0], [-9.0, 0.4, 0.0], true),
            None
        );
        assert!(ship_sight_obstruction([-6.5, 2.0, 0.0], [-9.0, 0.4, 0.0], false).is_some());
        assert!(ship_sight_obstruction([-5.8, 2.0, 0.0], [-5.8, 0.0, 0.0], true).is_some());
    }
}

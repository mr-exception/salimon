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
    if player.location != CharacterLocation::Surface {
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
    if player.location != CharacterLocation::Surface {
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
/// horizontally. Unsupported ship/cargo placement belongs to the transfer issue.
pub(crate) fn drop_pose(
    tool: &MiningTool,
    player: CharacterSnapshot,
    frame: ShipFrame,
    door_open: bool,
    seed: u64,
) -> Option<ResourceTransform> {
    if player.location != CharacterLocation::Surface {
        return None;
    }
    let id = tool.session.carried_id()?;
    let piece = *tool
        .session
        .fragments()
        .iter()
        .find(|piece| piece.id() == id)?;
    let ray = ray(player)?;
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
    if tool
        .session
        .fragments()
        .iter()
        .filter(|p| p.id() != id)
        .any(|other| {
            let offset = center.offset_from(other.transform().position());
            let extent =
                half + salimon_world::resource_fragments::side_meters(*other) * 0.5 + 0.005;
            offset.iter().all(|v| v.abs() < extent)
        })
    {
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

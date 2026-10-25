//! Explicit native evidence setup; subsequent actions use production commands.
use super::*;

/// Evidence-only initial source: one reserved four-cell room and one carried
/// 40 kg iron sphere. All later intake, paid work and movement use normal commands.
pub(crate) fn evidence_room(s: &mut GameplaySession) -> Result<(), String> {
    use salimon_world::{resources::*, spherical_mining::*};
    let y = SHIP_FLOOR_HEIGHT_METERS;
    s.character.set_supported_pose([-6.6, y + 1.75, 0.]);
    s.ship.toggle_door();
    s.advance_automation(std::time::Duration::from_secs(3));
    let target = [-14.5, y, -1.5];
    s.building
        .source
        .projects
        .place(
            registered_catalog(),
            "two-deck-room",
            target,
            Rotation::Forward,
            &s.building.structure,
            &PlacementContext {
                actor: OWNER,
                owner: OWNER,
                flight: FlightPermission::Landed,
                eye_meters: [-6.6, y + 1.75, 0.],
                reach_meters: 5.,
                reachable: true,
                physical_support: true,
                obstacles: &static_obstacles(Footprint::Structural),
            },
        )
        .map_err(|e| e.to_string())?;
    let frame = character_ship_frame(s.ship.snapshot().pose);
    let w = frame.local_to_world([-6.6, y + 1., 0.]);
    let pose = ResourceTransform::new(
        salimon_world::WorldPosition::new(w[0], w[1], w[2]),
        [0., 0., 0., 1.],
    )
    .map_err(|e| e.to_string())?;
    let mut deposit = ResourceDeposit::new(
        DepositId {
            body: salimon_world::CelestialBodyId::Earth,
            local: 545,
        },
        RawMaterial::new(ResourceId::IronOre, 40.).map_err(|e| e.to_string())?,
        pose.position(),
        40.,
    )
    .map_err(|e| e.to_string())?;
    let config = MiningConfig::new(40., 100.).map_err(|e| e.to_string())?;
    let mut id = None;
    for _ in 0..100 {
        id = s
            .mining
            .session
            .tick_spherical(true, Some(&mut deposit), SPHERE_TICK, config, pose)
            .or(id);
        if id.is_some() {
            break;
        }
    }
    if !s
        .mining
        .session
        .pick_up(id.ok_or("room evidence sphere absent")?)
    {
        return Err("room evidence carry rejected".into());
    }
    s.command(crate::session::Command::Look(
        std::f64::consts::PI / 0.0022,
        0.,
    ));
    s.building.active = true;
    s.building.selected = 5;
    s.equipment.stow();
    s.sync_carried();
    Ok(())
}

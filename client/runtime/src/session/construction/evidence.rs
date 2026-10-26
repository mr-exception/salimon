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

/// Explicit module evidence: a real reservation plus one declared carried input.
/// Upgrade/restoration fixtures additionally declare their prior paid history.
pub(crate) fn evidence_module(s: &mut GameplaySession, phase: u8) -> Result<(), String> {
    use crate::session::Command;
    use std::time::Duration;
    let y = SHIP_FLOOR_HEIGHT_METERS;
    s.character.set_supported_pose([-4.8, y + 1.75, 0.]);
    let mut placed = None;
    'search: for x in -14..=12 {
        for z in -8..=8 {
            let target = [f64::from(x) * 0.5, y, f64::from(z) * 0.5];
            let obstacles = s.construction_obstacles(Footprint::Interior);
            let c = PlacementContext {
                actor: OWNER,
                owner: OWNER,
                flight: FlightPermission::Landed,
                eye_meters: [-4.8, y + 1.75, 0.],
                reach_meters: 5.,
                reachable: true,
                physical_support: true,
                obstacles: &obstacles,
            };
            if let Ok(id) = s.building.source.projects.place(
                registered_catalog(),
                "utility-module",
                target,
                Rotation::Forward,
                &s.building.structure,
                &c,
            ) {
                if has_physical_support(
                    Footprint::Interior,
                    s.building.source.projects.inspect(id).unwrap().space,
                ) {
                    placed = Some(id);
                    break 'search;
                }
                s.building
                    .source
                    .projects
                    .cancel(id, OWNER)
                    .map_err(|e| e.to_string())?;
            }
        }
    }
    let id = placed.ok_or("no supported utility module site")?;
    let p = s.building.source.projects.inspect(id).unwrap();
    let target = std::array::from_fn(|i| (p.space.min[i] + p.space.max[i]) / 2.);
    let d = salimon_math::sub(target, [-4.8, y + 1.75, 0.]);
    s.command(Command::Look(
        d[2].atan2(d[0]) / 0.0022,
        -d[1].atan2(d[0].hypot(d[2])) / 0.0022,
    ));
    s.building.active = true;
    s.equipment.stow();
    seed_module_material(s, 3.)?;
    if phase > 0 {
        s.command(Command::Interact);
        s.advance_construction(Duration::from_secs(60));
        if s.utility_recovery_multiplier() != 1.25 {
            return Err(format!(
                "module commissioning failed: {}",
                s.construction_inspect()
            ));
        }
        if phase == 2 {
            s.construction_action(BuildAction::Module);
            seed_module_material(s, 2.)?;
            s.command(Command::Interact);
            s.advance_construction(Duration::from_secs(45));
            s.cancel_construction(id)
                .map_err(|_| "fixture maintenance cancel failed")?;
        }
        seed_module_material(s, if phase == 1 { 2. } else { 1. })?;
    }
    s.feedback = None;
    Ok(())
}
fn seed_module_material(s: &mut GameplaySession, mass: f64) -> Result<(), String> {
    use salimon_world::{resources::*, spherical_mining::*};
    let frame = character_ship_frame(s.ship.snapshot().pose);
    let w = frame.local_to_world([-4.8, SHIP_FLOOR_HEIGHT_METERS + 1., 0.]);
    let pose = ResourceTransform::new(
        salimon_world::WorldPosition::new(w[0], w[1], w[2]),
        [0., 0., 0., 1.],
    )
    .map_err(|e| e.to_string())?;
    let mut deposit = ResourceDeposit::new(
        DepositId {
            body: salimon_world::CelestialBodyId::Earth,
            local: 547 + s.mining.session.save_state().next_id,
        },
        RawMaterial::new(ResourceId::IronOre, mass).map_err(|e| e.to_string())?,
        pose.position(),
        mass,
    )
    .map_err(|e| e.to_string())?;
    let config = MiningConfig::new(mass, 100.).map_err(|e| e.to_string())?;
    let id = (0..1000)
        .find_map(|_| {
            s.mining
                .session
                .tick_spherical(true, Some(&mut deposit), SPHERE_TICK, config, pose)
        })
        .ok_or("module evidence fragment missing")?;
    if !s.mining.session.pick_up(id) {
        return Err("module evidence carry rejected".into());
    }
    s.sync_carried();
    Ok(())
}

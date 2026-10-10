//! Deterministic initial populations only; all later updates use production state.
use crate::app::{ClientApplication, character_ship_frame};
use crate::fragment_physics::{FragmentMotion, fragment_hull};
use salimon_character::SHIP_FLOOR_HEIGHT_METERS;
use salimon_world::{
    CelestialBodyId, WorldPosition,
    resources::{DepositId, RawMaterial, ResourceDeposit, ResourceId, ResourceTransform},
};
use std::time::Duration;

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub(crate) enum Layout {
    #[default]
    Scattered,
    Dense,
    Surface,
}
impl Layout {
    pub(crate) fn parse(value: &str) -> Result<Self, String> {
        match value {
            "scattered" => Ok(Self::Scattered),
            "dense" => Ok(Self::Dense),
            "surface" => Ok(Self::Surface),
            _ => Err("layout must be scattered, dense or surface".into()),
        }
    }
    pub(crate) fn name(self) -> &'static str {
        match self {
            Self::Scattered => "scattered",
            Self::Dense => "dense",
            Self::Surface => "surface",
        }
    }
}

/// Create one source per fragment through normal mass extraction. Fixture IDs,
/// materials, sizes and rotations are independent of machine/iteration timing.
pub(crate) fn initialize(
    app: &mut ClientApplication,
    count: usize,
    layout: Layout,
    seed: u64,
    active: bool,
) -> Result<(), String> {
    if count > 1000 {
        return Err("benchmark fixture count must be 0..1000".into());
    }
    let frame = character_ship_frame(app.ship.snapshot().pose);
    let scattered: Vec<_> = (0..160)
        .map(|i| {
            [
                -6.2 + (i % 16) as f64 * 0.7,
                SHIP_FLOOR_HEIGHT_METERS,
                -3.2 + (i / 16) as f64 * 0.7,
            ]
        })
        .filter(|p| salimon_character::ship_floor_placement(*p, 0.6).is_some())
        .collect();
    let dense: Vec<_> = (0..100)
        .map(|i| {
            [
                -5.8 + (i % 10) as f64 * 0.46,
                SHIP_FLOOR_HEIGHT_METERS,
                -2.0 + (i / 10) as f64 * 0.46,
            ]
        })
        .filter(|p| salimon_character::ship_floor_placement(*p, 0.6).is_some())
        .collect();
    if dense.is_empty() {
        return Err("fixture has no dense supported deck cells".into());
    }
    if scattered.is_empty() {
        return Err("fixture has no supported deck cells".into());
    }
    for index in 0..count {
        // Consecutive pairs exercise both authored ID-parity variants for each material.
        let resource = [
            ResourceId::IronOre,
            ResourceId::SilicateRock,
            ResourceId::WaterIce,
        ][(index / 2) % 3];
        let mass = if index % 4 < 2 { 2.0 } else { 0.5 };
        let material =
            RawMaterial::new(resource, mass).map_err(|e| format!("fixture mass: {e:?}"))?;
        let mut deposit = ResourceDeposit::new(
            DepositId {
                body: CelestialBodyId::Earth,
                local: 1_000_000 + index as u64,
            },
            material,
            WorldPosition::new(0.0, 0.0, 0.0),
            mass,
        )
        .map_err(|e| format!("fixture source: {e:?}"))?;
        app.mining
            .session
            .extract(&mut deposit, Duration::from_secs_f64(mass / 2.0));
        let piece = *app
            .mining
            .session
            .fragments()
            .last()
            .ok_or("fixture extraction emitted no fragment")?;
        if piece.source() != deposit.id() || piece.material() != material {
            return Err("fixture extraction did not conserve source/material".into());
        }
        let angle =
            ((seed.wrapping_add(index as u64 * 17)) % 360) as f64 * std::f64::consts::PI / 180.0;
        let orientation = [0.0, (angle / 2.0).sin(), 0.0, (angle / 2.0).cos()];
        let side = salimon_world::resource_fragments::side_meters(piece);
        let support = fragment_hull(piece).support(orientation, side, [0.0, -1.0, 0.0]);
        let mut local = match layout {
            Layout::Dense => {
                let base = dense[index % dense.len()];
                [
                    base[0],
                    SHIP_FLOOR_HEIGHT_METERS + support + (index / dense.len()) as f64 * 0.46,
                    base[2],
                ]
            }
            // One plane spanning the cabin for small loads; layers for stress loads.
            Layout::Scattered => {
                let base = scattered[index % scattered.len()];
                [
                    base[0],
                    SHIP_FLOOR_HEIGHT_METERS + support + (index / scattered.len()) as f64 * 0.7,
                    base[2],
                ]
            }
            // Outside the ship, still within the production 125m activation radius.
            Layout::Surface => [
                -12.0 - (index % 32) as f64 * 0.8,
                SHIP_FLOOR_HEIGHT_METERS + support,
                -12.0 + (index / 32) as f64 * 0.8,
            ],
        };
        if layout != Layout::Surface
            && salimon_character::ship_floor_placement(
                local,
                salimon_world::resource_size::fragment_contact_radius_meters(material),
            )
            .is_none()
        {
            return Err(format!(
                "fixture fragment {index} is outside deck containment"
            ));
        }
        if active {
            local[1] += 0.8;
        }
        let mut world = frame.local_to_world(local);
        if layout == Layout::Surface {
            let earth = salimon_world::CELESTIAL_BODIES
                .iter()
                .find(|b| b.id == CelestialBodyId::Earth)
                .expect("Earth catalog");
            let radial = salimon_math::sub(world, earth.center.meters());
            let up = salimon_math::scale(radial, 1.0 / salimon_math::length(radial));
            world = salimon_math::add(
                earth.center.meters(),
                salimon_math::scale(
                    up,
                    earth.radius_meters + support + if active { 0.8 } else { 0.0 },
                ),
            );
        }
        let world_orientation = crate::fragment_physics::to_world_orientation(frame, orientation);
        let pose = ResourceTransform::new(
            WorldPosition::new(world[0], world[1], world[2]),
            world_orientation,
        )
        .map_err(|e| format!("fixture pose: {e:?}"))?;
        if !app.mining.session.move_loose(piece.id(), pose) {
            return Err("fixture pose rejected".into());
        }
        let in_ship = layout != Layout::Surface;
        if in_ship {
            app.mining.ship_fragments.insert(piece.id(), local);
        }
        app.mining.fragment_motion.insert(
            piece.id(),
            FragmentMotion {
                velocity: if active {
                    let v = [0.12 * angle.cos(), 0.4, 0.12 * angle.sin()];
                    if in_ship {
                        v
                    } else {
                        std::array::from_fn(|axis| (0..3).map(|i| frame.axes[i][axis] * v[i]).sum())
                    }
                } else {
                    [0.0; 3]
                },
                angular_velocity: if active { [0.0, 0.3, 0.0] } else { [0.0; 3] },
                ship_orientation: in_ship.then_some(orientation),
            },
        );
    }
    if app.mining.session.fragments().len() != count {
        return Err("fixture count mismatch".into());
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn populations_conserve_mass_identity_variants_and_session_restore() {
        for count in [0, 10, 25, 50, 100, 250, 500, 1000] {
            let mut app = ClientApplication::default();
            initialize(&mut app, count, Layout::Dense, 155, true).unwrap();
            let pieces = app.mining.session.fragments();
            let total: f64 = pieces.iter().map(|p| p.material().mass_kg()).sum();
            assert_eq!(app.mining.session.extracted_mass_kg(), total);
            assert_eq!(pieces.len(), count);
            let before = pieces.to_vec();
            let mut other = ClientApplication::default();
            initialize(&mut other, count, Layout::Dense, 155, true).unwrap();
            assert_eq!(before, other.mining.session.fragments());
            for piece in before {
                let mut restored = ResourceDeposit::new(
                    piece.source(),
                    piece.material(),
                    WorldPosition::new(0.0, 0.0, 0.0),
                    piece.material().mass_kg(),
                )
                .unwrap();
                assert_eq!(
                    app.mining
                        .session
                        .extract(&mut restored, Duration::from_secs(1)),
                    0.0
                );
                assert_eq!(restored.remaining_mass_kg(), 0.0);
            }
            assert_eq!(app.mining.session.fragments().len(), count);
        }
    }
}

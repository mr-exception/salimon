//! Deterministic initial populations only; all later updates use production state.
use crate::app::{ClientApplication, character_ship_frame, surface_frame_for_ship};
use crate::fragment_physics::{FragmentMotion, fragment_hull};
use salimon_character::SHIP_FLOOR_HEIGHT_METERS;
use salimon_world::{
    CelestialBodyId, WorldPosition,
    resources::{DepositId, RawMaterial, ResourceDeposit, ResourceId, ResourceTransform},
};
use std::time::{Duration, Instant};

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub(crate) enum Layout {
    #[default]
    Scattered,
    Dense,
    Surface,
    Onboard,
}
impl Layout {
    pub(crate) fn parse(value: &str) -> Result<Self, String> {
        match value {
            "scattered" => Ok(Self::Scattered),
            "dense" => Ok(Self::Dense),
            "surface" => Ok(Self::Surface),
            "onboard" => Ok(Self::Onboard),
            _ => Err("layout must be scattered, dense, surface or onboard".into()),
        }
    }
    pub(crate) fn name(self) -> &'static str {
        match self {
            Self::Scattered => "scattered",
            Self::Dense => "dense",
            Self::Surface => "surface",
            Self::Onboard => "onboard",
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
    if layout == Layout::Onboard {
        crate::flight_regression::initialize_cargo(app, count)?;
        if active {
            for piece in app.mining.session.fragments() {
                app.mining.fragment_motion.insert(
                    piece.id(),
                    FragmentMotion {
                        velocity: [0.0, 0.4, 0.0],
                        angular_velocity: [0.0, 0.3, 0.0],
                        ship_orientation: Some([0.0, 0.0, 0.0, 1.0]),
                    },
                );
            }
        }
        return Ok(());
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
            Layout::Onboard => unreachable!("onboard fixture initialized above"),
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

/// Prepare a fixture only through production updates; never install rest poses or flags.
pub(crate) fn settle(app: &mut ClientApplication, count: usize, limit: u64) -> Result<(), String> {
    // Settled fixture preparation uses the real adapter at explicit 16 ms deltas,
    // before starting the native wall-clock measurement. No sleep flags/poses
    // are installed. This avoids a slow initial fall feeding large native deltas
    // back into a load intended to measure already settled cargo.
    if limit > 0 && count > 0 {
        let frame = character_ship_frame(app.ship.snapshot().pose);
        let eye = app
            .character
            .snapshot(frame, surface_frame_for_ship(app.ship.snapshot()))
            .eye_position_meters;
        let started = Instant::now();
        let mut sleeping = 0;
        let mut last_step = 0;
        checkpoint(app, frame, eye, 0)?;
        for step in 1..=limit {
            last_step = step;
            let measurements = crate::fragment_physics::advance_measured(
                &mut app.mining,
                frame,
                Duration::from_millis(16),
                eye,
                true,
            );
            sleeping = measurements.solver.sleeping_objects;
            if step % 50 == 0 {
                checkpoint(app, frame, eye, step)?;
                let states = crate::fragment_physics::snapshots(&app.mining, frame, eye);
                let mut speeds: Vec<_> = states
                    .iter()
                    .map(|(_, p)| salimon_math::length(p.velocity))
                    .collect();
                let mut spins: Vec<_> = states
                    .iter()
                    .map(|(_, p)| salimon_math::length(p.angular_velocity))
                    .collect();
                speeds.sort_by(f64::total_cmp);
                spins.sort_by(f64::total_cmp);
                eprintln!(
                    "motion: speed p50/p95 {:.5}/{:.5} m/s, spin p50/p95 {:.5}/{:.5} rad/s",
                    speeds[count / 2],
                    speeds[count * 95 / 100],
                    spins[count / 2],
                    spins[count * 95 / 100]
                );
                eprintln!(
                    "fixture preparation: {step} steps, {sleeping}/{} asleep, {} quiet, {} moving, {:.3} wall seconds",
                    count,
                    measurements.solver.quiet_objects,
                    measurements.moving_objects,
                    started.elapsed().as_secs_f64()
                );
            }
            if sleeping == count {
                eprintln!(
                    "fixture settled naturally after {step} steps ({:.3} simulation seconds, {:.3} wall seconds)",
                    step as f64 * 0.016,
                    started.elapsed().as_secs_f64()
                );
                break;
            }
        }
        checkpoint(app, frame, eye, last_step)?;
        if sleeping != count {
            return Err(format!(
                "fixture did not sleep within {} production steps: {sleeping}/{} asleep",
                limit, count
            ));
        }
    }
    Ok(())
}

/// Optional diagnostic export; never changes the fixture or activation state.
/// Each checkpoint replaces the file so failed preparation retains its last state.
fn checkpoint(
    app: &ClientApplication,
    frame: salimon_character::ShipFrame,
    eye: [f64; 3],
    step: u64,
) -> Result<(), String> {
    let Some(path) = std::env::var_os("SALIMON_FIXTURE_CHECKPOINT") else {
        return Ok(());
    };
    let states = crate::fragment_physics::snapshots(&app.mining, frame, eye);
    let data = serde_json::json!({"step":step, "states":states.iter().map(|(piece, p)| {
        serde_json::json!({"id":piece.id().0, "position":p.position,
            "orientation":p.orientation, "velocity":p.velocity,
            "angular_velocity":p.angular_velocity, "radius":p.radius,
            "mass_kg":p.mass_kg, "side_meters":p.side_meters,
            "sleeping":app.mining.fragment_sleep.is_sleeping(piece.id()),
            "vertices":crate::resource_presentation::fragment_mesh(*piece).mesh.unit_vertices()})
    }).collect::<Vec<_>>()});
    std::fs::write(path, serde_json::to_vec(&data).map_err(|e| e.to_string())?)
        .map_err(|e| format!("fixture checkpoint: {e}"))
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn onboard_benchmark_preserves_contained_flight_load_and_active_motion() {
        for active in [false, true] {
            let mut app = ClientApplication::default();
            initialize(&mut app, 500, Layout::Onboard, 199, active).unwrap();
            assert_eq!(app.mining.session.fragments().len(), 500);
            assert!((app.ship.snapshot().total_mass_kg - 100_016.0).abs() < 1e-6);
            assert!(
                app.mining
                    .ship_fragments
                    .values()
                    .all(|p| { salimon_character::ship_contains_object(*p) })
            );
            assert_eq!(
                app.mining.fragment_motion.len(),
                if active { 500 } else { 0 }
            );
            for motion in app.mining.fragment_motion.values() {
                assert_eq!(motion.velocity, [0.0, 0.4, 0.0]);
            }
        }
        assert!(
            initialize(
                &mut ClientApplication::default(),
                501,
                Layout::Onboard,
                199,
                false
            )
            .is_err()
        );
    }

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

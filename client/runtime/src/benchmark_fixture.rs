//! Deterministic initial populations only; all later updates use production state.
use crate::fragment_physics::{FragmentMotion, sphere_radius};
use crate::{
    session::GameplaySession,
    session::frames::{character_ship_frame, surface_frame_for_ship},
};
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
    SphereDeck,
    SpherePile,
    CurrentCap,
    CurrentMixed,
}
impl Layout {
    pub(crate) fn parse(value: &str) -> Result<Self, String> {
        match value {
            "scattered" => Ok(Self::Scattered),
            "dense" => Ok(Self::Dense),
            "surface" => Ok(Self::Surface),
            "onboard" => Ok(Self::Onboard),
            "sphere-deck" => Ok(Self::SphereDeck),
            "sphere-pile" => Ok(Self::SpherePile),
            "current-cap" => Ok(Self::CurrentCap),
            "current-mixed" => Ok(Self::CurrentMixed),
            _ => Err(
                "layout must be scattered, dense, surface, onboard, sphere-deck, sphere-pile, current-cap or current-mixed"
                    .into(),
            ),
        }
    }
    pub(crate) fn name(self) -> &'static str {
        match self {
            Self::Scattered => "scattered",
            Self::Dense => "dense",
            Self::Surface => "surface",
            Self::Onboard => "onboard",
            Self::SphereDeck => "sphere-deck",
            Self::SpherePile => "sphere-pile",
            Self::CurrentCap => "current-cap",
            Self::CurrentMixed => "current-mixed",
        }
    }
}

/// Create one source per fragment through normal mass extraction. Fixture IDs,
/// materials, sizes and rotations are independent of machine/iteration timing.
pub(crate) fn initialize(
    app: &mut GameplaySession,
    count: usize,
    layout: Layout,
    seed: u64,
    active: bool,
) -> Result<(), String> {
    if count > 1000 {
        return Err("benchmark fixture count must be 0..1000".into());
    }
    if matches!(layout, Layout::SphereDeck | Layout::SpherePile) {
        return initialize_spheres(app, count, layout == Layout::SpherePile, active);
    }
    if matches!(layout, Layout::CurrentCap | Layout::CurrentMixed) {
        return initialize_current_spheres(app, count, layout == Layout::CurrentMixed, active);
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
        let support = sphere_radius(piece);
        let mut local = match layout {
            Layout::Onboard
            | Layout::SphereDeck
            | Layout::SpherePile
            | Layout::CurrentCap
            | Layout::CurrentMixed => {
                unreachable!("specialized fixture initialized above")
            }
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
            app.mining
                .ship_fragments
                .insert(piece.id(), salimon_character::ShipLocalPoint::new(local));
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

/// Native counterpart of the #235 10 kg production-extraction workload.
/// Keep its mass, deck cells and 100-column pile distinct from legacy fixtures.
fn initialize_spheres(
    app: &mut GameplaySession,
    count: usize,
    pile: bool,
    active: bool,
) -> Result<(), String> {
    use salimon_world::spherical_mining::{MiningConfig, SPHERE_TICK};
    if count > 500 {
        return Err("10 kg sphere workload count must be <=500".into());
    }
    let frame = character_ship_frame(app.ship.snapshot().pose);
    let material = RawMaterial::new(ResourceId::IronOre, 10.0)
        .map_err(|e| format!("sphere workload material: {e:?}"))?;
    let radius = salimon_world::resource_size::fragment_contact_radius_meters(material);
    let mut cells: Vec<_> = (0..600)
        .map(|i| {
            [
                -6.3 + (i % 25) as f64 * 0.21,
                SHIP_FLOOR_HEIGHT_METERS + radius,
                -2.5 + (i / 25) as f64 * 0.21,
            ]
        })
        .filter(|p| salimon_character::ship_floor_placement(*p, radius).is_some())
        .collect();
    if pile {
        cells.truncate(100);
    }
    if cells.is_empty() {
        return Err("no supported sphere workload cells".into());
    }
    for i in 0..count {
        let mut local = cells[i % cells.len()];
        local[1] += (i / cells.len()) as f64 * (2.0 * radius + 0.00001);
        if active {
            local[1] += 0.3;
        }
        let world = frame.local_to_world(local);
        let orientation =
            crate::fragment_physics::to_world_orientation(frame, [0.0, 0.0, 0.0, 1.0]);
        let pose = ResourceTransform::new(
            WorldPosition::new(world[0], world[1], world[2]),
            orientation,
        )
        .map_err(|e| format!("sphere workload pose: {e:?}"))?;
        let mut deposit = ResourceDeposit::new(
            DepositId {
                body: CelestialBodyId::Earth,
                local: 236_000 + i as u64,
            },
            material,
            pose.position(),
            10.0,
        )
        .map_err(|e| format!("sphere workload deposit: {e:?}"))?;
        let mut released = None;
        for _ in 0..400 {
            released = app.mining.session.tick_spherical(
                true,
                Some(&mut deposit),
                SPHERE_TICK,
                MiningConfig::default(),
                pose,
            );
            if released.is_some() {
                break;
            }
        }
        let id = released.ok_or("sphere workload did not release at its 10 kg cap")?;
        app.mining
            .ship_fragments
            .insert(id, salimon_character::ShipLocalPoint::new(local));
        app.mining.fragment_motion.insert(
            id,
            FragmentMotion {
                velocity: if active { [0.0, 0.4, 0.0] } else { [0.0; 3] },
                angular_velocity: [0.0; 3],
                ship_orientation: Some([0.0, 0.0, 0.0, 1.0]),
            },
        );
    }
    app.sync_carried();
    Ok(())
}

/// Canonical deck cells and layer spacing for current-spheres-303-v1.
pub(crate) fn current_sphere_layout() -> (Vec<[f64; 3]>, f64) {
    let max_radius = salimon_world::resource_size::fragment_contact_radius_meters(
        RawMaterial::new(ResourceId::WaterIce, 10.0).expect("valid fixture mass"),
    );
    let cells = (0..600)
        .map(|i| {
            [
                -6.3 + (i % 25) as f64 * 0.30,
                SHIP_FLOOR_HEIGHT_METERS + max_radius,
                -3.4 + (i / 25) as f64 * 0.30,
            ]
        })
        .filter(|p| salimon_character::ship_floor_placement(*p, max_radius).is_some())
        .collect();
    (cells, 2.0 * max_radius + 0.000_01)
}

/// Native and CPU counterpart of current-spheres-303-v1, including 1,000-object stress.
/// Legacy sphere-deck/pile and onboard fixtures retain their original definitions.
fn initialize_current_spheres(
    app: &mut GameplaySession,
    count: usize,
    mixed: bool,
    active: bool,
) -> Result<(), String> {
    use salimon_world::spherical_mining::{MiningConfig, SPHERE_TICK};
    let frame = character_ship_frame(app.ship.snapshot().pose);
    let (cells, layer_spacing) = current_sphere_layout();
    if cells.is_empty() {
        return Err("no supported current-sphere cells".into());
    }
    for i in 0..count {
        let mass = if mixed {
            [1e-9, 0.000_003, 0.032, 0.25, 2.0, 10.0][i % 6]
        } else {
            10.0
        };
        let resource = if mixed {
            [
                ResourceId::IronOre,
                ResourceId::SilicateRock,
                ResourceId::WaterIce,
            ][(i / 6) % 3]
        } else {
            ResourceId::IronOre
        };
        let material =
            RawMaterial::new(resource, mass).map_err(|e| format!("fixture mass: {e:?}"))?;
        let mut local = cells[i % cells.len()];
        local[1] += (i / cells.len()) as f64 * layer_spacing + if active { 0.30 } else { 0.0 };
        if !salimon_character::ship_contains_object(local) {
            return Err(format!("current-sphere fixture leaves cabin: {local:?}"));
        }
        let world = frame.local_to_world(local);
        let pose = ResourceTransform::new(
            WorldPosition::new(world[0], world[1], world[2]),
            crate::fragment_physics::to_world_orientation(frame, [0.0, 0.0, 0.0, 1.0]),
        )
        .map_err(|e| format!("fixture pose: {e:?}"))?;
        let mut deposit = ResourceDeposit::new(
            DepositId {
                body: CelestialBodyId::Earth,
                local: 303_000 + i as u64,
            },
            material,
            pose.position(),
            mass,
        )
        .map_err(|e| format!("fixture deposit: {e:?}"))?;
        let mut released = None;
        for _ in 0..400 {
            released = app.mining.session.tick_spherical(
                true,
                Some(&mut deposit),
                SPHERE_TICK,
                MiningConfig::default(),
                pose,
            );
            if released.is_some() {
                break;
            }
        }
        let id = released.ok_or("current-sphere default extraction did not release")?;
        app.mining
            .ship_fragments
            .insert(id, salimon_character::ShipLocalPoint::new(local));
        app.mining.fragment_motion.insert(
            id,
            FragmentMotion {
                velocity: if active { [0.0, 0.4, 0.0] } else { [0.0; 3] },
                angular_velocity: [0.0; 3],
                ship_orientation: Some([0.0, 0.0, 0.0, 1.0]),
            },
        );
    }
    app.sync_carried();
    Ok(())
}

/// Prepare a fixture only through production updates; never install rest poses or flags.
pub(crate) fn settle(app: &mut GameplaySession, count: usize, limit: u64) -> Result<(), String> {
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
    app: &GameplaySession,
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
            "mass_kg":p.mass_kg, "side_meters":p.authored_mesh_scale,
            "sleeping":app.mining.fragment_sleep.is_sleeping(piece.id()),
            "collision_shape":"sphere"})
    }).collect::<Vec<_>>()});
    std::fs::write(path, serde_json::to_vec(&data).map_err(|e| e.to_string())?)
        .map_err(|e| format!("fixture checkpoint: {e}"))
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn current_native_profiles_have_real_count_mass_radius_and_containment() {
        for layout in [Layout::CurrentCap, Layout::CurrentMixed] {
            assert_eq!(Layout::parse(layout.name()).unwrap(), layout);
            let mut app = GameplaySession::default();
            initialize(&mut app, 1000, layout, 303, true).unwrap();
            let pieces = app.mining.session.fragments();
            assert_eq!(pieces.len(), 1000);
            assert_eq!(
                pieces
                    .iter()
                    .map(|p| p.id().0)
                    .collect::<std::collections::BTreeSet<_>>()
                    .len(),
                1000
            );
            for (i, piece) in pieces.iter().enumerate() {
                let expected_mass = if layout == Layout::CurrentMixed {
                    [1e-9, 0.000_003, 0.032, 0.25, 2.0, 10.0][i % 6]
                } else {
                    10.0
                };
                assert!((piece.material().mass_kg() - expected_mass).abs() < 1e-9);
                let radius = (3.0 * expected_mass
                    / (4.0
                        * std::f64::consts::PI
                        * piece.material().resource().definition().density_kg_per_m3))
                    .cbrt()
                    .clamp(0.005, 2.0);
                assert!((sphere_radius(*piece) - radius).abs() < 1e-12);
                assert!(salimon_character::ship_contains_object(
                    app.mining.ship_fragments[&piece.id()].meters()
                ));
                assert_eq!(
                    app.mining.fragment_motion[&piece.id()].velocity,
                    [0.0, 0.4, 0.0]
                );
            }
            let actual_mass: f64 = pieces.iter().map(|p| p.material().mass_kg()).sum();
            assert!((app.ship.snapshot().total_mass_kg - 100_000.0 - actual_mass).abs() < 1e-8);
        }
    }

    #[test]
    fn ten_kg_native_spheres_conserve_identity_mass_and_naturally_sleep() {
        for layout in [Layout::SphereDeck, Layout::SpherePile] {
            assert_eq!(Layout::parse(layout.name()).unwrap(), layout);
            let mut app = GameplaySession::default();
            initialize(&mut app, 500, layout, 236, false).unwrap();
            let initial = crate::benchmark::accounting(&app);
            assert_eq!(app.mining.session.fragments().len(), 500);
            // The session accumulator adds ~157,000 tick masses; allow only
            // 50 micrograms of f64 summation error across five tonnes.
            assert!((app.mining.session.extracted_mass_kg() - 5000.0).abs() < 5e-8);
            assert!(
                app.mining
                    .session
                    .fragments()
                    .iter()
                    .all(|p| p.material().mass_kg() == 10.0)
            );
            assert!((app.ship.snapshot().total_mass_kg - 105_000.0).abs() < 1e-6);
            settle(&mut app, 500, 1200).unwrap();
            assert_eq!(crate::benchmark::accounting(&app), initial);
            assert!(
                app.mining
                    .session
                    .fragments()
                    .iter()
                    .all(|p| app.mining.fragment_sleep.is_sleeping(p.id()))
            );
        }
        let mut active = GameplaySession::default();
        initialize(&mut active, 10, Layout::SphereDeck, 236, true).unwrap();
        assert!(
            active
                .mining
                .fragment_motion
                .values()
                .all(|m| m.velocity[1] > 0.0)
        );
        assert!(initialize(&mut active, 501, Layout::SphereDeck, 236, false).is_err());
    }

    #[test]
    fn onboard_benchmark_preserves_contained_flight_load_and_active_motion() {
        for active in [false, true] {
            let mut app = GameplaySession::default();
            initialize(&mut app, 500, Layout::Onboard, 199, active).unwrap();
            assert_eq!(app.mining.session.fragments().len(), 500);
            assert!((app.ship.snapshot().total_mass_kg - 100_016.0).abs() < 1e-6);
            assert!(
                app.mining
                    .ship_fragments
                    .values()
                    .all(|p| { salimon_character::ship_contains_object(p.meters()) })
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
                &mut GameplaySession::default(),
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
            let mut app = GameplaySession::default();
            initialize(&mut app, count, Layout::Dense, 155, true).unwrap();
            let pieces = app.mining.session.fragments();
            let total: f64 = pieces.iter().map(|p| p.material().mass_kg()).sum();
            assert_eq!(app.mining.session.extracted_mass_kg(), total);
            assert_eq!(pieces.len(), count);
            let before = pieces.to_vec();
            let mut other = GameplaySession::default();
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

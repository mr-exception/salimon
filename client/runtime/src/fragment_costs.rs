//! Corrected, deterministic portable attribution. No GPU or native FPS claim.
use crate::{
    app::character_ship_frame, carrying, fragment_physics, mining::MiningTool, sphere_fixtures,
};
use salimon_character::{SHIP_FLOOR_HEIGHT_METERS, ShipFrame};
use salimon_ship::{ShipController, ShipPose};
use salimon_world::{
    SURFACE_ANCHOR, WorldPosition,
    resources::{RawMaterial, ResourceId, ResourceTransform},
};
use serde_json::json;
use std::time::{Duration, Instant};

fn pose(p: [f64; 3]) -> ResourceTransform {
    ResourceTransform::new(WorldPosition::new(p[0], p[1], p[2]), [0.0, 0.0, 0.0, 1.0]).unwrap()
}

fn fixture(count: usize, history: usize, mode: &str) -> (MiningTool, ShipController, ShipFrame) {
    let onboard = matches!(mode, "moving-ship" | "zero-g");
    let mut tool = MiningTool::default();
    let ship = ShipController::flying(
        ShipPose {
            position_meters: SURFACE_ANCHOR.translated([0.0, 0.0, 20.0]).meters(),
            orientation: [0.0, 0.0, 0.0, 1.0],
        },
        0,
    );
    let frame = character_ship_frame(ship.snapshot().pose);
    for i in 0..history {
        let p = SURFACE_ANCHOR
            .translated([1000.0 + i as f64, 0.0, 0.0])
            .meters();
        sphere_fixtures::sphere(&mut tool.session, ResourceId::IronOre, 0.032, pose(p));
    }
    let cells: Vec<_> = (0..160 * 90)
        .map(|i| {
            [
                -6.3 + (i % 160) as f64 * 0.08,
                SHIP_FLOOR_HEIGHT_METERS + 0.03,
                -3.4 + (i / 160) as f64 * 0.08,
            ]
        })
        .filter(|p| salimon_character::ship_floor_placement(*p, 0.03).is_some())
        .collect();
    assert!(cells.len() >= count);
    for i in 0..count {
        let resource = [
            ResourceId::IronOre,
            ResourceId::SilicateRock,
            ResourceId::WaterIce,
        ][i % 3];
        let mass = if onboard {
            0.032
        } else if mode == "mixed" {
            [10.0, 0.032, 0.000003][i % 3]
        } else {
            10.0
        };
        let radius = salimon_world::resource_size::fragment_contact_radius_meters(
            RawMaterial::new(resource, mass).unwrap(),
        );
        let mut local = cells[i];
        local[1] = SHIP_FLOOR_HEIGHT_METERS + radius + if mode == "zero-g" { 0.5 } else { 0.0 };
        let p = if onboard {
            frame.local_to_world(local)
        } else {
            SURFACE_ANCHOR
                .translated([
                    (i % 32) as f64 * 0.4,
                    (i / 32) as f64 * 0.4,
                    radius + if mode == "active" { 1.0 } else { 0.0 },
                ])
                .meters()
        };
        let id = sphere_fixtures::sphere(&mut tool.session, resource, mass, pose(p));
        if onboard {
            tool.ship_fragments.insert(id, local);
            tool.fragment_motion.insert(
                id,
                fragment_physics::FragmentMotion {
                    velocity: if mode == "zero-g" {
                        [0.1, 0.0, 0.0]
                    } else {
                        [0.0; 3]
                    },
                    ship_orientation: Some([0.0, 0.0, 0.0, 1.0]),
                    ..Default::default()
                },
            );
        }
    }
    tool.ship_gravity_disabled = mode == "zero-g";
    assert_eq!(tool.session.fragments().len(), count + history);
    assert_eq!(
        fragment_physics::snapshots(&tool, frame, frame.origin_meters).len(),
        count
    );
    (tool, ship, frame)
}

#[test]
#[ignore = "release CPU adapter/selection/solver/journal/mass attribution; no native FPS"]
fn fragment_update_costs() {
    let path = std::env::var("SALIMON_FRAGMENT_COST_REPORT").expect("set output path");
    let quick = std::env::var_os("SALIMON_FRAGMENT_COST_QUICK").is_some();
    let allocation_mode =
        std::env::var("SALIMON_FRAGMENT_ALLOCATION_MODE").unwrap_or_else(|_| "asleep".to_owned());
    assert!(
        ["asleep", "active", "mixed", "moving-ship", "zero-g"].contains(&allocation_mode.as_str())
    );
    let sample_count: usize = std::env::var("SALIMON_FRAGMENT_COST_SAMPLES")
        .map(|value| value.parse().expect("integer sample count"))
        .unwrap_or(20);
    assert!((1..=1000).contains(&sample_count));
    let mut rows = Vec::new();
    for count in if quick { vec![500] } else { vec![500, 1000] } {
        for history in if quick { vec![0] } else { vec![0, 10_000] } {
            for mode in if quick {
                vec![allocation_mode.as_str()]
            } else {
                vec!["asleep", "active", "mixed", "moving-ship", "zero-g"]
            } {
                let (mut tool, mut ship, mut frame) = fixture(count, history, mode);
                let ids: Vec<_> = tool.session.fragments()[history..]
                    .iter()
                    .map(|p| p.id())
                    .collect();
                let identities: Vec<_> = tool
                    .session
                    .fragments()
                    .iter()
                    .map(|p| (p.id(), p.material()))
                    .collect();
                carrying::sync_ship_mass(&mut tool, frame, &mut ship);
                if matches!(mode, "asleep" | "mixed" | "moving-ship") {
                    for _ in 0..80 {
                        fragment_physics::advance(
                            &mut tool,
                            frame,
                            Duration::from_millis(16),
                            frame.origin_meters,
                        );
                        carrying::sync_ship_mass(&mut tool, frame, &mut ship);
                    }
                    assert!(
                        ids.iter().all(|id| tool.fragment_sleep.is_sleeping(*id)),
                        "settled fixture: {mode}"
                    );
                }
                let inventory:Vec<_>=tool.session.fragments()[history..].iter().map(|p|json!({"id":p.id().0,"mass_kg":p.material().mass_kg(),"resource":p.material().resource().key(),"radius_m":fragment_physics::sphere_radius(*p),"position_m":p.transform().position().meters()})).collect();
                let mut samples = Vec::new();
                for sample in 0..if quick { 5 } else { sample_count } {
                    if mode == "moving-ship" {
                        frame.origin_meters[0] += 0.05;
                    }
                    if matches!(mode, "active" | "mixed") {
                        for (i, id) in ids.iter().enumerate() {
                            if mode == "active" || i % 2 == 0 {
                                tool.fragment_motion.entry(*id).or_default().velocity =
                                    [0.2, 0.0, 0.5];
                            }
                        }
                    }
                    let now = Instant::now();
                    let input = fragment_physics::snapshots(&tool, frame, frame.origin_meters);
                    let selection_ns = now.elapsed().as_nanos();
                    let replay_ids: Vec<_> = input.iter().map(|(p, _)| p.id()).collect();
                    let mut replay: Vec<_> = input.into_iter().map(|(_, s)| s).collect();
                    let mut tracker = tool.fragment_sleep.clone();
                    let now = Instant::now();
                    let replay_stats = salimon_physics::advance_with_sleep(
                        &mut tracker,
                        &replay_ids,
                        &mut replay,
                        Duration::from_millis(16),
                        |p, r| salimon_character::ship_floor_placement(p, r).is_some(),
                        true,
                    );
                    let solver_ns = now.elapsed().as_nanos();
                    let update_start = Instant::now();
                    fragment_physics::capture_previous(&mut tool, frame, frame.origin_meters);
                    let m = fragment_physics::advance_measured(
                        &mut tool,
                        frame,
                        Duration::from_millis(16),
                        frame.origin_meters,
                        true,
                    );
                    let now = Instant::now();
                    carrying::sync_ship_fragments(&mut tool, frame);
                    let frame_sync_ns = now.elapsed().as_nanos();
                    let now = Instant::now();
                    let mass_visits = carrying::sync_ship_mass(&mut tool, frame, &mut ship);
                    let mass_ns = now.elapsed().as_nanos();
                    let total_ns = update_start.elapsed().as_nanos();
                    assert_eq!(m.solver.objects, count);
                    assert_eq!(m.solver.substeps, replay_stats.substeps);
                    let actual = fragment_physics::snapshots(&tool, frame, frame.origin_meters);
                    for ((_, a), b) in actual.iter().zip(&replay) {
                        assert_eq!(a.velocity, b.velocity);
                        assert_eq!(a.angular_velocity, b.angular_velocity);
                        for (x, y) in a.position.iter().zip(b.position) {
                            assert!((x - y).abs() < 0.0002);
                        }
                    }
                    if mode == "asleep" {
                        assert_eq!(m.solver.sleeping_objects, count);
                    }
                    samples.push(json!({"sample":sample,"selection_ns":selection_ns,"solver_replay_ns":solver_ns,"adapter_ns":m.adapter_time.as_nanos(),"physics_total_ns":m.total_time.as_nanos(),"frame_sync_ns":frame_sync_ns,"mass_ns":mass_ns,"mass_visits":format!("{mass_visits:?}"),"total_update_ns":total_ns,"sleeping":m.solver.sleeping_objects,"substeps":m.solver.substeps}));
                }
                assert_eq!(
                    tool.session
                        .fragments()
                        .iter()
                        .map(|p| (p.id(), p.material()))
                        .collect::<Vec<_>>(),
                    identities
                );
                rows.push(json!({"active":count,"inactive":history,"mode":mode,"population":inventory,"samples":samples}));
                eprintln!("fragment attribution {count}+{history} {mode} passed");
            }
        }
    }
    let report = json!({"schema":1,"fixture":"fragment-attribution-305-306-v1","step_ms":16,"samples_per_case":if quick {5} else {sample_count},"gpu_fps":null,"rows":rows});
    std::fs::write(path, serde_json::to_vec_pretty(&report).unwrap()).unwrap();
}

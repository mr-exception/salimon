//! Separately versioned current-sphere CPU workloads; never a native FPS gate.
use crate::{carrying, fragment_physics, mining::ResourceGameplay};
use salimon_ship::{ShipController, ShipPose};
use salimon_world::spherical_mining::SPHERE_TICK;
use serde_json::json;
use std::time::Instant;

fn population(count: usize, mixed: bool) -> (ResourceGameplay, ShipController, serde_json::Value) {
    let mut app = crate::session::GameplaySession::default();
    app.ship = ShipController::flying(
        ShipPose {
            position_meters: [1e12, 0.0, 0.0],
            orientation: [0.0, 0.0, 0.0, 1.0],
        },
        0,
    );
    crate::benchmark_fixture::initialize(
        &mut app,
        count,
        if mixed {
            crate::benchmark_fixture::Layout::CurrentMixed
        } else {
            crate::benchmark_fixture::Layout::CurrentCap
        },
        303,
        true,
    )
    .unwrap();
    let (cells, layer_spacing) = crate::benchmark_fixture::current_sphere_layout();
    let expected_mass: f64 = (0..count)
        .map(|i| {
            if mixed {
                [1e-9, 0.000_003, 0.032, 0.25, 2.0, 10.0][i % 6]
            } else {
                10.0
            }
        })
        .sum();
    let tool = app.mining;
    let ship = app.ship;
    let inventory: Vec<_> = tool.session.fragments().iter().map(|piece| json!({
        "id":piece.id().0,"source":piece.source().local,"resource":piece.material().resource().key(),
        "mass_kg":piece.material().mass_kg(),"radius_meters":fragment_physics::sphere_radius(*piece),
        "initial_ship_local_meters":tool.ship_fragments[&piece.id()].meters()
    })).collect();
    let actual_mass: f64 = tool
        .session
        .fragments()
        .iter()
        .map(|p| p.material().mass_kg())
        .sum();
    assert!((actual_mass - expected_mass).abs() < 1e-8);
    let metadata = json!({"requested_count":count,"actual_count":inventory.len(),
        "actual_mass_kg":actual_mass,"layout":"supported-deck-layers-active-v1",
        "columns":cells.len(),"layer_spacing_meters":layer_spacing,
        "profile":if mixed {"mixed-residuals-v1"} else {"cap-10kg-iron-v1"},
        "population":inventory});
    (tool, ship, metadata)
}

#[test]
fn current_sphere_population_contract() {
    let (tool, ship, metadata) = population(18, true);
    assert_eq!(tool.session.fragments().len(), 18);
    assert_eq!(metadata["actual_count"], 18);
    assert!(ship.snapshot().total_mass_kg > 100_000.0);
    let ids: std::collections::BTreeSet<_> =
        tool.session.fragments().iter().map(|p| p.id().0).collect();
    assert_eq!(ids.len(), 18);
}

#[test]
#[ignore = "opt-in versioned 500/1000 current-sphere CPU workloads; never native FPS"]
fn current_sphere_workloads_cpu() {
    let path = std::env::var("SALIMON_CURRENT_SPHERE_CPU_REPORT").expect("set output report path");
    let mut rows = Vec::new();
    for count in [500, 1000] {
        for mixed in [false, true] {
            let (mut tool, mut ship, mut metadata) = population(count, mixed);
            let frame = crate::session::frames::character_ship_frame(ship.snapshot().pose);
            let eye = frame.local_to_world([-5.0, 2.0, 0.0]);
            let identities: Vec<_> = tool
                .session
                .fragments()
                .iter()
                .map(|p| (p.id(), p.material()))
                .collect();
            let mut samples = Vec::new();
            for sample in 0..100 {
                let now = Instant::now();
                let m =
                    fragment_physics::advance_measured(&mut tool, frame, SPHERE_TICK, eye, true);
                let adapter_ms = now.elapsed().as_secs_f64() * 1000.0;
                let now = Instant::now();
                carrying::sync_ship_mass(&mut tool, frame, &mut ship);
                let mass_sync_ms = now.elapsed().as_secs_f64() * 1000.0;
                assert_eq!(m.solver.objects, count);
                assert_eq!(m.solver.projection_builds, 0);
                assert_eq!(
                    tool.session
                        .fragments()
                        .iter()
                        .map(|p| (p.id(), p.material()))
                        .collect::<Vec<_>>(),
                    identities
                );
                assert!(tool.session.carried_id().is_none());
                assert!(
                    (ship.snapshot().total_mass_kg
                        - 100_000.0
                        - metadata["actual_mass_kg"].as_f64().unwrap())
                    .abs()
                        < 1e-8
                );
                samples.push(json!({"sample":sample,"adapter_total_ms":adapter_ms,
                    "mass_sync_ms":mass_sync_ms,"activation_ms":m.solver.activation_time.as_secs_f64()*1000.0,
                    "contact_ms":m.solver.contact_time.as_secs_f64()*1000.0,
                    "moving":m.moving_objects,"sleeping":m.solver.sleeping_objects,
                    "objects":m.solver.objects,"substeps":m.solver.substeps}));
            }
            metadata["samples"] = json!(samples);
            eprintln!("current sphere CPU: {count}, mixed={mixed} passed");
            rows.push(metadata);
        }
    }
    std::fs::write(
        path,
        serde_json::to_vec_pretty(&json!({"schema":1,
        "workload":"current-spheres-303-v1","mode":"portable-cpu-runtime-adapter",
        "step_ms":16,"samples_per_case":100,"gpu_fps":null,"rows":rows}))
        .unwrap(),
    )
    .unwrap();
}

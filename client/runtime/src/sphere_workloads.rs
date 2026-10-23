//! Separately versioned current-sphere CPU workloads; never a native FPS gate.
use crate::{carrying, fragment_physics, mining::ResourceGameplay};
use salimon_ship::{ShipController, ShipPose};
use salimon_world::spherical_mining::SPHERE_TICK;
use serde_json::json;
use std::time::Instant;

fn population(
    count: usize,
    mixed: bool,
    active: bool,
) -> (ResourceGameplay, ShipController, serde_json::Value) {
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
        active,
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
    let (tool, ship, metadata) = population(18, true, true);
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
            let (mut tool, mut ship, mut metadata) = population(count, mixed, true);
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

/// Flushed records retain every completed step on a bounded subprocess timeout.
#[test]
#[ignore = "serial exact current-sphere stress evidence; set SALIMON_STRESS_OUTPUT"]
fn current_sphere_stress_cpu() {
    use std::io::Write;
    let path = std::env::var("SALIMON_STRESS_OUTPUT").unwrap();
    let count: usize = std::env::var("SALIMON_STRESS_COUNT")
        .unwrap()
        .parse()
        .unwrap();
    assert!([500, 1000].contains(&count));
    let mixed = std::env::var("SALIMON_STRESS_MIXED").unwrap() == "1";
    let settled = std::env::var("SALIMON_STRESS_SETTLED").unwrap() == "1";
    let archived: serde_json::Value =
        serde_json::from_str(include_str!("sphere_workloads/historical-inputs.json")).unwrap();
    let mut output = std::io::BufWriter::new(std::fs::File::create(path).unwrap());
    let mut emit = |value: serde_json::Value| {
        serde_json::to_writer(&mut output, &value).unwrap();
        writeln!(&mut output).unwrap();
        output.flush().unwrap();
    };
    // Diagnostic exact-prefix snapshots never qualify as the 100-sample pool.
    let target_samples = std::env::var("SALIMON_STRESS_PREFIX_SAMPLES")
        .ok()
        .map(|s| s.parse::<usize>().expect("prefix sample count"))
        .unwrap_or(100);
    assert!((1..=100).contains(&target_samples));
    let mut accepted = 0;
    for launch in 0..20 {
        let (tool, ship, mut metadata) = population(count, mixed, !settled);
        let frame = crate::session::frames::character_ship_frame(ship.snapshot().pose);
        let eye = frame.local_to_world([-5.0, 2.0, 0.0]);
        let initial = fragment_physics::snapshots(&tool, frame, eye);
        let ids: Vec<_> = initial.iter().map(|(p, _)| p.id()).collect();
        let mut states: Vec<_> = initial.into_iter().map(|(_, s)| s).collect();
        let mut tracker = salimon_physics::SleepTracker::default();
        for (i, state) in states.iter_mut().enumerate() {
            let entry = &archived["entries"][if mixed { i % 18 } else { 5 }];
            assert_eq!(metadata["population"][i]["resource"], entry["resource"]);
            state.radius = f64::from_bits(entry["radius_bits"].as_u64().unwrap());
            state.ground_support_meters = state.radius;
            state.mass_kg = f64::from_bits(entry["mass_bits"].as_u64().unwrap());
            metadata["population"][i]["solver_radius_bits"] = json!(state.radius.to_bits());
            metadata["population"][i]["solver_mass_bits"] = json!(state.mass_kg.to_bits());
        }
        metadata["mode"] = json!("exact-archived-snapshot-solver");
        emit(json!({"type":"population","launch":launch,"settled":settled,"metadata":metadata}));
        let state_record = |states: &[salimon_physics::ObjectState]| json!({"type":"state","fragments":states.iter().map(|p|json!({"position":p.position,"velocity":p.velocity,"orientation":p.orientation,"angular":p.angular_velocity,"mass_bits":p.mass_kg.to_bits(),"radius_bits":p.radius.to_bits()})).collect::<Vec<_>>()});
        let mut prepared = !settled;
        for step in 0..if settled { 400 } else { 150 } {
            let now = Instant::now();
            let m = salimon_physics::advance_with_sleep(
                &mut tracker,
                &ids,
                &mut states,
                SPHERE_TICK,
                |position, radius| {
                    salimon_character::ship_floor_placement(position, radius).is_some()
                },
                true,
            );
            let total_ms = now.elapsed().as_secs_f64() * 1000.0;
            let moving = states
                .iter()
                .filter(|p| {
                    salimon_math::length(p.velocity) >= 0.015
                        || salimon_math::length(p.angular_velocity) >= 0.015
                })
                .count();
            assert_eq!(m.objects, count);
            let preparation = !prepared;
            emit(
                json!({"type":"sample","launch":launch,"step":step,"preparation":preparation,
                "total_ms":total_ms,"contact_ms":m.contact_time.as_secs_f64()*1000.0,
                "activation_ms":m.activation_time.as_secs_f64()*1000.0,
                "integration_ms":m.integration_time.as_secs_f64()*1000.0,
                "grid_query_ms":m.grid_query_time.as_secs_f64()*1000.0,"grid_queries":m.grid_queries,
                "moving":moving,"sleeping":m.sleeping_objects,"substeps":m.substeps,
                "passes":m.solver_passes,"pair_visits":m.pair_visits,"contacts":m.contacts}),
            );
            if preparation {
                if m.sleeping_objects == count {
                    prepared = true;
                } else if step == 299 {
                    emit(state_record(&states));
                    emit(
                        json!({"type":"outcome","status":"failed-natural-sleep","sleeping":m.sleeping_objects,"requested":count,"steps":300}),
                    );
                    return;
                }
            } else if settled || moving > 0 {
                accepted += 1;
                if accepted == target_samples {
                    emit(state_record(&states));
                    emit(
                        json!({"type":"outcome","status":if target_samples == 100 {"completed"} else {"diagnostic-prefix"},"accepted_samples":accepted,"launches":launch+1}),
                    );
                    return;
                }
            } else {
                emit(state_record(&states));
                break;
            }
        }
        if settled {
            break;
        }
    }
    emit(json!({"type":"outcome","status":"insufficient-samples","accepted_samples":accepted}));
}

//! Opt-in portable measurements: production runtime adapter, no native FPS claim.
use crate::{fragment_physics, mining::MiningTool};
use salimon_character::{SHIP_FLOOR_HEIGHT_METERS, ShipFrame};
use salimon_world::{
    CelestialBodyId, WorldPosition,
    resources::{DepositId, RawMaterial, ResourceDeposit, ResourceId, ResourceTransform},
    spherical_mining::{MiningConfig, SPHERE_TICK},
};
use serde_json::json;
use std::time::Instant;

#[test]
#[ignore = "opt-in 10/100/500-sphere CPU benchmark; never native FPS"]
fn sphere_cpu_benchmark() {
    let frame = ShipFrame {
        origin_meters: [0.0; 3],
        axes: [[1.0, 0.0, 0.0], [0.0, 1.0, 0.0], [0.0, 0.0, 1.0]],
    };
    let eye = [-5.0, 2.0, 0.0];
    let mut rows = Vec::new();
    for count in [10, 100, 500] {
        for state in ["resting", "active", "pile-resting"] {
            let mut tool = MiningTool::default();
            let material = RawMaterial::new(ResourceId::IronOre, 10.0).unwrap();
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
            assert!(!cells.is_empty());
            if state == "pile-resting" {
                cells.truncate(100);
            }
            for i in 0..count {
                let mut local = cells[i % cells.len()];
                local[1] += (i / cells.len()) as f64 * (2.0 * radius + 0.00001);
                if state == "active" {
                    local[1] += 0.3;
                }
                let pose = ResourceTransform::new(
                    WorldPosition::new(local[0], local[1], local[2]),
                    [0.0, 0.0, 0.0, 1.0],
                )
                .unwrap();
                let mut deposit = ResourceDeposit::new(
                    DepositId {
                        body: CelestialBodyId::Earth,
                        local: 235_000 + i as u64,
                    },
                    material,
                    pose.position(),
                    10.0,
                )
                .unwrap();
                let mut id = None;
                for _ in 0..400 {
                    id = tool.session.tick_spherical(
                        true,
                        Some(&mut deposit),
                        SPHERE_TICK,
                        MiningConfig::default(),
                        pose,
                    );
                    if id.is_some() {
                        break;
                    }
                }
                let id = id.unwrap();
                tool.ship_fragments.insert(id, local);
            }
            let ids: Vec<_> = tool.session.fragments().iter().map(|p| p.id()).collect();
            let mut preparation = 0;
            if state != "active" {
                for step in 0..1200 {
                    let m = fragment_physics::advance_measured(
                        &mut tool,
                        frame,
                        SPHERE_TICK,
                        eye,
                        true,
                    );
                    preparation = step + 1;
                    if m.solver.sleeping_objects == count {
                        break;
                    }
                }
                assert!(
                    ids.iter().all(|id| tool.fragment_sleep.is_sleeping(*id)),
                    "{count} did not naturally sleep"
                );
            }
            let mut samples = Vec::new();
            for _ in 0..100 {
                let now = Instant::now();
                let m =
                    fragment_physics::advance_measured(&mut tool, frame, SPHERE_TICK, eye, true);
                let wall = now.elapsed().as_secs_f64() * 1000.0;
                assert_eq!(m.solver.objects, count);
                assert_eq!(m.solver.projection_builds, 0);
                if state != "active" {
                    assert_eq!(m.solver.integrated_objects, 0);
                    assert_eq!(m.solver.pair_visits, 0);
                }
                assert_eq!(
                    tool.session
                        .fragments()
                        .iter()
                        .map(|p| p.id())
                        .collect::<Vec<_>>(),
                    ids
                );
                assert!(
                    (tool
                        .session
                        .fragments()
                        .iter()
                        .map(|p| p.material().mass_kg())
                        .sum::<f64>()
                        - 10.0 * count as f64)
                        .abs()
                        < 1e-8
                );
                samples.push(json!({"adapter_total_ms":wall,"activation_ms":m.solver.activation_time.as_secs_f64()*1000.0,"contact_ms":m.solver.contact_time.as_secs_f64()*1000.0,"pair_visits":m.solver.pair_visits,"contacts":m.solver.contacts,"sleeping":m.solver.sleeping_objects,"moving":m.moving_objects,"substeps":m.solver.substeps}));
            }
            rows.push(json!({"count":count,"state":state,"mass_per_sphere_kg":10,"preparation_steps":preparation,"samples":samples}));
        }
    }
    let path = std::env::var("SALIMON_SPHERE_CPU_REPORT").expect("set SALIMON_SPHERE_CPU_REPORT");
    std::fs::write(path,serde_json::to_vec_pretty(&json!({"schema":1,"mode":"cpu-runtime-adapter","samples_per_case":100,"simulation_step_ms":16,"gpu_fps":null,"rows":rows})).unwrap()).unwrap();
}

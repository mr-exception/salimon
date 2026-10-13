//! Opt-in #160 evaluation. Box controls are rejected diagnostic geometry, not assets.
use salimon_physics::{ConvexHull, ObjectState, SleepTracker, Surface, advance_with_sleep};
use salimon_renderer::ResourceMesh;
use serde_json::json;
use std::{
    sync::Arc,
    time::{Duration, Instant},
};

const MESHES: [ResourceMesh; 18] = [
    ResourceMesh::IceShard,
    ResourceMesh::IceCluster,
    ResourceMesh::SilicateSlab,
    ResourceMesh::SilicateRidge,
    ResourceMesh::IronChunk,
    ResourceMesh::IronShard,
    ResourceMesh::IceDepositSpire,
    ResourceMesh::IceDepositCrown,
    ResourceMesh::IceDepositRidge,
    ResourceMesh::IceDepositShelf,
    ResourceMesh::SilicateDepositBoulder,
    ResourceMesh::SilicateDepositSlab,
    ResourceMesh::SilicateDepositRidge,
    ResourceMesh::SilicateDepositScree,
    ResourceMesh::IronDepositNodule,
    ResourceMesh::IronDepositVein,
    ResourceMesh::IronDepositLedge,
    ResourceMesh::IronDepositRubble,
];
const TICK: Duration = Duration::from_millis(16);

fn box_control(mesh: ResourceMesh) -> ConvexHull {
    let points = mesh.unit_vertices();
    let low: [f64; 3] =
        std::array::from_fn(|a| points.iter().map(|v| v[a]).fold(f64::INFINITY, f64::min));
    let high: [f64; 3] = std::array::from_fn(|a| {
        points
            .iter()
            .map(|v| v[a])
            .fold(f64::NEG_INFINITY, f64::max)
    });
    ConvexHull::new(
        (0..8).map(|i| std::array::from_fn(|a| if i & (1 << a) == 0 { low[a] } else { high[a] })),
    )
}

fn object(hull: Arc<ConvexHull>) -> ObjectState {
    let side = 0.4;
    ObjectState {
        position: [
            0.0,
            hull.support([0.0, 0.0, 0.0, 1.0], side, [0.0, -1.0, 0.0]),
            0.0,
        ],
        velocity: [0.0; 3],
        angular_velocity: [0.0; 3],
        radius: side * 0.48 * 3.0_f64.sqrt(),
        ground_support_meters: 0.0,
        side_meters: side,
        mass_kg: 2.0,
        orientation: [0.0, 0.0, 0.0, 1.0],
        surface: Surface::Floor { height_meters: 0.0 },
        hull: Some(hull),
    }
}

#[test]
#[ignore = "opt-in release authored geometry/contact-cost evaluation; no native FPS"]
fn resource_geometry_evaluation() {
    let mut audit = Vec::new();
    for mesh in MESHES {
        let hull = ConvexHull::new(mesh.unit_vertices().iter().copied());
        let c = hull.complexity();
        let proxy = box_control(mesh);
        let mut max_excess: f64 = 0.0;
        // Fibonacci directions include arbitrary future body rotations; extrema
        // are sampled lower bounds, not a certified maximum Hausdorff distance.
        for i in 0..1024 {
            let y = 1.0 - 2.0 * (i as f64 + 0.5) / 1024.0;
            let r = (1.0 - y * y).sqrt();
            let angle = i as f64 * 2.399963229728653;
            let direction = [r * angle.cos(), y, r * angle.sin()];
            let difference = proxy.support([0.0, 0.0, 0.0, 1.0], 1.0, direction)
                - hull.support([0.0, 0.0, 0.0, 1.0], 1.0, direction);
            assert!(difference >= -1e-9);
            max_excess = max_excess.max(difference);
        }
        audit.push(json!({"mesh": format!("{mesh:?}"), "vertices": c.vertices,
            "faces": c.faces, "face_axes": c.face_axes, "edge_axes": c.edge_axes,
            "same_variant_raw_pair_axis_bound": 2 * c.face_axes + c.edge_axes * c.edge_axes,
            "box_support_excess_per_unit_side": max_excess,
            "box_support_excess_at_0_4m_side_mm": max_excess * 400.0,
            "collision_used": audit.len() < 6}));
    }
    let mut rows = Vec::new();
    save(&audit, &rows);
    for proxy in [false, true] {
        let hulls: Vec<_> = MESHES[..6]
            .iter()
            .map(|mesh| {
                Arc::new(if proxy {
                    box_control(*mesh)
                } else {
                    ConvexHull::new(mesh.unit_vertices().iter().copied())
                })
            })
            .collect();
        // Settle six independent production shapes, then tile their actual poses.
        // This isolates geometry; it is not a dense cargo or ship-runtime fixture.
        let settled: Vec<_> = hulls
            .iter()
            .map(|hull| {
                let mut pieces = vec![object(hull.clone())];
                let mut tracker = SleepTracker::default();
                for _ in 0..2400 {
                    advance_with_sleep(&mut tracker, &[0], &mut pieces, TICK, |_, _| true, false);
                    if tracker.is_sleeping(0) {
                        break;
                    }
                }
                assert!(
                    tracker.is_sleeping(0),
                    "{proxy}: hull failed natural settling"
                );
                pieces.remove(0)
            })
            .collect();
        for count in [500, 1000] {
            let ids: Vec<_> = (0..count).collect();
            let mut initial: Vec<_> = (0..count)
                .map(|i| {
                    let mut p = settled[i % 6].clone();
                    p.position[0] += (i % 32) as f64 * 2.0;
                    p.position[2] += (i / 32) as f64 * 2.0;
                    p
                })
                .collect();
            let mut tracker = SleepTracker::default();
            for _ in 0..40 {
                advance_with_sleep(&mut tracker, &ids, &mut initial, TICK, |_, _| true, false);
            }
            assert!(ids.iter().all(|id| tracker.is_sleeping(*id)));
            for state in ["settled", "active_floor", "active_pairs"] {
                let mut source = initial.clone();
                if state != "settled" {
                    for (i, p) in source.iter_mut().enumerate() {
                        p.position[1] += 0.002;
                        p.velocity = [0.0, -1.0, 0.0];
                        p.angular_velocity = [0.0, 0.2, 0.0];
                        if state == "active_pairs" && i % 2 == 1 {
                            // Supported dimers exercise SAT/manifold response without
                            // deep initial penetration dominating the control workload.
                            let base = &initial[i - 1];
                            p.position[0] = base.position[0];
                            p.position[2] = initial[i - 1].position[2];
                            p.position[1] = base.position[1]
                                + 0.002
                                + base.hull.as_ref().unwrap().support(
                                    base.orientation,
                                    base.side_meters,
                                    [0.0, 1.0, 0.0],
                                )
                                + p.hull.as_ref().unwrap().support(
                                    p.orientation,
                                    p.side_meters,
                                    [0.0, -1.0, 0.0],
                                );
                        }
                    }
                }
                let mut samples = Vec::new();
                let mut last = Default::default();
                // Pair replays are exploratory: bound an intentionally expensive
                // active workload rather than turn this probe into native acceptance.
                let (warmup, sample_count) = if state == "active_pairs" {
                    (2, 20)
                } else {
                    (10, 100)
                };
                for sample in 0..warmup + sample_count {
                    let mut pieces = source.clone();
                    let mut cache = tracker.clone();
                    let start = Instant::now();
                    last =
                        advance_with_sleep(&mut cache, &ids, &mut pieces, TICK, |_, _| true, true);
                    if sample >= warmup {
                        samples.push(start.elapsed().as_secs_f64() * 1000.0);
                    }
                    assert_eq!(pieces.len(), count);
                    assert!(
                        pieces
                            .iter()
                            .all(|p| p.mass_kg == 2.0 && p.position.iter().all(|x| x.is_finite()))
                    );
                    if state == "settled" {
                        assert_eq!(pieces, source);
                    }
                }
                samples.sort_by(f64::total_cmp);
                rows.push(
                    json!({"geometry": if proxy {"box_control"} else {"authored"},
                    "count": count, "state": state, "median_ms": samples[sample_count / 2],
                    "p95_ms": samples[(sample_count * 95).div_ceil(100) - 1],
                    "p99_ms": samples[(sample_count * 99).div_ceil(100) - 1], "samples_ms": samples,
                    "warmup_replays": warmup, "samples": sample_count, "tails_reliable": sample_count >= 100,
                    "substeps": last.substeps, "integrated": last.integrated_objects,
                    "sleeping": last.sleeping_objects, "pair_visits": last.pair_visits,
                    "narrow_phase_tests": last.narrow_phase_tests, "contacts": last.contacts,
                    "projection_builds": last.projection_builds}),
                );
                eprintln!("completed box={proxy} count={count} state={state}");
                save(&audit, &rows);
            }
        }
    }
}

fn save(audit: &[serde_json::Value], rows: &[serde_json::Value]) {
    let output = serde_json::to_string_pretty(&json!({"audit": audit, "cpu": rows,
        "tick_ms": 16,
        "gpu_ms": null, "box_raw_pair_axis_bound": 15}))
    .unwrap();
    if let Ok(path) = std::env::var("SALIMON_GEOMETRY_EVIDENCE") {
        std::fs::write(path, output).unwrap();
    } else {
        println!("{output}");
    }
}

//! Identical-snapshot CPU timing, not a native GPU/FPS benchmark.
use salimon_physics::{ConvexHull, ObjectState, SleepTracker, Surface, advance_with_sleep};
use std::{
    sync::Arc,
    time::{Duration, Instant},
};

fn main() {
    let hull = Arc::new(ConvexHull::new((0..8).map(|i| {
        std::array::from_fn(|axis| if i & (1 << axis) == 0 { -0.5 } else { 0.5 })
    })));
    println!("count,state,delta_ms,samples,median_ms,p95_ms,p99_ms,substeps,integrated,sleeping");
    for count in [500, 1000] {
        let ids: Vec<_> = (0..count).collect();
        let mut objects: Vec<_> = (0..count)
            .map(|i| ObjectState {
                position: [(i % 32) as f64 * 0.5, 0.1, (i / 32) as f64 * 0.5],
                velocity: [0.0; 3],
                angular_velocity: [0.0; 3],
                radius: 0.18,
                ground_support_meters: 0.1,
                authored_mesh_scale: 0.2,
                mass_kg: if i % 2 == 0 { 0.5 } else { 2.0 },
                orientation: [0.0, 0.0, 0.0, 1.0],
                surface: Surface::Floor { height_meters: 0.0 },
                hull: Some(hull.clone()),
            })
            .collect();
        let mut tracker = SleepTracker::default();
        for _ in 0..40 {
            advance_with_sleep(
                &mut tracker,
                &ids,
                &mut objects,
                Duration::from_millis(16),
                |_, _| true,
                false,
            );
        }
        assert!(ids.iter().all(|id| tracker.is_sleeping(*id)));
        for state in ["settled", "fast_drop", "active"] {
            let mut initial = objects.clone();
            if state == "fast_drop" {
                initial[count - 1].velocity = [0.0, -40.0, 0.0];
            } else if state == "active" {
                for object in &mut initial {
                    object.position[1] = 1.0;
                    object.velocity = [0.0, -40.0, 0.0];
                }
            }
            for delta_ms in [16] {
                let mut times = Vec::new();
                let mut stats = Default::default();
                for sample in 0..110 {
                    let mut pieces = initial.clone();
                    let mut cache = tracker.clone();
                    let start = Instant::now();
                    stats = advance_with_sleep(
                        &mut cache,
                        &ids,
                        &mut pieces,
                        Duration::from_millis(delta_ms),
                        |_, _| true,
                        true,
                    );
                    if sample >= 10 {
                        times.push(start.elapsed().as_secs_f64() * 1000.0);
                    }
                    assert_eq!(pieces.len(), count);
                    assert!(pieces.iter().all(|p| p.position[1] >= 0.1 - 1e-6));
                }
                times.sort_by(f64::total_cmp);
                println!(
                    "{count},{state},{delta_ms},100,{:.6},{:.6},{:.6},{},{},{}",
                    times[50],
                    times[94],
                    times[98],
                    stats.substeps,
                    stats.integrated_objects,
                    stats.sleeping_objects
                );
            }
        }
    }
}

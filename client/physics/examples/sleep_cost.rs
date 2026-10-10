//! CPU-only solver isolation; complements (never substitutes for) native #155 matrix.
use salimon_physics::{
    ConvexHull, ObjectState, SleepTracker, Surface, advance_profiled, advance_with_sleep,
};
use std::{
    sync::Arc,
    time::{Duration, Instant},
};

fn distribution(mut values: Vec<f64>) -> [f64; 3] {
    values.sort_by(f64::total_cmp);
    [
        values[values.len() / 2],
        values[(values.len() * 95).div_ceil(100) - 1],
        values[(values.len() * 99).div_ceil(100) - 1],
    ]
}
fn main() {
    let hull = Arc::new(ConvexHull::new((0..8).map(|i| {
        std::array::from_fn(|axis| if i & (1 << axis) == 0 { -0.5 } else { 0.5 })
    })));
    let delta = Duration::from_millis(16);
    println!("count,state,path,samples,median_ms,p95_ms,p99_ms,awake,sleeping,integrated,pairs");
    for count in [500, 1000] {
        for settled in [true, false] {
            let initial: Vec<_> = (0..count)
                .map(|i| ObjectState {
                    position: [
                        (i % 32) as f64 * 0.5,
                        if settled { 0.1 } else { 1.0 },
                        (i / 32) as f64 * 0.5,
                    ],
                    velocity: if settled { [0.0; 3] } else { [0.1, -1.0, 0.0] },
                    angular_velocity: [0.0; 3],
                    radius: 0.18,
                    ground_support_meters: 0.1,
                    side_meters: 0.2,
                    mass_kg: if i % 2 == 0 { 0.5 } else { 2.0 },
                    orientation: [0.0, 0.0, 0.0, 1.0],
                    surface: Surface::Floor { height_meters: 0.0 },
                    hull: Some(hull.clone()),
                })
                .collect();
            let ids: Vec<_> = (0..count).collect();
            let mut tracker = SleepTracker::default();
            let mut sleeping = initial.clone();
            if settled {
                for _ in 0..40 {
                    advance_with_sleep(
                        &mut tracker,
                        &ids,
                        &mut sleeping,
                        delta,
                        |_, _| true,
                        false,
                    );
                }
            }
            let mut baseline_times = Vec::new();
            let mut sleep_times = Vec::new();
            let mut last_baseline = Default::default();
            let mut last_sleep = Default::default();
            // Replay identical active snapshots; settled snapshots were naturally
            // simulated through dwell, and both paths receive that same final pose.
            for _ in 0..100 {
                let mut baseline = sleeping.clone();
                let start = Instant::now();
                last_baseline = advance_profiled(&mut baseline, delta, |_, _| true);
                baseline_times.push(start.elapsed().as_secs_f64() * 1000.0);
                let mut objects = sleeping.clone();
                let mut cache = tracker.clone();
                let start = Instant::now();
                last_sleep =
                    advance_with_sleep(&mut cache, &ids, &mut objects, delta, |_, _| true, true);
                sleep_times.push(start.elapsed().as_secs_f64() * 1000.0);
                if !settled {
                    assert_eq!(baseline, objects);
                }
            }
            for (path, values, stats) in [
                ("stateless", baseline_times, last_baseline),
                ("sleep", sleep_times, last_sleep),
            ] {
                let [median, p95, p99] = distribution(values);
                println!(
                    "{count},{},{path},100,{median:.6},{p95:.6},{p99:.6},{},{},{},{}",
                    if settled { "settled" } else { "active" },
                    stats.awake_objects,
                    stats.sleeping_objects,
                    stats.integrated_objects,
                    stats.pair_visits
                );
            }
        }
    }
}

//! Conservative radius AABBs in one f64 simulation frame. Hash iteration never
//! selects solver order: every query sorts and deduplicates snapshot indices.
use crate::ObjectState;
use std::collections::HashMap;

#[derive(Clone, Copy, Debug, Default)]
struct Bounds {
    lo: [f64; 3],
    hi: [f64; 3],
}
impl Bounds {
    fn overlaps(self, other: Self) -> bool {
        (0..3).all(|a| self.lo[a] <= other.hi[a] && other.lo[a] <= self.hi[a])
    }
}
#[derive(Clone, Debug, Default)]
pub(crate) struct Grid {
    cells: HashMap<[i64; 3], Vec<usize>>,
    bounds: Vec<Bounds>,
    large: Vec<usize>,
    cell: f64,
    origin: [f64; 3],
    radii: Vec<f64>,
    obsolete_cells: bool,
}
impl Grid {
    pub(crate) fn reset(&mut self, pieces: &[ObjectState], gap: f64) {
        self.radii.clear();
        self.radii.extend(pieces.iter().map(|p| p.radius));
        self.radii.sort_unstable_by(f64::total_cmp);
        let cell = (2.0 * self.radii.get(pieces.len() / 2).copied().unwrap_or(1.0)).max(1e-6);
        let reuse = self.cell == cell && self.bounds.len() == pieces.len();
        self.cell = cell;
        self.origin = pieces.first().map_or([0.0; 3], |p| p.position);
        if reuse {
            // Refresh every exact AABB in the new local origin, but keep bucket
            // membership when its cell range is unchanged. Queries run only
            // after all rows have been refreshed.
            for (i, p) in pieces.iter().enumerate() {
                self.update(i, p, gap);
            }
            if self.obsolete_cells {
                self.cells.retain(|_, bucket| !bucket.is_empty());
                self.obsolete_cells = false;
            }
            return;
        }
        // Retain bucket vectors within a solve; discard obsolete cells between
        // poses so moving populations cannot accumulate unbounded empty buckets.
        for bucket in self.cells.values_mut() {
            bucket.clear();
        }
        self.large.clear();
        self.bounds.clear();
        for (i, p) in pieces.iter().enumerate() {
            self.bounds.push(self.bound(p, gap));
            self.insert(i);
        }
        self.cells.retain(|_, bucket| !bucket.is_empty());
        self.obsolete_cells = false;
    }
    fn bound(&self, p: &ObjectState, gap: f64) -> Bounds {
        // Relative coordinates retain precision at distant planet/ship anchors.
        // Extra rounding guard is conservative, never a narrow-phase tolerance.
        let center = std::array::from_fn::<_, 3, _>(|a| p.position[a] - self.origin[a]);
        let extent =
            p.radius + gap + 1e-12 * (1.0 + center.iter().map(|v| v.abs()).fold(0.0, f64::max));
        Bounds {
            lo: center.map(|v| v - extent),
            hi: center.map(|v| v + extent),
        }
    }
    fn keys(&self, b: Bounds) -> Option<([i64; 3], [i64; 3])> {
        let lo = b.lo.map(|v| (v / self.cell).floor() as i64);
        let hi = b.hi.map(|v| (v / self.cell).floor() as i64);
        let volume = (0..3).try_fold(1_u64, |n, a| {
            n.checked_mul(hi[a].checked_sub(lo[a])?.checked_add(1)?.try_into().ok()?)
        });
        // Mixed very large bodies/swept queries use an exact linear fallback,
        // avoiding huge cell enumeration or any collision/cargo cap.
        volume.filter(|n| *n <= 512).map(|_| (lo, hi))
    }
    fn each_key(lo: [i64; 3], hi: [i64; 3], mut f: impl FnMut([i64; 3])) {
        for x in lo[0]..=hi[0] {
            for y in lo[1]..=hi[1] {
                for z in lo[2]..=hi[2] {
                    f([x, y, z]);
                }
            }
        }
    }
    fn insert(&mut self, i: usize) {
        if let Some((lo, hi)) = self.keys(self.bounds[i]) {
            Self::each_key(lo, hi, |key| self.cells.entry(key).or_default().push(i));
        } else {
            self.large.push(i);
        }
    }
    pub(crate) fn update(&mut self, i: usize, p: &ObjectState, gap: f64) {
        let next = self.bound(p, gap);
        if self.keys(next) == self.keys(self.bounds[i]) {
            self.bounds[i] = next;
            return;
        }
        self.obsolete_cells = true;
        if let Some((lo, hi)) = self.keys(self.bounds[i]) {
            Self::each_key(lo, hi, |key| {
                if let Some(bucket) = self.cells.get_mut(&key) {
                    bucket.retain(|&j| j != i);
                }
            });
        } else {
            self.large.retain(|&j| j != i);
        }
        self.bounds[i] = next;
        self.insert(i);
    }
    pub(crate) fn query(&self, i: usize, reach: f64, out: &mut Vec<usize>) {
        out.clear();
        #[cfg(test)]
        if BRUTE_FORCE.with(|flag| flag.get()) {
            out.extend((0..self.bounds.len()).filter(|&j| j != i));
            return;
        }
        let mut b = self.bounds[i];
        b.lo = b.lo.map(|v| v - reach);
        b.hi = b.hi.map(|v| v + reach);
        if let Some((lo, hi)) = self.keys(b) {
            Self::each_key(lo, hi, |key| {
                if let Some(bucket) = self.cells.get(&key) {
                    out.extend(bucket.iter().copied());
                }
            });
            out.extend(self.large.iter().copied());
        } else {
            out.extend(0..self.bounds.len());
        }
        out.retain(|&j| j != i && b.overlaps(self.bounds[j]));
        out.sort_unstable();
        out.dedup();
    }
    pub(crate) fn overlaps(&self, i: usize, j: usize) -> bool {
        self.bounds[i].overlaps(self.bounds[j])
    }
}

#[cfg(test)]
thread_local! { pub(crate) static BRUTE_FORCE: std::cell::Cell<bool> = const { std::cell::Cell::new(false) }; }

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{ConvexHull, Surface, advance_profiled};
    use std::{sync::Arc, time::Duration};

    fn cube(position: [f64; 3], side: f64) -> ObjectState {
        ObjectState {
            position,
            velocity: [0.0; 3],
            radius: side * 3_f64.sqrt() / 2.0,
            ground_support_meters: side / 2.0,
            surface: Surface::Floor { height_meters: 0.0 },
            hull: Some(Arc::new(ConvexHull::new((0..8).map(|i| {
                std::array::from_fn(|a| if i & (1 << a) == 0 { -0.5 } else { 0.5 })
            })))),
            side_meters: side,
            orientation: [0.0, 0.0, 0.0, 1.0],
            angular_velocity: [0.0; 3],
            mass_kg: 2.0,
        }
    }
    fn reference(pieces: &mut [ObjectState], delta: Duration) -> crate::StepStats {
        BRUTE_FORCE.with(|flag| flag.set(true));
        let result = advance_profiled(pieces, delta, |_, _| true);
        BRUTE_FORCE.with(|flag| flag.set(false));
        result
    }
    #[test]
    fn spatial_solver_matches_all_pairs_on_dense_scattered_rotated_mixed_fast_cases() {
        for layout in 0..5 {
            let mut seed = 157_u64;
            let mut random = || {
                seed = seed.wrapping_mul(6364136223846793005).wrapping_add(1);
                (seed >> 32) as f64 / u32::MAX as f64
            };
            let mut pieces: Vec<_> = (0..32)
                .map(|i| {
                    let side = 0.06 + random() * 0.25;
                    let extent = if layout == 0 { 0.3 } else { 3.0 };
                    let mut p = cube(
                        [
                            (random() - 0.5) * extent,
                            0.15 + random() * 0.7,
                            (random() - 0.5) * extent,
                        ],
                        side,
                    );
                    let angle = random() * 3.0;
                    p.orientation = [0.0, (angle / 2.0).sin(), 0.0, (angle / 2.0).cos()];
                    p.velocity = [(random() - 0.5) * 2.0, -random() * 2.0, 0.0];
                    p.angular_velocity = [0.2, random(), 0.1];
                    if layout == 2 && i == 0 {
                        p.velocity = [80.0, -20.0, 0.0];
                    }
                    if layout == 4 && i % 3 == 0 {
                        p.hull = None;
                    }
                    if layout == 3 {
                        p.hull = None;
                        p.surface = if i % 2 == 0 {
                            Surface::Unsupported
                        } else {
                            p.surface
                        };
                    }
                    p.mass_kg = 0.5 + random() * 10.0;
                    p
                })
                .collect();
            let mut brute = pieces.clone();
            for step in 0..8 {
                advance_profiled(&mut pieces, Duration::from_millis(16), |_, _| true);
                reference(&mut brute, Duration::from_millis(16));
                assert_eq!(pieces, brute, "layout {layout}, step {step}");
            }
        }
    }
    #[test]
    fn correction_created_later_pair_is_visited_in_the_same_pass() {
        // 0 overlaps 1 deeply. Their first correction pushes 0 toward 2,
        // initially outside 0's conservative AABB, before the (0,2) visit.
        let mut pieces = vec![
            cube([0.0, 1.0, 0.0], 0.2),
            cube([0.01, 1.0, 0.0], 0.2),
            cube([-0.39, 1.0, 0.0], 0.2),
        ];
        for p in &mut pieces {
            p.hull = None;
            p.surface = Surface::Unsupported;
        }
        let mut grid = Grid::default();
        grid.reset(&pieces, 0.0);
        let mut candidates = Vec::new();
        grid.query(0, 0.0, &mut candidates);
        assert!(!candidates.contains(&2));
        let mut brute = pieces.clone();
        let stats = advance_profiled(&mut pieces, Duration::from_millis(1), |_, _| true);
        let expected = reference(&mut brute, Duration::from_millis(1));
        assert_eq!(pieces, brute);
        assert_eq!(stats.contacts, expected.contacts);
        assert!(pieces[2].position[0] < -0.39);
    }
    #[test]
    fn grid_covers_reference_radius_pairs_and_swept_queries_at_distant_anchors() {
        let mut pieces: Vec<_> = (0..100)
            .map(|i| {
                cube(
                    [1e12 + (i % 10) as f64 * 0.3, (i / 10) as f64 * 0.3, 0.0],
                    0.1 + (i % 7) as f64 * 0.03,
                )
            })
            .collect();
        pieces[0].radius = 10.0; // Oversize fallback, no huge cell enumeration.
        let mut grid = Grid::default();
        grid.reset(&pieces, 0.001);
        let mut out = Vec::new();
        for reach in [0.0, 0.1, 80.0] {
            for i in 0..pieces.len() {
                grid.query(i, reach, &mut out);
                for j in 0..pieces.len() {
                    if i != j
                        && salimon_math::length(salimon_math::sub(
                            pieces[i].position,
                            pieces[j].position,
                        )) <= pieces[i].radius + pieces[j].radius + reach + 0.001
                    {
                        assert!(out.contains(&j), "{i} {j} {reach}");
                    }
                }
                assert!(out.windows(2).all(|p| p[0] < p[1]));
            }
        }
    }
    #[test]
    fn refreshed_membership_matches_fresh_grid_after_origin_shape_and_population_changes() {
        let mut pieces: Vec<_> = (0..40)
            .map(|i| cube([1e12 + i as f64 * 0.1, 1.0, 0.0], 0.1))
            .collect();
        let mut retained = Grid::default();
        let mut actual = Vec::new();
        let mut expected = Vec::new();
        for step in 0..20 {
            for (i, p) in pieces.iter_mut().enumerate() {
                p.position[0] += if step < 5 { 0.3 } else { (i % 3) as f64 * 0.07 };
                if step == 7 {
                    p.radius *= 2.0;
                }
            }
            if step == 9 {
                pieces[0].radius = 20.0; // Normal-to-oversize membership.
            } else if step == 10 {
                pieces[0].radius = 0.01; // Oversize-to-normal membership.
                pieces.reverse();
            } else if step == 12 {
                pieces.truncate(21);
            } else if step == 14 {
                pieces.push(cube([1e12, 1.0, 0.0], 0.2));
            }
            let gap = if step % 2 == 0 { 0.001 } else { 0.0 };
            retained.reset(&pieces, gap);
            let mut fresh = Grid::default();
            fresh.reset(&pieces, gap);
            assert!(retained.cells.values().all(|bucket| !bucket.is_empty()));
            for reach in [0.0, 0.1, 100.0] {
                for i in 0..pieces.len() {
                    retained.query(i, reach, &mut actual);
                    fresh.query(i, reach, &mut expected);
                    assert_eq!(actual, expected, "step {step}, row {i}, reach {reach}");
                }
            }
        }
    }
    #[test]
    fn separated_populations_have_no_pair_storage_or_pair_visits() {
        for count in [500, 1000] {
            let mut pieces: Vec<_> = (0..count)
                .map(|i| cube([i as f64 * 2.0, 0.1, 0.0], 0.2))
                .collect();
            let stats = advance_profiled(&mut pieces, Duration::from_millis(16), |_, _| true);
            assert_eq!(stats.pair_visits, 0);
            assert_eq!(stats.projection_storage_bytes, 0);
            assert_eq!(stats.matrix_allocations, 0);
        }
    }
    #[test]
    fn empty_earlier_row_is_requeried_after_a_later_pair_correction() {
        let mut pieces = vec![
            cube([0.0, 1.0, 0.0], 0.2),
            cube([0.39, 1.0, 0.0], 0.2),
            cube([0.40, 1.0, 0.0], 0.2),
        ];
        for p in &mut pieces {
            p.hull = None;
        }
        let mut grid = Grid::default();
        grid.reset(&pieces, 2.0 * crate::CONTACT_BIAS_METERS);
        let mut candidates = Vec::new();
        grid.query(0, 0.0, &mut candidates);
        assert!(candidates.is_empty());
        let original = pieces[0].position;
        let mut brute = pieces.clone();
        advance_profiled(&mut pieces, Duration::from_millis(1), |_, _| true);
        reference(&mut brute, Duration::from_millis(1));
        assert_eq!(pieces, brute);
        assert!(pieces[0].position[0] < original[0]);
    }

    #[test]
    fn empty_rows_are_refreshed_after_integration_and_between_updates() {
        let mut pieces = vec![cube([0.0, 1.0, 0.0], 0.2), cube([0.36, 1.0, 0.0], 0.2)];
        for (i, p) in pieces.iter_mut().enumerate() {
            p.hull = None;
            p.velocity[0] = if i == 0 { 1.0 } else { -1.0 };
        }
        let mut brute = pieces.clone();
        let mut contacts = 0;
        for _ in 0..10 {
            contacts +=
                advance_profiled(&mut pieces, Duration::from_millis(16), |_, _| true).contacts;
            reference(&mut brute, Duration::from_millis(16));
            assert_eq!(pieces, brute);
        }
        assert!(contacts > 0);
    }

    #[test]
    #[ignore = "CPU-only sparse-storage versus spatial-filtering attribution"]
    fn candidate_filtering_cost() {
        use std::time::Instant;
        println!("count,state,path,samples,median_ms,p95_ms,pairs,storage_bytes,growth");
        for count in [500, 1000] {
            for active in [false, true] {
                let initial: Vec<_> = (0..count)
                    .map(|i| {
                        let mut p = cube(
                            [
                                (i % 32) as f64 * 0.5,
                                if active { 1.0 } else { 0.1 },
                                (i / 32) as f64 * 0.5,
                            ],
                            0.2,
                        );
                        if active {
                            p.velocity = [0.1, -1.0, 0.0];
                        }
                        p
                    })
                    .collect();
                for brute in [true, false] {
                    let mut times = Vec::new();
                    let mut stats = crate::StepStats::default();
                    for _ in 0..20 {
                        let mut pieces = initial.clone();
                        let start = Instant::now();
                        stats = if brute {
                            reference(&mut pieces, Duration::from_millis(16))
                        } else {
                            advance_profiled(&mut pieces, Duration::from_millis(16), |_, _| true)
                        };
                        times.push(start.elapsed().as_secs_f64() * 1000.0);
                    }
                    times.sort_by(f64::total_cmp);
                    println!(
                        "{count},{},{},20,{:.6},{:.6},{},{},{}",
                        if active { "active" } else { "settled" },
                        if brute { "sparse_all_pairs" } else { "spatial" },
                        times[10],
                        times[18],
                        stats.pair_visits,
                        stats.projection_storage_bytes,
                        stats.matrix_allocations
                    );
                }
            }
        }
    }

    #[test]
    #[ignore = "CPU-only attribution of repeated empty contact queries (#255)"]
    fn separated_sphere_contact_cost() {
        use std::{hint::black_box, time::Instant};
        println!(
            "count,samples,total_p95_ms,contact_p95_ms,query_replay_p95_ms,wake_replay_p95_ms,reset_replay_p95_ms,substeps,passes"
        );
        for count in [500, 1000] {
            let initial: Vec<_> = (0..count)
                .map(|i| {
                    let mut p = cube([(i % 25) as f64 * 0.12, 1.0, (i / 25) as f64 * 0.12], 0.02);
                    p.hull = None;
                    p.radius = 0.01;
                    p.velocity = [0.0, 0.4, 0.0];
                    p.angular_velocity = [0.0, 0.3, 0.0];
                    p
                })
                .collect();
            let mut total = Vec::new();
            let mut contact = Vec::new();
            let mut queries = Vec::new();
            let mut wakes = Vec::new();
            let mut resets = Vec::new();
            let mut stats = crate::StepStats::default();
            let mut grid = Grid::default();
            let mut candidates = Vec::new();
            for sample in 0..110 {
                let mut pieces = initial.clone();
                let start = Instant::now();
                stats = advance_profiled(&mut pieces, Duration::from_millis(16), |_, _| true);
                let elapsed = start.elapsed();
                assert_eq!(stats.pair_visits, 0);
                // Replay the pre-#255 repeated-query path on the same separated
                // bounds. This isolates query cost, excluding response/integration.
                let start = Instant::now();
                for _ in 0..stats.solver_passes / 16 {
                    grid.reset(&initial, 2.0 * crate::CONTACT_BIAS_METERS);
                    for _ in 0..16 {
                        for i in 0..count {
                            grid.query(i, 0.0, &mut candidates);
                            black_box(&candidates);
                            assert!(candidates.is_empty());
                        }
                    }
                }
                let query_elapsed = start.elapsed();
                let ids: Vec<_> = (0..count).collect();
                let mut tracker = crate::SleepTracker::default();
                let mut activation = tracker.prepare(&ids, &initial);
                let start = Instant::now();
                for _ in 0..stats.substeps {
                    activation.impacts(&initial, 0.002);
                    activation.impacts(&initial, 0.0);
                    activation.impacts(&initial, 0.0);
                }
                let wake_elapsed = start.elapsed();
                let start = Instant::now();
                for _ in 0..stats.solver_passes / 16 {
                    grid.reset(black_box(&initial), 2.0 * crate::CONTACT_BIAS_METERS);
                }
                if sample >= 10 {
                    total.push(elapsed.as_secs_f64() * 1000.0);
                    contact.push(stats.contact_time.as_secs_f64() * 1000.0);
                    queries.push(query_elapsed.as_secs_f64() * 1000.0);
                    wakes.push(wake_elapsed.as_secs_f64() * 1000.0);
                    resets.push(start.elapsed().as_secs_f64() * 1000.0);
                }
            }
            total.sort_by(f64::total_cmp);
            contact.sort_by(f64::total_cmp);
            queries.sort_by(f64::total_cmp);
            wakes.sort_by(f64::total_cmp);
            resets.sort_by(f64::total_cmp);
            println!(
                "{count},100,{:.6},{:.6},{:.6},{:.6},{:.6},{},{}",
                total[94],
                contact[94],
                queries[94],
                wakes[94],
                resets[94],
                stats.substeps,
                stats.solver_passes
            );
        }
    }
}

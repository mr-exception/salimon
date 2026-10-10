//! Persistent contact-island activation, independent of feature identities and frames.
use std::{collections::HashMap, hash::Hash, time::Duration};

use salimon_math::{dot, length, scale, sub};

use crate::{ObjectState, Surface, convex, normalize};

const CONTACT_GAP_METERS: f64 = 0.001;
const DWELL_SECONDS: f64 = 0.5;
// Authored faceted ground response has bounded residual chatter above damping's
// zero cutoff. A dwell-wide pose envelope prevents these velocity tolerances
// from sleeping a slowly sliding/tipping object.
const QUIET_SPEED: f64 = 0.05;
const QUIET_ANGULAR_SPEED: f64 = 0.2;

#[derive(Clone, Debug)]
struct Entry<K> {
    state: ObjectState,
    neighbors: Vec<K>,
    quiet_seconds: f64,
    quiet_position: [f64; 3],
    quiet_orientation: [f64; 4],
    sleeping: bool,
}

/// Caller-owned cache keyed by stable identity. Omitted identities are invalidated,
/// so streaming return starts awake and can never retain an absent support.
/// A changed caller-defined frame/environment must change the snapshots or call
/// `wake_all`; rigid motion of a local frame need not invalidate local snapshots.
#[derive(Clone, Debug)]
pub struct SleepTracker<K> {
    entries: HashMap<K, Entry<K>>,
}
impl<K> Default for SleepTracker<K> {
    fn default() -> Self {
        Self {
            entries: HashMap::new(),
        }
    }
}

pub(crate) struct Activation {
    pub(crate) sleeping: Vec<bool>,
    neighbors: Vec<Vec<usize>>,
    woken: Vec<bool>,
}
impl Activation {
    pub(crate) fn wake(&mut self, index: usize) {
        let mut pending = vec![index];
        while let Some(i) = pending.pop() {
            if self.sleeping[i] {
                self.sleeping[i] = false;
                self.woken[i] = true;
                pending.extend(self.neighbors[i].iter().copied());
            }
        }
    }
    // Conservative swept-sphere wake, before integration, includes fast contacts.
    // Narrow phase still determines actual collision response.
    pub(crate) fn impacts(&mut self, pieces: &[ObjectState], dt: f64) {
        let active: Vec<_> = self
            .sleeping
            .iter()
            .enumerate()
            .filter_map(|(i, asleep)| (!asleep).then_some(i))
            .collect();
        for i in active {
            for j in 0..pieces.len() {
                if self.sleeping[j]
                    && same_environment(&pieces[i], &pieces[j])
                    && length(sub(pieces[i].position, pieces[j].position))
                        <= pieces[i].radius
                            + pieces[j].radius
                            + length(pieces[i].velocity) * dt
                            + 0.5 * crate::GRAVITY * dt * dt
                {
                    self.wake(j);
                }
            }
        }
    }
}

impl<K: Copy + Eq + Hash> SleepTracker<K> {
    /// Explicitly invalidate support/forces not represented by ObjectState fields.
    pub fn wake_all(&mut self) {
        for entry in self.entries.values_mut() {
            entry.sleeping = false;
            entry.quiet_seconds = 0.0;
        }
    }

    pub(crate) fn prepare(&mut self, ids: &[K], pieces: &[ObjectState]) -> Activation {
        assert_eq!(ids.len(), pieces.len());
        let indices: HashMap<_, _> = ids.iter().enumerate().map(|(i, id)| (*id, i)).collect();
        assert_eq!(indices.len(), ids.len(), "unique simulation identities");
        let mut activation = Activation {
            woken: vec![false; ids.len()],
            sleeping: ids
                .iter()
                .map(|id| self.entries.get(id).is_some_and(|e| e.sleeping))
                .collect(),
            neighbors: ids
                .iter()
                .map(|id| {
                    self.entries.get(id).map_or_else(Vec::new, |e| {
                        e.neighbors
                            .iter()
                            .filter_map(|id| indices.get(id).copied())
                            .collect()
                    })
                })
                .collect(),
        };
        for (i, id) in ids.iter().enumerate() {
            if let Some(entry) = self.entries.get(id)
                && (!unchanged(&entry.state, &pieces[i])
                    || entry.neighbors.iter().any(|id| !indices.contains_key(id)))
            {
                // Wake old contacts even if this changed object was already awake.
                activation.wake(i);
                for &neighbor in &activation.neighbors[i].clone() {
                    activation.wake(neighbor);
                }
                if let Some(entry) = self.entries.get_mut(id) {
                    entry.quiet_seconds = 0.0;
                    entry.quiet_position = pieces[i].position;
                    entry.quiet_orientation = pieces[i].orientation;
                }
            }
        }
        self.entries.retain(|id, _| indices.contains_key(id));
        activation
    }

    pub(crate) fn finish(
        &mut self,
        ids: &[K],
        pieces: &mut [ObjectState],
        delta: Duration,
        activation: &mut Activation,
        floor_contains: impl Fn([f64; 3], f64) -> bool,
    ) {
        if activation.sleeping.iter().all(|s| *s) {
            return;
        }
        let before_neighbors = std::mem::take(&mut activation.neighbors);
        let mut neighbors = vec![Vec::new(); pieces.len()];
        for i in 0..pieces.len() {
            for j in i + 1..pieces.len() {
                let contact = if activation.sleeping[i] && activation.sleeping[j] {
                    before_neighbors[i].contains(&j)
                } else {
                    near_contact(&pieces[i], &pieces[j])
                };
                if contact {
                    neighbors[i].push(j);
                    neighbors[j].push(i);
                }
            }
        }
        let quiet: Vec<_> = ids
            .iter()
            .zip(pieces.iter())
            .enumerate()
            .map(|(i, (id, p))| {
                let stable = self.entries.get(id).is_some_and(|e| {
                    length(sub(p.position, e.quiet_position)) < 0.001
                        && p.orientation
                            .iter()
                            .zip(e.quiet_orientation)
                            .all(|(a, b)| (a - b).abs() < 0.005)
                });
                if stable
                    && !activation.woken[i]
                    && length(p.velocity) < QUIET_SPEED
                    && length(p.angular_velocity) < QUIET_ANGULAR_SPEED
                {
                    self.entries.get(id).map_or(0.0, |e| e.quiet_seconds) + delta.as_secs_f64()
                } else {
                    0.0
                }
            })
            .collect();
        let mut visited = vec![false; pieces.len()];
        for root in 0..pieces.len() {
            if visited[root] {
                continue;
            }
            let mut island = Vec::new();
            let mut pending = vec![root];
            visited[root] = true;
            while let Some(i) = pending.pop() {
                island.push(i);
                for &j in &neighbors[i] {
                    if !visited[j] {
                        visited[j] = true;
                        pending.push(j);
                    }
                }
            }
            // Airborne zero-speed islands cannot sleep. All members must dwell;
            // an unstable tilted contact or moving neighbor keeps the island awake.
            let supported = island
                .iter()
                .any(|&i| grounded(&pieces[i], &floor_contains));
            let sleep = supported && island.iter().all(|&i| quiet[i] >= DWELL_SECONDS);
            for i in island {
                activation.sleeping[i] = sleep;
                if sleep {
                    pieces[i].velocity = [0.0; 3];
                    pieces[i].angular_velocity = [0.0; 3];
                }
            }
        }
        for (i, id) in ids.iter().enumerate() {
            let (quiet_position, quiet_orientation) = if quiet[i] > 0.0 {
                let old = &self.entries[id];
                (old.quiet_position, old.quiet_orientation)
            } else {
                (pieces[i].position, pieces[i].orientation)
            };
            self.entries.insert(
                *id,
                Entry {
                    state: pieces[i].clone(),
                    neighbors: neighbors[i].iter().map(|&j| ids[j]).collect(),
                    quiet_seconds: quiet[i],
                    quiet_position,
                    quiet_orientation,
                    sleeping: activation.sleeping[i],
                },
            );
        }
        activation.neighbors = neighbors;
    }
}

fn unchanged(a: &ObjectState, b: &ObjectState) -> bool {
    a.position == b.position
        // Snapshot adapters may renormalize an already unit quaternion by an ULP.
        && a.orientation.iter().zip(b.orientation).all(|(a, b)| (a-b).abs() <= 1e-12)
        && a.velocity == b.velocity
        && a.angular_velocity == b.angular_velocity
        && a.mass_kg == b.mass_kg
        && a.side_meters == b.side_meters
        && a.radius == b.radius
        && a.surface == b.surface
        && match (&a.hull, &b.hull) {
            (Some(a), Some(b)) => std::sync::Arc::ptr_eq(a, b),
            (None, None) => a.ground_support_meters == b.ground_support_meters,
            _ => false,
        }
}
fn same_environment(a: &ObjectState, b: &ObjectState) -> bool {
    a.surface == b.surface
}
fn near_contact(a: &ObjectState, b: &ObjectState) -> bool {
    if !same_environment(a, b) {
        return false;
    }
    let offset = sub(b.position, a.position);
    if length(offset) > a.radius + b.radius + CONTACT_GAP_METERS {
        return false;
    }
    if a.hull.is_some() && b.hull.is_some() {
        convex::near_contact(a, b, CONTACT_GAP_METERS)
    } else {
        true
    }
}
fn grounded(p: &ObjectState, contains: &impl Fn([f64; 3], f64) -> bool) -> bool {
    let (up, gap) = match p.surface {
        Surface::Floor { height_meters } if contains(p.position, p.radius) => {
            ([0.0, 1.0, 0.0], p.position[1] - height_meters)
        }
        Surface::Sphere(body) => (
            normalize(sub(p.position, body.center)),
            length(sub(p.position, body.center)) - body.radius,
        ),
        _ => return false,
    };
    let support = p.hull.as_ref().map_or(p.ground_support_meters, |h| {
        h.support(p.orientation, p.side_meters, scale(up, -1.0))
    });
    (gap - support).abs() <= CONTACT_GAP_METERS && dot(p.velocity, up).abs() < QUIET_SPEED
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{ConvexHull, advance_profiled, advance_with_sleep};
    use std::sync::Arc;

    fn cube(x: f64, y: f64) -> ObjectState {
        ObjectState {
            position: [x, y, 0.0],
            velocity: [0.0; 3],
            angular_velocity: [0.0; 3],
            radius: 0.18,
            ground_support_meters: 0.1,
            side_meters: 0.2,
            mass_kg: 1.0,
            orientation: [0.0, 0.0, 0.0, 1.0],
            surface: Surface::Floor { height_meters: 0.0 },
            hull: Some(Arc::new(ConvexHull::new((0..8).map(|i| {
                std::array::from_fn(|axis| if i & (1 << axis) == 0 { -0.5 } else { 0.5 })
            })))),
        }
    }
    fn settle(ids: &[u64], objects: &mut [ObjectState], tracker: &mut SleepTracker<u64>) {
        for _ in 0..150 {
            advance_with_sleep(
                tracker,
                ids,
                objects,
                Duration::from_millis(16),
                |_, _| true,
                false,
            );
        }
        assert!(ids.iter().all(|id| tracker.entries[id].sleeping));
    }
    #[test]
    fn support_removal_wakes_only_the_connected_pile_and_reordering_keeps_identity() {
        let mut objects = vec![cube(0.0, 0.1), cube(0.0, 0.30001), cube(2.0, 0.1)];
        let mut tracker = SleepTracker::default();
        settle(&[10, 20, 30], &mut objects, &mut tracker);
        let isolated = objects[2].clone();
        let mut remaining = vec![objects[2].clone(), objects[1].clone()];
        let stats = advance_with_sleep(
            &mut tracker,
            &[30, 20],
            &mut remaining,
            Duration::from_millis(16),
            |_, _| true,
            true,
        );
        assert_eq!(stats.sleeping_objects, 1);
        assert_eq!(remaining[0], isolated);
        assert!(remaining[1].position[1] < objects[1].position[1] - 0.001);
        assert!(!tracker.entries.contains_key(&10));
    }
    #[test]
    fn airborne_zero_velocity_and_disconnected_unsupported_contacts_never_sleep() {
        let mut tracker = SleepTracker::default();
        let mut objects = vec![cube(0.0, 4.0), cube(2.0, 0.1)];
        objects[0].surface = Surface::Unsupported;
        for _ in 0..60 {
            advance_with_sleep(
                &mut tracker,
                &[1, 2],
                &mut objects,
                Duration::from_millis(16),
                |_, _| true,
                false,
            );
        }
        assert!(!tracker.entries[&1].sleeping);
        assert!(objects[0].position[1] < 3.0);
        assert!(tracker.entries[&2].sleeping);
    }
    #[test]
    fn fast_impact_wakes_whole_island_without_waking_unrelated_support() {
        let mut tracker = SleepTracker::default();
        let mut objects = vec![cube(0.0, 0.1), cube(0.0, 0.30001), cube(2.0, 0.1)];
        settle(&[1, 2, 3], &mut objects, &mut tracker);
        let isolated = objects[2].clone();
        let mut impact = cube(-0.7, 0.3);
        impact.velocity = [40.0, 0.0, 0.0];
        objects.push(impact);
        let stats = advance_with_sleep(
            &mut tracker,
            &[1, 2, 3, 4],
            &mut objects,
            Duration::from_millis(16),
            |_, _| true,
            true,
        );
        assert!(!tracker.entries[&1].sleeping && !tracker.entries[&2].sleeping);
        assert_eq!(stats.sleeping_objects, 1);
        assert_eq!(objects[2], isolated);
    }
    #[test]
    fn geometry_force_frame_and_streaming_changes_invalidate_sleep() {
        for change in 0..6 {
            let mut tracker = SleepTracker::default();
            let mut objects = vec![cube(0.0, 0.1)];
            settle(&[1], &mut objects, &mut tracker);
            match change {
                0 => objects[0].mass_kg = 2.0,
                1 => objects[0].side_meters *= 1.5,
                2 => objects[0].surface = Surface::Unsupported,
                3 => objects[0].velocity = [0.0, 3.0, 0.0],
                4 => tracker.wake_all(),
                _ => {
                    tracker.prepare(&[], &[]);
                }
            }
            let stats = advance_with_sleep(
                &mut tracker,
                &[1],
                &mut objects,
                Duration::from_millis(16),
                |_, _| true,
                true,
            );
            assert!(stats.integrated_objects > 0, "change {change}");
            assert_eq!(stats.sleeping_objects, 0, "change {change}");
        }
    }
    #[test]
    fn five_hundred_settled_objects_skip_integration_pairs_and_allocations() {
        for count in [500, 1000] {
            let ids: Vec<_> = (0..count as u64).collect();
            let mut objects: Vec<_> = (0..count).map(|i| cube(i as f64 * 0.5, 0.1)).collect();
            let mut tracker = SleepTracker::default();
            // Real solver + dwell, no pre-installed sleeping flags.
            for _ in 0..8 {
                advance_with_sleep(
                    &mut tracker,
                    &ids,
                    &mut objects,
                    Duration::from_millis(100),
                    |_, _| true,
                    false,
                );
            }
            let before = objects.clone();
            let stats = advance_with_sleep(
                &mut tracker,
                &ids,
                &mut objects,
                Duration::from_millis(16),
                |_, _| true,
                true,
            );
            assert_eq!(stats.sleeping_objects, count);
            assert_eq!(stats.integrated_objects, 0);
            assert_eq!(stats.pair_visits, 0);
            assert_eq!(stats.matrix_allocations, 0);
            assert_eq!(objects, before);
            let mut baseline = objects.clone();
            let baseline = advance_profiled(&mut baseline, Duration::from_millis(16), |_, _| true);
            assert!(baseline.pair_visits >= (count * (count - 1) / 2) as u64);
        }
    }
    #[test]
    fn overlapping_bounds_without_convex_contact_do_not_connect_sleep_islands() {
        let mut tracker = SleepTracker::default();
        let mut objects = vec![cube(0.0, 0.1), cube(0.34, 0.1)];
        settle(&[1, 2], &mut objects, &mut tracker);
        assert!(tracker.entries[&1].neighbors.is_empty());
        let isolated = objects[1].clone();
        let mut remaining = vec![isolated.clone()];
        let stats = advance_with_sleep(
            &mut tracker,
            &[2],
            &mut remaining,
            Duration::from_millis(16),
            |_, _| true,
            true,
        );
        assert_eq!(stats.sleeping_objects, 1);
        assert_eq!(remaining[0], isolated);
    }

    #[test]
    fn adapter_quaternion_rounding_preserves_sleep_but_real_rotation_wakes() {
        let mut tracker = SleepTracker::default();
        let mut objects = vec![cube(0.0, 0.1)];
        settle(&[1], &mut objects, &mut tracker);
        objects[0].orientation[0] = 1e-14;
        let stats = advance_with_sleep(
            &mut tracker,
            &[1],
            &mut objects,
            Duration::from_millis(16),
            |_, _| true,
            true,
        );
        assert_eq!(stats.sleeping_objects, 1);
        objects[0].orientation = [0.1_f64.sin(), 0.0, 0.0, 0.1_f64.cos()];
        let stats = advance_with_sleep(
            &mut tracker,
            &[1],
            &mut objects,
            Duration::from_millis(16),
            |_, _| true,
            true,
        );
        assert!(stats.integrated_objects > 0);
        assert_eq!(stats.sleeping_objects, 0);
    }

    #[test]
    fn disappearing_floor_containment_wakes_sleeping_objects() {
        let mut tracker = SleepTracker::default();
        let mut objects = vec![cube(0.0, 0.1)];
        settle(&[1], &mut objects, &mut tracker);
        let stats = advance_with_sleep(
            &mut tracker,
            &[1],
            &mut objects,
            Duration::from_millis(16),
            |_, _| false,
            true,
        );
        assert!(stats.integrated_objects > 0);
        assert_eq!(stats.sleeping_objects, 0);
    }

    #[test]
    fn profiling_and_zero_delta_preserve_activation_behavior() {
        let mut a = vec![cube(0.0, 0.1), cube(0.0, 0.3)];
        let mut b = a.clone();
        let mut ta = SleepTracker::default();
        let mut tb = SleepTracker::default();
        for _ in 0..100 {
            advance_with_sleep(
                &mut ta,
                &[1, 2],
                &mut a,
                Duration::from_millis(16),
                |_, _| true,
                false,
            );
            advance_with_sleep(
                &mut tb,
                &[1, 2],
                &mut b,
                Duration::from_millis(16),
                |_, _| true,
                true,
            );
            assert_eq!(a, b);
        }
        let before = a.clone();
        advance_with_sleep(
            &mut ta,
            &[],
            &mut a,
            Duration::ZERO,
            |_, _| panic!("zero delta"),
            true,
        );
        assert_eq!(a, before);
    }
}

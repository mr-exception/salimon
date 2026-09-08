//! Persistent contact-island activation, independent of feature identities and frames.
use std::{
    collections::{HashMap, VecDeque},
    hash::Hash,
    time::Duration,
};

use salimon_math::{add, dot, length, scale, sub};

use crate::Surface;
use crate::{ObjectState, convex};

const CONTACT_GAP_METERS: f64 = 0.001;
const DWELL_SECONDS: f64 = 0.5;
// Radius growth of each observed material-point envelope, in metres/second.
// A 0.5 s window permits a 15 mm radius, rather than axis-dependent angle limits.
const POINT_ENVELOPE_SPEED: f64 = 0.03;
// Authored faceted ground response has bounded residual chatter above damping's
// zero cutoff. A supported observation window distinguishes bounded contact
// chatter from point motion exceeding the rest envelope.
const QUIET_SPEED: f64 = 0.15;
const QUIET_ANGULAR_SPEED: f64 = 1.0;

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
struct MotionSample {
    time: f64,
    position: [f64; 3],
    orientation: [f64; 4],
    supported: bool,
    quiet: bool,
}
#[derive(Clone, Debug, Default, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
struct MotionWindow {
    time: f64,
    samples: VecDeque<MotionSample>,
    points: Vec<[f64; 3]>,
}
impl MotionWindow {
    fn observe(&mut self, p: &ObjectState, delta: f64, supported: bool) {
        if self.points.is_empty() {
            self.points.push([0.; 3]);
            if let Some(h) = &p.hull {
                self.points
                    .extend(h.vertices.iter().map(|v| scale(*v, p.authored_mesh_scale)));
            } else {
                for i in 0..3 {
                    for sign in [-1., 1.] {
                        let mut v = [0.; 3];
                        v[i] = sign * p.radius;
                        self.points.push(v);
                    }
                }
            }
        }
        // Credit only time between actual observations. A first long update
        // cannot manufacture a preceding quiet/support interval.
        self.time += delta;
        self.samples.push_back(MotionSample {
            time: self.time,
            position: p.position,
            orientation: p.orientation,
            supported,
            quiet: length(p.velocity) < QUIET_SPEED
                && length(p.angular_velocity) < QUIET_ANGULAR_SPEED,
        });
        let start = self.time - DWELL_SECONDS;
        while self.samples.len() > 2 && self.samples[1].time <= start {
            self.samples.pop_front();
        }
    }
    fn stable(&self) -> bool {
        let Some(first) = self.samples.front() else {
            return false;
        };
        let start = self.time - DWELL_SECONDS;
        if first.time > start + 1e-12 {
            return false;
        }
        let next = &self.samples[1];
        let weight = ((start - first.time) / (next.time - first.time)).clamp(0., 1.);
        let position = add(
            first.position,
            scale(sub(next.position, first.position), weight),
        );
        let flip = if first
            .orientation
            .iter()
            .zip(next.orientation)
            .map(|(a, b)| a * b)
            .sum::<f64>()
            < 0.
        {
            -1.
        } else {
            1.
        };
        let mut q: [f64; 4] = std::array::from_fn(|i| {
            first.orientation[i] + (flip * next.orientation[i] - first.orientation[i]) * weight
        });
        let norm = q.iter().map(|v| v * v).sum::<f64>().sqrt();
        if norm < 1e-12 {
            return false;
        }
        q = q.map(|v| v / norm);
        if !first.supported || !next.supported || !first.quiet || !next.quiet {
            return false;
        }
        if self
            .samples
            .iter()
            .filter(|p| p.time >= start)
            .any(|p| !p.supported || !p.quiet)
        {
            return false;
        }
        for point in &self.points {
            let reference = convex::rotate(q, *point);
            // Grow an enclosing sphere in coordinates relative to the window
            // boundary, retaining precision at large world positions.
            let mut center = [0.; 3];
            let mut radius = 0.;
            for sample in self.samples.iter().filter(|p| p.time >= start) {
                let offset = add(
                    sub(sample.position, position),
                    sub(convex::rotate(sample.orientation, *point), reference),
                );
                let direction = sub(offset, center);
                let distance = length(direction);
                if distance > radius {
                    let new_radius = (radius + distance) * 0.5;
                    center = add(center, scale(direction, (new_radius - radius) / distance));
                    radius = new_radius;
                }
                if radius >= POINT_ENVELOPE_SPEED * DWELL_SECONDS {
                    return false;
                }
            }
        }
        true
    }
}

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
struct Entry<K> {
    state: ObjectState,
    neighbors: Vec<K>,
    quiet_seconds: f64,
    motion: MotionWindow,
    sleeping: bool,
}

/// Caller-owned cache keyed by stable identity. Omitted identities are invalidated,
/// so streaming return starts awake and can never retain an absent support.
/// A changed caller-defined frame/environment must change the snapshots or call
/// `wake_all`; rigid motion of a local frame need not invalidate local snapshots.
#[derive(Clone, Debug)]
pub struct SleepTracker<K> {
    entries: HashMap<K, Entry<K>>,
    geometry_ids: Vec<K>,
    geometry: Option<convex::PairGeometry>,
    warm_pairs: HashMap<(K, K), crate::ContactResponse>,
    warm_ground: HashMap<K, crate::ContactResponse>,
    warm_dt: f64,
    spatial: crate::broad_phase::Grid,
    candidates: Vec<usize>,
    pub(crate) scratch: crate::SolverScratch,
}
impl<K> Default for SleepTracker<K> {
    fn default() -> Self {
        Self {
            entries: HashMap::new(),
            geometry_ids: Vec::new(),
            geometry: None,
            warm_pairs: HashMap::new(),
            warm_ground: HashMap::new(),
            warm_dt: 0.0,
            spatial: Default::default(),
            candidates: Vec::new(),
            scratch: Default::default(),
        }
    }
}

pub(crate) struct Activation {
    spatial: crate::broad_phase::Grid,
    candidates: Vec<usize>,
    pub(crate) warm_pairs: HashMap<(usize, usize), crate::ContactResponse>,
    pub(crate) warm_ground: HashMap<usize, crate::ContactResponse>,
    pub(crate) warm_dt: f64,
    pub(crate) sleeping: Vec<bool>,
    neighbors: Vec<Vec<usize>>,
    woken: Vec<bool>,
    // Last substep's exact SAT spans remain valid: contact iterations only move
    // centers; damping does not rotate hulls. Reuse them for the support graph.
    pub(crate) geometry: Option<convex::PairGeometry>,
}
impl Activation {
    pub(crate) fn invalidate_support(&mut self, index: usize) {
        self.warm_ground.remove(&index);
        self.warm_pairs
            .retain(|&(a, b), _| a != index && b != index);
    }
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
        // This query can only wake sleeping participants. Once all are awake,
        // there is no activation state to change, even during a fast sweep.
        if !self.sleeping.iter().any(|asleep| *asleep) {
            return;
        }
        let active: Vec<_> = self
            .sleeping
            .iter()
            .enumerate()
            .filter_map(|(i, asleep)| (!asleep).then_some(i))
            .collect();
        self.spatial.reset(pieces, 0.0);
        for i in active {
            let reach = length(pieces[i].velocity) * dt
                + length(pieces[i].angular_velocity) * pieces[i].radius * dt
                + 0.5 * crate::GRAVITY * dt * dt;
            self.spatial.query(i, reach, &mut self.candidates);
            for index in 0..self.candidates.len() {
                let j = self.candidates[index];
                if self.sleeping[j]
                    && same_environment(&pieces[i], &pieces[j])
                    && length(sub(pieces[i].position, pieces[j].position))
                        <= pieces[i].radius
                            + pieces[j].radius
                            + reach
                    // Bounding spheres of faceted objects can overlap at rest
                    // without touching. Expand the SAT gap by swept motion so
                    // fast impacts still wake before integration.
                    && (pieces[i].hull.is_none()
                        || pieces[j].hull.is_none()
                        || convex::near_contact(&pieces[i], &pieces[j], reach))
                {
                    self.wake(j);
                }
            }
        }
    }
}

impl<K: Copy + Eq + Hash> SleepTracker<K> {
    /// Activation from the most recent completed update; absent identities are awake.
    pub fn is_sleeping(&self, id: K) -> bool {
        self.entries.get(&id).is_some_and(|entry| entry.sleeping)
    }

    /// Contact identities from the last completed update; absent entries have none.
    pub fn contact_neighbors(&self, id: K) -> &[K] {
        self.entries
            .get(&id)
            .map_or(&[], |entry| entry.neighbors.as_slice())
    }

    /// Observe actual environmental support in the last completed snapshot.
    pub fn environment_supported(
        &self,
        id: K,
        floor_contains: impl Fn([f64; 3], f64) -> bool,
    ) -> bool {
        self.entries
            .get(&id)
            .is_some_and(|entry| grounded(&entry.state, &floor_contains))
    }

    pub(crate) fn quiet_objects(&self) -> usize {
        self.entries
            .values()
            .filter(|entry| entry.quiet_seconds >= DWELL_SECONDS)
            .count()
    }

    /// Explicitly invalidate support/forces not represented by ObjectState fields.
    pub fn wake_all(&mut self) {
        self.geometry = None;
        self.warm_pairs.clear();
        self.warm_ground.clear();
        self.warm_dt = 0.0;
        for entry in self.entries.values_mut() {
            entry.sleeping = false;
            entry.quiet_seconds = 0.0;
            entry.motion = MotionWindow::default();
        }
    }

    pub(crate) fn prepare(&mut self, ids: &[K], pieces: &[ObjectState]) -> Activation {
        assert_eq!(ids.len(), pieces.len());
        let indices: HashMap<_, _> = ids.iter().enumerate().map(|(i, id)| (*id, i)).collect();
        assert_eq!(indices.len(), ids.len(), "unique simulation identities");
        let valid_warm: Vec<_> = ids
            .iter()
            .enumerate()
            .map(|(i, id)| {
                self.entries
                    .get(id)
                    .is_some_and(|e| unchanged(&e.state, &pieces[i]))
            })
            .collect();
        // Projection spans depend on exact hull orientation/scale. Center
        // translations are handled by ContactGeometry's exact SAT bounds.
        // External changes or reordered/streamed IDs conservatively rebuild.
        let geometry = if self.geometry_ids == ids
            && valid_warm.iter().all(|valid| *valid)
            && ids
                .iter()
                .enumerate()
                .all(|(i, id)| self.entries[id].state.orientation == pieces[i].orientation)
        {
            self.geometry.take()
        } else {
            self.geometry = None;
            None
        };
        self.warm_pairs.retain(|(a, b), _| {
            indices.get(a).is_some_and(|&i| valid_warm[i])
                && indices.get(b).is_some_and(|&i| valid_warm[i])
        });
        self.warm_ground
            .retain(|id, _| indices.get(id).is_some_and(|&i| valid_warm[i]));
        let mut activation = Activation {
            spatial: std::mem::take(&mut self.spatial),
            candidates: std::mem::take(&mut self.candidates),
            warm_pairs: HashMap::new(),
            warm_ground: HashMap::new(),
            warm_dt: self.warm_dt,
            geometry,
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
                    entry.motion = MotionWindow::default();
                }
            }
        }
        self.entries.retain(|id, _| indices.contains_key(id));
        if activation.sleeping.iter().any(|asleep| !asleep) {
            activation.warm_pairs = std::mem::take(&mut self.warm_pairs)
                .into_iter()
                .filter_map(|((a, b), response)| {
                    let (i, j) = (indices[&a], indices[&b]);
                    (i < j).then_some(((i, j), response))
                })
                .collect();
            activation.warm_ground = std::mem::take(&mut self.warm_ground)
                .into_iter()
                .map(|(id, response)| (indices[&id], response))
                .collect();
        }
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
        self.warm_dt = activation.warm_dt;
        self.warm_pairs = std::mem::take(&mut activation.warm_pairs)
            .into_iter()
            .map(|((i, j), response)| ((ids[i], ids[j]), response))
            .collect();
        self.warm_ground = std::mem::take(&mut activation.warm_ground)
            .into_iter()
            .map(|(i, response)| (ids[i], response))
            .collect();
        let before_neighbors = std::mem::take(&mut activation.neighbors);
        let mut neighbors = vec![Vec::new(); pieces.len()];
        let mut deep_contact = vec![false; pieces.len()];
        activation.spatial.reset(pieces, CONTACT_GAP_METERS);
        for i in 0..pieces.len() {
            activation.spatial.query(i, 0.0, &mut activation.candidates);
            for &j in activation.candidates.iter().filter(|&&j| j > i) {
                let contact = if activation.sleeping[i] && activation.sleeping[j] {
                    before_neighbors[i].contains(&j)
                } else if let Some(geometry) = activation
                    .geometry
                    .as_ref()
                    .and_then(|cache| cache.pairs.get(&(i, j)).and_then(Option::as_ref))
                    .filter(|geometry| geometry.used_this_substep)
                {
                    same_environment(&pieces[i], &pieces[j])
                        && length(sub(pieces[j].position, pieces[i].position))
                            <= pieces[i].radius + pieces[j].radius + CONTACT_GAP_METERS
                        && convex::near_projected_contact(
                            &geometry.projections,
                            sub(pieces[j].position, pieces[i].position),
                            CONTACT_GAP_METERS,
                        )
                } else {
                    near_contact(&pieces[i], &pieces[j])
                };
                if contact {
                    neighbors[i].push(j);
                    neighbors[j].push(i);
                    if !(activation.sleeping[i] && activation.sleeping[j]) {
                        let offset = sub(pieces[j].position, pieces[i].position);
                        let depth = if let Some(geometry) = activation
                            .geometry
                            .as_ref()
                            .and_then(|cache| cache.pairs.get(&(i, j)).and_then(Option::as_ref))
                            .filter(|geometry| geometry.used_this_substep)
                        {
                            convex::projected_contact(&geometry.projections, offset)
                        } else if pieces[i].hull.is_some() && pieces[j].hull.is_some() {
                            convex::projected_contact(
                                &convex::projections(&pieces[i], &pieces[j]),
                                offset,
                            )
                        } else {
                            Some((
                                [0.0, 1.0, 0.0],
                                pieces[i].radius + pieces[j].radius - length(offset),
                            ))
                        };
                        if depth.is_some_and(|(_, depth)| depth > CONTACT_GAP_METERS) {
                            deep_contact[i] = true;
                            deep_contact[j] = true;
                        }
                    }
                }
            }
        }
        let mut groups = Vec::new();
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
            let supported = island
                .iter()
                .any(|&i| grounded(&pieces[i], &floor_contains));
            groups.push((island, supported));
        }
        let mut supported = vec![false; pieces.len()];
        for (island, is_supported) in &groups {
            for &i in island {
                supported[i] = *is_supported && !deep_contact[i];
            }
        }
        let mut motions: Vec<_> = ids
            .iter()
            .enumerate()
            .map(|(i, id)| {
                let mut motion = self
                    .entries
                    .get_mut(id)
                    .map_or_else(MotionWindow::default, |entry| {
                        std::mem::take(&mut entry.motion)
                    });
                if activation.woken[i] {
                    motion = MotionWindow::default();
                }
                motion.observe(&pieces[i], delta.as_secs_f64(), supported[i]);
                motion
            })
            .collect();
        let quiet: Vec<_> = pieces
            .iter()
            .enumerate()
            .map(|(i, p)| {
                if !activation.woken[i]
                    && length(p.velocity) < QUIET_SPEED
                    && length(p.angular_velocity) < QUIET_ANGULAR_SPEED
                    && motions[i].stable()
                {
                    DWELL_SECONDS
                } else {
                    0.0
                }
            })
            .collect();
        for (island, supported) in groups {
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
            self.entries.insert(
                *id,
                Entry {
                    state: pieces[i].clone(),
                    neighbors: neighbors[i].iter().map(|&j| ids[j]).collect(),
                    quiet_seconds: quiet[i],
                    motion: std::mem::take(&mut motions[i]),
                    sleeping: activation.sleeping[i],
                },
            );
        }
        activation.neighbors = neighbors;
        self.spatial = std::mem::take(&mut activation.spatial);
        self.candidates = std::mem::take(&mut activation.candidates);
        self.geometry_ids = ids.to_vec();
        self.geometry = if activation.sleeping.iter().all(|s| *s) {
            None
        } else {
            activation.geometry.take()
        };
    }
}

fn unchanged(a: &ObjectState, b: &ObjectState) -> bool {
    a.position == b.position
        // Snapshot adapters may renormalize an already unit quaternion by an ULP.
        && a.orientation.iter().zip(b.orientation).all(|(a, b)| (a-b).abs() <= 1e-12)
        && a.velocity == b.velocity
        && a.angular_velocity == b.angular_velocity
        && a.mass_kg == b.mass_kg
        && a.authored_mesh_scale == b.authored_mesh_scale
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
    if p.surface.is_floor() && !contains(p.position, p.radius) {
        return false;
    }
    let Some(ground) = p.surface.ground(p.position) else {
        return false;
    };
    let up = ground.normal;
    let gap = if let Surface::Sphere(body) = p.surface {
        length(sub(p.position, body.center)) - body.radius
    } else {
        dot(sub(p.position, ground.point), up)
    };
    if let Some(contact) = crate::sampled_contact(p) {
        return contact.penetration.abs() <= CONTACT_GAP_METERS
            && dot(p.velocity, contact.normal).abs() < QUIET_SPEED;
    }
    let support = p.hull.as_ref().map_or(p.ground_support_meters, |h| {
        h.support(p.orientation, p.authored_mesh_scale, scale(up, -1.0))
    });
    (gap - support).abs() <= CONTACT_GAP_METERS && dot(p.velocity, up).abs() < QUIET_SPEED
}

/// Exact activation/dwell/contact impulses; geometry and solver buffers regenerate.
#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SleepSaveState<K> {
    entries: Vec<(K, Entry<K>)>,
    warm_pairs: Vec<((K, K), crate::ContactResponse)>,
    warm_ground: Vec<(K, crate::ContactResponse)>,
    warm_dt: f64,
}
impl<K: Clone + Eq + Hash + serde::Serialize> SleepTracker<K> {
    pub fn save_state(&self) -> SleepSaveState<K> {
        fn ordered<K: serde::Serialize, V>(mut v: Vec<(K, V)>) -> Vec<(K, V)> {
            v.sort_by_cached_key(|(k, _)| {
                serde_json::to_string(k).expect("stable identity serializes")
            });
            v
        }
        SleepSaveState {
            entries: ordered(
                self.entries
                    .iter()
                    .map(|(k, v)| (k.clone(), v.clone()))
                    .collect(),
            ),
            warm_pairs: ordered(
                self.warm_pairs
                    .iter()
                    .map(|(k, v)| (k.clone(), v.clone()))
                    .collect(),
            ),
            warm_ground: ordered(
                self.warm_ground
                    .iter()
                    .map(|(k, v)| (k.clone(), v.clone()))
                    .collect(),
            ),
            warm_dt: self.warm_dt,
        }
    }
    pub fn from_save(
        state: SleepSaveState<K>,
        mut rebuild: impl FnMut(&K, &ObjectState) -> Option<ObjectState>,
    ) -> Result<Self, salimon_persistence::SaveError> {
        use salimon_persistence::invalid;
        let mut tracker = Self::default();
        let ids: std::collections::HashSet<_> =
            state.entries.iter().map(|(id, _)| id.clone()).collect();
        if ids.len() != state.entries.len() || !state.warm_dt.is_finite() || state.warm_dt < 0. {
            return Err(invalid("sleep identities/timestep"));
        }
        for (id, mut e) in state.entries {
            let p = &e.state;
            let norm: f64 = p.orientation.iter().map(|v| v * v).sum();
            if p.position
                .iter()
                .chain(p.velocity.iter())
                .chain(p.angular_velocity.iter())
                .any(|v| !v.is_finite())
                || !norm.is_finite()
                || (norm - 1.).abs() > 1e-9
                || !p.radius.is_finite()
                || p.radius <= 0.
                || !p.mass_kg.is_finite()
                || p.mass_kg <= 0.
                || !e.quiet_seconds.is_finite()
                || e.quiet_seconds < 0.
                || e.neighbors.iter().any(|n| !ids.contains(n))
                || !e.motion.time.is_finite()
                || e.motion.time < 0.
                || e.motion.points.iter().flatten().any(|v| !v.is_finite())
            {
                return Err(invalid("sleep/contact state"));
            }
            if (e.motion.samples.is_empty() && e.motion.time != 0.)
                || (e.motion.samples.len() == 1 && e.motion.samples[0].time != e.motion.time)
            {
                return Err(invalid("incomplete sleep dwell window"));
            }
            let mut previous = None;
            for sample in &e.motion.samples {
                let n: f64 = sample.orientation.iter().map(|v| v * v).sum();
                if !sample.time.is_finite()
                    || sample.time < 0.
                    || sample.time > e.motion.time
                    || previous.is_some_and(|t| sample.time <= t)
                    || sample.position.iter().any(|v| !v.is_finite())
                    || !n.is_finite()
                    || (n - 1.).abs() > 1e-9
                {
                    return Err(invalid("sleep dwell samples"));
                }
                previous = Some(sample.time);
            }
            e.state = rebuild(&id, &e.state)
                .ok_or_else(|| invalid("sleep state has no physical identity"))?;
            tracker.entries.insert(id, e);
        }
        for ((a, b), r) in state.warm_pairs {
            r.validate_save()?;
            if !ids.contains(&a)
                || !ids.contains(&b)
                || a == b
                || tracker.warm_pairs.insert((a, b), r).is_some()
            {
                return Err(invalid("contact pair identity"));
            }
        }
        for (id, r) in state.warm_ground {
            r.validate_save()?;
            if !ids.contains(&id) || tracker.warm_ground.insert(id, r).is_some() {
                return Err(invalid("ground contact identity"));
            }
        }
        tracker.warm_dt = state.warm_dt;
        Ok(tracker)
    }
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
            authored_mesh_scale: 0.2,
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
        assert_eq!(tracker.contact_neighbors(10), &[20]);
        assert!(tracker.environment_supported(10, |_, _| true));
        assert!(!tracker.environment_supported(20, |_, _| true));
        assert!(!tracker.environment_supported(999, |_, _| true));
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
    fn warm_reactions_follow_identity_and_discard_changed_bodies_and_forces() {
        let prepared = || {
            let mut tracker = SleepTracker::default();
            let mut objects = vec![cube(0.0, 0.1), cube(0.2, 0.1)];
            advance_with_sleep(
                &mut tracker,
                &[10, 20],
                &mut objects,
                Duration::from_millis(16),
                |_, _| true,
                false,
            );
            assert!(tracker.warm_ground.contains_key(&10));
            assert!(tracker.warm_pairs.contains_key(&(10, 20)));
            (tracker, objects)
        };
        let (mut tracker, mut objects) = prepared();
        objects.reverse();
        let activation = tracker.prepare(&[20, 10], &objects);
        assert!(activation.warm_ground.contains_key(&1));
        assert!(
            activation.warm_pairs.is_empty(),
            "reversed pair response starts fresh"
        );
        for change in 0..5 {
            let (mut tracker, mut objects) = prepared();
            match change {
                0 => objects[0].mass_kg *= 2.0,
                1 => objects[0].position[0] += 0.01,
                2 => objects[0].velocity[0] += 1.0,
                3 => objects[0].surface = Surface::Unsupported,
                _ => tracker.wake_all(),
            }
            let activation = tracker.prepare(&[10, 20], &objects);
            assert!(!activation.warm_ground.contains_key(&0));
            assert!(activation.warm_pairs.is_empty());
        }
        let (mut tracker, objects) = prepared();
        let activation = tracker.prepare(&[20], &objects[1..]);
        assert!(activation.warm_pairs.is_empty());
        assert_eq!(activation.warm_ground.len(), 1);
        assert!(activation.warm_ground.contains_key(&0));
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
    fn fast_deck_impact_releases_substep_pressure_without_waking_remote_cargo() {
        for count in [500, 1000] {
            let ids: Vec<u64> = (0..=count as u64).collect();
            let mut objects: Vec<_> = (0..count).map(|i| cube(i as f64 * 0.5, 0.1)).collect();
            let mut tracker = SleepTracker::default();
            settle(&ids[..count], &mut objects, &mut tracker);
            let settled = objects.clone();
            let mut drop = cube(-2.0, 0.1);
            drop.velocity = [0.0, -40.0, 0.0];
            objects.push(drop);
            let stats = advance_with_sleep(
                &mut tracker,
                &ids,
                &mut objects,
                Duration::from_millis(16),
                |_, _| true,
                true,
            );
            assert_eq!(&objects[..count], &settled);
            assert_eq!(stats.sleeping_objects, count);
            // The old frame-wide speed selection requested 22 substeps even
            // though the first deck reaction removes the drop's downward speed.
            assert!(stats.substeps < 10, "{stats:?}");
            assert_eq!(stats.integrated_objects, stats.substeps);
            assert!(objects[count].position[1] >= 0.1 - 1e-6);
        }
    }
    #[test]
    fn geometry_force_frame_and_streaming_changes_invalidate_sleep() {
        for change in 0..6 {
            let mut tracker = SleepTracker::default();
            let mut objects = vec![cube(0.0, 0.1)];
            settle(&[1], &mut objects, &mut tracker);
            match change {
                0 => objects[0].mass_kg = 2.0,
                1 => objects[0].authored_mesh_scale *= 1.5,
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
    fn retained_projection_spans_match_fresh_substep_rebuilds_exactly() {
        let ids = [10, 20, 30];
        let mut cached = vec![cube(0.0, 0.1), cube(0.0, 0.3), cube(0.0, 0.5)];
        cached[2].angular_velocity = [0.0, 0.3, 0.0];
        let mut fresh = cached.clone();
        let mut tracker = SleepTracker::default();
        let mut reference = SleepTracker::default();
        let mut cached_builds = 0;
        let mut fresh_builds = 0;
        for update in 0..20 {
            cached_builds += advance_with_sleep(
                &mut tracker,
                &ids,
                &mut cached,
                Duration::from_millis(16),
                |_, _| true,
                true,
            )
            .projection_builds;
            // Two identical 8 ms substeps with fresh SAT spans, retaining
            // the same accumulated contact reactions and physical state.
            for _ in 0..2 {
                reference.geometry = None;
                fresh_builds += advance_with_sleep(
                    &mut reference,
                    &ids,
                    &mut fresh,
                    Duration::from_millis(8),
                    |_, _| true,
                    true,
                )
                .projection_builds;
            }
            assert_eq!(cached, fresh, "update {update}");
        }
        assert!(cached_builds < fresh_builds);
        assert!(tracker.geometry.is_some());
        let mut changed = cached.clone();
        changed[0].orientation[1] += 1e-14;
        assert!(tracker.clone().prepare(&ids, &changed).geometry.is_none());
        changed = cached.clone();
        changed.reverse();
        assert!(
            tracker
                .clone()
                .prepare(&[30, 20, 10], &changed)
                .geometry
                .is_none()
        );
        tracker.wake_all();
        assert!(tracker.geometry.is_none());
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
            assert_eq!(baseline.pair_visits, 0);
            assert!(baseline.integrated_objects > 0);
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
    fn overlapping_bounds_of_separated_hulls_do_not_wake_resting_neighbors() {
        let mut tracker = SleepTracker::default();
        let mut objects = vec![cube(0.0, 0.1)];
        settle(&[1], &mut objects, &mut tracker);
        // 0.1 m hull gap, but 0.36 m combined bounding radius.
        let resting = objects[0].clone();
        objects.push(cube(0.3, 0.1));
        objects[1].velocity = [0.06, 0.0, 0.0];
        for _ in 0..10 {
            let stats = advance_with_sleep(
                &mut tracker,
                &[1, 2],
                &mut objects,
                Duration::from_millis(16),
                |_, _| true,
                true,
            );
            assert!(stats.sleeping_objects >= 1);
            assert_eq!(objects[0], resting);
        }
    }

    #[test]
    fn point_window_credits_observed_time_and_handles_quaternion_sign() {
        let mut p = cube(0.0, 0.1);
        let mut window = MotionWindow::default();
        window.observe(&p, 2.0, true);
        assert!(!window.stable(), "one observation cannot establish dwell");
        for step in 0..40 {
            p.orientation[3] = if step % 2 == 0 { -1.0 } else { 1.0 };
            window.observe(&p, 0.016, true);
        }
        assert!(
            window.stable(),
            "antipodal quaternions represent the same pose"
        );
    }

    #[test]
    fn supported_motion_below_the_point_envelope_can_rest() {
        let mut p = cube(0.0, 0.1);
        p.velocity = [0.02, 0.0, 0.0];
        let mut window = MotionWindow::default();
        for _ in 0..40 {
            p.position[0] += 0.02 * 0.016;
            window.observe(&p, 0.016, true);
        }
        assert!(window.stable());
        window.observe(&p, 0.016, false);
        assert!(!window.stable(), "support loss invalidates the full window");
    }

    #[test]
    fn point_envelope_rejects_sliding_and_tipping_above_the_movement_bound() {
        for tipping in [false, true] {
            let mut tracker = SleepTracker::default();
            let mut objects = vec![cube(0.0, 0.1)];
            let mut activation = tracker.prepare(&[1], &objects);
            // Isolate the post-solver dwell contract with prescribed poses;
            // speeds remain below the chatter guards throughout this window.
            for step in 0..100 {
                if tipping {
                    let angle = step as f64 * 0.004;
                    objects[0].orientation = [angle.sin(), 0.0, 0.0, angle.cos()];
                    objects[0].position[1] = objects[0].hull.as_ref().unwrap().support(
                        objects[0].orientation,
                        0.2,
                        [0.0, -1.0, 0.0],
                    );
                    objects[0].angular_velocity = [0.5, 0.0, 0.0];
                } else {
                    objects[0].position[0] += 0.00128;
                    objects[0].velocity = [0.08, 0.0, 0.0];
                }
                tracker.finish(
                    &[1],
                    &mut objects,
                    Duration::from_millis(16),
                    &mut activation,
                    |_, _| true,
                );
                assert!(!activation.sleeping[0], "tipping={tipping}, step={step}");
            }
        }
    }

    #[test]
    fn bounded_constraint_projection_does_not_masquerade_as_physical_sliding() {
        for supported in [true, false] {
            let mut tracker = SleepTracker::default();
            let mut objects = vec![cube(0.0, 0.1)];
            if !supported {
                objects[0].surface = Surface::Unsupported;
            }
            let mut activation = tracker.prepare(&[1], &objects);
            for _ in 0..40 {
                objects[0].position[0] += 0.0001;
                tracker.finish(
                    &[1],
                    &mut objects,
                    Duration::from_millis(16),
                    &mut activation,
                    |_, _| true,
                );
                if activation.sleeping[0] {
                    break;
                }
            }
            assert_eq!(activation.sleeping[0], supported);
        }
    }

    #[test]
    fn bounded_angular_chatter_sleeps_but_fast_point_motion_stays_awake() {
        for phase in [1.0, -1.0] {
            for drifting in [false, true] {
                let mut tracker = SleepTracker::default();
                let mut objects = vec![cube(0.0, 0.1)];
                let mut activation = tracker.prepare(&[1], &objects);
                for step in 0..80 {
                    let chatter = phase * if step % 2 == 0 { 1.0 } else { -1.0 };
                    let angle = if drifting {
                        step as f64 * 0.004 + chatter * 0.001
                    } else {
                        chatter * 0.004
                    };
                    objects[0].orientation = [angle.sin(), 0.0, 0.0, angle.cos()];
                    objects[0].position[1] = objects[0].hull.as_ref().unwrap().support(
                        objects[0].orientation,
                        0.2,
                        [0.0, -1.0, 0.0],
                    );
                    tracker.finish(
                        &[1],
                        &mut objects,
                        Duration::from_millis(16),
                        &mut activation,
                        |_, _| true,
                    );
                    if activation.sleeping[0] {
                        break;
                    }
                }
                assert_eq!(
                    activation.sleeping[0], !drifting,
                    "phase={phase} drifting={drifting}"
                );
            }
        }
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

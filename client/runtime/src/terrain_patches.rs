//! Bounded visible cap of cube-sphere quadtree patches, never a full-planet mesh.
use salimon_math::{length, scale, sub};
use salimon_renderer::{TerrainMask, TerrainPatch, TerrainPatchKey, TerrainVertex};
#[cfg(test)]
use salimon_world::CELESTIAL_BODIES;
use salimon_world::{
    CelestialBody,
    terrain::{TerrainDefinition, TerrainMode},
};
use std::{
    collections::{BTreeMap, BTreeSet},
    sync::Arc,
    time::Instant,
};

pub(crate) const MAX_ACTIVE_PATCHES: usize = 512;
pub(crate) const MAX_CACHE_PATCHES: usize = 1024;
pub(crate) const MAX_GENERATED_PER_FRAME: usize = 8;
pub(crate) const PREPARATION_BUDGET_MICROS: u128 = 2000;
const GRID: u32 = 16;
const COVERAGE_METERS: f64 = 8192.0;
const MASK_METERS: f64 = 4096.0;

#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd)]
struct Cell {
    face: u8,
    level: u8,
    u: u32,
    v: u32,
}
fn unit(v: [f64; 3]) -> [f64; 3] {
    scale(v, 1.0 / length(v))
}
fn cube(face: u8, u: f64, v: f64) -> [f64; 3] {
    match face {
        0 => [1.0, u, v],
        1 => [-1.0, u, -v],
        2 => [u, 1.0, v],
        3 => [u, -1.0, -v],
        4 => [u, v, 1.0],
        5 => [-u, v, -1.0],
        _ => unreachable!("six cube faces"),
    }
}
fn face_uv(d: [f64; 3]) -> (u8, f64, f64) {
    let [x, y, z] = d;
    if x.abs() >= y.abs().max(z.abs()) {
        if x >= 0.0 {
            (0, y / x, z / x)
        } else {
            (1, -y / x, z / x)
        }
    } else if y.abs() >= z.abs() {
        if y >= 0.0 {
            (2, x / y, z / y)
        } else {
            (3, -x / y, z / y)
        }
    } else if z >= 0.0 {
        (4, x / z, y / z)
    } else {
        (5, x / z, -y / z)
    }
}
impl Cell {
    fn direction(self, u: f64, v: f64) -> [f64; 3] {
        let n = f64::from(1_u32 << self.level);
        unit(cube(
            self.face,
            2.0 * (f64::from(self.u) + u) / n - 1.0,
            2.0 * (f64::from(self.v) + v) / n - 1.0,
        ))
    }
    fn children(self) -> [Self; 4] {
        std::array::from_fn(|i| Self {
            face: self.face,
            level: self.level + 1,
            u: self.u * 2 + (i as u32 & 1),
            v: self.v * 2 + (i as u32 >> 1),
        })
    }
}
fn select(
    cell: Cell,
    radius: f64,
    anchor: [f64; 3],
    whole_planet: bool,
    leaves: &mut BTreeSet<Cell>,
    pending: &mut Vec<Cell>,
) {
    let center = scale(cell.direction(0.5, 0.5), radius);
    let corners = [(0.0, 0.0), (1.0, 0.0), (1.0, 1.0), (0.0, 1.0)]
        .map(|(u, v)| scale(cell.direction(u, v), radius));
    let size = (0..4)
        .map(|i| length(sub(corners[i], corners[(i + 1) % 4])))
        .fold(0.0_f64, f64::max);
    let bound = corners
        .iter()
        .map(|p| length(sub(*p, center)))
        .fold(0.0_f64, f64::max);
    let distance = length(sub(center, anchor));
    if !whole_planet && distance > COVERAGE_METERS + bound * 1.1 {
        return;
    }
    let target_size = if whole_planet {
        ((distance - bound * 1.1).max(0.0) * 3.0).max(64.0)
    } else {
        (distance * 0.45).clamp(64.0, 4096.0)
    };
    if cell.level < 24 && size > target_size {
        pending.extend(cell.children());
    } else {
        leaves.insert(cell);
    }
}
fn neighbor_at(cell: Cell, edge: usize, along: f64, leaves: &BTreeSet<Cell>) -> Option<Cell> {
    let (u, v) = match edge {
        0 => (-0.00001, along),
        1 => (1.00001, along),
        2 => (along, -0.00001),
        _ => (along, 1.00001),
    };
    let (face, u, v) = face_uv(cell.direction(u, v));
    for level in 0..=24 {
        let n = 1_u32 << level;
        let candidate = Cell {
            face,
            level,
            u: (((u + 1.0) * 0.5 * f64::from(n)).floor() as u32).min(n - 1),
            v: (((v + 1.0) * 0.5 * f64::from(n)).floor() as u32).min(n - 1),
        };
        if leaves.contains(&candidate) {
            return Some(candidate);
        }
    }
    None
}
fn neighbor(cell: Cell, edge: usize, leaves: &BTreeSet<Cell>) -> Option<Cell> {
    // The midpoint can straddle two differently refined neighbors; inspect the
    // whole edge so a fine half cannot leave an unstitched T junction.
    (0..8)
        .filter_map(|i| neighbor_at(cell, edge, (f64::from(i) + 0.5) / 8.0, leaves))
        .max_by_key(|c| c.level)
}
#[cfg(test)]
fn balanced_selection(radius: f64, anchor: [f64; 3]) -> Result<BTreeSet<Cell>, String> {
    let mut leaves = BTreeSet::new();
    let mut pending: Vec<_> = (0..6)
        .map(|face| Cell {
            face,
            level: 0,
            u: 0,
            v: 0,
        })
        .collect();
    while let Some(cell) = pending.pop() {
        select(cell, radius, anchor, false, &mut leaves, &mut pending);
    }
    loop {
        let split: Vec<_> = leaves
            .iter()
            .copied()
            .filter(|c| {
                (0..4)
                    .any(|edge| neighbor(*c, edge, &leaves).is_some_and(|n| n.level > c.level + 1))
            })
            .collect();
        if split.is_empty() {
            break;
        }
        for c in split {
            leaves.remove(&c);
            leaves.extend(c.children());
        }
        if leaves.len() > MAX_ACTIVE_PATCHES {
            return Err("terrain active patch budget exceeded".into());
        }
    }
    if leaves.len() > MAX_ACTIVE_PATCHES {
        return Err("terrain active patch budget exceeded".into());
    }
    Ok(leaves)
}
fn key(
    cell: Cell,
    body: CelestialBody,
    definition: &TerrainDefinition,
    leaves: &BTreeSet<Cell>,
) -> TerrainPatchKey {
    TerrainPatchKey {
        body: body.id as u8,
        definition: [
            b"SLMMAT01".as_slice(),
            definition.geology().to_bytes().as_slice(),
        ]
        .concat(),
        face: cell.face,
        level: cell.level,
        u: cell.u,
        v: cell.v,
        edge_levels: std::array::from_fn(|edge| {
            neighbor(cell, edge, leaves).map_or(cell.level, |n| n.level.max(cell.level))
        }),
    }
}
fn generate(
    body: CelestialBody,
    definition: &TerrainDefinition,
    key: TerrainPatchKey,
) -> TerrainPatch {
    let cell = Cell {
        face: key.face,
        level: key.level,
        u: key.u,
        v: key.v,
    };
    let anchor = definition
        .sample(cell.direction(0.5, 0.5))
        .expect("cube direction")
        .body_local_position_meters;
    let far_radius = body.radius_meters + definition.elevation_bounds_meters()[0];
    let geology = definition.geology();
    let vertex = |u, v| {
        let d = cell.direction(u, v);
        let observed = geology.sample(d).expect("cube direction");
        let sample = observed.surface;
        let (albedo_factor, strata) = crate::terrain_materials::factors(
            geology.profile(),
            &observed.geology,
            sample.elevation_meters,
        );
        TerrainVertex {
            offset_meters: sub(sample.body_local_position_meters, anchor).map(|v| v as f32),
            normal: sample.outward_normal.map(|v| v as f32),
            albedo_factor,
            strata,
            far_offset_meters: sub(scale(d, far_radius), anchor).map(|v| v as f32),
        }
    };
    let mut vertices = Vec::with_capacity(400);
    for v in 0..=GRID {
        for u in 0..=GRID {
            vertices.push(vertex(
                f64::from(u) / f64::from(GRID),
                f64::from(v) / f64::from(GRID),
            ));
        }
    }
    let mut indices = Vec::with_capacity(2000);
    for v in 0..GRID {
        for u in 0..GRID {
            let corners = [
                v * (GRID + 1) + u,
                v * (GRID + 1) + u + 1,
                (v + 1) * (GRID + 1) + u + 1,
                (v + 1) * (GRID + 1) + u,
            ];
            // On a coarse 2:1 boundary, exact midpoint samples match the fine neighbor.
            let split = [
                v == 0 && key.edge_levels[2] > cell.level,
                u == GRID - 1 && key.edge_levels[1] > cell.level,
                v == GRID - 1 && key.edge_levels[3] > cell.level,
                u == 0 && key.edge_levels[0] > cell.level,
            ];
            if !split.iter().any(|v| *v) {
                indices.extend([
                    corners[0], corners[1], corners[2], corners[0], corners[2], corners[3],
                ]);
                continue;
            }
            let center = vertices.len() as u32;
            vertices.push(vertex(
                (f64::from(u) + 0.5) / f64::from(GRID),
                (f64::from(v) + 0.5) / f64::from(GRID),
            ));
            let mut ring = Vec::with_capacity(8);
            for edge in 0..4 {
                ring.push(corners[edge]);
                if split[edge] {
                    let (du, dv) = [(0.5, 0.0), (1.0, 0.5), (0.5, 1.0), (0.0, 0.5)][edge];
                    ring.push(vertices.len() as u32);
                    vertices.push(vertex(
                        (f64::from(u) + du) / f64::from(GRID),
                        (f64::from(v) + dv) / f64::from(GRID),
                    ));
                }
            }
            for i in 0..ring.len() {
                indices.extend([center, ring[i], ring[(i + 1) % ring.len()]]);
            }
        }
    }
    TerrainPatch {
        key,
        body_center_meters: body.center.meters(),
        anchor_meters: anchor,
        vertices,
        indices,
        material: if geology.profile() == salimon_world::terrain::geology::GeologyProfile::Ice {
            salimon_renderer::SurfaceMaterial::Ice
        } else {
            match body.id {
                salimon_world::CelestialBodyId::Earth => salimon_renderer::SurfaceMaterial::Oceanic,
                salimon_world::CelestialBodyId::Moon | salimon_world::CelestialBodyId::Vesper => {
                    salimon_renderer::SurfaceMaterial::Slate
                }
                _ => salimon_renderer::SurfaceMaterial::Rust,
            }
        },
    }
}
// Cooperative CPU work, not a worker thread: bounded queues and an exact target
// identity prevent stale partial selections from becoming visible after traversal.
#[derive(Default)]
enum SelectionPhase {
    #[default]
    Select,
    Balance,
    Keys,
}
struct SelectionWork {
    id: (u8, Vec<u8>, [f64; 3]),
    leaves: BTreeSet<Cell>,
    pending: Vec<Cell>,
    scan: Vec<Cell>,
    split: Vec<Cell>,
    index: usize,
    phase: SelectionPhase,
    keys: Vec<TerrainPatchKey>,
}
impl SelectionWork {
    fn new(id: (u8, Vec<u8>, [f64; 3])) -> Self {
        Self {
            id,
            leaves: BTreeSet::new(),
            pending: (0..6)
                .map(|face| Cell {
                    face,
                    level: 0,
                    u: 0,
                    v: 0,
                })
                .collect(),
            scan: Vec::new(),
            split: Vec::new(),
            index: 0,
            phase: SelectionPhase::Select,
            keys: Vec::new(),
        }
    }
    fn advance(
        &mut self,
        body: CelestialBody,
        definition: &TerrainDefinition,
        start: Instant,
    ) -> Result<Option<Vec<TerrainPatchKey>>, String> {
        while start.elapsed().as_micros() < PREPARATION_BUDGET_MICROS {
            match self.phase {
                SelectionPhase::Select => {
                    if let Some(cell) = self.pending.pop() {
                        select(
                            cell,
                            body.radius_meters,
                            self.id.2,
                            body.id == salimon_world::CelestialBodyId::Vesper,
                            &mut self.leaves,
                            &mut self.pending,
                        );
                        if self.leaves.len() > MAX_ACTIVE_PATCHES || self.pending.len() > 128 {
                            return Err("terrain selection work budget exceeded".into());
                        }
                    } else {
                        self.scan = self.leaves.iter().copied().collect();
                        self.phase = SelectionPhase::Balance;
                    }
                }
                SelectionPhase::Balance => {
                    if let Some(cell) = self.scan.get(self.index).copied() {
                        if (0..4).any(|edge| {
                            neighbor(cell, edge, &self.leaves)
                                .is_some_and(|n| n.level > cell.level + 1)
                        }) {
                            self.split.push(cell);
                        }
                        self.index += 1;
                    } else if self.split.is_empty() {
                        self.index = 0;
                        self.phase = SelectionPhase::Keys;
                    } else {
                        for cell in self.split.drain(..) {
                            self.leaves.remove(&cell);
                            self.leaves.extend(cell.children());
                        }
                        if self.leaves.len() > MAX_ACTIVE_PATCHES {
                            return Err("terrain active patch budget exceeded".into());
                        }
                        self.scan = self.leaves.iter().copied().collect();
                        self.index = 0;
                    }
                }
                SelectionPhase::Keys => {
                    if let Some(cell) = self.scan.get(self.index) {
                        self.keys.push(key(*cell, body, definition, &self.leaves));
                        self.index += 1;
                    } else {
                        return Ok(Some(std::mem::take(&mut self.keys)));
                    }
                }
            }
        }
        Ok(None)
    }
}

#[derive(Default)]
pub(crate) struct TerrainStream {
    cache: BTreeMap<TerrainPatchKey, (Arc<TerrainPatch>, u64)>,
    selection_id: Option<(u8, Vec<u8>, [f64; 3])>,
    selection_work: Option<SelectionWork>,
    desired_keys: Vec<TerrainPatchKey>,
    desired_set: BTreeSet<TerrainPatchKey>,
    pub(crate) active: Vec<Arc<TerrainPatch>>,
    pub(crate) mask: Option<TerrainMask>,
    frame: u64,
    pub(crate) generated: usize,
    pub(crate) selection_micros: u128,
    pub(crate) preparation_micros: u128,
    pub(crate) preparation_peak_micros: u128,
    pub(crate) preparation_overruns: u64,
    pub(crate) generation_micros: u128,
    pub(crate) generation_peak_micros: u128,
    pub(crate) generation_total_micros: u128,
    pub(crate) generated_peak: usize,
    pub(crate) pending: usize,
    pub(crate) has_target: bool,
    pub(crate) total_generated: u64,
}
impl TerrainStream {
    pub(crate) fn update(&mut self, camera: [f64; 3], mode: TerrainMode) -> Result<(), String> {
        let start = Instant::now();
        self.selection_micros = 0;
        self.preparation_micros = 0;
        self.frame += 1;
        self.generated = 0;
        self.generation_micros = 0;
        let target = mode
            .bodies()
            .iter()
            .filter_map(|body| {
                let definition = mode.definition(body.id)?;
                let local = sub(camera, body.center.meters());
                let altitude = length(local) - body.radius_meters;
                (mode == TerrainMode::Canonical || altitude.abs() <= 10_000.0).then_some((
                    *body,
                    definition,
                    local,
                    altitude.abs(),
                ))
            })
            .min_by(|a, b| a.3.total_cmp(&b.3));
        self.has_target = target.is_some();
        let Some((body, definition, local, _)) = target else {
            self.selection_id = None;
            self.selection_work = None;
            self.active.clear();
            self.mask = None;
            self.pending = 0;
            self.record_preparation(start);
            return Ok(());
        };
        let (face, u, v) = face_uv(local);
        // Stable ~128–256 m streaming anchor; camera motion does not rewrite geometry.
        let level = (2.0 * body.radius_meters / 256.0).log2().ceil() as u8;
        let n = f64::from(1_u32 << level);
        let direction = unit(cube(
            face,
            2.0 * (((u + 1.0) * 0.5 * n).floor() + 0.5) / n - 1.0,
            2.0 * (((v + 1.0) * 0.5 * n).floor() + 0.5) / n - 1.0,
        ));
        let anchor = if mode == TerrainMode::Canonical {
            let altitude = (length(local) - body.radius_meters).max(0.0);
            let bin = 256.0_f64.max(altitude * 0.125);
            scale(
                direction,
                body.radius_meters + (altitude / bin).floor() * bin,
            )
        } else {
            scale(direction, body.radius_meters)
        };
        let selection_id = (body.id as u8, definition.to_bytes(), anchor);
        if self.selection_id.as_ref() != Some(&selection_id) {
            let selection_started = Instant::now();
            if self
                .selection_work
                .as_ref()
                .is_none_or(|work| work.id != selection_id)
            {
                self.selection_work = Some(SelectionWork::new(selection_id.clone()));
            }
            let ready = self
                .selection_work
                .as_mut()
                .expect("selection work initialized")
                .advance(body, definition, start)?;
            self.selection_micros = selection_started.elapsed().as_micros();
            let Some(keys) = ready else {
                self.pending = self.desired_keys.len().max(1);
                if self
                    .mask
                    .is_some_and(|m| m.body_center_meters != body.center.meters())
                {
                    self.active.clear();
                    self.mask = None;
                }
                self.record_preparation(start);
                return Ok(());
            };
            self.desired_keys = keys;
            self.desired_set = self.desired_keys.iter().cloned().collect();
            self.selection_id = Some(selection_id);
            self.selection_work = None;
        } else {
            self.selection_work = None;
        }
        let keys = &self.desired_keys;
        let desired = &self.desired_set;
        for k in keys {
            if let Some((_, age)) = self.cache.get_mut(k) {
                *age = self.frame;
                continue;
            }
            if self.generated >= MAX_GENERATED_PER_FRAME
                || start.elapsed().as_micros() >= PREPARATION_BUDGET_MICROS
            {
                continue;
            }
            while self.cache.len() >= MAX_CACHE_PATCHES {
                let oldest = self
                    .cache
                    .iter()
                    .filter(|(k, _)| {
                        !desired.contains(*k) && !self.active.iter().any(|p| &p.key == *k)
                    })
                    .min_by_key(|(_, (_, age))| *age)
                    .map(|(k, _)| k.clone());
                if let Some(k) = oldest {
                    self.cache.remove(&k);
                } else {
                    return Err("terrain CPU cache budget exceeded".into());
                }
            }
            let generated_at = Instant::now();
            let patch = Arc::new(generate(body, definition, k.clone()));
            self.generation_micros += generated_at.elapsed().as_micros();
            self.cache.insert(k.clone(), (patch, self.frame));
            self.generated += 1;
            self.total_generated += 1;
        }
        self.generation_total_micros += self.generation_micros;
        self.generation_peak_micros = self.generation_peak_micros.max(self.generation_micros);
        self.generated_peak = self.generated_peak.max(self.generated);
        self.pending = keys.iter().filter(|k| !self.cache.contains_key(*k)).count();
        if self.pending == 0 {
            self.active = keys.iter().map(|k| self.cache[k].0.clone()).collect();
            self.mask = Some(TerrainMask {
                body_center_meters: body.center.meters(),
                anchor_meters: anchor,
                radius_meters: if mode == TerrainMode::Canonical {
                    body.radius_meters * 4.0 + length(anchor)
                } else {
                    MASK_METERS
                },
            });
        } else if self
            .mask
            .is_some_and(|m| m.body_center_meters != body.center.meters())
        {
            self.active.clear();
            self.mask = None;
        }
        self.record_preparation(start);
        Ok(())
    }
    fn record_preparation(&mut self, start: Instant) {
        self.preparation_micros = start.elapsed().as_micros();
        self.preparation_peak_micros = self.preparation_peak_micros.max(self.preparation_micros);
        if self.preparation_micros > PREPARATION_BUDGET_MICROS {
            self.preparation_overruns += 1;
        }
    }
    pub(crate) fn cpu_bytes(&self) -> usize {
        self.cache
            .values()
            .map(|(p, _)| {
                p.vertices.capacity() * std::mem::size_of::<TerrainVertex>()
                    + p.indices.capacity() * 4
                    + p.key.definition.capacity()
            })
            .sum::<usize>()
            + self.cache.len()
                * (std::mem::size_of::<TerrainPatch>()
                    + std::mem::size_of::<TerrainPatchKey>()
                    + 32)
            + self.desired_keys.capacity() * std::mem::size_of::<TerrainPatchKey>()
            + self
                .desired_keys
                .iter()
                .map(|k| k.definition.capacity())
                .sum::<usize>()
            + self.desired_set.len() * (std::mem::size_of::<TerrainPatchKey>() + 32)
            + self
                .desired_set
                .iter()
                .map(|k| k.definition.capacity())
                .sum::<usize>()
            + self.active.capacity() * std::mem::size_of::<Arc<TerrainPatch>>()
            + self.selection_work.as_ref().map_or(0, |w| {
                w.leaves.len() * (std::mem::size_of::<Cell>() + 32)
                    + (w.pending.capacity() + w.scan.capacity() + w.split.capacity())
                        * std::mem::size_of::<Cell>()
                    + w.keys.capacity() * std::mem::size_of::<TerrainPatchKey>()
                    + w.keys
                        .iter()
                        .map(|k| k.definition.capacity())
                        .sum::<usize>()
                    + w.id.1.capacity()
            })
    }
    pub(crate) fn triangles(&self) -> usize {
        self.active.iter().map(|p| p.indices.len() / 3).sum()
    }
    pub(crate) fn cached(&self) -> usize {
        self.cache.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use salimon_world::CelestialBodyId;
    fn body(id: CelestialBodyId) -> CelestialBody {
        *CELESTIAL_BODIES.iter().find(|b| b.id == id).unwrap()
    }
    #[test]
    fn cap_covers_poles_face_edges_and_corners_with_balanced_lod() {
        for id in [
            CelestialBodyId::Earth,
            CelestialBodyId::Moon,
            CelestialBodyId::Mars,
        ] {
            let b = body(id);
            for d in [
                [0.0, 1.0, 0.0],
                [0.0, 0.0, 1.0],
                unit([1.0, 1.0, 0.0]),
                unit([1.0, 1.0, 1.0]),
                unit([-1.0, -1.0, -1.0]),
            ] {
                let leaves =
                    balanced_selection(b.radius_meters, scale(d, b.radius_meters)).unwrap();
                assert!(leaves.len() <= MAX_ACTIVE_PATCHES);
                eprintln!("{} {:?}: {} patches", b.name, d, leaves.len());
                for c in &leaves {
                    for e in 0..4 {
                        if let Some(n) = neighbor(*c, e, &leaves) {
                            assert!(c.level.abs_diff(n.level) <= 1);
                        }
                    }
                }
                let axis = unit(salimon_math::cross(
                    d,
                    if d[0].abs() < 0.8 {
                        [1.0, 0.0, 0.0]
                    } else {
                        [0.0, 1.0, 0.0]
                    },
                ));
                let side = salimon_math::cross(d, axis);
                for x in -10..=10 {
                    for y in -10..=10 {
                        if x * x + y * y > 100 {
                            continue;
                        }
                        let point = unit(salimon_math::add(
                            scale(d, b.radius_meters),
                            salimon_math::add(
                                scale(axis, f64::from(x) * 409.6),
                                scale(side, f64::from(y) * 409.6),
                            ),
                        ));
                        let (face, u, v) = face_uv(point);
                        assert!(
                            leaves.iter().any(|c| {
                                let n = f64::from(1_u32 << c.level);
                                c.face == face
                                    && (u + 1.0) * 0.5 * n >= f64::from(c.u)
                                    && (u + 1.0) * 0.5 * n <= f64::from(c.u + 1)
                                    && (v + 1.0) * 0.5 * n >= f64::from(c.v)
                                    && (v + 1.0) * 0.5 * n <= f64::from(c.v + 1)
                            }),
                            "cap hole"
                        );
                    }
                }
            }
        }
    }
    #[test]
    fn patch_vertices_and_stitched_borders_follow_authoritative_query() {
        let b = body(CelestialBodyId::Earth);
        let definition = TerrainMode::Representative.definition(b.id).unwrap();
        let anchor = scale(unit([1.0, 1.0, 1.0]), b.radius_meters);
        let leaves = balanced_selection(b.radius_meters, anchor).unwrap();
        let mut shared: BTreeMap<[i64; 3], [f64; 3]> = BTreeMap::new();
        let mut duplicates = 0;
        let mut max_error = 0.0_f64;
        for c in &leaves {
            let patch = generate(b, definition, key(*c, b, definition, &leaves));
            for vertex in &patch.vertices {
                let p: [f64; 3] = std::array::from_fn(|i| {
                    patch.anchor_meters[i] + f64::from(vertex.offset_meters[i])
                });
                let expected = definition
                    .sample(unit(p))
                    .unwrap()
                    .body_local_position_meters;
                let error = length(sub(p, expected));
                max_error = max_error.max(error);
                assert!(error < 0.0002, "sample error {error}");
                let rounded = p.map(|v| (v * 1000.0).round() as i64);
                if let Some(old) = shared.insert(rounded, p) {
                    assert!(length(sub(p, old)) < 0.0002);
                    duplicates += 1;
                }
            }
        }
        assert!(duplicates > 100);
        eprintln!("border vertices max query error {max_error} m, {duplicates} shared nodes");
    }
    #[test]
    fn mesh_contact_tolerance_is_measured_at_all_profile_craters_and_plain_spawns() {
        for id in [
            CelestialBodyId::Earth,
            CelestialBodyId::Moon,
            CelestialBodyId::Mars,
        ] {
            let b = body(id);
            let d = TerrainMode::Representative.definition(id).unwrap();
            for direction in [[0.0, 1.0, 0.0], [0.0, 0.0, 1.0]] {
                let anchor = scale(direction, b.radius_meters);
                let leaves = balanced_selection(b.radius_meters, anchor).unwrap();
                let mut max_error = 0.0_f64;
                let mut count = 0;
                for c in &leaves {
                    if length(sub(scale(c.direction(0.5, 0.5), b.radius_meters), anchor)) > 200.0 {
                        continue;
                    }
                    let p = generate(b, d, key(*c, b, d, &leaves));
                    for tri in p.indices.as_chunks::<3>().0 {
                        let local: [f64; 3] = std::array::from_fn(|i| {
                            p.anchor_meters[i]
                                + tri
                                    .iter()
                                    .map(|v| {
                                        f64::from(p.vertices[*v as usize].offset_meters[i]) / 3.0
                                    })
                                    .sum::<f64>()
                        });
                        if length(sub(scale(unit(local), b.radius_meters), anchor)) > 150.0 {
                            continue;
                        }
                        let ground = d.sample(unit(local)).unwrap();
                        max_error =
                            max_error.max(length(sub(local, ground.body_local_position_meters)));
                        count += 1;
                    }
                }
                eprintln!(
                    "{} {:?}: {count} near triangle centers, max error {max_error} m",
                    b.name, direction
                );
                assert!(
                    count > 100 && max_error < 0.25,
                    "mesh/contact tolerance exceeded: {max_error}"
                );
            }
        }
    }
    #[test]
    fn stream_generation_memory_and_revisit_identity_are_bounded() {
        let b = body(CelestialBodyId::Earth);
        let mut stream = TerrainStream::default();
        let camera = b
            .center
            .translated([0.0, b.radius_meters + 10.0, 0.0])
            .meters();
        for _ in 0..600 {
            stream.update(camera, TerrainMode::Representative).unwrap();
            assert!(
                stream.generated <= MAX_GENERATED_PER_FRAME && stream.cached() <= MAX_CACHE_PATCHES
            );
            assert!(stream.cpu_bytes() < 48 * 1024 * 1024);
            if stream.pending == 0 {
                break;
            }
        }
        assert_eq!(stream.pending, 0);
        assert!(!stream.active.is_empty());
        let first = stream.active.clone();
        let total = stream.total_generated;
        for _ in 0..20 {
            stream.update(camera, TerrainMode::Representative).unwrap();
        }
        assert_eq!(stream.total_generated, total);
        for (a, b) in first.iter().zip(&stream.active) {
            assert!(Arc::ptr_eq(a, b));
        }
        eprintln!(
            "{} active, {} triangles, {} CPU bytes, {} generated",
            stream.active.len(),
            stream.triangles(),
            stream.cpu_bytes(),
            total
        );
        for _ in 0..600 {
            stream
                .update(
                    salimon_math::add(camera, [1200.0, 0.0, 0.0]),
                    TerrainMode::Representative,
                )
                .unwrap();
            if stream.pending == 0 {
                break;
            }
        }
        assert_eq!(stream.pending, 0);
        for _ in 0..600 {
            stream.update(camera, TerrainMode::Representative).unwrap();
            if stream.pending == 0 {
                break;
            }
        }
        assert_eq!(stream.active, first);
    }
}

#[cfg(test)]
mod topology_tests {
    use super::*;
    #[test]
    fn mixed_lod_and_cross_face_segments_have_no_open_edges_inside_the_mask() {
        for mode in [TerrainMode::Representative, TerrainMode::Canonical] {
            let body = if mode == TerrainMode::Canonical {
                salimon_world::canonical::BODIES[0]
            } else {
                *CELESTIAL_BODIES
                    .iter()
                    .find(|b| b.id == salimon_world::CelestialBodyId::Earth)
                    .unwrap()
            };
            let definition = mode.definition(body.id).unwrap();
            for direction in [unit([1.0, 1.0, 0.0]), unit([1.0, 1.0, 1.0])] {
                let anchor = scale(direction, body.radius_meters);
                let leaves = if mode == TerrainMode::Canonical {
                    let mut work =
                        SelectionWork::new((body.id as u8, definition.to_bytes(), anchor));
                    let mut selected = None;
                    for _ in 0..5000 {
                        if let Some(keys) = work.advance(body, definition, Instant::now()).unwrap()
                        {
                            selected = Some(
                                keys.into_iter()
                                    .map(|k| Cell {
                                        face: k.face,
                                        level: k.level,
                                        u: k.u,
                                        v: k.v,
                                    })
                                    .collect(),
                            );
                            break;
                        }
                    }
                    selected.expect("bounded canonical selection")
                } else {
                    balanced_selection(body.radius_meters, anchor).unwrap()
                };
                let mut boundary =
                    std::collections::HashMap::<[[i64; 3]; 2], ([f64; 3], u32)>::new();
                for cell in &leaves {
                    let patch = generate(body, definition, key(*cell, body, definition, &leaves));
                    let mut local = std::collections::HashMap::<[u32; 2], u32>::new();
                    for tri in patch.indices.as_chunks::<3>().0 {
                        for i in 0..3 {
                            let mut edge = [tri[i], tri[(i + 1) % 3]];
                            edge.sort();
                            *local.entry(edge).or_default() += 1;
                        }
                    }
                    for (edge, count) in local {
                        if count != 1 {
                            continue;
                        }
                        let points = edge.map(|index| {
                            std::array::from_fn::<_, 3, _>(|i| {
                                patch.anchor_meters[i]
                                    + f64::from(patch.vertices[index as usize].offset_meters[i])
                            })
                        });
                        let mid = salimon_math::scale(salimon_math::add(points[0], points[1]), 0.5);
                        let mut positions = points.map(|p| {
                            let cube =
                                scale(p, 1.0 / p.iter().map(|v| v.abs()).fold(0.0_f64, f64::max));
                            cube.map(|v| (v * f64::from(1_u32 << 24)).round() as i64)
                        });
                        positions.sort();
                        boundary.entry(positions).or_insert((mid, 0)).1 += 1;
                    }
                }
                let mut checked = 0;
                for (_, (mid, count)) in boundary {
                    if length(sub(scale(unit(mid), body.radius_meters), anchor)) < MASK_METERS {
                        assert_eq!(count, 2, "unpaired near terrain edge at {mid:?}");
                        checked += 1;
                    }
                }
                assert!(checked > 1000);
            }
        }
    }
}

#[cfg(test)]
mod selection_work_tests {
    use super::*;
    #[test]
    fn cooperative_selection_matches_reference_and_rejects_stale_body_targets() {
        let mut stream = TerrainStream::default();
        let mode = TerrainMode::Representative;
        for body in CELESTIAL_BODIES
            .iter()
            .filter(|b| mode.definition(b.id).is_some())
        {
            let camera = body
                .center
                .translated([0.0, body.radius_meters + 2.0, 0.0])
                .meters();
            // Change bodies before previous work can publish a stale cap.
            stream.update(camera, mode).unwrap();
            if let Some(work) = &stream.selection_work {
                assert_eq!(work.id.0, body.id as u8);
            }
        }
        let body = *CELESTIAL_BODIES
            .iter()
            .find(|b| b.id == salimon_world::CelestialBodyId::Earth)
            .unwrap();
        let camera = body
            .center
            .translated([0.0, body.radius_meters + 2.0, 0.0])
            .meters();
        for _ in 0..2000 {
            stream.update(camera, mode).unwrap();
            if stream.pending == 0 {
                break;
            }
        }
        assert_eq!(stream.pending, 0);
        let anchor = stream.selection_id.as_ref().unwrap().2;
        let leaves = balanced_selection(body.radius_meters, anchor).unwrap();
        let expected: Vec<_> = leaves
            .iter()
            .map(|c| key(*c, body, mode.definition(body.id).unwrap(), &leaves))
            .collect();
        assert_eq!(stream.desired_keys, expected);
        assert!(stream.active.iter().all(|p| p.key.body == body.id as u8));
    }
}

#[cfg(test)]
mod canonical_tests {
    use super::*;
    fn selection(body: CelestialBody, anchor: [f64; 3]) -> Vec<TerrainPatchKey> {
        let d = TerrainMode::Canonical.definition(body.id).unwrap();
        let mut work = SelectionWork::new((body.id as u8, d.to_bytes(), anchor));
        for _ in 0..5000 {
            if let Some(keys) = work.advance(body, d, Instant::now()).unwrap() {
                return keys;
            }
        }
        panic!("bounded selection did not complete");
    }
    #[test]
    fn canonical_orbit_to_surface_covers_all_faces_and_balances_seams() {
        let b = salimon_world::canonical::BODIES[0];
        for altitude in [b.radius_meters * 10., b.radius_meters, 100_000., 1000., 3.] {
            for d in [[0., 1., 0.], unit([1., 1., 0.]), unit([1., 1., 1.])] {
                let keys = selection(b, scale(d, b.radius_meters + altitude));
                assert!(keys.len() <= MAX_ACTIVE_PATCHES);
                if altitude < 10.0 {
                    assert!(
                        keys.iter().any(|k| k.level >= 16),
                        "surface detail did not refine"
                    );
                }
                assert_eq!(
                    keys.iter().map(|k| k.face).collect::<BTreeSet<_>>().len(),
                    6
                );
                let leaves: BTreeSet<_> = keys
                    .iter()
                    .map(|k| Cell {
                        face: k.face,
                        level: k.level,
                        u: k.u,
                        v: k.v,
                    })
                    .collect();
                // Sum exact dyadic face areas: all six faces have full coverage.
                for face in 0..6 {
                    let area: f64 = leaves
                        .iter()
                        .filter(|c| c.face == face)
                        .map(|c| 4.0_f64.powi(-i32::from(c.level)))
                        .sum();
                    assert_eq!(area, 1.);
                }
                for c in &leaves {
                    for edge in 0..4 {
                        let n = neighbor(*c, edge, &leaves).unwrap();
                        assert!(c.level.abs_diff(n.level) <= 1);
                    }
                }
                eprintln!(
                    "canonical altitude={altitude} direction={d:?}: {} patches",
                    keys.len()
                );
            }
        }
    }
    #[test]
    fn canonical_cpu_residency_eviction_and_revisit_keep_identical_geometry() {
        let b = salimon_world::canonical::BODIES[0];
        let mut stream = TerrainStream::default();
        let cameras = [
            [0., b.radius_meters + 3., 0.],
            [b.radius_meters + 3., 0., 0.],
            [0., b.radius_meters * 3., 0.],
            [0., b.radius_meters + 3., 0.],
        ];
        let mut first = None;
        for camera in cameras {
            for _ in 0..5000 {
                stream.update(camera, TerrainMode::Canonical).unwrap();
                if stream.pending == 0 && !stream.active.is_empty() {
                    break;
                }
            }
            assert_eq!(stream.pending, 0);
            assert!(!stream.active.is_empty());
            assert!(stream.cached() <= MAX_CACHE_PATCHES);
            assert!(stream.cpu_bytes() < 48 * 1024 * 1024);
            let geometry: Vec<_> = stream.active.iter().map(|p| (**p).clone()).collect();
            if camera == cameras[0] {
                if let Some(old) = &first {
                    assert_eq!(&geometry, old);
                } else {
                    first = Some(geometry);
                }
            }
        }
    }
}

#[cfg(test)]
mod geology_material_tests {
    use super::*;
    #[test]
    fn shared_world_vertices_keep_material_and_physical_identity_across_lod() {
        let body = *CELESTIAL_BODIES
            .iter()
            .find(|b| b.id == salimon_world::CelestialBodyId::Moon)
            .unwrap();
        let definition = TerrainDefinition::representative(body).unwrap();
        let parent = Cell {
            face: 2,
            level: 8,
            u: 128,
            v: 128,
        };
        let child = Cell {
            face: 2,
            level: 9,
            u: 256,
            v: 256,
        };
        let patches = [parent, child].map(|cell| {
            let leaves = BTreeSet::from([cell]);
            generate(body, &definition, key(cell, body, &definition, &leaves))
        });
        assert_eq!(
            patches[0].vertices[0].albedo_factor,
            patches[1].vertices[0].albedo_factor
        );
        assert_eq!(patches[0].vertices[0].strata, patches[1].vertices[0].strata);
        assert!(patches[0].key.definition.starts_with(b"SLMMAT01SLMGEO01"));
        for (cell, patch) in [parent, child].into_iter().zip(patches) {
            for v in 0..=GRID {
                for u in 0..=GRID {
                    let vertex = patch.vertices[(v * (GRID + 1) + u) as usize];
                    let sample = definition
                        .sample(cell.direction(
                            f64::from(u) / f64::from(GRID),
                            f64::from(v) / f64::from(GRID),
                        ))
                        .unwrap();
                    let position = std::array::from_fn(|i| {
                        patch.anchor_meters[i] + f64::from(vertex.offset_meters[i])
                    });
                    assert!(length(sub(position, sample.body_local_position_meters)) < 0.001);
                    assert_eq!(vertex.strata[0], sample.elevation_meters as f32);
                    assert!((0.8..1.0).contains(&vertex.albedo_factor));
                }
            }
        }
    }
}

#[cfg(test)]
mod landmark_tests {
    use super::*;
    use salimon_world::environment::landmarks::{Geography, Landmarks, Scale, Signature};
    fn source() -> (CelestialBody, TerrainDefinition, Cell, Landmarks) {
        let body = *CELESTIAL_BODIES
            .iter()
            .find(|b| b.id == salimon_world::CelestialBodyId::Moon)
            .unwrap();
        let cell = Cell {
            face: 2,
            level: 12,
            u: 2048,
            v: 2048,
        };
        let d = cell.direction(0.5, 0.5);
        let terrain = TerrainDefinition::new(
            body,
            1,
            91,
            vec![],
            vec![salimon_world::terrain::Feature {
                direction: d,
                radius_meters: 400.,
                elevation_meters: -160.,
            }],
        )
        .unwrap();
        let sites = Landmarks::publish(
            terrain.geology(),
            Geography::Observed,
            0,
            &[Signature {
                key: "declared-basin-fixture".into(),
                direction: d,
                scale: Scale::Regional,
            }],
        )
        .unwrap();
        (body, terrain, cell, sites)
    }
    #[test]
    fn landmark_render_lods_and_checkpoint_binding_share_physical_terrain() {
        let (body, terrain, cell, sites) = source();
        let binding = sites.binding();
        let original = terrain.to_bytes();
        for level in [11, 12, 13] {
            let c = Cell {
                level,
                u: cell.u >> (12 - level.min(12)),
                v: cell.v >> (12 - level.min(12)),
                ..cell
            };
            let c = if level > 12 {
                Cell {
                    u: cell.u * 2,
                    v: cell.v * 2,
                    ..c
                }
            } else {
                c
            };
            let patches = BTreeSet::from([c]);
            let p = generate(body, &terrain, key(c, body, &terrain, &patches));
            for v in &p.vertices {
                let position =
                    std::array::from_fn(|i| p.anchor_meters[i] + f64::from(v.offset_meters[i]));
                let physical = terrain.sample(unit(position)).unwrap();
                assert!(length(sub(position, physical.body_local_position_meters)) < 0.001);
            }
        }
        let restored = TerrainDefinition::from_bytes(body, &original).unwrap();
        let site = &sites.sites()[0];
        let rebuilt = Landmarks::publish(
            restored.geology(),
            Geography::Observed,
            0,
            &[Signature {
                key: "declared-basin-fixture".into(),
                direction: site.direction,
                scale: Scale::Regional,
            }],
        )
        .unwrap();
        rebuilt.verify_binding(&binding).unwrap();
        assert_eq!(terrain.to_bytes(), original);
        // The publication is authoritative metadata only, never player-known chart input.
        assert!(salimon_world::canonical::LANDMARKS.sites().is_empty());
    }
    #[test]
    #[ignore = "exports production terrain patch for actual landmark source; set SALIMON_LANDMARK_PATCH_FILE"]
    fn landmark_production_patch_export() {
        let (body, terrain, cell, sites) = source();
        let leaves = BTreeSet::from([cell]);
        let p = generate(body, &terrain, key(cell, body, &terrain, &leaves));
        let vertices:Vec<_>=p.vertices.iter().map(|v|serde_json::json!({"offset":v.offset_meters,"normal":v.normal,"far":v.far_offset_meters,"albedo":v.albedo_factor,"strata":v.strata})).collect();
        let site = &sites.sites()[0];
        let data = serde_json::json!({"id":site.id,"publication":sites.identity(),"binding":sites.binding(),"geology":terrain.geology().to_bytes(),"anchor":p.anchor_meters,"position":site.position_meters,"radius":site.radius_meters,"relief":site.relief_meters,"vertices":vertices,"indices":p.indices,"definition":p.key.definition});
        std::fs::write(
            std::env::var("SALIMON_LANDMARK_PATCH_FILE").unwrap(),
            serde_json::to_vec_pretty(&data).unwrap(),
        )
        .unwrap();
    }
}

//! Bounded visible cap of cube-sphere quadtree patches, never a full-planet mesh.
use salimon_math::{length, scale, sub};
use salimon_renderer::{TerrainMask, TerrainPatch, TerrainPatchKey, TerrainVertex};
use salimon_world::{
    CELESTIAL_BODIES, CelestialBody,
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
fn select(cell: Cell, radius: f64, anchor: [f64; 3], leaves: &mut BTreeSet<Cell>) {
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
    if distance > COVERAGE_METERS + bound * 1.1 {
        return;
    }
    if cell.level < 24 && size > (distance * 0.45).clamp(64.0, 4096.0) {
        for child in cell.children() {
            select(child, radius, anchor, leaves);
        }
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
fn balanced_selection(radius: f64, anchor: [f64; 3]) -> Result<BTreeSet<Cell>, String> {
    let mut leaves = BTreeSet::new();
    for face in 0..6 {
        select(
            Cell {
                face,
                level: 0,
                u: 0,
                v: 0,
            },
            radius,
            anchor,
            &mut leaves,
        );
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
        definition: definition.to_bytes(),
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
    let vertex = |u, v| {
        let d = cell.direction(u, v);
        let sample = definition.sample(d).expect("cube direction");
        TerrainVertex {
            offset_meters: sub(sample.body_local_position_meters, anchor).map(|v| v as f32),
            normal: sample.outward_normal.map(|v| v as f32),
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
        material: match body.id {
            salimon_world::CelestialBodyId::Earth => salimon_renderer::SurfaceMaterial::Oceanic,
            salimon_world::CelestialBodyId::Moon => salimon_renderer::SurfaceMaterial::Slate,
            _ => salimon_renderer::SurfaceMaterial::Rust,
        },
    }
}
#[derive(Default)]
pub(crate) struct TerrainStream {
    cache: BTreeMap<TerrainPatchKey, (Arc<TerrainPatch>, u64)>,
    selection_id: Option<(u8, Vec<u8>, [f64; 3])>,
    desired_keys: Vec<TerrainPatchKey>,
    desired_set: BTreeSet<TerrainPatchKey>,
    pub(crate) active: Vec<Arc<TerrainPatch>>,
    pub(crate) mask: Option<TerrainMask>,
    frame: u64,
    pub(crate) generated: usize,
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
        self.frame += 1;
        self.generated = 0;
        self.generation_micros = 0;
        let target = CELESTIAL_BODIES
            .iter()
            .filter_map(|body| {
                let definition = mode.definition(body.id)?;
                let local = sub(camera, body.center.meters());
                let altitude = length(local) - body.radius_meters;
                (altitude.abs() <= 10_000.0).then_some((*body, definition, local, altitude.abs()))
            })
            .min_by(|a, b| a.3.total_cmp(&b.3));
        self.has_target = target.is_some();
        let Some((body, definition, local, _)) = target else {
            self.active.clear();
            self.mask = None;
            self.pending = 0;
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
        let anchor = scale(direction, body.radius_meters);
        let selection_id = (body.id as u8, definition.to_bytes(), anchor);
        if self.selection_id.as_ref() != Some(&selection_id) {
            let selection = balanced_selection(body.radius_meters, anchor)?;
            self.desired_keys = selection
                .iter()
                .map(|c| key(*c, body, definition, &selection))
                .collect();
            self.desired_set = self.desired_keys.iter().cloned().collect();
            self.selection_id = Some(selection_id);
        }
        let keys = &self.desired_keys;
        let desired = &self.desired_set;
        let start = Instant::now();
        for k in keys {
            if let Some((_, age)) = self.cache.get_mut(k) {
                *age = self.frame;
                continue;
            }
            if self.generated >= MAX_GENERATED_PER_FRAME
                || (self.generated > 0 && start.elapsed().as_micros() >= 2000)
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
                radius_meters: MASK_METERS,
            });
        } else if self
            .mask
            .is_some_and(|m| m.body_center_meters != body.center.meters())
        {
            self.active.clear();
            self.mask = None;
        }
        Ok(())
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
        let body = *CELESTIAL_BODIES
            .iter()
            .find(|b| b.id == salimon_world::CelestialBodyId::Earth)
            .unwrap();
        let definition = TerrainMode::Representative.definition(body.id).unwrap();
        for direction in [unit([1.0, 1.0, 0.0]), unit([1.0, 1.0, 1.0])] {
            let anchor = scale(direction, body.radius_meters);
            let leaves = balanced_selection(body.radius_meters, anchor).unwrap();
            let mut boundary = std::collections::HashMap::<[[i64; 3]; 2], ([f64; 3], u32)>::new();
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

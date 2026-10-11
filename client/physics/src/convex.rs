//! Immutable convex proxies derived from authored vertices; no asset/platform dependencies.
use salimon_math::{add, cross, dot, length, scale, sub};

// Include a nearly coplanar supporting feature rather than switching between
// individual vertices as a resting face rocks by floating-point/contact noise.
const SUPPORT_FEATURE_TOLERANCE: f64 = 0.001;

#[derive(Clone, Debug, PartialEq)]
pub struct ConvexHull {
    pub(crate) vertices: Vec<[f64; 3]>,
    normals: Vec<[f64; 3]>,
    edges: Vec<[f64; 3]>,
}

impl ConvexHull {
    /// Build once at unit scale. Duplicate face vertices are merged. Supporting
    /// planes and edges describe the convex envelope, including non-triangular faces.
    pub fn new(points: impl IntoIterator<Item = [f64; 3]>) -> Self {
        let mut vertices = Vec::new();
        for p in points {
            assert!(p.iter().all(|v| v.is_finite()));
            if !vertices.iter().any(|v| length(sub(p, *v)) < 1e-8) {
                vertices.push(p);
            }
        }
        assert!(
            vertices.len() >= 4,
            "convex geometry needs a solid vertex set"
        );
        let mut normals = Vec::new();
        let mut planes: Vec<([f64; 3], f64)> = Vec::new();
        for i in 0..vertices.len() {
            for j in i + 1..vertices.len() {
                for k in j + 1..vertices.len() {
                    let axis = cross(sub(vertices[j], vertices[i]), sub(vertices[k], vertices[i]));
                    if length(axis) < 1e-10 {
                        continue;
                    }
                    let mut n = scale(axis, 1.0 / length(axis));
                    let mut d = dot(n, vertices[i]);
                    let low = vertices.iter().all(|v| dot(n, *v) <= d + 1e-8);
                    let high = vertices.iter().all(|v| dot(n, *v) >= d - 1e-8);
                    if !low && !high {
                        continue;
                    }
                    if high {
                        n = scale(n, -1.0);
                        d = -d;
                    }
                    if planes
                        .iter()
                        .any(|(p, h)| dot(*p, n) > 1.0 - 1e-8 && (d - h).abs() < 1e-8)
                    {
                        continue;
                    }
                    planes.push((n, d));
                    push_axis(&mut normals, n);
                }
            }
        }
        assert!(
            planes.len() >= 4,
            "convex geometry must have nonzero volume"
        );
        let mut edges = Vec::new();
        for i in 0..vertices.len() {
            for j in i + 1..vertices.len() {
                let shared: Vec<_> = planes
                    .iter()
                    .filter(|(n, d)| {
                        (dot(*n, vertices[i]) - d).abs() < 1e-8
                            && (dot(*n, vertices[j]) - d).abs() < 1e-8
                    })
                    .collect();
                if shared
                    .iter()
                    .any(|a| shared.iter().any(|b| dot(a.0, b.0).abs() < 1.0 - 1e-8))
                {
                    push_axis(&mut edges, sub(vertices[j], vertices[i]));
                }
            }
        }
        Self {
            vertices,
            normals,
            edges,
        }
    }

    pub fn support(&self, orientation: [f64; 4], side: f64, direction: [f64; 3]) -> f64 {
        let inverse = [
            -orientation[0],
            -orientation[1],
            -orientation[2],
            orientation[3],
        ];
        let local = rotate(inverse, direction);
        self.vertices
            .iter()
            .map(|v| dot(*v, local) * side)
            .fold(f64::NEG_INFINITY, f64::max)
    }

    /// Closest point of the supporting feature to the center-of-mass projection.
    /// A resting face distributes reaction beneath the COM instead of applying
    /// a torque at its geometric centroid. The 1 mm/unit tolerance prevents
    /// facet-edge chatter while retaining visibly close terrain contact.
    pub(crate) fn ground_arm(
        &self,
        orientation: [f64; 4],
        side: f64,
        direction: [f64; 3],
    ) -> [f64; 3] {
        let extreme = self.support(orientation, side, direction);
        let points: Vec<_> = self
            .vertices
            .iter()
            .map(|v| rotate(orientation, scale(*v, side)))
            .filter(|p| dot(*p, direction) >= extreme - SUPPORT_FEATURE_TOLERANCE * side)
            .collect();
        let target = scale(direction, extreme);
        let mut nearest = points[0];
        for &a in &points {
            for &b in &points {
                let ab = sub(b, a);
                let squared = dot(ab, ab);
                if squared > 1e-16 {
                    let p = add(
                        a,
                        scale(ab, (dot(sub(target, a), ab) / squared).clamp(0.0, 1.0)),
                    );
                    if length(sub(p, target)) < length(sub(nearest, target)) {
                        nearest = p;
                    }
                }
                for &c in &points {
                    let ac = sub(c, a);
                    let at = sub(target, a);
                    let d00 = dot(ab, ab);
                    let d01 = dot(ab, ac);
                    let d11 = dot(ac, ac);
                    let denom = d00 * d11 - d01 * d01;
                    if denom < 1e-16 {
                        continue;
                    }
                    let u = (d11 * dot(at, ab) - d01 * dot(at, ac)) / denom;
                    let v = (d00 * dot(at, ac) - d01 * dot(at, ab)) / denom;
                    if u >= 0.0 && v >= 0.0 && u + v <= 1.0 {
                        let p = add(a, add(scale(ab, u), scale(ac, v)));
                        if length(sub(p, target)) < length(sub(nearest, target)) {
                            nearest = p;
                        }
                    }
                }
            }
        }
        nearest
    }

    /// A resting face needs several normal constraints to resist pitch/roll
    /// under an off-center load. A single reaction beneath its own COM cannot
    /// oppose that torque. Faces/edges use their actual near-coplanar vertices;
    /// a vertex retains the existing reaction arm and can still tip.
    pub(crate) fn ground_arms(
        &self,
        orientation: [f64; 4],
        side: f64,
        direction: [f64; 3],
    ) -> Vec<[f64; 3]> {
        let extreme = self.support(orientation, side, direction);
        let points: Vec<_> = self
            .vertices
            .iter()
            .map(|v| rotate(orientation, scale(*v, side)))
            .filter(|p| dot(*p, direction) >= extreme - SUPPORT_FEATURE_TOLERANCE * side)
            .collect();
        if points.len() < 2 {
            vec![self.ground_arm(orientation, side, direction)]
        } else {
            points
        }
    }
}

fn push_axis(axes: &mut Vec<[f64; 3]>, axis: [f64; 3]) {
    let size = length(axis);
    if size < 1e-10 {
        return;
    }
    let n = scale(axis, 1.0 / size);
    if !axes.iter().any(|v| dot(*v, n).abs() > 1.0 - 1e-8) {
        axes.push(n);
    }
}

/// Rotate a vector by a caller-validated unit XYZW quaternion.
pub fn rotate(q: [f64; 4], p: [f64; 3]) -> [f64; 3] {
    let v = [q[0], q[1], q[2]];
    let t = scale(cross(v, p), 2.0);
    add(p, add(scale(t, q[3]), cross(v, t)))
}

pub fn compose(a: [f64; 4], b: [f64; 4]) -> [f64; 4] {
    let av = [a[0], a[1], a[2]];
    let bv = [b[0], b[1], b[2]];
    let xyz = add(add(scale(bv, a[3]), scale(av, b[3])), cross(av, bv));
    [xyz[0], xyz[1], xyz[2], a[3] * b[3] - dot(av, bv)]
}

pub(crate) fn integrate(q: [f64; 4], omega: [f64; 3], dt: f64) -> [f64; 4] {
    let speed = length(omega);
    if speed < 1e-12 {
        return q;
    }
    let angle = speed * dt * 0.5;
    let v = scale(omega, angle.sin() / speed);
    let result = compose([v[0], v[1], v[2], angle.cos()], q);
    let norm = result.iter().map(|v| v * v).sum::<f64>().sqrt();
    result.map(|v| v / norm)
}

#[derive(Clone)]
pub(crate) struct Projection {
    axis: [f64; 3],
    forward: f64,
    backward: f64,
}

/// Reject common sphere-overlapping but hull-separated pairs before constructing
/// crossed-edge axes. Recheck each pass because other contacts can move centers.
pub(crate) fn separated_by_faces(a: &super::ObjectState, b: &super::ObjectState) -> bool {
    separated_by_faces_with_gap(a, b, 0.0)
}

fn separated_by_faces_with_gap(a: &super::ObjectState, b: &super::ObjectState, gap: f64) -> bool {
    let ah = a.hull.as_ref().expect("convex caller");
    let bh = b.hull.as_ref().expect("convex caller");
    let separation = sub(b.position, a.position);
    ah.normals
        .iter()
        .map(|n| rotate(a.orientation, *n))
        .chain(bh.normals.iter().map(|n| rotate(b.orientation, *n)))
        .any(|axis| {
            let offset = dot(separation, axis);
            offset
                > ah.support(a.orientation, a.side_meters, axis)
                    + bh.support(b.orientation, b.side_meters, scale(axis, -1.0))
                    + gap
                || -offset
                    > ah.support(a.orientation, a.side_meters, scale(axis, -1.0))
                        + bh.support(b.orientation, b.side_meters, axis)
                        + gap
        })
}

/// Orientation/scale are constant through the contact iterations in a substep.
/// Cache relative projections once rather than rotating vertices in every pass.
pub(crate) fn projections(a: &super::ObjectState, b: &super::ObjectState) -> Vec<Projection> {
    let ah = a.hull.as_ref().expect("convex caller");
    let bh = b.hull.as_ref().expect("convex caller");
    let mut axes = Vec::new();
    for n in &ah.normals {
        push_axis(&mut axes, rotate(a.orientation, *n));
    }
    for n in &bh.normals {
        push_axis(&mut axes, rotate(b.orientation, *n));
    }
    // Keep crossed-edge axes without a quadratic duplicate search. Native dense
    // profiling shows repeated parallel axes dominate projection/graph work;
    // quantized membership below removes those repeats in deterministic order.
    let ae: Vec<_> = ah.edges.iter().map(|v| rotate(a.orientation, *v)).collect();
    let be: Vec<_> = bh.edges.iter().map(|v| rotate(b.orientation, *v)).collect();
    for a in &ae {
        for b in &be {
            let axis = cross(*a, *b);
            let size = length(axis);
            if size > 1e-10 {
                axes.push(scale(axis, 1.0 / size));
            }
        }
    }
    let mut seen = std::collections::HashSet::with_capacity(axes.len());
    axes.retain(|axis| {
        // Opposite axes describe the same separating planes. 1e-9 component
        // bins merge only numerical parallel repeats (sub-nanometre projection
        // error at fragment scales, far below the 10 micrometre separation bias).
        let sign = axis
            .iter()
            .find(|v| v.abs() > 1e-9)
            .map_or(1.0, |v| v.signum());
        let key: [i64; 3] = axis.map(|v| (v * sign * 1e9).round() as i64);
        seen.insert(key)
    });
    let av: Vec<_> = ah
        .vertices
        .iter()
        .map(|v| rotate(a.orientation, scale(*v, a.side_meters)))
        .collect();
    let bv: Vec<_> = bh
        .vertices
        .iter()
        .map(|v| rotate(b.orientation, scale(*v, b.side_meters)))
        .collect();
    let span = |vertices: &[[f64; 3]], axis| {
        vertices
            .iter()
            .map(|v| dot(*v, axis))
            .fold((f64::INFINITY, f64::NEG_INFINITY), |(lo, hi), x| {
                (lo.min(x), hi.max(x))
            })
    };
    axes.into_iter()
        .map(|axis| {
            let (amin, amax) = span(&av, axis);
            let (bmin, bmax) = span(&bv, axis);
            Projection {
                axis,
                forward: amax - bmin,
                backward: bmax - amin,
            }
        })
        .collect()
}

/// Reuse orientation/scale spans after center-only contact corrections.
pub(crate) fn near_projected_contact(
    projections: &[Projection],
    offset: [f64; 3],
    gap: f64,
) -> bool {
    projections.iter().all(|p| {
        let shift = dot(offset, p.axis);
        p.forward - shift >= -gap && p.backward + shift >= -gap
    })
}

/// Full SAT with a small support-gap tolerance; spheres alone never anchor sleep.
pub(crate) fn near_contact(a: &super::ObjectState, b: &super::ObjectState, gap: f64) -> bool {
    if separated_by_faces_with_gap(a, b, gap) {
        return false;
    }
    // Faces have already passed. Test crossed edges directly and stop at the
    // first separating axis; the support graph needs a boolean, not a cached
    // minimum-translation manifold. Rotated vertices are shared across axes.
    let ah = a.hull.as_ref().expect("convex caller");
    let bh = b.hull.as_ref().expect("convex caller");
    let offset = sub(b.position, a.position);
    let vertices = |h: &ConvexHull, orientation, side| {
        h.vertices
            .iter()
            .map(|v| rotate(orientation, scale(*v, side)))
            .collect::<Vec<_>>()
    };
    let av = vertices(ah, a.orientation, a.side_meters);
    let bv = vertices(bh, b.orientation, b.side_meters);
    let span = |vertices: &[[f64; 3]], axis| {
        vertices
            .iter()
            .map(|v| dot(*v, axis))
            .fold((f64::INFINITY, f64::NEG_INFINITY), |(lo, hi), x| {
                (lo.min(x), hi.max(x))
            })
    };
    let be: Vec<_> = bh.edges.iter().map(|v| rotate(b.orientation, *v)).collect();
    for edge in &ah.edges {
        let ae = rotate(a.orientation, *edge);
        for &be in &be {
            let axis = cross(ae, be);
            let size = length(axis);
            if size <= 1e-10 {
                continue;
            }
            let axis = scale(axis, 1.0 / size);
            let (amin, amax) = span(&av, axis);
            let (bmin, bmax) = span(&bv, axis);
            let shift = dot(offset, axis);
            if amax - bmin - shift < -gap || bmax - amin + shift < -gap {
                return false;
            }
        }
    }
    true
}

/// Minimum translation of B relative to A, with universe-scale-safe subtraction.
pub(crate) fn projected_contact(
    projections: &[Projection],
    offset: [f64; 3],
) -> Option<([f64; 3], f64)> {
    let mut best = ([0.0, 1.0, 0.0], f64::INFINITY);
    for p in projections {
        let shift = dot(offset, p.axis);
        let forward = p.forward - shift;
        let backward = p.backward + shift;
        // Position separation leaves a 10 um bias. Keep velocity constraints
        // participating across that bias during later passes; otherwise a light
        // supported body receives only one impulse from a heavy upper body and
        // the upper body retains an artificial downward terminal velocity.
        let skin = 2.0 * crate::CONTACT_BIAS_METERS;
        if forward < -skin || backward < -skin {
            return None;
        }
        let candidate = if forward <= backward {
            (p.axis, forward)
        } else {
            (scale(p.axis, -1.0), backward)
        };
        if candidate.1 < best.1 {
            best = candidate;
        }
    }
    Some(best)
}

#[cfg(test)]
fn contact(a: &super::ObjectState, b: &super::ObjectState) -> Option<([f64; 3], f64)> {
    projected_contact(&projections(a, b), sub(b.position, a.position))
}

/// Approximate manifold center from overlapping support-feature tangent bounds.
/// A face against a vertex uses the incident vertex rather than the face centroid,
/// so an off-center contact has the correct torque arm. Coordinates are relative to A.
pub(crate) fn contact_arms(
    a: &super::ObjectState,
    b: &super::ObjectState,
    normal: [f64; 3],
) -> ([f64; 3], [f64; 3]) {
    let offset = sub(b.position, a.position);
    let ah = a.hull.as_ref().expect("convex caller");
    let bh = b.hull.as_ref().expect("convex caller");
    let feature = |h: &ConvexHull, q, side, direction| {
        let support = h.support(q, side, direction);
        h.vertices
            .iter()
            .map(|v| rotate(q, scale(*v, side)))
            .filter(|p| dot(*p, direction) >= support - side * SUPPORT_FEATURE_TOLERANCE)
            .collect::<Vec<_>>()
    };
    let av = feature(ah, a.orientation, a.side_meters, normal);
    let bv: Vec<_> = feature(bh, b.orientation, b.side_meters, scale(normal, -1.0))
        .into_iter()
        .map(|v| add(v, offset))
        .collect();
    let seed = if normal[1].abs() < 0.9 {
        [0.0, 1.0, 0.0]
    } else {
        [1.0, 0.0, 0.0]
    };
    let tangent = cross(normal, seed);
    let tangent = scale(tangent, 1.0 / length(tangent));
    let bitangent = cross(normal, tangent);
    let interval = |vertices: &[[f64; 3]], axis| {
        vertices
            .iter()
            .map(|v| dot(*v, axis))
            .fold((f64::INFINITY, f64::NEG_INFINITY), |(lo, hi), v| {
                (lo.min(v), hi.max(v))
            })
    };
    let middle = |axis| {
        let (al, ah) = interval(&av, axis);
        let (bl, bh) = interval(&bv, axis);
        (al.max(bl) + ah.min(bh)) * 0.5
    };
    let height = (dot(av[0], normal) + dot(bv[0], normal)) * 0.5;
    let point = add(
        scale(normal, height),
        add(
            scale(tangent, middle(tangent)),
            scale(bitangent, middle(bitangent)),
        ),
    );
    (point, sub(point, offset))
}

/// Intersect the two supporting faces in the contact tangent plane. Normal
/// constraints at the actual polygon corners balance loads without inventing
/// support outside either hull. A face/edge intersection keeps both segment
/// endpoints; vertex contacts retain one reaction arm.
pub(crate) fn contact_manifold(
    a: &super::ObjectState,
    b: &super::ObjectState,
    normal: [f64; 3],
) -> Vec<([f64; 3], [f64; 3])> {
    let offset = sub(b.position, a.position);
    let seed = if normal[1].abs() < 0.9 {
        [0.0, 1.0, 0.0]
    } else {
        [1.0, 0.0, 0.0]
    };
    let tangent = cross(normal, seed);
    let tangent = scale(tangent, 1.0 / length(tangent));
    let bitangent = cross(normal, tangent);
    let feature = |p: &super::ObjectState, direction| {
        let hull = p.hull.as_ref().expect("convex caller");
        let support = hull.support(p.orientation, p.side_meters, direction);
        hull.vertices
            .iter()
            .map(|v| rotate(p.orientation, scale(*v, p.side_meters)))
            .filter(|v| dot(*v, direction) >= support - p.side_meters * SUPPORT_FEATURE_TOLERANCE)
            .collect::<Vec<_>>()
    };
    let av = feature(a, normal);
    let bv: Vec<_> = feature(b, scale(normal, -1.0))
        .into_iter()
        .map(|v| add(v, offset))
        .collect();
    let fallback = || vec![contact_arms(a, b, normal)];
    if av.len() < 2 || bv.len() < 2 {
        return fallback();
    }
    let project = |v| [dot(v, tangent), dot(v, bitangent)];
    let mut polygon = tangent_hull(av.iter().copied().map(project).collect());
    let mut clip = tangent_hull(bv.iter().copied().map(project).collect());
    if clip.len() == 2 && polygon.len() >= 3 {
        std::mem::swap(&mut polygon, &mut clip);
    }
    if polygon.len() < 2 || clip.len() < 3 {
        return fallback();
    }
    for i in 0..clip.len() {
        let start = clip[i];
        let end = clip[(i + 1) % clip.len()];
        let distance = |p| cross2(start, end, p);
        let input = std::mem::take(&mut polygon);
        if input.is_empty() {
            return fallback();
        }
        let mut previous = *input.last().expect("nonempty polygon");
        let mut previous_distance = distance(previous);
        for current in input {
            let current_distance = distance(current);
            if (current_distance >= 0.0) != (previous_distance >= 0.0) {
                let t = previous_distance / (previous_distance - current_distance);
                polygon.push(std::array::from_fn(|axis| {
                    previous[axis] + t * (current[axis] - previous[axis])
                }));
            }
            if current_distance >= 0.0 {
                polygon.push(current);
            }
            previous = current;
            previous_distance = current_distance;
        }
    }
    polygon = tangent_hull(polygon);
    if polygon.len() < 2 {
        return fallback();
    }
    let height = (dot(av[0], normal) + dot(bv[0], normal)) * 0.5;
    polygon
        .into_iter()
        .map(|p| {
            let point = add(
                scale(normal, height),
                add(scale(tangent, p[0]), scale(bitangent, p[1])),
            );
            (point, sub(point, offset))
        })
        .collect()
}

fn cross2(a: [f64; 2], b: [f64; 2], c: [f64; 2]) -> f64 {
    (b[0] - a[0]) * (c[1] - a[1]) - (b[1] - a[1]) * (c[0] - a[0])
}

fn tangent_hull(mut points: Vec<[f64; 2]>) -> Vec<[f64; 2]> {
    points.sort_by(|a, b| a[0].total_cmp(&b[0]).then(a[1].total_cmp(&b[1])));
    points.dedup_by(|a, b| (a[0] - b[0]).abs() < 1e-12 && (a[1] - b[1]).abs() < 1e-12);
    if points.len() < 3 {
        return points;
    }
    let mut lower = Vec::new();
    let mut upper = Vec::new();
    for &p in &points {
        while lower.len() >= 2 && cross2(lower[lower.len() - 2], lower[lower.len() - 1], p) <= 1e-14
        {
            lower.pop();
        }
        lower.push(p);
    }
    for &p in points.iter().rev() {
        while upper.len() >= 2 && cross2(upper[upper.len() - 2], upper[upper.len() - 1], p) <= 1e-14
        {
            upper.pop();
        }
        upper.push(p);
    }
    lower.pop();
    upper.pop();
    lower.extend(upper);
    lower
}

/// Unit quaternion from right-handed orthonormal basis columns.
pub fn orientation_from_axes(axes: [[f64; 3]; 3]) -> [f64; 4] {
    let m = |row: usize, column: usize| axes[column][row];
    let trace = m(0, 0) + m(1, 1) + m(2, 2);
    let q = if trace > 0.0 {
        let s = (trace + 1.0).sqrt() * 2.0;
        [
            (m(2, 1) - m(1, 2)) / s,
            (m(0, 2) - m(2, 0)) / s,
            (m(1, 0) - m(0, 1)) / s,
            s * 0.25,
        ]
    } else {
        let i = (0..3)
            .max_by(|&i, &j| m(i, i).total_cmp(&m(j, j)))
            .expect("three axes");
        let j = (i + 1) % 3;
        let k = (i + 2) % 3;
        let s = (1.0 + m(i, i) - m(j, j) - m(k, k)).sqrt() * 2.0;
        let mut q = [0.0; 4];
        q[i] = s * 0.25;
        q[j] = (m(j, i) + m(i, j)) / s;
        q[k] = (m(k, i) + m(i, k)) / s;
        q[3] = (m(k, j) - m(j, k)) / s;
        q
    };
    let norm = q.iter().map(|v| v * v).sum::<f64>().sqrt();
    q.map(|v| v / norm)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{ObjectState, Surface, advance};
    use std::{sync::Arc, time::Duration};

    #[test]
    fn support_graph_short_circuit_matches_full_sat() {
        for index in 0..400 {
            let mut a = object([0.0; 3], [0.3, 0.5, 0.2]);
            let mut b = object(
                [
                    0.1 * (index % 13) as f64,
                    0.03 * (index % 7) as f64,
                    0.02 * (index % 11) as f64,
                ],
                [0.4, 0.2, 0.3],
            );
            let angle = index as f64 * 0.17;
            a.orientation = [0.0, (angle / 2.0).sin(), 0.0, (angle / 2.0).cos()];
            b.orientation = [(angle / 3.0).sin(), 0.0, 0.0, (angle / 3.0).cos()];
            for gap in [0.0, 0.001, 0.05] {
                let offset = sub(b.position, a.position);
                let reference = projections(&a, &b).iter().all(|p| {
                    let shift = dot(offset, p.axis);
                    p.forward - shift >= -gap && p.backward + shift >= -gap
                });
                assert_eq!(near_contact(&a, &b, gap), reference, "{index} {gap}");
            }
        }
    }

    #[test]
    fn loaded_deck_support_does_not_undo_pair_separation() {
        let mut bottom = object([0.0, 0.2, 0.0], [0.2; 3]);
        let mut top = object([0.0, 0.45, 0.0], [0.2; 3]);
        bottom.mass_kg = 0.5;
        top.mass_kg = 2.0;
        let mut pieces = [bottom, top];
        advance(&mut pieces, Duration::from_millis(1), |_, _| true);
        assert!((pieces[0].position[1] - 0.2).abs() < 1e-9);
        assert!((pieces[1].position[1] - 0.60001).abs() < 1e-6);
        assert!(!near_contact(&pieces[0], &pieces[1], 0.0));
        // The velocity solver retains contact across its separation bias.
        assert!(contact(&pieces[0], &pieces[1]).unwrap().1 <= 0.0);
        assert_eq!(pieces[0].mass_kg, 0.5);
        assert_eq!(pieces[1].mass_kg, 2.0);
    }

    #[test]
    fn resting_face_balances_an_off_center_load_without_rocking() {
        let mut bottom = object([0.0, 0.1, 0.0], [0.3, 0.1, 0.3]);
        bottom.mass_kg = 0.5;
        let top = object([0.15, 0.35, 0.0], [0.1; 3]);
        let mut pieces = [bottom, top];
        let mut tracker = crate::SleepTracker::default();
        for _ in 0..360 {
            crate::advance_with_sleep(
                &mut tracker,
                &[1, 2],
                &mut pieces,
                Duration::from_millis(16),
                |_, _| true,
                false,
            );
        }
        assert!(
            tracker.is_sleeping(1) && tracker.is_sleeping(2),
            "{:?}",
            pieces
                .iter()
                .map(|p| (p.position, p.velocity, p.angular_velocity, p.orientation))
                .collect::<Vec<_>>()
        );
        assert!(pieces[0].orientation[0].abs() < 0.005);
        assert!(pieces[0].orientation[2].abs() < 0.005);
        assert!((pieces[0].position[1] - 0.1).abs() < 0.001);
        assert!((pieces[1].position[1] - 0.3).abs() < 0.002);
    }

    #[test]
    fn asymmetric_resting_face_supports_its_center_of_mass() {
        let bottom = object([0.0, 0.1, 0.0], [0.5, 0.1, 0.5]);
        let mut top = object([0.0, 0.35, 0.0], [0.4; 3]);
        top.hull =
            Some(Arc::new(ConvexHull::new([-0.1, 0.1].into_iter().flat_map(
                |y| [[-0.1, y, -0.2], [0.3, y, -0.2], [0.05, y, 0.4]],
            ))));
        let mut pieces = [bottom, top];
        let mut tracker = crate::SleepTracker::default();
        for _ in 0..360 {
            crate::advance_with_sleep(
                &mut tracker,
                &[1, 2],
                &mut pieces,
                Duration::from_millis(16),
                |_, _| true,
                false,
            );
        }
        assert!(tracker.is_sleeping(1) && tracker.is_sleeping(2));
        assert!(pieces[1].orientation[0].abs() < 0.005);
        assert!(pieces[1].orientation[2].abs() < 0.005);
        assert!((pieces[1].position[1] - 0.3).abs() < 0.002);
    }

    #[test]
    fn supported_face_in_a_stack_balances_an_off_center_load() {
        let bottom = object([0.0, 0.1, 0.0], [0.5, 0.1, 0.5]);
        let mut middle = object([0.0, 0.3, 0.0], [0.3, 0.1, 0.3]);
        middle.mass_kg = 0.5;
        let top = object([0.15, 0.55, 0.0], [0.1; 3]);
        let mut pieces = [bottom, middle, top];
        let mut tracker = crate::SleepTracker::default();
        for _ in 0..360 {
            crate::advance_with_sleep(
                &mut tracker,
                &[1, 2, 3],
                &mut pieces,
                Duration::from_millis(16),
                |_, _| true,
                false,
            );
        }
        assert!((1..=3).all(|id| tracker.is_sleeping(id)), "{:?}", pieces);
        assert!(pieces[1].orientation[0].abs() < 0.005);
        assert!(pieces[1].orientation[2].abs() < 0.005);
        assert!((pieces[2].position[1] - 0.5).abs() < 0.002);
    }

    #[test]
    fn face_manifold_corners_stay_inside_both_rotated_support_faces() {
        let a = object([0.0, 0.1, 0.0], [0.3, 0.1, 0.3]);
        let mut b = object([0.15, 0.3, 0.05], [0.25, 0.1, 0.2]);
        b.orientation = [0.0, 0.37_f64.sin(), 0.0, 0.37_f64.cos()];
        let arms = contact_manifold(&a, &b, [0.0, 1.0, 0.0]);
        assert!(arms.len() >= 3);
        for (aa, ba) in arms {
            assert!(aa[0].abs() <= 0.3 + 1e-12 && aa[2].abs() <= 0.3 + 1e-12);
            let local_b = rotate([0.0, -b.orientation[1], 0.0, b.orientation[3]], ba);
            assert!(local_b[0].abs() <= 0.25 + 1e-12 && local_b[2].abs() <= 0.2 + 1e-12);
            assert!(length(sub(add(a.position, aa), add(b.position, ba))) < 1e-12);
        }
    }

    #[test]
    fn supporting_edge_balances_a_load_along_its_length() {
        let mut bottom = object([0.0, 0.2, 0.0], [0.5, 0.2, 0.5]);
        bottom.hull =
            Some(Arc::new(ConvexHull::new([-0.5, 0.5].into_iter().flat_map(
                |z| [[-0.5, -0.2, z], [0.5, -0.2, z], [0.0, 0.2, z]],
            ))));
        let mut middle = object([0.0, 0.5, 0.0], [0.3, 0.1, 0.3]);
        middle.mass_kg = 0.5;
        let top = object([0.0, 0.75, 0.15], [0.1; 3]);
        let mut pieces = [bottom, middle, top];
        let mut tracker = crate::SleepTracker::default();
        for _ in 0..360 {
            crate::advance_with_sleep(
                &mut tracker,
                &[1, 2, 3],
                &mut pieces,
                Duration::from_millis(16),
                |_, _| true,
                false,
            );
        }
        assert!(
            (1..=3).all(|id| tracker.is_sleeping(id)),
            "{:?}",
            pieces
                .iter()
                .map(|p| (p.position, p.velocity, p.angular_velocity, p.orientation))
                .collect::<Vec<_>>()
        );
        assert!(pieces[1].orientation[0].abs() < 0.005);
        assert!((pieces[2].position[1] - 0.7).abs() < 0.002);
    }

    #[test]
    fn resting_face_friction_resists_twist() {
        let bottom = object([0.0, 0.1, 0.0], [0.5, 0.1, 0.5]);
        let mut top = object([0.0, 0.3, 0.0], [0.2, 0.1, 0.2]);
        top.angular_velocity = [0.0, 0.4, 0.0];
        let mut pieces = [bottom, top];
        advance(&mut pieces, Duration::from_millis(16), |_, _| true);
        assert!(
            pieces[1].angular_velocity[1].abs() < 0.35,
            "{:?}",
            pieces[1].angular_velocity
        );
    }

    #[test]
    fn distributed_friction_preserves_momentum_and_dissipates_energy() {
        let mut a = object([0.0, 0.1, 0.0], [0.4, 0.1, 0.4]);
        a.mass_kg = 0.5;
        let mut b = object([0.1, 0.3, 0.0], [0.2, 0.1, 0.2]);
        b.velocity = [0.3, -0.2, 0.1];
        b.angular_velocity = [0.1, 0.5, -0.2];
        let momentum = |p: &ObjectState| scale(p.velocity, p.mass_kg);
        let angular = |p: &ObjectState| {
            add(
                cross(p.position, momentum(p)),
                scale(p.angular_velocity, 0.4 * p.mass_kg * p.radius * p.radius),
            )
        };
        let energy = |p: &ObjectState| {
            0.5 * p.mass_kg * dot(p.velocity, p.velocity)
                + 0.2
                    * p.mass_kg
                    * p.radius
                    * p.radius
                    * dot(p.angular_velocity, p.angular_velocity)
        };
        let before = (
            add(momentum(&a), momentum(&b)),
            add(angular(&a), angular(&b)),
            energy(&a) + energy(&b),
        );
        crate::pair_impulse(&mut a, &mut b, [0.0, 1.0, 0.0]);
        assert!(length(sub(add(momentum(&a), momentum(&b)), before.0)) < 1e-12);
        assert!(length(sub(add(angular(&a), angular(&b)), before.1)) < 1e-12);
        assert!(energy(&a) + energy(&b) <= before.2 + 1e-12);
    }

    fn box_hull(half: [f64; 3]) -> Arc<ConvexHull> {
        Arc::new(ConvexHull::new((0..8).map(|i| {
            std::array::from_fn(|axis| half[axis] * if i & (1 << axis) == 0 { -1.0 } else { 1.0 })
        })))
    }
    fn object(position: [f64; 3], half: [f64; 3]) -> ObjectState {
        ObjectState {
            position,
            velocity: [0.0; 3],
            radius: length(half),
            ground_support_meters: half[1],
            surface: Surface::Floor { height_meters: 0.0 },
            hull: Some(box_hull(half)),
            side_meters: 1.0,
            orientation: [0.0, 0.0, 0.0, 1.0],
            angular_velocity: [0.0; 3],
            mass_kg: 2.0,
        }
    }
    #[test]
    fn flat_shapes_touch_without_bounding_sphere_gaps_and_rotation_changes_contact() {
        let a = object([0.0, 0.1, 0.0], [0.3, 0.1, 0.2]);
        let mut b = object([0.0, 0.3, 0.0], [0.2, 0.1, 0.2]);
        assert!(length(sub(a.position, b.position)) < a.radius + b.radius);
        assert!(contact(&a, &b).unwrap().1 < 1e-12);
        b.position[1] += 0.001;
        assert!(contact(&a, &b).is_none());
        b.orientation = [
            0.0,
            0.0,
            (std::f64::consts::FRAC_PI_4).sin(),
            (std::f64::consts::FRAC_PI_4).cos(),
        ];
        assert!(contact(&a, &b).unwrap().1 > 0.05);
    }

    #[test]
    fn repeated_edge_axes_keep_rotated_box_separation_and_support_gap() {
        for angle in [0.0_f64, 0.17, 0.73] {
            let mut a = object([0.0, 0.1, 0.0], [0.3, 0.1, 0.2]);
            a.orientation = [0.0, angle.sin(), 0.0, angle.cos()];
            let mut b = a.clone();
            b.position[1] = 0.3;
            assert_eq!(projections(&a, &b).len(), 3);
            assert!(contact(&a, &b).unwrap().1.abs() < 1e-12);
            b.position[1] += 0.0005;
            assert!(contact(&a, &b).is_none());
            assert!(near_contact(&a, &b, 0.001));
            b.position[1] += 0.001;
            assert!(!near_contact(&a, &b, 0.001));
        }
    }
    #[test]
    fn convex_pile_settles_and_support_removal_reactivates_gravity() {
        let initial = [
            object([0.0, 0.1, 0.0], [0.3, 0.1, 0.3]),
            object([0.0, 0.3, 0.0], [0.25, 0.1, 0.25]),
            object([0.0, 0.5, 0.0], [0.2, 0.1, 0.2]),
        ];
        let mut pieces = initial.clone();
        let mut repeat = initial;
        for _ in 0..240 {
            advance(&mut pieces, Duration::from_millis(16), |_, _| true);
            advance(&mut repeat, Duration::from_millis(16), |_, _| true);
        }
        assert_eq!(pieces, repeat);
        for (p, expected) in pieces.iter().zip([0.1, 0.3, 0.5]) {
            assert!((p.position[1] - expected).abs() < 0.003, "{p:?}");
            assert!(length(p.velocity) < 0.03);
        }
        let mut unsupported = [pieces[2].clone()];
        for _ in 0..180 {
            advance(&mut unsupported, Duration::from_millis(16), |_, _| true);
        }
        assert!((unsupported[0].position[1] - 0.1).abs() < 0.002);
    }
    #[test]
    fn off_center_ground_contact_tips_and_fast_drop_does_not_cross_floor() {
        let mut p = object([0.0, 0.3, 0.0], [0.3, 0.1, 0.2]);
        p.orientation = [0.0, 0.0, (0.2_f64).sin(), (0.2_f64).cos()];
        let before = p.orientation;
        p.velocity = [0.0, -8.0, 0.0];
        let mut pieces = [p];
        for _ in 0..100 {
            advance(&mut pieces, Duration::from_millis(16), |_, _| true);
        }
        assert_ne!(pieces[0].orientation, before);
        let support =
            pieces[0]
                .hull
                .as_ref()
                .unwrap()
                .support(pieces[0].orientation, 1.0, [0.0, -1.0, 0.0]);
        assert!(pieces[0].position[1] >= support - 1e-6);
        assert!(pieces[0].orientation.iter().all(|v| v.is_finite()));
    }
    #[test]
    fn growth_refreshes_contacts_without_changing_the_shared_hull() {
        let mut pieces = [
            object([0.0, 0.1, 0.0], [0.3, 0.1, 0.3]),
            object([0.0, 0.3, 0.0], [0.2, 0.1, 0.2]),
        ];
        let hull = pieces[1].hull.clone().unwrap();
        pieces[1].side_meters = 1.5;
        pieces[1].radius *= 1.5;
        pieces[1].mass_kg = 3.0;
        for _ in 0..120 {
            advance(&mut pieces, Duration::from_millis(16), |_, _| true);
        }
        assert!(Arc::ptr_eq(&hull, pieces[1].hull.as_ref().unwrap()));
        assert!((pieces[0].position[1] - 0.1).abs() < 0.003);
        assert!((pieces[1].position[1] - 0.35).abs() < 0.003);
        assert!(contact(&pieces[0], &pieces[1]).is_none_or(|(_, depth)| depth < 0.001));
    }

    #[test]
    fn ship_basis_orientation_roundtrips_vectors() {
        let axes = [[0.0, 0.0, -1.0], [0.0, 1.0, 0.0], [1.0, 0.0, 0.0]];
        let q = orientation_from_axes(axes);
        for (axis, expected) in axes.iter().enumerate() {
            let mut basis = [0.0; 3];
            basis[axis] = 1.0;
            assert!(length(sub(rotate(q, basis), *expected)) < 1e-12);
        }
    }
}

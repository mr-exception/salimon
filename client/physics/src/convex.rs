//! Immutable convex proxies derived from authored vertices; no asset/platform dependencies.
use salimon_math::{add, cross, dot, length, scale, sub};

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
        assert!(vertices.len() >= 4, "convex geometry needs a solid vertex set");
        let mut normals = Vec::new();
        let mut planes: Vec<([f64; 3], f64)> = Vec::new();
        for i in 0..vertices.len() {
            for j in i + 1..vertices.len() {
                for k in j + 1..vertices.len() {
                    let axis = cross(sub(vertices[j], vertices[i]), sub(vertices[k], vertices[i]));
                    if length(axis) < 1e-10 { continue; }
                    let mut n = scale(axis, 1.0 / length(axis));
                    let mut d = dot(n, vertices[i]);
                    let low = vertices.iter().all(|v| dot(n, *v) <= d + 1e-8);
                    let high = vertices.iter().all(|v| dot(n, *v) >= d - 1e-8);
                    if !low && !high { continue; }
                    if high { n = scale(n, -1.0); d = -d; }
                    if planes.iter().any(|(p, h)| dot(*p, n) > 1.0 - 1e-8 && (d - h).abs() < 1e-8) { continue; }
                    planes.push((n, d));
                    push_axis(&mut normals, n);
                }
            }
        }
        assert!(planes.len() >= 4, "convex geometry must have nonzero volume");
        let mut edges = Vec::new();
        for i in 0..vertices.len() {
            for j in i + 1..vertices.len() {
                let shared: Vec<_> = planes.iter().filter(|(n, d)|
                    (dot(*n, vertices[i]) - d).abs() < 1e-8 && (dot(*n, vertices[j]) - d).abs() < 1e-8
                ).collect();
                if shared.iter().any(|a| shared.iter().any(|b| dot(a.0, b.0).abs() < 1.0 - 1e-8)) {
                    push_axis(&mut edges, sub(vertices[j], vertices[i]));
                }
            }
        }
        Self { vertices, normals, edges }
    }

    pub fn support(&self, orientation: [f64; 4], side: f64, direction: [f64; 3]) -> f64 {
        self.vertices.iter().map(|v| dot(rotate(orientation, scale(*v, side)), direction)).fold(f64::NEG_INFINITY, f64::max)
    }

    pub(crate) fn support_point(&self, orientation: [f64; 4], side: f64, direction: [f64; 3]) -> [f64; 3] {
        let extreme = self.support(orientation, side, direction);
        let mut sum = [0.0; 3];
        let mut count = 0.0;
        for v in &self.vertices {
            let p = rotate(orientation, scale(*v, side));
            if dot(p, direction) >= extreme - 1e-7 * side {
                sum = add(sum, p); count += 1.0;
            }
        }
        scale(sum, 1.0 / count)
    }
}

fn push_axis(axes: &mut Vec<[f64; 3]>, axis: [f64; 3]) {
    let size = length(axis);
    if size < 1e-10 { return; }
    let n = scale(axis, 1.0 / size);
    if !axes.iter().any(|v| dot(*v, n).abs() > 1.0 - 1e-8) { axes.push(n); }
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
    if speed < 1e-12 { return q; }
    let angle = speed * dt * 0.5;
    let v = scale(omega, angle.sin() / speed);
    let result = compose([v[0], v[1], v[2], angle.cos()], q);
    let norm = result.iter().map(|v| v * v).sum::<f64>().sqrt();
    result.map(|v| v / norm)
}

/// Minimum separating translation of B relative to A. Projection is relative to
/// A's origin to preserve precision at universe-scale coordinates.
pub(crate) fn contact(a: &super::ObjectState, b: &super::ObjectState) -> Option<([f64; 3], f64)> {
    let (Some(ah), Some(bh)) = (&a.hull, &b.hull) else { return None; };
    let offset = sub(b.position, a.position);
    let mut axes = Vec::new();
    for n in &ah.normals { push_axis(&mut axes, rotate(a.orientation, *n)); }
    for n in &bh.normals { push_axis(&mut axes, rotate(b.orientation, *n)); }
    for ae in &ah.edges {
        for be in &bh.edges {
            push_axis(&mut axes, cross(rotate(a.orientation, *ae), rotate(b.orientation, *be)));
        }
    }
    let mut best = ([0.0, 1.0, 0.0], f64::INFINITY);
    for axis in axes {
        let opposite = scale(axis, -1.0);
        let amax = ah.support(a.orientation, a.side_meters, axis);
        let amin = -ah.support(a.orientation, a.side_meters, opposite);
        let shift = dot(offset, axis);
        let bmax = bh.support(b.orientation, b.side_meters, axis) + shift;
        let bmin = -bh.support(b.orientation, b.side_meters, opposite) + shift;
        let forward = amax - bmin;
        let backward = bmax - amin;
        if forward < 0.0 || backward < 0.0 { return None; }
        let candidate = if forward <= backward { (axis, forward) } else { (opposite, backward) };
        if candidate.1 < best.1 { best = candidate; }
    }
    Some(best)
}

/// Unit quaternion from right-handed orthonormal basis columns.
pub fn orientation_from_axes(axes: [[f64; 3]; 3]) -> [f64; 4] {
    let m = |row: usize, column: usize| axes[column][row];
    let trace = m(0, 0) + m(1, 1) + m(2, 2);
    let q = if trace > 0.0 {
        let s = (trace + 1.0).sqrt() * 2.0;
        [(m(2, 1) - m(1, 2)) / s, (m(0, 2) - m(2, 0)) / s, (m(1, 0) - m(0, 1)) / s, s * 0.25]
    } else {
        let i = (0..3).max_by(|&i, &j| m(i, i).total_cmp(&m(j, j))).expect("three axes");
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

    fn box_hull(half: [f64; 3]) -> Arc<ConvexHull> {
        Arc::new(ConvexHull::new((0..8).map(|i| std::array::from_fn(|axis|
            half[axis] * if i & (1 << axis) == 0 { -1.0 } else { 1.0 }))))
    }
    fn object(position: [f64; 3], half: [f64; 3]) -> ObjectState {
        ObjectState {
            position, velocity: [0.0; 3], radius: length(half), ground_support_meters: half[1],
            surface: Surface::Floor { height_meters: 0.0 }, hull: Some(box_hull(half)),
            side_meters: 1.0, orientation: [0.0, 0.0, 0.0, 1.0], angular_velocity: [0.0; 3], mass_kg: 2.0,
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
        b.orientation = [0.0, 0.0, (std::f64::consts::FRAC_PI_4).sin(), (std::f64::consts::FRAC_PI_4).cos()];
        assert!(contact(&a, &b).unwrap().1 > 0.05);
    }
    #[test]
    fn convex_pile_settles_and_support_removal_reactivates_gravity() {
        let initial = [object([0.0, 0.1, 0.0], [0.3, 0.1, 0.3]),
            object([0.0, 0.3, 0.0], [0.25, 0.1, 0.25]),
            object([0.0, 0.5, 0.0], [0.2, 0.1, 0.2])];
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
        for _ in 0..180 { advance(&mut unsupported, Duration::from_millis(16), |_, _| true); }
        assert!((unsupported[0].position[1] - 0.1).abs() < 0.002);
    }
    #[test]
    fn off_center_ground_contact_tips_and_fast_drop_does_not_cross_floor() {
        let mut p = object([0.0, 0.3, 0.0], [0.3, 0.1, 0.2]);
        p.orientation = [0.0, 0.0, (0.2_f64).sin(), (0.2_f64).cos()];
        let before = p.orientation;
        p.velocity = [0.0, -8.0, 0.0];
        let mut pieces = [p];
        for _ in 0..100 { advance(&mut pieces, Duration::from_millis(16), |_, _| true); }
        assert_ne!(pieces[0].orientation, before);
        let support = pieces[0].hull.as_ref().unwrap().support(pieces[0].orientation, 1.0, [0.0, -1.0, 0.0]);
        assert!(pieces[0].position[1] >= support - 1e-6);
        assert!(pieces[0].orientation.iter().all(|v| v.is_finite()));
    }
    #[test]
    fn ship_basis_orientation_roundtrips_vectors() {
        let axes = [[0.0, 0.0, -1.0], [0.0, 1.0, 0.0], [1.0, 0.0, 0.0]];
        let q = orientation_from_axes(axes);
        for (axis, expected) in axes.iter().enumerate() {
            let mut basis = [0.0; 3]; basis[axis] = 1.0;
            assert!(length(sub(rotate(q, basis), *expected)) < 1e-12);
        }
    }
}

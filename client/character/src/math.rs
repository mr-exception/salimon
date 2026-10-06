//! Character-owned normalization, projection and interpolation policies.
use salimon_math::{add, cross, dot, scale, sub};
pub(crate) fn orthogonal_tangent(normal: [f64; 3]) -> [f64; 3] {
    let reference = if normal[1].abs() < 0.9 {
        [0.0, 1.0, 0.0]
    } else {
        [1.0, 0.0, 0.0]
    };
    normalize(cross(normal, reference))
}

pub(crate) fn normalize(vector: [f64; 3]) -> [f64; 3] {
    normalize_or(vector, [0.0, 1.0, 0.0])
}

pub(crate) fn normalize_or(vector: [f64; 3], fallback: [f64; 3]) -> [f64; 3] {
    let length = dot(vector, vector).sqrt();
    if length > 1.0e-12 {
        scale(vector, length.recip())
    } else {
        fallback
    }
}

pub(crate) fn reject(vector: [f64; 3], normal: [f64; 3]) -> [f64; 3] {
    sub(vector, scale(normal, dot(vector, normal)))
}

pub(crate) fn lerp(from: [f64; 3], to: [f64; 3], amount: f64) -> [f64; 3] {
    add(scale(from, 1.0 - amount), scale(to, amount))
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn normalization_preserves_domain_threshold_and_fallback() {
        let fallback = [1.0, 0.0, 0.0];
        for vector in [[0.0; 3], [1.0e-12, 0.0, 0.0], [f64::NAN, 0.0, 0.0]] {
            assert_eq!(normalize_or(vector, fallback), fallback);
        }
        assert_eq!(normalize([0.0; 3]), [0.0, 1.0, 0.0]);
        assert_eq!(normalize_or([0.0, 2.0e-12, 0.0], fallback), [0.0, 1.0, 0.0]);
        let unit = normalize([3.0, 4.0, 0.0]);
        assert!((dot(unit, unit) - 1.0).abs() < 1.0e-15);
    }
}

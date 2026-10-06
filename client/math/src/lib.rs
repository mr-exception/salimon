//! Portable component arithmetic on `[f64; 3]`, without frame or unit ownership.
//!
//! Callers own finite-input validation, normalization thresholds/fallbacks, units
//! and coordinate frames. These operations preserve ordinary IEEE arithmetic;
//! length uses the existing sum-of-squares order, not overflow-safe rescaling.

/// Component-wise sum. Both operands must use the same frame and units.
#[must_use]
pub fn add(left: [f64; 3], right: [f64; 3]) -> [f64; 3] {
    [left[0] + right[0], left[1] + right[1], left[2] + right[2]]
}

/// Component-wise difference. Subtract absolute positions before any `f32` cast.
#[must_use]
pub fn sub(left: [f64; 3], right: [f64; 3]) -> [f64; 3] {
    [left[0] - right[0], left[1] - right[1], left[2] - right[2]]
}

/// Multiply every component by a scalar.
#[must_use]
pub fn scale(vector: [f64; 3], amount: f64) -> [f64; 3] {
    vector.map(|value| value * amount)
}

/// Sum component products in X, Y, Z order.
#[must_use]
pub fn dot(left: [f64; 3], right: [f64; 3]) -> f64 {
    left.into_iter().zip(right).map(|(a, b)| a * b).sum()
}

/// Right-handed cross product: +X crossed with +Y is +Z.
#[must_use]
pub fn cross(left: [f64; 3], right: [f64; 3]) -> [f64; 3] {
    [
        left[1] * right[2] - left[2] * right[1],
        left[2] * right[0] - left[0] * right[2],
        left[0] * right[1] - left[1] * right[0],
    ]
}

/// Euclidean length using sum of squares. Very large components may overflow.
#[must_use]
pub fn length(vector: [f64; 3]) -> f64 {
    dot(vector, vector).sqrt()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn right_handed_axes_and_cross_order() {
        let x = [1.0, 0.0, 0.0];
        let y = [0.0, 1.0, 0.0];
        let z = [0.0, 0.0, 1.0];
        assert_eq!(cross(x, y), z);
        assert_eq!(cross(y, z), x);
        assert_eq!(cross(z, x), y);
        assert_eq!(cross(y, x), scale(z, -1.0));
        assert_eq!(cross(x, x), [0.0; 3]);
        assert_eq!(dot(x, y), 0.0);
    }

    #[test]
    fn arithmetic_and_length_keep_f64_precision() {
        let origin = [1.0e12, -7.5e11, 2.5e11];
        let offset = [0.125, -0.25, 1.0];
        let position = add(origin, offset);
        assert_eq!(sub(position, origin), offset);
        assert_eq!(sub(position, origin).map(|v| v as f32), [0.125, -0.25, 1.0]);
        assert_eq!(length([3.0, 4.0, 0.0]), 5.0);
        assert_eq!(length([0.0; 3]), 0.0);
        assert_eq!(scale([3.0, 4.0, 0.0], 2.0), [6.0, 8.0, 0.0]);
    }

    #[test]
    fn length_preserves_existing_ieee_edge_behavior() {
        assert!(length([f64::MAX, 0.0, 0.0]).is_infinite());
        assert!(length([f64::NAN, 0.0, 0.0]).is_nan());
        assert!(dot([f64::INFINITY, 0.0, 0.0], [0.0; 3]).is_nan());
    }
}

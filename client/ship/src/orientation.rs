//! Ship-local quaternion and normalization policy.

use salimon_math::{cross, dot, length as vector_length, scale, sub};

pub(crate) fn quaternion_slerp(start: [f64; 4], end: [f64; 4], amount: f64) -> [f64; 4] {
    if amount <= 0.0 {
        return start;
    }
    if amount >= 1.0 {
        return end;
    }
    let start = normalized_quaternion(start);
    let mut end = normalized_quaternion(end);
    let mut cosine: f64 = start.iter().zip(end).map(|(a, b)| a * b).sum();
    // Quaternion signs describe the same orientation; always follow the short arc.
    if cosine < 0.0 {
        end = end.map(|value| -value);
        cosine = -cosine;
    }
    let (start_weight, end_weight) = if cosine > 0.9995 {
        (1.0 - amount, amount)
    } else {
        let angle = cosine.clamp(-1.0, 1.0).acos();
        (
            ((1.0 - amount) * angle).sin() / angle.sin(),
            (amount * angle).sin() / angle.sin(),
        )
    };
    normalized_quaternion(std::array::from_fn(|i| {
        start[i] * start_weight + end[i] * end_weight
    }))
}

pub(crate) fn normalize_or(vector: [f64; 3], fallback: [f64; 3]) -> [f64; 3] {
    let length = vector_length(vector);
    if length > 1.0e-12 {
        scale(vector, length.recip())
    } else {
        fallback
    }
}

pub(crate) fn surface_aligned_orientation(up: [f64; 3], prior_forward: [f64; 3]) -> [f64; 4] {
    let tangent = sub(prior_forward, scale(up, dot(prior_forward, up)));
    let forward = normalize_or(
        tangent,
        normalize_or(cross([0.0, 0.0, 1.0], up), [1.0, 0.0, 0.0]),
    );
    let port = normalize_or(cross(forward, up), [0.0, 0.0, 1.0]);
    quaternion_from_axes(forward, up, port)
}

fn quaternion_from_axes(forward: [f64; 3], up: [f64; 3], port: [f64; 3]) -> [f64; 4] {
    let m00 = forward[0];
    let m01 = up[0];
    let m02 = port[0];
    let m10 = forward[1];
    let m11 = up[1];
    let m12 = port[1];
    let m20 = forward[2];
    let m21 = up[2];
    let m22 = port[2];
    let trace = m00 + m11 + m22;
    let quaternion = if trace > 0.0 {
        let s = (trace + 1.0).sqrt() * 2.0;
        [(m21 - m12) / s, (m02 - m20) / s, (m10 - m01) / s, 0.25 * s]
    } else if m00 > m11 && m00 > m22 {
        let s = (1.0 + m00 - m11 - m22).sqrt() * 2.0;
        [0.25 * s, (m01 + m10) / s, (m02 + m20) / s, (m21 - m12) / s]
    } else if m11 > m22 {
        let s = (1.0 + m11 - m00 - m22).sqrt() * 2.0;
        [(m01 + m10) / s, 0.25 * s, (m12 + m21) / s, (m02 - m20) / s]
    } else {
        let s = (1.0 + m22 - m00 - m11).sqrt() * 2.0;
        [(m02 + m20) / s, (m12 + m21) / s, 0.25 * s, (m10 - m01) / s]
    };
    normalized_quaternion(quaternion)
}

pub(crate) fn axis_angle_quaternion(axis: [f64; 3], angle: f64) -> [f64; 4] {
    let half = angle * 0.5;
    let sine = half.sin();
    [axis[0] * sine, axis[1] * sine, axis[2] * sine, half.cos()]
}

pub(crate) fn quaternion_multiply(left: [f64; 4], right: [f64; 4]) -> [f64; 4] {
    let [lx, ly, lz, lw] = left;
    let [rx, ry, rz, rw] = right;
    [
        lw * rx + lx * rw + ly * rz - lz * ry,
        lw * ry - lx * rz + ly * rw + lz * rx,
        lw * rz + lx * ry - ly * rx + lz * rw,
        lw * rw - lx * rx - ly * ry - lz * rz,
    ]
}

pub(crate) fn normalized_quaternion(quaternion: [f64; 4]) -> [f64; 4] {
    let length = quaternion
        .iter()
        .map(|value| value * value)
        .sum::<f64>()
        .sqrt();
    if length <= 1.0e-12 || !length.is_finite() {
        [0.0, 0.0, 0.0, 1.0]
    } else {
        quaternion.map(|value| value / length)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ShipPose;
    use crate::controller::assist::smoothstep;

    #[test]
    fn local_quaternion_composition_preserves_right_handed_axes() {
        let yaw = axis_angle_quaternion([0.0, 1.0, 0.0], std::f64::consts::FRAC_PI_2);
        let roll = axis_angle_quaternion([1.0, 0.0, 0.0], std::f64::consts::FRAC_PI_2);
        let pose = ShipPose {
            position_meters: [0.0; 3],
            orientation: normalized_quaternion(quaternion_multiply(yaw, roll)),
        };
        let [forward, up, port] = pose.axes();
        for (actual, expected) in [
            (forward, [0.0, 0.0, -1.0]),
            (up, [1.0, 0.0, 0.0]),
            (port, [0.0, -1.0, 0.0]),
        ] {
            assert!(vector_length(sub(actual, expected)) < 1.0e-15);
        }
        assert!(vector_length(sub(cross(forward, up), port)) < 1.0e-15);
        let identity = [0.0, 0.0, 0.0, 1.0];
        for invalid in [
            [0.0; 4],
            [1.0e-12, 0.0, 0.0, 0.0],
            [f64::NAN; 4],
            [f64::INFINITY; 4],
        ] {
            assert_eq!(normalized_quaternion(invalid), identity);
        }
        assert_eq!(normalized_quaternion([0.0, 0.0, 0.0, 2.0]), identity);
        assert_eq!(
            normalize_or([1.0e-12, 0.0, 0.0], [0.0, 1.0, 0.0]),
            [0.0, 1.0, 0.0]
        );
    }

    #[test]
    fn axes_rotate_local_forward_with_pose() {
        let half_turn_about_y = ShipPose {
            position_meters: [0.0; 3],
            orientation: [0.0, 1.0, 0.0, 0.0],
        };
        let axes = half_turn_about_y.axes();
        assert!((axes[0][0] + 1.0).abs() < 1.0e-12);
        assert!(axes[0][1].abs() < 1.0e-12);
        assert!(axes[0][2].abs() < 1.0e-12);
    }

    #[test]
    fn quaternion_alignment_follows_the_short_arc_without_endpoint_snaps() {
        let start = axis_angle_quaternion([0.0, 0.0, 1.0], 179.0_f64.to_radians());
        let end = axis_angle_quaternion([0.0, 0.0, 1.0], -179.0_f64.to_radians());
        let middle = ShipPose {
            position_meters: [0.0; 3],
            orientation: quaternion_slerp(start, end, 0.5),
        };
        assert!(middle.axes()[0][0] < -0.99999);
        assert_eq!(quaternion_slerp(start, end, 0.0), start);
        assert_eq!(quaternion_slerp(start, end, 1.0), end);
        for amount in [0.0, 1.0] {
            let edge = ShipPose {
                position_meters: [0.0; 3],
                orientation: quaternion_slerp(start, end, amount),
            };
            let near = ShipPose {
                position_meters: [0.0; 3],
                orientation: quaternion_slerp(
                    start,
                    end,
                    smoothstep(if amount == 0.0 { 0.0001 } else { 0.9999 }),
                ),
            };
            assert!(vector_length(sub(edge.axes()[0], near.axes()[0])) < 1.0e-8);
        }
    }
}

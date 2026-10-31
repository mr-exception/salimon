//! Backend-independent spatial prototype. No accounts, ranking or unlock authority.
//! A rank is only a deterministic synthetic layout index, never a real inscription.
use std::f64::consts::TAU;

pub const MAX_SIGNS: usize = 1024;
pub const VIEW_METERS: f64 = 128.;
pub const EYE_METERS: f64 = 1.7;
const RING_GAP: f64 = 16.;
const CAPSULE_RADIUS: f64 = 0.3;

pub fn capacity(ring: u32) -> Option<u64> {
    (ring > 0).then(|| 20 * u64::from(ring) + 20)
}
fn end(ring: u32) -> u128 {
    let r = u128::from(ring);
    10 * r * (r + 3)
}
pub fn radius(ring: u32) -> Option<f64> {
    capacity(ring).map(|_| 32. + RING_GAP * f64::from(ring - 1))
}
/// Zero-based rank; integer search preserves exact boundaries even beyond f64 IDs.
pub fn slot(rank: u64) -> (u32, u64) {
    let (mut low, mut high) = (1u32, u32::MAX);
    while low < high {
        let mid = low + (high - low) / 2;
        if end(mid) > u128::from(rank) {
            high = mid;
        } else {
            low = mid + 1;
        }
    }
    (low, (u128::from(rank) - end(low - 1)) as u64)
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Sign {
    pub rank: u64,
    pub ring: u32,
    pub center: [f64; 3],
    /// Rotation around the Hall's +Y axis; the inscribed face points inward.
    pub yaw: f64,
    pub detailed: bool,
}
pub fn sign(rank: u64) -> Sign {
    let (ring, index) = slot(rank);
    let yaw = TAU * index as f64 / capacity(ring).expect("positive ring") as f64;
    let r = radius(ring).expect("positive ring");
    Sign {
        rank,
        ring,
        center: [r * yaw.sin(), 0., r * yaw.cos()],
        yaw,
        detailed: false,
    }
}

/// Enumerate only nearby arcs of nearby rings. Work and retained geometry never
/// scale with the synthetic registry size. Stable rank ordering makes revisits exact.
pub fn visible(eye: [f64; 3], population: u64) -> Result<Vec<Sign>, &'static str> {
    if !eye.iter().all(|v| v.is_finite()) || eye[0].abs().max(eye[2].abs()) > 1e12 {
        return Err("invalid Hall camera");
    }
    let distance = eye[0].hypot(eye[2]);
    let first =
        (((distance - VIEW_METERS - 32.) / RING_GAP).ceil().max(0.) as u32).saturating_add(1);
    let last =
        (((distance + VIEW_METERS - 32.) / RING_GAP).floor().max(0.) as u32).saturating_add(1);
    let mut out = Vec::with_capacity(MAX_SIGNS);
    for ring in first..=last {
        let begin = end(ring - 1);
        if begin >= u128::from(population) {
            break;
        }
        let count = capacity(ring).expect("positive ring");
        let r = radius(ring).expect("positive ring");
        if (distance - r).abs() > VIEW_METERS {
            continue;
        }
        let angle = eye[0].atan2(eye[2]).rem_euclid(TAU);
        let center = (angle / TAU * count as f64).round() as i64;
        let half_angle = if distance == 0. || distance + r <= VIEW_METERS {
            std::f64::consts::PI
        } else {
            2. * ((VIEW_METERS * VIEW_METERS - (distance - r).powi(2)) / (4. * distance * r))
                .clamp(0., 1.)
                .sqrt()
                .asin()
        };
        let span = (half_angle / TAU * count as f64).ceil() as i64 + 1;
        // Each candidate slot appears at most once, including wrap and full circles.
        let n = ((2 * span + 1) as u64).min(count);
        for offset in 0..n {
            let index = (center - span + offset as i64).rem_euclid(count as i64) as u64;
            let rank = begin + u128::from(index);
            if rank >= u128::from(population) {
                continue;
            }
            let mut s = sign(rank as u64);
            let separation = (eye[0] - s.center[0]).hypot(eye[2] - s.center[2]);
            if separation <= VIEW_METERS {
                s.detailed = separation <= 48.;
                out.push(s);
            }
        }
    }
    out.sort_by_key(|s| s.rank);
    if out.len() > MAX_SIGNS {
        return Err("Hall spatial budget exceeded");
    }
    Ok(out)
}

/// Horizontal capsule sweep against the actual plinth envelope, with infinite
/// floor support. Invalid/large steps reject rather than tunnelling past signs.
pub fn walk(eye: [f64; 3], delta: [f64; 2], population: u64) -> Result<[f64; 3], &'static str> {
    if !eye.iter().all(|v| v.is_finite()) {
        return Err("invalid Hall camera");
    }
    if !delta.iter().all(|v| v.is_finite()) || delta[0].hypot(delta[1]) > 1. {
        return Err("Hall walk requires finite steps of at most one metre");
    }
    let candidate = [eye[0] + delta[0], EYE_METERS, eye[2] + delta[1]];
    let nearby = visible(candidate, population)?;
    let start = [eye[0], eye[2]];
    let finish = [candidate[0], candidate[2]];
    let d2 = delta[0] * delta[0] + delta[1] * delta[1];
    let t = if d2 == 0. {
        0.
    } else {
        (-(start[0] * delta[0] + start[1] * delta[1]) / d2).clamp(0., 1.)
    };
    if (start[0] + t * delta[0]).hypot(start[1] + t * delta[1]) <= 10. + CAPSULE_RADIUS {
        return Ok(eye);
    }
    for x in [-4.5, 4.5] {
        if segment_box(
            [start[0] - x, start[1] - 14.],
            [finish[0] - x, finish[1] - 14.],
            [0.8 + CAPSULE_RADIUS; 2],
        ) {
            return Ok(eye);
        }
    }
    for s in &nearby {
        let local = |p: [f64; 2]| {
            let dx = p[0] - s.center[0];
            let dz = p[1] - s.center[2];
            [
                dx * s.yaw.cos() - dz * s.yaw.sin(),
                dx * s.yaw.sin() + dz * s.yaw.cos(),
            ]
        };
        if segment_box(
            local(start),
            local(finish),
            [1. + CAPSULE_RADIUS, 0.6 + CAPSULE_RADIUS],
        ) {
            return Ok(eye);
        }
    }
    Ok(candidate)
}

/// Slab intersection in the sign's local frame. This conservative Minkowski
/// rectangle covers the horizontal capsule at corners and cannot tunnel.
fn segment_box(a: [f64; 2], b: [f64; 2], half: [f64; 2]) -> bool {
    let (mut enter, mut exit) = (0f64, 1f64);
    for axis in 0..2 {
        let delta = b[axis] - a[axis];
        if delta == 0. {
            if a[axis].abs() > half[axis] {
                return false;
            }
        } else {
            let t0 = (-half[axis] - a[axis]) / delta;
            let t1 = (half[axis] - a[axis]) / delta;
            enter = enter.max(t0.min(t1));
            exit = exit.min(t0.max(t1));
            if enter > exit {
                return false;
            }
        }
    }
    true
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn exact_capacity_boundaries_and_extreme_rank_round_trip() {
        assert_eq!(capacity(0), None);
        for (rank, expected) in [
            (0, (1, 0)),
            (39, (1, 39)),
            (40, (2, 0)),
            (99, (2, 59)),
            (100, (3, 0)),
        ] {
            assert_eq!(slot(rank), expected);
        }
        for rank in [u64::MAX, 1_000_000_000, 9_007_199_254_740_993] {
            let (ring, index) = slot(rank);
            assert_eq!(end(ring - 1) + u128::from(index), u128::from(rank));
            assert!(index < capacity(ring).unwrap());
        }
    }
    #[test]
    fn billion_sign_window_is_bounded_unique_stable_and_has_distant_lod() {
        for rank in [0, 39, 40, 1_000_000, 999_999_999, u64::MAX - 1] {
            let anchor = sign(rank).center;
            let eye = [anchor[0], EYE_METERS, anchor[2] + 4.];
            let a = visible(eye, u64::MAX).unwrap();
            assert!(!a.is_empty() && a.len() <= MAX_SIGNS);
            assert_eq!(a, visible(eye, u64::MAX).unwrap());
            assert!(a.windows(2).all(|w| w[0].rank < w[1].rank));
            assert!(a.iter().any(|s| s.detailed));
            assert!(a.iter().any(|s| !s.detailed));
        }
        assert!(visible([0., EYE_METERS, 50.], 0).unwrap().is_empty());
        assert!(visible([f64::NAN, 0., 0.], 100).is_err());
    }
    #[test]
    fn capsule_stops_at_sign_and_core_but_walks_between_rings_without_outer_wall() {
        let mut eye = [0., EYE_METERS, 35.];
        for _ in 0..20 {
            eye = walk(eye, [0., -0.5], 1_000_000_000).unwrap();
        }
        assert!(eye[2] >= 32.9);
        assert!(walk(eye, [0., -2.], 1_000_000_000).is_err());
        assert!(walk(eye, [f64::NAN, 0.], 100).is_err());
        let outside = [0., EYE_METERS, 1e6];
        assert_eq!(walk(outside, [0., 1.], 0).unwrap()[2], 1e6 + 1.);
        assert_eq!(walk([0., EYE_METERS, 10.5], [0., -1.], 0).unwrap()[2], 10.5);
        assert_eq!(walk([0., EYE_METERS, 24.], [1., 0.], 100).unwrap()[0], 1.);
    }
}

//! Conservative continuous sweeps for caller-owned static box publications.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum StaticCollisionError {
    InvalidBounds,
    InvalidSweep,
    TooManyBoxes,
}
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct StaticBox {
    bounds: [f64; 6],
}
impl StaticBox {
    pub fn new(bounds: [f64; 6]) -> Result<Self, StaticCollisionError> {
        if !bounds.iter().all(|v| v.is_finite())
            || !(0..3).all(|i| bounds[2 * i] < bounds[2 * i + 1])
        {
            return Err(StaticCollisionError::InvalidBounds);
        }
        Ok(Self { bounds })
    }
    pub fn bounds(self) -> [f64; 6] {
        self.bounds
    }
}
#[derive(Clone, Debug, Default, PartialEq)]
pub struct StaticField {
    boxes: Vec<StaticBox>,
}
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct StaticHit {
    pub fraction: f64,
    pub normal: [f64; 3],
    pub box_index: usize,
}
impl StaticField {
    pub fn new(boxes: Vec<StaticBox>) -> Result<Self, StaticCollisionError> {
        if boxes.len() > 4096 {
            return Err(StaticCollisionError::TooManyBoxes);
        }
        Ok(Self { boxes })
    }
    pub fn boxes(&self) -> &[StaticBox] {
        &self.boxes
    }
    /// Exact slab interval for an axis-aligned swept box; callers may conservatively
    /// bound capsules/rotated hulls. All values use the same caller-owned metre frame.
    pub fn sweep(
        &self,
        start: [f64; 3],
        end: [f64; 3],
        half: [f64; 3],
    ) -> Result<Option<StaticHit>, StaticCollisionError> {
        if !start.iter().chain(&end).all(|v| v.is_finite())
            || !half.iter().all(|v| v.is_finite() && *v >= 0.)
        {
            return Err(StaticCollisionError::InvalidSweep);
        }
        let movement: [f64; 3] = std::array::from_fn(|i| end[i] - start[i]);
        if !movement.iter().all(|v| v.is_finite()) {
            return Err(StaticCollisionError::InvalidSweep);
        }
        let mut first = None;
        for (index, b) in self.boxes.iter().enumerate() {
            let lo: [f64; 3] = std::array::from_fn(|i| b.bounds[2 * i] - half[i]);
            let hi: [f64; 3] = std::array::from_fn(|i| b.bounds[2 * i + 1] + half[i]);
            if !lo.iter().chain(&hi).all(|v| v.is_finite()) {
                return Err(StaticCollisionError::InvalidSweep);
            }
            let mut enter = f64::NEG_INFINITY;
            let mut exit = f64::INFINITY;
            let mut normal = [0.; 3];
            let mut missed = false;
            for axis in 0..3 {
                let d = movement[axis];
                if d == 0. {
                    if start[axis] < lo[axis] || start[axis] > hi[axis] {
                        missed = true;
                        break;
                    }
                    continue;
                }
                let a = (lo[axis] - start[axis]) / d;
                let b = (hi[axis] - start[axis]) / d;
                let near = a.min(b);
                let far = a.max(b);
                if near > enter {
                    enter = near;
                    normal = [0.; 3];
                    normal[axis] = if d > 0. { -1. } else { 1. };
                }
                exit = exit.min(far);
            }
            if missed || enter > exit || exit < 0. || enter > 1. {
                continue;
            }
            if enter < 0. {
                let mut nearest = f64::INFINITY;
                for axis in 0..3 {
                    for (edge, sign) in [(lo[axis], -1.), (hi[axis], 1.)] {
                        let distance = (start[axis] - edge).abs();
                        if distance < nearest {
                            nearest = distance;
                            normal = [0.; 3];
                            normal[axis] = sign;
                        }
                    }
                }
                if (0..3).map(|i| movement[i] * normal[i]).sum::<f64>() > 0. {
                    continue;
                }
            }
            let hit = StaticHit {
                fraction: enter.max(0.),
                normal,
                box_index: index,
            };
            if first.is_none_or(|old: StaticHit| hit.fraction < old.fraction) {
                first = Some(hit)
            }
        }
        Ok(first)
    }
    pub fn overlaps(&self, center: [f64; 3], half: [f64; 3]) -> Result<bool, StaticCollisionError> {
        Ok(self.sweep(center, center, half)?.is_some())
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn static_sweeps_stop_fast_crossings_and_keep_tangent_retreat_clear() {
        let f = StaticField::new(vec![StaticBox::new([0., 1., 0., 3., 0., 1.]).unwrap()]).unwrap();
        let h = f
            .sweep([-100., 1., 0.5], [100., 1., 0.5], [0.3; 3])
            .unwrap()
            .unwrap();
        assert!((h.fraction - 0.4985).abs() < 1e-12);
        assert_eq!(h.normal, [-1., 0., 0.]);
        assert!(
            f.sweep([-1., 1., 2.], [2., 1., 2.], [0.3; 3])
                .unwrap()
                .is_none()
        );
        assert!(
            f.sweep([0.1, 1., 0.5], [-1., 1., 0.5], [0.; 3])
                .unwrap()
                .is_none()
        );
        assert!(
            f.sweep([0.1, 1., 0.5], [0.2, 1., 0.5], [0.; 3])
                .unwrap()
                .is_some()
        );
    }
    #[test]
    fn static_sweeps_preserve_large_translated_frames_and_reject_invalid_input() {
        let make = |t: f64| {
            StaticField::new(vec![
                StaticBox::new([t, t + 1., t, t + 3., t, t + 1.]).unwrap(),
            ])
            .unwrap()
        };
        let a = make(0.)
            .sweep([-100., 1., 0.5], [100., 1., 0.5], [0.5; 3])
            .unwrap()
            .unwrap();
        let b = make(1e12)
            .sweep(
                [1e12 - 100., 1e12 + 1., 1e12 + 0.5],
                [1e12 + 100., 1e12 + 1., 1e12 + 0.5],
                [0.5; 3],
            )
            .unwrap()
            .unwrap();
        assert_eq!(a, b);
        assert!(StaticBox::new([0., f64::NAN, 0., 1., 0., 1.]).is_err());
        assert!(make(0.).sweep([0.; 3], [1.; 3], [-1.; 3]).is_err());
    }
}

//! Reversible, persistable manual mechanism travel; occupancy is caller supplied.
use salimon_persistence::{SaveError, Validate, invalid};
use std::time::Duration;
#[derive(Clone, Copy, Debug, Default, PartialEq, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Mechanism {
    pub fraction: f64,
    pub opening: bool,
}
impl Validate for Mechanism {
    fn validate(&self) -> Result<(), SaveError> {
        if !self.fraction.is_finite() || !(0.0..=1.0).contains(&self.fraction) {
            return Err(invalid("mechanism travel"));
        }
        Ok(())
    }
}
impl Mechanism {
    pub fn open() -> Self {
        Self {
            fraction: 1.,
            opening: true,
        }
    }
    pub fn reverse(&mut self) {
        self.opening = !self.opening;
    }
    pub fn next(self, delta: Duration, duration: Duration) -> Self {
        let step = delta.as_secs_f64() / duration.as_secs_f64().max(0.001);
        Self {
            fraction: if self.opening {
                (self.fraction + step).min(1.)
            } else {
                (self.fraction - step).max(0.)
            },
            ..self
        }
    }
    pub fn eased(self) -> f64 {
        self.fraction * self.fraction * (3. - 2. * self.fraction)
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn reversed_motion_and_saved_intermediate_state_retain_endpoints() {
        let mut m = Mechanism::default();
        m.reverse();
        m = m.next(Duration::from_millis(320), Duration::from_millis(800));
        assert!((m.fraction - 0.4).abs() < 1e-12);
        let mut r: Mechanism = serde_json::from_slice(&serde_json::to_vec(&m).unwrap()).unwrap();
        r.reverse();
        r = r.next(Duration::from_millis(320), Duration::from_millis(800));
        assert_eq!(r.fraction, 0.);
        r.validate().unwrap();
    }
}

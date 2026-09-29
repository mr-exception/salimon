use std::time::{Duration, Instant};

// A hidden/blocked native surface must not become a large domain update step.
const MAX_UPDATE_STEP: Duration = Duration::from_millis(250);

/// Monotonic delta source for the runtime's pre-render prototype update.
#[derive(Debug, Default)]
pub(crate) struct UpdateClock {
    last_update_at: Option<Instant>,
    fixed_step: Option<Duration>,
}

impl UpdateClock {
    pub(crate) fn step(&mut self, now: Instant) -> Duration {
        let elapsed = self.last_update_at
            .replace(now)
            .map_or(Duration::ZERO, |previous| {
                now.saturating_duration_since(previous).min(MAX_UPDATE_STEP)
            });
        self.fixed_step.unwrap_or(elapsed)
    }

    pub(crate) fn set_fixed_step(&mut self, step: Duration) {
        self.fixed_step = Some(step);
    }

    /// Excludes lifecycle gaps from the next portable-domain update.
    pub(crate) fn reset(&mut self) {
        self.last_update_at = None;
    }
}

#[cfg(test)]
mod tests {
    use super::UpdateClock;
    use std::time::{Duration, Instant};

    #[test]
    fn first_step_is_zero_and_following_steps_are_monotonic() {
        let origin = Instant::now();
        let mut clock = UpdateClock::default();

        assert_eq!(clock.step(origin), Duration::ZERO);
        assert_eq!(
            clock.step(origin + Duration::from_millis(16)),
            Duration::from_millis(16)
        );
    }

    #[test]
    fn reset_excludes_a_lifecycle_gap() {
        let origin = Instant::now();
        let mut clock = UpdateClock::default();
        clock.step(origin);
        clock.reset();

        assert_eq!(clock.step(origin + Duration::from_secs(30)), Duration::ZERO);
    }

    #[test]
    fn clamps_an_unreported_stall() {
        let origin = Instant::now();
        let mut clock = UpdateClock::default();
        clock.step(origin);

        assert_eq!(
            clock.step(origin + Duration::from_secs(2)),
            Duration::from_millis(250)
        );
    }

    #[test]
    fn fixed_step_survives_lifecycle_reset() {
        let origin = Instant::now();
        let mut clock = UpdateClock::default();
        clock.set_fixed_step(Duration::from_millis(16));
        assert_eq!(clock.step(origin), Duration::from_millis(16));
        clock.reset();
        assert_eq!(clock.step(origin + Duration::from_secs(5)), Duration::from_millis(16));
    }
}

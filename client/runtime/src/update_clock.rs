use std::time::{Duration, Instant};

pub(crate) const GAME_TICK: Duration = Duration::from_millis(16);
pub(crate) const MAX_TICKS_PER_RENDER: usize = 2;
pub(crate) const CATCH_UP_BUDGET: Duration = Duration::from_millis(4);

/// Monotonic delta source for the runtime's pre-render prototype update.
#[derive(Debug, Default)]
pub(crate) struct UpdateClock {
    last_update_at: Option<Instant>,
    fixed_step: Option<Duration>,
    backlog: Duration,
    lifecycle_excluded: Duration,
}

impl UpdateClock {
    pub(crate) fn step(&mut self, now: Instant) -> Duration {
        let elapsed = self
            .last_update_at
            .replace(now)
            .map_or(Duration::ZERO, |previous| {
                now.saturating_duration_since(previous)
            });
        self.fixed_step.unwrap_or(elapsed)
    }

    pub(crate) fn accumulate(&mut self, elapsed: Duration) {
        self.backlog = self.backlog.saturating_add(elapsed);
    }

    /// The caller enforces both the per-render count and wall budget. Neither
    /// overload nor an OS stall enlarges a tick or discards elapsed game time.
    pub(crate) fn take_tick(&mut self) -> Option<Duration> {
        if self.backlog < GAME_TICK {
            return None;
        }
        self.backlog -= GAME_TICK;
        Some(GAME_TICK)
    }

    pub(crate) fn backlog(&self) -> Duration {
        self.backlog
    }

    pub(crate) fn lifecycle_excluded(&self) -> Duration {
        self.lifecycle_excluded
    }

    pub(crate) fn interpolation(&self) -> f64 {
        // On overload render the latest pose, rather than a backwards sawtooth
        // from the fractional remainder of an arbitrarily large backlog.
        (self.backlog.as_secs_f64() / GAME_TICK.as_secs_f64()).min(1.0)
    }

    pub(crate) fn set_fixed_step(&mut self, step: Duration) {
        self.fixed_step = Some(step);
    }

    /// Excludes lifecycle gaps from the next portable-domain update.
    pub(crate) fn reset(&mut self) {
        self.last_update_at = None;
        self.lifecycle_excluded = self.lifecycle_excluded.saturating_add(self.backlog);
        self.backlog = Duration::ZERO;
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
    fn retains_an_unreported_stall() {
        let origin = Instant::now();
        let mut clock = UpdateClock::default();
        clock.step(origin);

        assert_eq!(
            clock.step(origin + Duration::from_secs(2)),
            Duration::from_secs(2)
        );
    }

    #[test]
    fn bounded_catch_up_keeps_exact_time_and_drains_without_large_ticks() {
        let mut clock = UpdateClock::default();
        clock.accumulate(Duration::from_secs(2));
        let mut advanced = Duration::ZERO;
        for _ in 0..super::MAX_TICKS_PER_RENDER {
            advanced += clock.take_tick().unwrap();
        }
        assert_eq!(advanced, Duration::from_millis(32));
        assert_eq!(clock.backlog(), Duration::from_millis(1968));
        assert_eq!(clock.interpolation(), 1.0);
        while let Some(tick) = clock.take_tick() {
            assert_eq!(tick, super::GAME_TICK);
            advanced += tick;
        }
        assert_eq!(advanced + clock.backlog(), Duration::from_secs(2));
        clock.accumulate(Duration::from_millis(5));
        assert_eq!(clock.interpolation(), 5.0 / 16.0);
        clock.reset();
        assert_eq!(clock.backlog(), Duration::ZERO);
        assert_eq!(clock.lifecycle_excluded(), Duration::from_millis(5));
    }

    #[test]
    fn fixed_step_survives_lifecycle_reset() {
        let origin = Instant::now();
        let mut clock = UpdateClock::default();
        clock.set_fixed_step(Duration::from_millis(16));
        assert_eq!(clock.step(origin), Duration::from_millis(16));
        clock.reset();
        assert_eq!(
            clock.step(origin + Duration::from_secs(5)),
            Duration::from_millis(16)
        );
    }
}

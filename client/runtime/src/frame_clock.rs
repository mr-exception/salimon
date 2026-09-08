use std::time::{Duration, Instant};

// Keep an unreported OS sleep or debugger pause from becoming a simulation-sized step.
const MAX_CONTIGUOUS_FRAME_GAP: Duration = Duration::from_millis(250);

/// Timing captured for one successfully presented frame.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct FrameTiming {
    pub(crate) frame_number: u64,
    pub(crate) frame_interval: Duration,
    pub(crate) submission_wall_time: Duration,
}

/// Monotonic frame counter owned by runtime orchestration, not the renderer.
#[derive(Debug, Default)]
pub(crate) struct FrameClock {
    presented_frames: u64,
    last_presented_at: Option<Instant>,
}

impl FrameClock {
    pub(crate) fn record_presented(
        &mut self,
        render_started_at: Instant,
        presented_at: Instant,
    ) -> FrameTiming {
        self.presented_frames = self.presented_frames.saturating_add(1);
        let frame_interval = self
            .last_presented_at
            .replace(presented_at)
            .map_or(Duration::ZERO, |previous| {
                presented_at.saturating_duration_since(previous)
            })
            .min(MAX_CONTIGUOUS_FRAME_GAP);

        FrameTiming {
            frame_number: self.presented_frames,
            frame_interval,
            submission_wall_time: presented_at.saturating_duration_since(render_started_at),
        }
    }

    /// Prevents a suspended or occluded interval from becoming one huge frame.
    pub(crate) fn reset_interval(&mut self) {
        self.last_presented_at = None;
    }
}

#[cfg(test)]
mod tests {
    use super::FrameClock;
    use std::time::{Duration, Instant};

    #[test]
    fn records_monotonic_frame_interval_and_submission_time() {
        let origin = Instant::now();
        let mut clock = FrameClock::default();

        let first = clock.record_presented(origin, origin + Duration::from_millis(4));
        let second = clock.record_presented(
            origin + Duration::from_millis(16),
            origin + Duration::from_millis(20),
        );

        assert_eq!(first.frame_number, 1);
        assert_eq!(first.frame_interval, Duration::ZERO);
        assert_eq!(first.submission_wall_time, Duration::from_millis(4));
        assert_eq!(second.frame_number, 2);
        assert_eq!(second.frame_interval, Duration::from_millis(16));
        assert_eq!(second.submission_wall_time, Duration::from_millis(4));
    }

    #[test]
    fn reset_excludes_suspension_from_next_frame_interval() {
        let origin = Instant::now();
        let mut clock = FrameClock::default();
        clock.record_presented(origin, origin + Duration::from_millis(3));

        clock.reset_interval();
        let resumed = clock.record_presented(
            origin + Duration::from_secs(30),
            origin + Duration::from_secs(30) + Duration::from_millis(2),
        );

        assert_eq!(resumed.frame_number, 2);
        assert_eq!(resumed.frame_interval, Duration::ZERO);
    }

    #[test]
    fn clamps_unreported_os_stalls() {
        let origin = Instant::now();
        let mut clock = FrameClock::default();
        clock.record_presented(origin, origin + Duration::from_millis(2));

        let after_stall = clock.record_presented(
            origin + Duration::from_secs(30),
            origin + Duration::from_secs(30) + Duration::from_millis(2),
        );

        assert_eq!(after_stall.frame_interval, Duration::from_millis(250));
    }
}

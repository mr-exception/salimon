//! Bounded observations, validated owned domain snapshots, and timing statistics.

use std::collections::VecDeque;
use std::time::Duration;

use crate::{DomainMetrics, FrameSample};

const FRAME_WINDOW_CAPACITY: usize = 120;
const OVERLAY_REFRESH_INTERVAL: Duration = Duration::from_millis(250);

#[derive(Clone, Debug, Default, PartialEq)]
pub(super) struct OwnedDomainMetrics {
    pub(super) camera_position: Option<[f64; 3]>,
    pub(super) camera_altitude_meters: Option<f64>,
    pub(super) camera_phase: Option<String>,
    pub(super) camera_paused: Option<bool>,
    pub(super) player_position: Option<[f64; 3]>,
    pub(super) ship_position: Option<[f64; 3]>,
    pub(super) ship_velocity: Option<[f64; 3]>,
    pub(super) ship_speed_mps: Option<f64>,
    pub(super) thruster_percent: Option<u8>,
    pub(super) nearby_bodies: Vec<OwnedBodyDistance>,
}

#[derive(Clone, Debug, PartialEq)]
pub(super) struct OwnedBodyDistance {
    pub(super) name: String,
    pub(super) distance_meters: f64,
}

impl From<DomainMetrics<'_>> for OwnedDomainMetrics {
    fn from(metrics: DomainMetrics<'_>) -> Self {
        Self {
            camera_position: valid_vector(metrics.camera_position),
            camera_altitude_meters: valid_distance(metrics.camera_altitude_meters),
            camera_phase: metrics
                .camera_phase
                .map(str::trim)
                .filter(|phase| !phase.is_empty())
                .map(str::to_owned),
            camera_paused: metrics.camera_paused,
            player_position: valid_vector(metrics.player_position),
            ship_position: valid_vector(metrics.ship_position),
            ship_velocity: valid_vector(metrics.ship_velocity),
            ship_speed_mps: valid_speed(metrics.ship_speed_mps),
            thruster_percent: metrics.thruster_percent.filter(|percent| *percent <= 100),
            nearby_bodies: metrics
                .nearby_bodies
                .iter()
                .filter(|body| body.distance_meters.is_finite() && body.distance_meters >= 0.0)
                .map(|body| OwnedBodyDistance {
                    name: body.name.to_owned(),
                    distance_meters: body.distance_meters,
                })
                .collect(),
        }
    }
}

fn valid_vector(value: Option<[f64; 3]>) -> Option<[f64; 3]> {
    value.filter(|components| components.iter().all(|component| component.is_finite()))
}

fn valid_speed(value: Option<f64>) -> Option<f64> {
    value.filter(|speed| speed.is_finite() && *speed >= 0.0)
}

fn valid_distance(value: Option<f64>) -> Option<f64> {
    value.filter(|distance| distance.is_finite() && *distance >= 0.0)
}

#[derive(Debug)]
pub(super) struct Aggregation {
    pub(super) frame_samples: VecDeque<FrameSample>,
    pub(super) latest_sample: Option<FrameSample>,
    pub(super) domain_metrics: OwnedDomainMetrics,
    pub(super) memory_warning_count: u64,
    elapsed_since_refresh: Duration,
}

impl Default for Aggregation {
    fn default() -> Self {
        Self {
            frame_samples: VecDeque::with_capacity(FRAME_WINDOW_CAPACITY),
            latest_sample: None,
            domain_metrics: OwnedDomainMetrics::default(),
            memory_warning_count: 0,
            elapsed_since_refresh: Duration::ZERO,
        }
    }
}

impl Aggregation {
    /// Returns whether the snapshot cadence permits a visible rebuild.
    pub(super) fn record_presented(
        &mut self,
        sample: FrameSample,
        domain_metrics: DomainMetrics<'_>,
    ) -> bool {
        self.latest_sample = Some(sample);

        let first_valid_sample = !sample.frame_interval.is_zero() && self.frame_samples.is_empty();
        if !sample.frame_interval.is_zero() {
            if self.frame_samples.len() == FRAME_WINDOW_CAPACITY {
                self.frame_samples.pop_front();
            }
            self.frame_samples.push_back(sample);
            self.elapsed_since_refresh = self
                .elapsed_since_refresh
                .saturating_add(sample.frame_interval);
        }

        if first_valid_sample || self.elapsed_since_refresh >= OVERLAY_REFRESH_INTERVAL {
            self.elapsed_since_refresh = Duration::ZERO;
            let snapshot = OwnedDomainMetrics::from(domain_metrics);
            if self.domain_metrics != snapshot {
                self.domain_metrics = snapshot;
            }
            return true;
        }
        false
    }

    pub(super) fn reset_refresh_cadence(&mut self) {
        self.elapsed_since_refresh = Duration::ZERO;
    }

    pub(super) fn reset_frame_window(&mut self) {
        self.frame_samples.clear();
        self.latest_sample = None;
        self.reset_refresh_cadence();
    }

    pub(super) fn record_memory_warning(&mut self) {
        self.memory_warning_count = self.memory_warning_count.saturating_add(1);
    }
}

pub(super) fn average_milliseconds(durations: &[Duration]) -> f64 {
    durations
        .iter()
        .map(|duration| duration.as_secs_f64() * 1_000.0)
        .sum::<f64>()
        / durations.len() as f64
}

pub(super) fn percentile_95_milliseconds(durations: &[Duration]) -> f64 {
    let mut milliseconds: Vec<f64> = durations
        .iter()
        .map(|duration| duration.as_secs_f64() * 1_000.0)
        .collect();
    milliseconds.sort_by(f64::total_cmp);
    let index = ((milliseconds.len() as f64 * 0.95).ceil() as usize)
        .saturating_sub(1)
        .min(milliseconds.len() - 1);
    milliseconds[index]
}

#[cfg(test)]
mod tests {
    use super::{
        Aggregation, FRAME_WINDOW_CAPACITY, average_milliseconds, percentile_95_milliseconds,
    };
    use crate::test_support::sample;
    use crate::{DomainMetrics, GpuTime};
    use std::time::Duration;

    #[test]
    fn window_evicts_oldest_valid_samples_and_zero_only_updates_latest() {
        let mut aggregation = Aggregation::default();
        for frame_number in 1..=125 {
            aggregation.record_presented(
                sample(
                    frame_number,
                    Duration::from_millis(frame_number),
                    GpuTime::Pending,
                ),
                DomainMetrics::default(),
            );
        }
        aggregation.record_presented(
            sample(126, Duration::ZERO, GpuTime::Unsupported),
            DomainMetrics::default(),
        );
        assert_eq!(aggregation.frame_samples.len(), FRAME_WINDOW_CAPACITY);
        assert_eq!(aggregation.frame_samples.front().unwrap().frame_number, 6);
        assert_eq!(aggregation.frame_samples.back().unwrap().frame_number, 125);
        assert_eq!(aggregation.latest_sample.unwrap().frame_number, 126);
    }

    #[test]
    fn timing_statistics_use_average_and_nearest_rank_p95() {
        let durations: Vec<_> = (1..=20).rev().map(Duration::from_millis).collect();
        assert_eq!(average_milliseconds(&durations), 10.5);
        assert_eq!(percentile_95_milliseconds(&durations), 19.0);
        assert_eq!(percentile_95_milliseconds(&[Duration::ZERO]), 0.0);
    }

    #[test]
    fn refresh_threshold_copies_domain_snapshot_and_reset_retains_it() {
        let mut aggregation = Aggregation::default();
        let domain = |speed| DomainMetrics {
            ship_speed_mps: Some(speed),
            ..DomainMetrics::default()
        };
        assert!(aggregation.record_presented(
            sample(1, Duration::from_millis(1), GpuTime::Pending),
            domain(1.0)
        ));
        assert!(
            !aggregation.record_presented(sample(2, Duration::ZERO, GpuTime::Pending), domain(2.0))
        );
        assert!(!aggregation.record_presented(
            sample(3, Duration::from_millis(249), GpuTime::Pending),
            domain(3.0)
        ));
        assert_eq!(aggregation.domain_metrics.ship_speed_mps, Some(1.0));
        assert!(aggregation.record_presented(
            sample(4, Duration::from_millis(1), GpuTime::Pending),
            domain(4.0)
        ));
        aggregation.reset_frame_window();
        assert!(aggregation.frame_samples.is_empty());
        assert!(aggregation.latest_sample.is_none());
        assert_eq!(aggregation.domain_metrics.ship_speed_mps, Some(4.0));
        assert!(aggregation.record_presented(
            sample(5, Duration::from_millis(1), GpuTime::Pending),
            domain(5.0)
        ));
    }
}

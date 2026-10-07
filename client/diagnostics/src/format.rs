//! Diagnostic labels, availability states, and display units.

use std::fmt::Write as _;
use std::time::Duration;

use crate::GpuTime;
use crate::aggregation::{Aggregation, average_milliseconds, percentile_95_milliseconds};

impl Aggregation {
    pub(super) fn format_overlay_text(&self) -> String {
        let mut text = String::new();
        let _ = writeln!(text, "SALIMON ENGINEERING DIAGNOSTICS");
        let _ = writeln!(text, "F3: HIDE");

        if self.frame_samples.is_empty() {
            let _ = writeln!(text, "FPS              WARMING UP");
            let _ = writeln!(text, "FRAME TIME       WARMING UP");
            let _ = writeln!(text, "CPU RENDER       WARMING UP");
        } else {
            let frame_times: Vec<Duration> = self
                .frame_samples
                .iter()
                .map(|sample| sample.frame_interval)
                .collect();
            let cpu_times: Vec<Duration> = self
                .frame_samples
                .iter()
                .map(|sample| sample.cpu_render_time)
                .collect();
            let average_frame_ms = average_milliseconds(&frame_times);
            let fps = if average_frame_ms > 0.0 {
                1_000.0 / average_frame_ms
            } else {
                0.0
            };
            let _ = writeln!(text, "FPS              {fps:.1}");
            write_timing_summary(&mut text, "FRAME TIME", &frame_times);
            write_timing_summary(&mut text, "CPU RENDER", &cpu_times);
        }

        let cpu_update_times: Vec<Duration> = self
            .frame_samples
            .iter()
            .filter_map(|sample| sample.cpu_update_time)
            .collect();
        if cpu_update_times.is_empty() {
            let _ = writeln!(text, "CPU UPDATE       N/A (NO UPDATE LOOP)");
        } else {
            write_timing_summary(&mut text, "CPU UPDATE", &cpu_update_times);
        }

        self.write_gpu_time(&mut text);
        if let Some(sample) = self.latest_sample {
            let _ = writeln!(
                text,
                "OBJECTS          {} VISIBLE / {} RENDERED",
                sample.visible_objects, sample.rendered_objects
            );
            let _ = writeln!(
                text,
                "DRAW CALLS       {} SCENE / {} TOTAL",
                sample.scene_draw_calls, sample.total_draw_calls
            );
            if let Some(memory) = sample.gpu_memory {
                let _ = writeln!(
                    text,
                    "GPU MEMORY       {} ALLOC / {} RESERVED",
                    format_bytes(memory.allocated_bytes),
                    format_bytes(memory.reserved_bytes)
                );
            } else {
                let _ = writeln!(text, "GPU MEMORY       N/A (BACKEND REPORT)");
            }
        } else {
            let _ = writeln!(text, "OBJECTS          WARMING UP");
            let _ = writeln!(text, "DRAW CALLS       WARMING UP");
            let _ = writeln!(text, "GPU MEMORY       WARMING UP");
        }
        let _ = writeln!(text, "MEM WARNINGS     {}", self.memory_warning_count);

        let _ = writeln!(
            text,
            "CAMERA POSITION  {}",
            self.domain_metrics
                .camera_position
                .map(format_position)
                .unwrap_or_else(|| "N/A".to_owned())
        );
        let _ = writeln!(
            text,
            "CAMERA ALTITUDE  {}",
            self.domain_metrics
                .camera_altitude_meters
                .map(format_distance)
                .unwrap_or_else(|| "N/A".to_owned())
        );
        let camera_phase = self
            .domain_metrics
            .camera_phase
            .as_deref()
            .map(str::to_ascii_uppercase)
            .unwrap_or_else(|| "N/A".to_owned());
        let camera_motion = match self.domain_metrics.camera_paused {
            Some(true) => "PAUSED",
            Some(false) => "RUNNING",
            None => "N/A",
        };
        let _ = writeln!(text, "CAMERA TRANSITION {camera_phase} / {camera_motion}");

        let _ = writeln!(
            text,
            "PLAYER POSITION  {}",
            self.domain_metrics
                .player_position
                .filter(|position| position.iter().all(|value| value.is_finite()))
                .map(format_position)
                .unwrap_or_else(|| "N/A (TASK 8+)".to_owned())
        );
        let _ = writeln!(
            text,
            "SHIP POSITION    {}",
            self.domain_metrics
                .ship_position
                .filter(|position| position.iter().all(|value| value.is_finite()))
                .map(format_position)
                .unwrap_or_else(|| "N/A (TASK 9+)".to_owned())
        );
        let _ = writeln!(
            text,
            "SHIP VELOCITY    {}",
            self.domain_metrics
                .ship_velocity
                .filter(|velocity| velocity.iter().all(|value| value.is_finite()))
                .map(format_velocity)
                .unwrap_or_else(|| "N/A (TASK 9+)".to_owned())
        );
        let _ = writeln!(
            text,
            "SHIP SPEED       {}",
            self.domain_metrics
                .ship_speed_mps
                .filter(|speed| speed.is_finite() && *speed >= 0.0)
                .map(format_speed)
                .unwrap_or_else(|| "N/A (TASK 9+)".to_owned())
        );
        let _ = writeln!(
            text,
            "THRUSTER         {}",
            self.domain_metrics
                .thruster_percent
                .filter(|percent| *percent <= 100)
                .map(|percent| format!("{percent}%"))
                .unwrap_or_else(|| "N/A (TASK 9+)".to_owned())
        );
        let nearby = self
            .domain_metrics
            .nearby_bodies
            .iter()
            .filter(|body| body.distance_meters.is_finite() && body.distance_meters >= 0.0)
            .min_by(|left, right| left.distance_meters.total_cmp(&right.distance_meters))
            .map(|body| {
                format!(
                    "{}  {}",
                    body.name.to_ascii_uppercase(),
                    format_distance(body.distance_meters)
                )
            })
            .unwrap_or_else(|| "N/A (NO BODY DATA)".to_owned());
        let _ = writeln!(text, "NEARBY BODY      {nearby}");

        text.pop();
        text
    }

    fn write_gpu_time(&self, text: &mut String) {
        if matches!(
            self.latest_sample.map(|sample| sample.gpu_time),
            Some(GpuTime::Unsupported)
        ) {
            let _ = writeln!(text, "GPU FRAME        UNSUPPORTED");
            return;
        }

        let gpu_times: Vec<Duration> = self
            .frame_samples
            .iter()
            .filter_map(|sample| match sample.gpu_time {
                GpuTime::Measured(duration) => Some(duration),
                GpuTime::Unsupported | GpuTime::Pending => None,
            })
            .collect();
        if gpu_times.is_empty() {
            let state = if self.latest_sample.is_some() {
                "PENDING"
            } else {
                "WARMING UP"
            };
            let _ = writeln!(text, "GPU FRAME        {state}");
        } else {
            write_timing_summary(text, "GPU FRAME", &gpu_times);
        }
    }
}

fn write_timing_summary(text: &mut String, label: &str, durations: &[Duration]) {
    let average = average_milliseconds(durations);
    let p95 = percentile_95_milliseconds(durations);
    let _ = writeln!(text, "{label:<16} AVG {average:>6.2} MS  P95 {p95:>6.2} MS");
}

fn format_position(value: [f64; 3]) -> String {
    format_vector(value, "")
}

fn format_velocity(value: [f64; 3]) -> String {
    format_vector(value, "/s")
}

fn format_vector(value: [f64; 3], suffix: &str) -> String {
    let magnitude = value.iter().copied().map(f64::abs).fold(0.0, f64::max);
    let (divisor, unit) = metric_scale(magnitude);
    format!(
        "[{:.2}, {:.2}, {:.2}] {unit}{suffix}",
        value[0] / divisor,
        value[1] / divisor,
        value[2] / divisor
    )
}

fn format_speed(meters_per_second: f64) -> String {
    let (divisor, unit) = metric_scale(meters_per_second.abs());
    format!("{:.2} {unit}/s", meters_per_second / divisor)
}

fn format_distance(meters: f64) -> String {
    let (divisor, unit) = metric_scale(meters.abs());
    format!("{:.2} {unit}", meters / divisor)
}

fn metric_scale(magnitude: f64) -> (f64, &'static str) {
    if magnitude >= 1_000_000_000_000.0 {
        (1_000_000_000_000.0, "Tm")
    } else if magnitude >= 1_000_000_000.0 {
        (1_000_000_000.0, "Gm")
    } else if magnitude >= 1_000_000.0 {
        (1_000_000.0, "Mm")
    } else if magnitude >= 1_000.0 {
        (1_000.0, "km")
    } else {
        (1.0, "m")
    }
}

fn format_bytes(bytes: u64) -> String {
    const KIB: f64 = 1_024.0;
    const MIB: f64 = KIB * 1_024.0;
    const GIB: f64 = MIB * 1_024.0;
    let bytes = bytes as f64;
    if bytes >= GIB {
        format!("{:.1} GIB", bytes / GIB)
    } else if bytes >= MIB {
        format!("{:.1} MIB", bytes / MIB)
    } else if bytes >= KIB {
        format!("{:.1} KIB", bytes / KIB)
    } else {
        format!("{bytes:.0} B")
    }
}

#[cfg(test)]
mod tests {
    use crate::test_support::{refresh_with, sample};
    use crate::{BodyDistance, Diagnostics, DomainMetrics, GpuTime};
    use std::time::Duration;

    #[test]
    fn aggregates_sixty_hertz_and_ignores_initial_zero_interval() {
        let mut diagnostics = Diagnostics::default();
        diagnostics.toggle();
        diagnostics.record_presented(
            sample(1, Duration::ZERO, GpuTime::Pending),
            DomainMetrics::default(),
        );
        refresh_with(
            &mut diagnostics,
            GpuTime::Measured(Duration::from_micros(800)),
            DomainMetrics::default(),
        );

        let text = diagnostics.overlay_text().expect("overlay is visible");
        assert!(text.contains("FPS              60.0"));
        assert!(text.contains("FRAME TIME       AVG  16.67 MS"));
        assert!(text.contains("CPU RENDER       AVG   0.50 MS"));
        assert!(text.contains("GPU FRAME        AVG   0.80 MS"));
    }

    #[test]
    fn reports_gpu_capability_states_without_faking_a_number() {
        let mut diagnostics = Diagnostics::default();
        diagnostics.record_presented(
            sample(1, Duration::ZERO, GpuTime::Unsupported),
            DomainMetrics::default(),
        );
        diagnostics.toggle();
        assert!(
            diagnostics
                .overlay_text()
                .expect("overlay is visible")
                .contains("GPU FRAME        UNSUPPORTED")
        );

        diagnostics.record_presented(
            sample(2, Duration::ZERO, GpuTime::Pending),
            DomainMetrics::default(),
        );
        diagnostics.toggle();
        diagnostics.toggle();
        assert!(
            diagnostics
                .overlay_text()
                .expect("overlay is visible")
                .contains("GPU FRAME        PENDING")
        );
    }

    #[test]
    fn distinguishes_unavailable_and_populated_domain_metrics() {
        let mut diagnostics = Diagnostics::default();
        diagnostics.toggle();
        assert!(
            diagnostics
                .overlay_text()
                .expect("overlay is visible")
                .contains("SHIP SPEED       N/A (TASK 9+)")
        );

        let bodies = [BodyDistance {
            name: "Mars",
            distance_meters: 2_500_000.0,
        }];
        diagnostics.record_presented(
            sample(1, Duration::from_millis(16), GpuTime::Pending),
            DomainMetrics {
                camera_position: Some([1_000_000_000_000.0, 0.0, 12.0]),
                camera_altitude_meters: Some(12.0),
                camera_phase: Some("approach"),
                camera_paused: Some(false),
                player_position: Some([1_000.0, 2_000.0, 3_000.0]),
                ship_position: Some([1_000_000.0, 0.0, -2_000_000.0]),
                ship_velocity: Some([0.0, 1_500.0, 0.0]),
                ship_speed_mps: Some(2_500_000.0),
                thruster_percent: Some(73),
                nearby_bodies: &bodies,
            },
        );

        let text = diagnostics.overlay_text().expect("overlay is visible");
        assert!(text.contains("PLAYER POSITION  [1.00, 2.00, 3.00] km"));
        assert!(text.contains("CAMERA POSITION  [1.00, 0.00, 0.00] Tm"));
        assert!(text.contains("CAMERA ALTITUDE  12.00 m"));
        assert!(text.contains("CAMERA TRANSITION APPROACH / RUNNING"));
        assert!(text.contains("SHIP SPEED       2.50 Mm/s"));
        assert!(text.contains("THRUSTER         73%"));
        assert!(text.contains("NEARBY BODY      MARS  2.50 Mm"));
    }

    #[test]
    fn invalid_domain_values_remain_unavailable() {
        let mut diagnostics = Diagnostics::default();
        diagnostics.toggle();
        let bodies = [BodyDistance {
            name: "invalid",
            distance_meters: -1.0,
        }];
        diagnostics.record_presented(
            sample(1, Duration::from_millis(16), GpuTime::Pending),
            DomainMetrics {
                camera_position: Some([f64::NAN, 0.0, 0.0]),
                camera_altitude_meters: Some(f64::NEG_INFINITY),
                camera_phase: Some("  "),
                camera_paused: None,
                player_position: Some([f64::NAN, 0.0, 0.0]),
                ship_position: Some([f64::INFINITY, 0.0, 0.0]),
                ship_velocity: Some([0.0, f64::NEG_INFINITY, 0.0]),
                ship_speed_mps: Some(-1.0),
                thruster_percent: Some(101),
                nearby_bodies: &bodies,
            },
        );

        let text = diagnostics.overlay_text().expect("overlay is visible");
        assert!(text.contains("CAMERA POSITION  N/A"));
        assert!(text.contains("CAMERA ALTITUDE  N/A"));
        assert!(text.contains("CAMERA TRANSITION N/A / N/A"));
        assert!(text.contains("PLAYER POSITION  N/A (TASK 8+)"));
        assert!(text.contains("SHIP SPEED       N/A (TASK 9+)"));
        assert!(text.contains("THRUSTER         N/A (TASK 9+)"));
        assert!(text.contains("NEARBY BODY      N/A (NO BODY DATA)"));
    }
}

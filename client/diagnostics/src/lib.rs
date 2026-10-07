//! Observational Phase 0 engineering metrics and overlay rasterization.
//!
//! This crate deliberately has no windowing, GPU, or gameplay dependencies.
//! The runtime supplies typed observations and decides when the resulting RGBA
//! image is visible; the renderer only composites those pixels.

mod aggregation;
mod format;
mod raster;

use aggregation::Aggregation;
use raster::rasterize_panel;
use std::time::Duration;

const BASE_GLYPH_SCALE: f64 = 2.0;
const MIN_GLYPH_SCALE: u32 = 2;
const MAX_GLYPH_SCALE: u32 = 6;

/// Availability and value of a completed GPU timestamp measurement.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum GpuTime {
    Unsupported,
    Pending,
    Measured(Duration),
}

/// Optional allocator totals supplied by the renderer.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct GpuMemory {
    pub allocated_bytes: u64,
    pub reserved_bytes: u64,
}

/// Measurements associated with one successfully presented frame.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct FrameSample {
    pub frame_number: u64,
    pub frame_interval: Duration,
    pub cpu_render_time: Duration,
    pub cpu_update_time: Option<Duration>,
    pub gpu_time: GpuTime,
    pub visible_objects: u32,
    pub rendered_objects: u32,
    pub scene_draw_calls: u32,
    pub total_draw_calls: u32,
    pub gpu_memory: Option<GpuMemory>,
}

/// Nonnegative surface distance from the producer's documented reference point
/// to one world body.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct BodyDistance<'a> {
    pub name: &'a str,
    pub distance_meters: f64,
}

/// Optional domain-owned values observed by diagnostics.
///
/// Missing values are rendered as unavailable rather than as synthetic zeroes.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct DomainMetrics<'a> {
    pub camera_position: Option<[f64; 3]>,
    pub camera_altitude_meters: Option<f64>,
    pub camera_phase: Option<&'a str>,
    pub camera_paused: Option<bool>,
    pub player_position: Option<[f64; 3]>,
    pub ship_position: Option<[f64; 3]>,
    pub ship_velocity: Option<[f64; 3]>,
    pub ship_speed_mps: Option<f64>,
    pub thruster_percent: Option<u8>,
    pub nearby_bodies: &'a [BodyDistance<'a>],
}

/// Borrowed RGBA panel ready for a renderer to composite.
#[derive(Clone, Copy, Debug)]
pub struct OverlayImage<'a> {
    pub width: u32,
    pub height: u32,
    pub rgba8: &'a [u8],
    pub revision: u64,
}

/// Rolling diagnostics collector and optional presentation state.
#[derive(Debug)]
pub struct Diagnostics {
    visible: bool,
    aggregation: Aggregation,
    glyph_scale: u32,
    overlay_text: String,
    overlay_pixels: Vec<u8>,
    overlay_width: u32,
    overlay_height: u32,
    overlay_revision: u64,
}

impl Default for Diagnostics {
    fn default() -> Self {
        Self {
            visible: false,
            aggregation: Aggregation::default(),
            glyph_scale: MIN_GLYPH_SCALE,
            overlay_text: String::new(),
            overlay_pixels: Vec::new(),
            overlay_width: 0,
            overlay_height: 0,
            overlay_revision: 0,
        }
    }
}

impl Diagnostics {
    /// Toggles the developer view and returns its new visibility.
    pub fn toggle(&mut self) -> bool {
        self.visible = !self.visible;
        if self.visible {
            self.aggregation.reset_refresh_cadence();
            self.rebuild_overlay();
        }
        self.visible
    }

    #[must_use]
    pub const fn is_visible(&self) -> bool {
        self.visible
    }

    /// Adjusts bitmap density without depending on a platform window type.
    pub fn set_scale_factor(&mut self, scale_factor: f64) {
        let finite_scale = if scale_factor.is_finite() {
            scale_factor
        } else {
            1.0
        };
        let glyph_scale = (finite_scale.clamp(1.0, 3.0) * BASE_GLYPH_SCALE).round() as u32;
        let glyph_scale = glyph_scale.clamp(MIN_GLYPH_SCALE, MAX_GLYPH_SCALE);
        if self.glyph_scale != glyph_scale {
            self.glyph_scale = glyph_scale;
            if self.visible {
                self.rebuild_overlay();
            }
        }
    }

    /// Records one successful present and refreshes visible pixels at most four
    /// times per second under ordinary frame pacing.
    pub fn record_presented(&mut self, sample: FrameSample, domain_metrics: DomainMetrics<'_>) {
        if self.aggregation.record_presented(sample, domain_metrics) && self.visible {
            self.rebuild_overlay();
        }
    }

    /// Drops timing history across lifecycle discontinuities such as minimize
    /// and resume, preventing a pause from polluting the FPS window.
    pub fn reset_frame_window(&mut self) {
        self.aggregation.reset_frame_window();
        if self.visible {
            self.rebuild_overlay();
        }
    }

    /// Records an operating-system memory pressure notification.
    pub fn record_memory_warning(&mut self) {
        self.aggregation.record_memory_warning();
        if self.visible {
            self.rebuild_overlay();
        }
    }

    #[must_use]
    pub fn overlay(&self) -> Option<OverlayImage<'_>> {
        self.visible.then_some(OverlayImage {
            width: self.overlay_width,
            height: self.overlay_height,
            rgba8: &self.overlay_pixels,
            revision: self.overlay_revision,
        })
    }

    /// Text backing the visible panel, useful for logs and deterministic tests.
    #[must_use]
    pub fn overlay_text(&self) -> Option<&str> {
        self.visible.then_some(self.overlay_text.as_str())
    }

    fn rebuild_overlay(&mut self) {
        self.overlay_text = self.aggregation.format_overlay_text();
        let (width, height, pixels) = rasterize_panel(&self.overlay_text, self.glyph_scale);
        self.overlay_width = width;
        self.overlay_height = height;
        self.overlay_pixels = pixels;
        self.overlay_revision = self.overlay_revision.saturating_add(1);
    }
}

#[cfg(test)]
mod test_support {
    use super::{Diagnostics, DomainMetrics, FrameSample, GpuMemory, GpuTime};
    use std::time::Duration;

    pub(super) fn sample(
        frame_number: u64,
        frame_interval: Duration,
        gpu_time: GpuTime,
    ) -> FrameSample {
        FrameSample {
            frame_number,
            frame_interval,
            cpu_render_time: Duration::from_micros(500),
            cpu_update_time: None,
            gpu_time,
            visible_objects: 1,
            rendered_objects: 1,
            scene_draw_calls: 1,
            total_draw_calls: 2,
            gpu_memory: Some(GpuMemory {
                allocated_bytes: 2 * 1_024 * 1_024,
                reserved_bytes: 4 * 1_024 * 1_024,
            }),
        }
    }

    pub(super) fn refresh_with(
        diagnostics: &mut Diagnostics,
        gpu_time: GpuTime,
        domain: DomainMetrics<'_>,
    ) {
        for frame_number in 1..=15 {
            diagnostics.record_presented(
                sample(frame_number, Duration::from_micros(16_667), gpu_time),
                domain,
            );
        }
    }
}

#[cfg(test)]
mod tests {
    use super::test_support::{refresh_with, sample};
    use super::{Diagnostics, DomainMetrics, GpuTime};
    use std::time::Duration;

    #[test]
    fn starts_hidden_and_toggles_both_directions() {
        let mut diagnostics = Diagnostics::default();

        assert!(!diagnostics.is_visible());
        assert!(diagnostics.overlay().is_none());
        assert!(diagnostics.toggle());
        assert!(diagnostics.overlay().is_some());
        assert!(!diagnostics.toggle());
        assert!(diagnostics.overlay().is_none());
    }

    #[test]
    fn reset_discards_pre_pause_frame_history() {
        let mut diagnostics = Diagnostics::default();
        diagnostics.toggle();
        refresh_with(&mut diagnostics, GpuTime::Pending, DomainMetrics::default());

        diagnostics.reset_frame_window();

        assert!(
            diagnostics
                .overlay_text()
                .expect("overlay is visible")
                .contains("FPS              WARMING UP")
        );
    }

    #[test]
    fn refresh_is_throttled_until_a_quarter_second_elapses() {
        let mut diagnostics = Diagnostics::default();
        diagnostics.toggle();
        diagnostics.record_presented(
            sample(1, Duration::from_micros(16_667), GpuTime::Pending),
            DomainMetrics::default(),
        );
        let first_revision = diagnostics.overlay().expect("overlay is visible").revision;

        for frame_number in 2..=15 {
            diagnostics.record_presented(
                sample(
                    frame_number,
                    Duration::from_micros(16_667),
                    GpuTime::Pending,
                ),
                DomainMetrics::default(),
            );
        }
        assert_eq!(
            diagnostics.overlay().expect("overlay is visible").revision,
            first_revision
        );

        diagnostics.record_presented(
            sample(16, Duration::from_micros(16_667), GpuTime::Pending),
            DomainMetrics::default(),
        );
        assert!(diagnostics.overlay().expect("overlay is visible").revision > first_revision);
    }

    #[test]
    fn changing_domain_metrics_do_not_bypass_the_refresh_cadence() {
        let mut diagnostics = Diagnostics::default();
        diagnostics.toggle();
        diagnostics.record_presented(
            sample(1, Duration::from_micros(16_667), GpuTime::Pending),
            DomainMetrics {
                ship_speed_mps: Some(1.0),
                ..DomainMetrics::default()
            },
        );
        let first_revision = diagnostics.overlay().expect("overlay is visible").revision;

        for frame_number in 2..=15 {
            diagnostics.record_presented(
                sample(
                    frame_number,
                    Duration::from_micros(16_667),
                    GpuTime::Pending,
                ),
                DomainMetrics {
                    ship_speed_mps: Some(frame_number as f64),
                    ..DomainMetrics::default()
                },
            );
        }
        assert_eq!(
            diagnostics.overlay().expect("overlay is visible").revision,
            first_revision
        );
        assert!(
            diagnostics
                .overlay_text()
                .expect("overlay is visible")
                .contains("SHIP SPEED       1.00 m/s")
        );

        diagnostics.record_presented(
            sample(16, Duration::from_micros(16_667), GpuTime::Pending),
            DomainMetrics {
                ship_speed_mps: Some(16.0),
                ..DomainMetrics::default()
            },
        );
        assert!(diagnostics.overlay().expect("overlay is visible").revision > first_revision);
        assert!(
            diagnostics
                .overlay_text()
                .expect("overlay is visible")
                .contains("SHIP SPEED       16.00 m/s")
        );
    }

    #[test]
    fn scale_and_memory_warning_changes_rebuild_the_visible_panel() {
        let mut diagnostics = Diagnostics::default();
        diagnostics.toggle();
        let initial = diagnostics.overlay().expect("overlay is visible");
        let initial_size = (initial.width, initial.height);
        let initial_revision = initial.revision;

        diagnostics.set_scale_factor(2.0);
        let scaled = diagnostics.overlay().expect("overlay is visible");
        assert!(scaled.width > initial_size.0);
        assert!(scaled.height > initial_size.1);
        assert!(scaled.revision > initial_revision);

        let scaled_revision = scaled.revision;
        diagnostics.record_memory_warning();
        assert!(diagnostics.overlay().expect("overlay is visible").revision > scaled_revision);
        assert!(
            diagnostics
                .overlay_text()
                .expect("overlay is visible")
                .contains("MEM WARNINGS     1")
        );
    }
}

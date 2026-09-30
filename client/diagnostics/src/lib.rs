//! Observational Phase 0 engineering metrics and overlay rasterization.
//!
//! This crate deliberately has no windowing, GPU, or gameplay dependencies.
//! The runtime supplies typed observations and decides when the resulting RGBA
//! image is visible; the renderer only composites those pixels.

use std::collections::VecDeque;
use std::fmt::Write as _;
use std::time::Duration;

const FRAME_WINDOW_CAPACITY: usize = 120;
const OVERLAY_REFRESH_INTERVAL: Duration = Duration::from_millis(250);
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

#[derive(Clone, Debug, Default, PartialEq)]
struct OwnedDomainMetrics {
    camera_position: Option<[f64; 3]>,
    camera_altitude_meters: Option<f64>,
    camera_phase: Option<String>,
    camera_paused: Option<bool>,
    player_position: Option<[f64; 3]>,
    ship_position: Option<[f64; 3]>,
    ship_velocity: Option<[f64; 3]>,
    ship_speed_mps: Option<f64>,
    thruster_percent: Option<u8>,
    nearby_bodies: Vec<OwnedBodyDistance>,
}

#[derive(Clone, Debug, PartialEq)]
struct OwnedBodyDistance {
    name: String,
    distance_meters: f64,
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

/// Rolling diagnostics collector and optional presentation state.
#[derive(Debug)]
pub struct Diagnostics {
    visible: bool,
    frame_samples: VecDeque<FrameSample>,
    latest_sample: Option<FrameSample>,
    domain_metrics: OwnedDomainMetrics,
    memory_warning_count: u64,
    elapsed_since_refresh: Duration,
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
            frame_samples: VecDeque::with_capacity(FRAME_WINDOW_CAPACITY),
            latest_sample: None,
            domain_metrics: OwnedDomainMetrics::default(),
            memory_warning_count: 0,
            elapsed_since_refresh: Duration::ZERO,
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
            self.elapsed_since_refresh = Duration::ZERO;
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
            if self.visible {
                self.rebuild_overlay();
            }
        }
    }

    /// Drops timing history across lifecycle discontinuities such as minimize
    /// and resume, preventing a pause from polluting the FPS window.
    pub fn reset_frame_window(&mut self) {
        self.frame_samples.clear();
        self.latest_sample = None;
        self.elapsed_since_refresh = Duration::ZERO;
        if self.visible {
            self.rebuild_overlay();
        }
    }

    /// Records an operating-system memory pressure notification.
    pub fn record_memory_warning(&mut self) {
        self.memory_warning_count = self.memory_warning_count.saturating_add(1);
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
        self.overlay_text = self.format_overlay_text();
        let (width, height, pixels) = rasterize_panel(&self.overlay_text, self.glyph_scale);
        self.overlay_width = width;
        self.overlay_height = height;
        self.overlay_pixels = pixels;
        self.overlay_revision = self.overlay_revision.saturating_add(1);
    }

    fn format_overlay_text(&self) -> String {
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

fn average_milliseconds(durations: &[Duration]) -> f64 {
    durations
        .iter()
        .map(|duration| duration.as_secs_f64() * 1_000.0)
        .sum::<f64>()
        / durations.len() as f64
}

fn percentile_95_milliseconds(durations: &[Duration]) -> f64 {
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

fn rasterize_panel(text: &str, scale: u32) -> (u32, u32, Vec<u8>) {
    const GLYPH_WIDTH: u32 = 5;
    const GLYPH_HEIGHT: u32 = 7;
    const CHARACTER_ADVANCE: u32 = 6;
    const LINE_ADVANCE: u32 = 9;
    const PADDING: u32 = 3;
    const BACKGROUND: [u8; 4] = [5, 12, 24, 226];
    const BORDER: [u8; 4] = [34, 211, 238, 255];
    const HEADING: [u8; 4] = [103, 232, 249, 255];
    const MUTED: [u8; 4] = [148, 163, 184, 255];
    const TEXT: [u8; 4] = [226, 232, 240, 255];

    let lines: Vec<&str> = text.lines().collect();
    let max_characters = lines
        .iter()
        .map(|line| line.chars().count() as u32)
        .max()
        .unwrap_or(1);
    let line_count = lines.len().max(1) as u32;
    let content_width = max_characters
        .saturating_mul(CHARACTER_ADVANCE)
        .saturating_sub(CHARACTER_ADVANCE - GLYPH_WIDTH);
    let content_height = line_count
        .saturating_mul(LINE_ADVANCE)
        .saturating_sub(LINE_ADVANCE - GLYPH_HEIGHT);
    let width = (content_width + PADDING * 2) * scale;
    let height = (content_height + PADDING * 2) * scale;
    let pixel_count = width as usize * height as usize;
    let mut pixels = vec![0_u8; pixel_count * 4];
    for pixel in pixels.as_chunks_mut::<4>().0 {
        pixel.copy_from_slice(&BACKGROUND);
    }

    fill_rect(&mut pixels, width, height, 0, 0, width, scale, BORDER);
    fill_rect(&mut pixels, width, height, 0, 0, scale, height, BORDER);
    fill_rect(
        &mut pixels,
        width,
        height,
        0,
        height - scale,
        width,
        scale,
        BORDER,
    );
    fill_rect(
        &mut pixels,
        width,
        height,
        width - scale,
        0,
        scale,
        height,
        BORDER,
    );

    for (line_index, line) in lines.iter().enumerate() {
        let color = match line_index {
            0 => HEADING,
            1 => MUTED,
            _ => TEXT,
        };
        let glyph_y = (PADDING + line_index as u32 * LINE_ADVANCE) * scale;
        for (character_index, character) in line.chars().enumerate() {
            if character == ' ' {
                continue;
            }
            let glyph_x = (PADDING + character_index as u32 * CHARACTER_ADVANCE) * scale;
            for (row, bits) in glyph_rows(character).into_iter().enumerate() {
                for column in 0..GLYPH_WIDTH {
                    if bits & (1 << (GLYPH_WIDTH - 1 - column)) != 0 {
                        fill_rect(
                            &mut pixels,
                            width,
                            height,
                            glyph_x + column * scale,
                            glyph_y + row as u32 * scale,
                            scale,
                            scale,
                            color,
                        );
                    }
                }
            }
        }
    }

    (width, height, pixels)
}

#[allow(clippy::too_many_arguments)]
fn fill_rect(
    pixels: &mut [u8],
    image_width: u32,
    image_height: u32,
    x: u32,
    y: u32,
    width: u32,
    height: u32,
    color: [u8; 4],
) {
    let end_x = x.saturating_add(width).min(image_width);
    let end_y = y.saturating_add(height).min(image_height);
    for pixel_y in y.min(image_height)..end_y {
        for pixel_x in x.min(image_width)..end_x {
            let offset = (pixel_y as usize * image_width as usize + pixel_x as usize) * 4;
            pixels[offset..offset + 4].copy_from_slice(&color);
        }
    }
}

fn glyph_rows(character: char) -> [u8; 7] {
    match character.to_ascii_uppercase() {
        'A' => [
            0b01110, 0b10001, 0b10001, 0b11111, 0b10001, 0b10001, 0b10001,
        ],
        'B' => [
            0b11110, 0b10001, 0b10001, 0b11110, 0b10001, 0b10001, 0b11110,
        ],
        'C' => [
            0b01111, 0b10000, 0b10000, 0b10000, 0b10000, 0b10000, 0b01111,
        ],
        'D' => [
            0b11110, 0b10001, 0b10001, 0b10001, 0b10001, 0b10001, 0b11110,
        ],
        'E' => [
            0b11111, 0b10000, 0b10000, 0b11110, 0b10000, 0b10000, 0b11111,
        ],
        'F' => [
            0b11111, 0b10000, 0b10000, 0b11110, 0b10000, 0b10000, 0b10000,
        ],
        'G' => [
            0b01111, 0b10000, 0b10000, 0b10111, 0b10001, 0b10001, 0b01111,
        ],
        'H' => [
            0b10001, 0b10001, 0b10001, 0b11111, 0b10001, 0b10001, 0b10001,
        ],
        'I' => [
            0b11111, 0b00100, 0b00100, 0b00100, 0b00100, 0b00100, 0b11111,
        ],
        'J' => [
            0b00111, 0b00010, 0b00010, 0b00010, 0b10010, 0b10010, 0b01100,
        ],
        'K' => [
            0b10001, 0b10010, 0b10100, 0b11000, 0b10100, 0b10010, 0b10001,
        ],
        'L' => [
            0b10000, 0b10000, 0b10000, 0b10000, 0b10000, 0b10000, 0b11111,
        ],
        'M' => [
            0b10001, 0b11011, 0b10101, 0b10101, 0b10001, 0b10001, 0b10001,
        ],
        'N' => [
            0b10001, 0b11001, 0b10101, 0b10011, 0b10001, 0b10001, 0b10001,
        ],
        'O' => [
            0b01110, 0b10001, 0b10001, 0b10001, 0b10001, 0b10001, 0b01110,
        ],
        'P' => [
            0b11110, 0b10001, 0b10001, 0b11110, 0b10000, 0b10000, 0b10000,
        ],
        'Q' => [
            0b01110, 0b10001, 0b10001, 0b10001, 0b10101, 0b10010, 0b01101,
        ],
        'R' => [
            0b11110, 0b10001, 0b10001, 0b11110, 0b10100, 0b10010, 0b10001,
        ],
        'S' => [
            0b01111, 0b10000, 0b10000, 0b01110, 0b00001, 0b00001, 0b11110,
        ],
        'T' => [
            0b11111, 0b00100, 0b00100, 0b00100, 0b00100, 0b00100, 0b00100,
        ],
        'U' => [
            0b10001, 0b10001, 0b10001, 0b10001, 0b10001, 0b10001, 0b01110,
        ],
        'V' => [
            0b10001, 0b10001, 0b10001, 0b10001, 0b10001, 0b01010, 0b00100,
        ],
        'W' => [
            0b10001, 0b10001, 0b10001, 0b10101, 0b10101, 0b10101, 0b01010,
        ],
        'X' => [
            0b10001, 0b10001, 0b01010, 0b00100, 0b01010, 0b10001, 0b10001,
        ],
        'Y' => [
            0b10001, 0b10001, 0b01010, 0b00100, 0b00100, 0b00100, 0b00100,
        ],
        'Z' => [
            0b11111, 0b00001, 0b00010, 0b00100, 0b01000, 0b10000, 0b11111,
        ],
        '0' => [
            0b01110, 0b10001, 0b10011, 0b10101, 0b11001, 0b10001, 0b01110,
        ],
        '1' => [
            0b00100, 0b01100, 0b00100, 0b00100, 0b00100, 0b00100, 0b01110,
        ],
        '2' => [
            0b01110, 0b10001, 0b00001, 0b00010, 0b00100, 0b01000, 0b11111,
        ],
        '3' => [
            0b11110, 0b00001, 0b00001, 0b01110, 0b00001, 0b00001, 0b11110,
        ],
        '4' => [
            0b00010, 0b00110, 0b01010, 0b10010, 0b11111, 0b00010, 0b00010,
        ],
        '5' => [
            0b11111, 0b10000, 0b10000, 0b11110, 0b00001, 0b00001, 0b11110,
        ],
        '6' => [
            0b01110, 0b10000, 0b10000, 0b11110, 0b10001, 0b10001, 0b01110,
        ],
        '7' => [
            0b11111, 0b00001, 0b00010, 0b00100, 0b01000, 0b01000, 0b01000,
        ],
        '8' => [
            0b01110, 0b10001, 0b10001, 0b01110, 0b10001, 0b10001, 0b01110,
        ],
        '9' => [
            0b01110, 0b10001, 0b10001, 0b01111, 0b00001, 0b00001, 0b01110,
        ],
        ':' => [0, 0b00100, 0b00100, 0, 0b00100, 0b00100, 0],
        '.' => [0, 0, 0, 0, 0, 0b00110, 0b00110],
        ',' => [0, 0, 0, 0, 0b00110, 0b00100, 0b01000],
        '-' => [0, 0, 0, 0b11111, 0, 0, 0],
        '/' => [
            0b00001, 0b00010, 0b00010, 0b00100, 0b01000, 0b01000, 0b10000,
        ],
        '%' => [0b11001, 0b11010, 0b00100, 0b01000, 0b10110, 0b00110, 0],
        '(' => [
            0b00010, 0b00100, 0b01000, 0b01000, 0b01000, 0b00100, 0b00010,
        ],
        ')' => [
            0b01000, 0b00100, 0b00010, 0b00010, 0b00010, 0b00100, 0b01000,
        ],
        '[' => [
            0b01110, 0b01000, 0b01000, 0b01000, 0b01000, 0b01000, 0b01110,
        ],
        ']' => [
            0b01110, 0b00010, 0b00010, 0b00010, 0b00010, 0b00010, 0b01110,
        ],
        '+' => [0, 0b00100, 0b00100, 0b11111, 0b00100, 0b00100, 0],
        '=' => [0, 0, 0b11111, 0, 0b11111, 0, 0],
        '|' => [
            0b00100, 0b00100, 0b00100, 0b00100, 0b00100, 0b00100, 0b00100,
        ],
        ' ' => [0; 7],
        _ => [0b01110, 0b10001, 0b00010, 0b00100, 0b00100, 0, 0b00100],
    }
}

#[cfg(test)]
mod tests {
    use super::{BodyDistance, Diagnostics, DomainMetrics, FrameSample, GpuMemory, GpuTime};
    use std::time::Duration;

    fn sample(frame_number: u64, frame_interval: Duration, gpu_time: GpuTime) -> FrameSample {
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

    fn refresh_with(diagnostics: &mut Diagnostics, gpu_time: GpuTime, domain: DomainMetrics<'_>) {
        for frame_number in 1..=15 {
            diagnostics.record_presented(
                sample(frame_number, Duration::from_micros(16_667), gpu_time),
                domain,
            );
        }
    }

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

    #[test]
    fn rasterized_image_has_exact_rgba_length_and_visible_pixels() {
        let mut diagnostics = Diagnostics::default();
        diagnostics.toggle();
        let overlay = diagnostics.overlay().expect("overlay is visible");

        assert_eq!(
            overlay.rgba8.len(),
            overlay.width as usize * overlay.height as usize * 4
        );
        assert!(
            overlay
                .rgba8
                .as_chunks::<4>()
                .0
                .iter()
                .any(|pixel| pixel[3] > 0)
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

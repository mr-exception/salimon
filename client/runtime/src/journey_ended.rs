//! Terminal presentation and pointer geometry. Durable restart stays in checkpoints.
mod raster;
use crate::session::lifecycle::EndedRun;
use salimon_renderer::{OverlayImage, OverlayPlacement};
use salimon_survival::effects::Source;
use std::time::Duration;
#[derive(Clone, Debug, PartialEq, Eq, Hash, serde::Serialize)]
pub(crate) struct Report {
    pub(crate) cause: String,
    pub(crate) duration: String,
    pub(crate) distance: String,
    pub(crate) location: String,
}
impl From<&EndedRun> for Report {
    fn from(run: &EndedRun) -> Self {
        Self {
            cause: match run.cause {
                Source::Starvation => "STARVATION".into(),
                Source::Radiation => "RADIATION".into(),
                Source::Burning => "BURNING".into(),
                Source::Fall => "FALL".into(),
                Source::Environment => "ENVIRONMENT".into(),
                Source::Registered(id) => format!("REGISTERED HAZARD #{id}"),
            },
            duration: format_duration(run.active_duration),
            distance: format_distance(run.journey.distance_meters),
            location: run.final_location.name.to_ascii_uppercase(),
        }
    }
}
fn format_duration(duration: Duration) -> String {
    let s = duration.as_secs();
    if s == 0 {
        return format!("0.{:03}S", duration.subsec_millis());
    }
    if s >= 86400 {
        format!(
            "{}D {:02}H {:02}M {:02}S",
            s / 86400,
            s / 3600 % 24,
            s / 60 % 60,
            s % 60
        )
    } else {
        format!("{:02}H {:02}M {:02}S", s / 3600, s / 60 % 60, s % 60)
    }
}
fn format_distance(meters: f64) -> String {
    if meters < 1000. {
        format!("{meters:.1} M")
    } else if meters < 1e9 {
        format!("{:.2} KM", meters / 1000.)
    } else {
        format!("{meters:.3e} M").to_ascii_uppercase()
    }
}
const PANEL: [u32; 2] = [280, 160];
const BUTTON: [u32; 4] = [56, 116, 168, 21];
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub(crate) struct Layout {
    canvas: [u32; 2],
    scale: u32,
    panel_origin: [u32; 2],
    screen: [u32; 2],
}
impl Layout {
    pub(crate) fn new(screen: [u32; 2], dpi: f64) -> Option<Self> {
        let available = salimon_renderer::overlay_available_size(screen);
        let desired = if dpi.is_finite() {
            (2. * dpi.clamp(1., 1.5)).round() as u32
        } else {
            2
        };
        let scale = desired
            .min(available[0] / PANEL[0])
            .min(available[1] / PANEL[1]);
        if scale == 0 {
            return None;
        }
        let panel = PANEL.map(|v| v * scale);
        // Shade ordinary windows; bound the terminal-only RGBA cache at 64 MiB.
        // Very large displays retain the centered panel without a huge backdrop.
        let canvas = if u64::from(available[0]) * u64::from(available[1]) <= 16 * 1024 * 1024 {
            available
        } else {
            panel
        };
        Some(Self {
            canvas,
            scale,
            panel_origin: std::array::from_fn(|i| (canvas[i] - panel[i]) / 2),
            screen,
        })
    }
    fn button_bounds(self) -> [f64; 4] {
        let origin: [f64; 2] = std::array::from_fn(|i| {
            f64::from(self.screen[i] - self.canvas[i]) / 2. + f64::from(self.panel_origin[i])
        });
        [
            origin[0] + f64::from(BUTTON[0] * self.scale),
            origin[1] + f64::from(BUTTON[1] * self.scale),
            f64::from(BUTTON[2] * self.scale),
            f64::from(BUTTON[3] * self.scale),
        ]
    }
    pub(crate) fn contains(self, point: [f64; 2]) -> bool {
        let [x, y, w, h] = self.button_bounds();
        point.into_iter().all(f64::is_finite)
            && point[0] >= x
            && point[0] < x + w
            && point[1] >= y
            && point[1] < y + h
    }
}
#[derive(Default)]
pub(crate) struct Screen {
    pointer: Option<[f64; 2]>,
    failed: bool,
    cache: Option<(Report, Layout, bool, bool)>,
    pixels: Vec<u8>,
    revision: u64,
}
impl Screen {
    pub(crate) fn pointer_moved(&mut self, point: [f64; 2]) {
        self.pointer = point.into_iter().all(f64::is_finite).then_some(point);
    }
    pub(crate) fn clear_pointer(&mut self) {
        self.pointer = None;
    }
    pub(crate) fn restart_failed(&mut self) {
        self.failed = true;
    }
    pub(crate) fn clicked_restart(&self, screen: [u32; 2], dpi: f64) -> bool {
        self.pointer
            .zip(Layout::new(screen, dpi))
            .is_some_and(|(p, l)| l.contains(p))
    }
    pub(crate) fn image(
        &mut self,
        report: Option<Report>,
        screen: [u32; 2],
        dpi: f64,
    ) -> Option<OverlayImage<'_>> {
        let Some(report) = report else {
            *self = Self::default();
            return None;
        };
        let layout = Layout::new(screen, dpi)?;
        let hover = self.pointer.is_some_and(|p| layout.contains(p));
        let key = (report, layout, hover, self.failed);
        if self.cache.as_ref() != Some(&key) {
            self.pixels = raster::draw(&key.0, layout, hover, self.failed);
            use std::hash::{Hash, Hasher};
            let mut hash = std::collections::hash_map::DefaultHasher::new();
            key.hash(&mut hash);
            self.revision = hash.finish();
            self.cache = Some(key);
        }
        Some(OverlayImage {
            width: layout.canvas[0],
            height: layout.canvas[1],
            rgba8: &self.pixels,
            revision: self.revision,
            placement: OverlayPlacement::Center,
        })
    }
}
#[cfg(test)]
mod tests;

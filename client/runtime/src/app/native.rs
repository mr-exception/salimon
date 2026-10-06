//! Window configuration, drawable size and cursor operations.

use salimon_renderer::SurfaceSize;
use winit::dpi::{LogicalSize, PhysicalSize};
use winit::window::{CursorGrabMode, Window, WindowAttributes};

pub(super) const WINDOW_TITLE: &str = "Salimon — Compressed Solar System";
pub(super) const INITIAL_WIDTH: u32 = 1920;
pub(super) const INITIAL_HEIGHT: u32 = 1080;
const MINIMUM_WIDTH: f64 = 640.0;
const MINIMUM_HEIGHT: f64 = 360.0;
pub(super) const fn initial_window_size() -> PhysicalSize<u32> {
    PhysicalSize::new(INITIAL_WIDTH, INITIAL_HEIGHT)
}

pub(super) fn window_attributes(e2e: bool) -> WindowAttributes {
    WindowAttributes::default()
        .with_title(WINDOW_TITLE)
        .with_inner_size(if e2e {
            PhysicalSize::new(1280, 800)
        } else {
            initial_window_size()
        })
        .with_min_inner_size(LogicalSize::new(MINIMUM_WIDTH, MINIMUM_HEIGHT))
        .with_resizable(true)
}

pub(super) fn surface_size(window: &Window) -> SurfaceSize {
    let size = window.inner_size();
    SurfaceSize::new(size.width, size.height)
}

pub(super) fn capture_cursor(window: &Window) -> bool {
    if let Err(error) = window.set_cursor_grab(CursorGrabMode::Locked) {
        log::warn!("could not lock cursor for mouse look: {error}");
        window.set_cursor_visible(true);
        return false;
    }
    window.set_cursor_visible(false);
    true
}

pub(super) fn release_cursor(window: &Window) {
    if let Err(error) = window.set_cursor_grab(CursorGrabMode::None) {
        log::warn!("could not release cursor: {error}");
    }
    window.set_cursor_visible(true);
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn initial_drawable_requests_the_phase_zero_benchmark_resolution() {
        assert_eq!(initial_window_size().width, 1920);
        assert_eq!(initial_window_size().height, 1080);
    }
}

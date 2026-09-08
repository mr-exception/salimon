//! Drawable resize and surface configuration; recovery policy remains with the host.
use super::{DepthTarget, Renderer, SurfaceSize};

impl Renderer {
    /// Applies a new physical surface size.
    ///
    /// Zero-sized windows are marked idle because `wgpu` rejects zero-sized
    /// surface configurations. A later non-zero resize resumes presentation.
    pub fn resize(&mut self, size: SurfaceSize) {
        self.drawable = size.is_drawable();
        if !self.drawable {
            return;
        }

        self.configuration.width = size.width;
        self.configuration.height = size.height;
        self.depth_target = DepthTarget::new(&self.device, size.width, size.height);
        self.configure_surface();
    }

    pub(super) fn configure_surface(&self) {
        debug_assert!(self.configuration.width > 0 && self.configuration.height > 0);
        self.surface.configure(&self.device, &self.configuration);
    }
}

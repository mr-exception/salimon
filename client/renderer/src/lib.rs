//! Custom GPU presentation for the Salimon client.
//!
//! This crate owns `wgpu` resources and surface presentation. It deliberately
//! has no dependency on world, character, or ship state.

use std::error::Error;
use std::fmt;

const CLEAR_COLOR: wgpu::Color = wgpu::Color {
    r: 0.008,
    g: 0.015,
    b: 0.035,
    a: 1.0,
};

/// Physical pixel dimensions for the renderer's presentation surface.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct SurfaceSize {
    width: u32,
    height: u32,
}

impl SurfaceSize {
    #[must_use]
    pub const fn new(width: u32, height: u32) -> Self {
        Self { width, height }
    }

    #[must_use]
    pub const fn is_drawable(self) -> bool {
        self.width > 0 && self.height > 0
    }
}

/// Stable information about the GPU selected for presentation.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RendererInfo {
    pub adapter_name: String,
    pub backend: String,
    pub device_type: String,
}

/// Result of attempting to render one frame.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum RenderOutcome {
    /// Commands were submitted and the surface image was presented.
    Presented,
    /// No drawable image is currently available; wait for a lifecycle event.
    Idle,
    /// The surface is temporarily unavailable or was reconfigured; retry later.
    Retry,
    /// The platform surface was lost and the renderer must be rebuilt.
    SurfaceLost,
}

/// Failure while initializing or using the GPU renderer.
#[derive(Debug)]
pub struct RendererError {
    context: &'static str,
    detail: String,
}

impl RendererError {
    fn new(context: &'static str, detail: impl fmt::Display) -> Self {
        Self {
            context,
            detail: detail.to_string(),
        }
    }
}

impl fmt::Display for RendererError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "{}: {}", self.context, self.detail)
    }
}

impl Error for RendererError {}

/// Owns the surface, device, queue, and bootstrap render pipeline.
pub struct Renderer {
    surface: wgpu::Surface<'static>,
    device: wgpu::Device,
    queue: wgpu::Queue,
    configuration: wgpu::SurfaceConfiguration,
    pipeline: wgpu::RenderPipeline,
    drawable: bool,
    info: RendererInfo,
}

impl Renderer {
    /// Creates a renderer for an owned native surface target.
    ///
    /// Passing an owned target (for example `Arc<winit::window::Window>`) gives
    /// the surface a static lifetime without unsafe code.
    pub async fn new(
        surface_target: impl Into<wgpu::SurfaceTarget<'static>>,
        initial_size: SurfaceSize,
    ) -> Result<Self, RendererError> {
        let instance = wgpu::Instance::default();
        let surface = instance
            .create_surface(surface_target)
            .map_err(|error| RendererError::new("failed to create rendering surface", error))?;

        let adapter = instance
            .request_adapter(&wgpu::RequestAdapterOptions {
                power_preference: wgpu::PowerPreference::HighPerformance,
                compatible_surface: Some(&surface),
                force_fallback_adapter: false,
                ..Default::default()
            })
            .await
            .map_err(|error| {
                RendererError::new("failed to find a compatible GPU adapter", error)
            })?;

        let adapter_info = adapter.get_info();
        let info = RendererInfo {
            adapter_name: adapter_info.name,
            backend: format!("{:?}", adapter_info.backend),
            device_type: format!("{:?}", adapter_info.device_type),
        };

        let (device, queue) = adapter
            .request_device(&wgpu::DeviceDescriptor {
                label: Some("Salimon Phase 0 device"),
                required_features: wgpu::Features::empty(),
                required_limits: wgpu::Limits::default(),
                experimental_features: wgpu::ExperimentalFeatures::disabled(),
                memory_hints: wgpu::MemoryHints::MemoryUsage,
                trace: wgpu::Trace::Off,
            })
            .await
            .map_err(|error| RendererError::new("failed to create GPU device", error))?;

        let configured_width = initial_size.width.max(1);
        let configured_height = initial_size.height.max(1);
        let mut configuration = surface
            .get_default_config(&adapter, configured_width, configured_height)
            .ok_or_else(|| {
                RendererError::new(
                    "failed to configure rendering surface",
                    "adapter exposes no compatible surface format",
                )
            })?;
        configuration.present_mode = wgpu::PresentMode::AutoVsync;
        configuration.desired_maximum_frame_latency = 2;

        let shader = device.create_shader_module(wgpu::include_wgsl!("bootstrap.wgsl"));
        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("Salimon bootstrap pipeline layout"),
            bind_group_layouts: &[],
            immediate_size: 0,
        });
        let pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("Salimon bootstrap triangle pipeline"),
            layout: Some(&pipeline_layout),
            vertex: wgpu::VertexState {
                module: &shader,
                entry_point: Some("vs_main"),
                buffers: &[],
                compilation_options: wgpu::PipelineCompilationOptions::default(),
            },
            primitive: wgpu::PrimitiveState::default(),
            depth_stencil: None,
            multisample: wgpu::MultisampleState::default(),
            fragment: Some(wgpu::FragmentState {
                module: &shader,
                entry_point: Some("fs_main"),
                targets: &[Some(wgpu::ColorTargetState {
                    format: configuration.format,
                    blend: Some(wgpu::BlendState::REPLACE),
                    write_mask: wgpu::ColorWrites::ALL,
                })],
                compilation_options: wgpu::PipelineCompilationOptions::default(),
            }),
            multiview_mask: None,
            cache: None,
        });

        let mut renderer = Self {
            surface,
            device,
            queue,
            configuration,
            pipeline,
            drawable: false,
            info,
        };
        renderer.resize(initial_size);
        Ok(renderer)
    }

    #[must_use]
    pub const fn info(&self) -> &RendererInfo {
        &self.info
    }

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
        self.configure_surface();
    }

    /// Draws and presents the bootstrap scene.
    ///
    /// `before_present` lets the platform runtime issue its presentation
    /// notification at the exact boundary without introducing a `winit`
    /// dependency into this crate.
    pub fn render(
        &mut self,
        before_present: impl FnOnce(),
    ) -> Result<RenderOutcome, RendererError> {
        if !self.drawable {
            return Ok(RenderOutcome::Idle);
        }

        let frame = match self.surface.get_current_texture() {
            wgpu::CurrentSurfaceTexture::Success(frame) => frame,
            wgpu::CurrentSurfaceTexture::Suboptimal(frame) => {
                drop(frame);
                self.configure_surface();
                return Ok(RenderOutcome::Retry);
            }
            wgpu::CurrentSurfaceTexture::Timeout => return Ok(RenderOutcome::Retry),
            wgpu::CurrentSurfaceTexture::Occluded => return Ok(RenderOutcome::Idle),
            wgpu::CurrentSurfaceTexture::Outdated => {
                self.configure_surface();
                return Ok(RenderOutcome::Retry);
            }
            wgpu::CurrentSurfaceTexture::Lost => return Ok(RenderOutcome::SurfaceLost),
            wgpu::CurrentSurfaceTexture::Validation => {
                return Err(RendererError::new(
                    "failed to acquire surface image",
                    "wgpu reported a validation error",
                ));
            }
        };

        let view = frame
            .texture
            .create_view(&wgpu::TextureViewDescriptor::default());
        let mut encoder = self
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("Salimon bootstrap frame encoder"),
            });
        {
            let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("Salimon bootstrap render pass"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &view,
                    depth_slice: None,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(CLEAR_COLOR),
                        store: wgpu::StoreOp::Store,
                    },
                })],
                depth_stencil_attachment: None,
                timestamp_writes: None,
                occlusion_query_set: None,
                multiview_mask: None,
            });
            render_pass.set_pipeline(&self.pipeline);
            render_pass.draw(0..3, 0..1);
        }

        self.queue.submit([encoder.finish()]);
        before_present();
        self.queue.present(frame);
        Ok(RenderOutcome::Presented)
    }

    fn configure_surface(&self) {
        debug_assert!(self.configuration.width > 0 && self.configuration.height > 0);
        self.surface.configure(&self.device, &self.configuration);
    }
}

#[cfg(test)]
mod tests {
    use super::SurfaceSize;

    #[test]
    fn surface_size_requires_two_non_zero_dimensions() {
        assert!(SurfaceSize::new(1, 1).is_drawable());
        assert!(!SurfaceSize::new(0, 1).is_drawable());
        assert!(!SurfaceSize::new(1, 0).is_drawable());
        assert!(!SurfaceSize::new(0, 0).is_drawable());
    }
}

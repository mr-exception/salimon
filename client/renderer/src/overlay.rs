// Keep this physical-pixel inset aligned with the vertex shader's margin.
const OVERLAY_MARGIN_PIXELS: u32 = 16;

/// Borrowed RGBA image supplied by a diagnostics or UI producer.
#[derive(Clone, Copy, Debug)]
pub struct OverlayImage<'a> {
    pub width: u32,
    pub height: u32,
    pub rgba8: &'a [u8],
    pub revision: u64,
    pub placement: OverlayPlacement,
}

/// Screen-space or absolute world-object placement selected by the producer.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub enum OverlayPlacement {
    #[default]
    TopLeft,
    BottomCenter,
    /// Bottom-center message above a renderer-reserved physical-pixel band.
    BottomCenterInset(u32),
    Center,
    /// Absolute object center and conservative visible-surface radius, in metres.
    World {
        anchor_meters: [f64; 3],
        radius_meters: f64,
    },
}

impl OverlayImage<'_> {
    pub(crate) fn validate(self) -> Result<(), OverlayImageError> {
        if self.width == 0 || self.height == 0 {
            return Err(OverlayImageError::Empty);
        }
        let expected_len = usize::try_from(self.width)
            .ok()
            .and_then(|width| {
                usize::try_from(self.height)
                    .ok()
                    .and_then(|height| width.checked_mul(height))
            })
            .and_then(|pixels| pixels.checked_mul(4))
            .ok_or(OverlayImageError::DimensionsOverflow)?;
        if self.rgba8.len() != expected_len {
            return Err(OverlayImageError::InvalidByteLength {
                expected: expected_len,
                actual: self.rgba8.len(),
            });
        }
        Ok(())
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum OverlayImageError {
    Empty,
    DimensionsOverflow,
    InvalidByteLength { expected: usize, actual: usize },
}

impl std::fmt::Display for OverlayImageError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Empty => formatter.write_str("overlay image dimensions must be nonzero"),
            Self::DimensionsOverflow => formatter.write_str("overlay image dimensions overflow"),
            Self::InvalidByteLength { expected, actual } => write!(
                formatter,
                "overlay image has {actual} RGBA bytes; expected {expected}"
            ),
        }
    }
}

pub(crate) fn fitted_overlay_size(surface_size: [u32; 2], image_size: [u32; 2]) -> [f32; 2] {
    let available = surface_size.map(|size| size.saturating_sub(2 * OVERLAY_MARGIN_PIXELS) as f32);
    let image_size = image_size.map(|size| size as f32);
    let scale = (available[0] / image_size[0])
        .min(available[1] / image_size[1])
        .min(1.0);
    image_size.map(|size| size * scale)
}

fn overlay_origin(
    surface_size: [u32; 2],
    display_size: [f32; 2],
    placement: OverlayPlacement,
) -> [f32; 2] {
    match placement {
        OverlayPlacement::World { .. } => unreachable!("world placement is projected separately"),
        OverlayPlacement::TopLeft => [OVERLAY_MARGIN_PIXELS as f32; 2],
        OverlayPlacement::Center => [
            (surface_size[0] as f32 - display_size[0]) * 0.5,
            (surface_size[1] as f32 - display_size[1]) * 0.5,
        ],
        OverlayPlacement::BottomCenterInset(inset) => [
            (surface_size[0] as f32 - display_size[0]).max(0.0) * 0.5,
            (surface_size[1] as f32
                - OVERLAY_MARGIN_PIXELS as f32
                - inset as f32
                - display_size[1])
                .max(0.0),
        ],
        OverlayPlacement::BottomCenter => [
            (surface_size[0] as f32 - display_size[0]).max(0.0) * 0.5,
            (surface_size[1] as f32 - OVERLAY_MARGIN_PIXELS as f32 - display_size[1]).max(0.0),
        ],
    }
}

/// Project the center for placement and the camera-facing bound for reverse-Z visibility.
fn project_anchor(
    camera: crate::CameraFrame,
    matrix: [f32; 16],
    anchor: [f64; 3],
    radius: f64,
) -> Option<[f32; 3]> {
    if !radius.is_finite() || radius < 0.0 {
        return None;
    }
    let relative: [f64; 3] = std::array::from_fn(|i| anchor[i] - camera.position_meters[i]);
    if !relative.iter().all(|v| v.is_finite()) {
        return None;
    }
    let clip: [f64; 4] = std::array::from_fn(|row| {
        (0..3)
            .map(|col| f64::from(matrix[col * 4 + row]) * relative[col])
            .sum::<f64>()
            + f64::from(matrix[12 + row])
    });
    if !clip.iter().all(|v| v.is_finite()) || clip[3] <= 0.0 {
        return None;
    }
    let ndc = clip.map(|v| v / clip[3]);
    if ndc[0].abs() > 1.0 || ndc[1].abs() > 1.0 || !(0.0..=1.0).contains(&ndc[2]) {
        return None;
    }
    let distance = relative.iter().map(|v| v * v).sum::<f64>().sqrt();
    let factor = 1.0 - (radius / distance).min(0.5);
    let depth = (ndc[2] / factor).min(1.0);
    Some([ndc[0] as f32, ndc[1] as f32, depth as f32])
}

pub(crate) struct OverlayRenderer {
    pipeline: wgpu::RenderPipeline,
    bind_group_layout: wgpu::BindGroupLayout,
    dimensions_buffer: wgpu::Buffer,
    texture: Option<wgpu::Texture>,
    bind_group: Option<wgpu::BindGroup>,
    depth_view: Option<wgpu::TextureView>,
    image_size: Option<(u32, u32)>,
    last_revision: Option<u64>,
    visible: bool,
}

impl OverlayRenderer {
    pub(crate) fn new(device: &wgpu::Device, surface_format: wgpu::TextureFormat) -> Self {
        let bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("Salimon overlay bind group layout"),
            entries: &[
                wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Texture {
                        sample_type: wgpu::TextureSampleType::Float { filterable: false },
                        view_dimension: wgpu::TextureViewDimension::D2,
                        multisampled: false,
                    },
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 1,
                    visibility: wgpu::ShaderStages::VERTEX_FRAGMENT,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 2,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Texture {
                        sample_type: wgpu::TextureSampleType::Depth,
                        view_dimension: wgpu::TextureViewDimension::D2,
                        multisampled: false,
                    },
                    count: None,
                },
            ],
        });
        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("Salimon overlay pipeline layout"),
            bind_group_layouts: &[Some(&bind_group_layout)],
            immediate_size: 0,
        });
        let shader = device.create_shader_module(wgpu::include_wgsl!("overlay.wgsl"));
        let pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("Salimon overlay pipeline"),
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
                    format: surface_format,
                    blend: Some(wgpu::BlendState::ALPHA_BLENDING),
                    write_mask: wgpu::ColorWrites::ALL,
                })],
                compilation_options: wgpu::PipelineCompilationOptions::default(),
            }),
            multiview_mask: None,
            cache: None,
        });
        let dimensions_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("Salimon overlay dimensions"),
            size: 48,
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        Self {
            pipeline,
            bind_group_layout,
            dimensions_buffer,
            texture: None,
            bind_group: None,
            depth_view: None,
            image_size: None,
            last_revision: None,
            visible: false,
        }
    }

    #[allow(clippy::too_many_arguments)]
    pub(crate) fn prepare(
        &mut self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        surface_width: u32,
        surface_height: u32,
        image: Option<OverlayImage<'_>>,
        camera: crate::CameraFrame,
        view_projection: [f32; 16],
        depth_view: &wgpu::TextureView,
    ) -> Result<(), OverlayImageError> {
        let Some(image) = image else {
            self.visible = false;
            return Ok(());
        };
        image.validate()?;

        // Surface reconstruction replaces the depth view; rebuild its binding then.
        if self.depth_view.as_ref() != Some(depth_view) {
            self.depth_view = Some(depth_view.clone());
            self.image_size = None;
        }
        let image_size = (image.width, image.height);
        if self.image_size != Some(image_size) {
            self.recreate_texture(device, image.width, image.height);
            self.last_revision = None;
        }
        if self.last_revision != Some(image.revision) {
            let texture = self
                .texture
                .as_ref()
                .expect("a prepared overlay has a texture");
            queue.write_texture(
                wgpu::TexelCopyTextureInfo {
                    texture,
                    mip_level: 0,
                    origin: wgpu::Origin3d::ZERO,
                    aspect: wgpu::TextureAspect::All,
                },
                image.rgba8,
                wgpu::TexelCopyBufferLayout {
                    offset: 0,
                    bytes_per_row: Some(image.width * 4),
                    rows_per_image: Some(image.height),
                },
                wgpu::Extent3d {
                    width: image.width,
                    height: image.height,
                    depth_or_array_layers: 1,
                },
            );
            self.last_revision = Some(image.revision);
        }

        let layout_height = match image.placement {
            OverlayPlacement::BottomCenterInset(inset) => surface_height.saturating_sub(inset),
            _ => surface_height,
        };
        let mut display_size =
            fitted_overlay_size([surface_width, layout_height], [image.width, image.height]);
        if matches!(image.placement, OverlayPlacement::World { .. }) {
            let scale = (surface_width as f32 * 0.6 / display_size[0]).min(1.0);
            display_size = display_size.map(|value| value * scale);
        }
        let (origin, visibility) = match image.placement {
            OverlayPlacement::World {
                anchor_meters,
                radius_meters,
            } => {
                let Some(projected) =
                    project_anchor(camera, view_projection, anchor_meters, radius_meters)
                else {
                    self.visible = false;
                    return Ok(());
                };
                let pixel = [
                    (projected[0] * 0.5 + 0.5) * surface_width as f32,
                    (0.5 - projected[1] * 0.5) * surface_height as f32,
                ];
                let origin = [
                    pixel[0] - display_size[0] * 0.5,
                    pixel[1] - display_size[1] - 12.0,
                ];
                // Hide rather than clamp a label whose full rectangle does not fit.
                if (0..2).any(|i| {
                    origin[i] < 0.0
                        || origin[i] + display_size[i] > [surface_width, surface_height][i] as f32
                }) {
                    self.visible = false;
                    return Ok(());
                }
                (origin, [pixel[0], pixel[1], projected[2], 1.0])
            }
            placement => (
                overlay_origin([surface_width, surface_height], display_size, placement),
                [0.0; 4],
            ),
        };
        let dimensions = [
            surface_width as f32,
            surface_height as f32,
            display_size[0],
            display_size[1],
            origin[0],
            origin[1],
            0.0,
            0.0,
            visibility[0],
            visibility[1],
            visibility[2],
            visibility[3],
        ];
        let mut bytes = [0_u8; 48];
        for (chunk, value) in bytes.as_chunks_mut::<4>().0.iter_mut().zip(dimensions) {
            chunk.copy_from_slice(&value.to_ne_bytes());
        }
        queue.write_buffer(&self.dimensions_buffer, 0, &bytes);
        self.visible = display_size.into_iter().all(|size| size > 0.0);
        Ok(())
    }

    pub(crate) fn draw<'pass>(&'pass self, render_pass: &mut wgpu::RenderPass<'pass>) {
        if !self.visible {
            return;
        }
        let bind_group = self
            .bind_group
            .as_ref()
            .expect("a visible overlay has a bind group");
        render_pass.set_pipeline(&self.pipeline);
        render_pass.set_bind_group(0, bind_group, &[]);
        render_pass.draw(0..6, 0..1);
    }

    pub(crate) const fn is_visible(&self) -> bool {
        self.visible
    }

    fn recreate_texture(&mut self, device: &wgpu::Device, width: u32, height: u32) {
        let texture = device.create_texture(&wgpu::TextureDescriptor {
            label: Some("Salimon overlay image"),
            size: wgpu::Extent3d {
                width,
                height,
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Rgba8UnormSrgb,
            usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
            view_formats: &[],
        });
        let texture_view = texture.create_view(&wgpu::TextureViewDescriptor::default());
        let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("Salimon overlay bind group"),
            layout: &self.bind_group_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::TextureView(&texture_view),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: self.dimensions_buffer.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 2,
                    resource: wgpu::BindingResource::TextureView(
                        self.depth_view
                            .as_ref()
                            .expect("prepare installs depth view"),
                    ),
                },
            ],
        });
        self.texture = Some(texture);
        self.bind_group = Some(bind_group);
        self.image_size = Some((width, height));
    }
}

#[cfg(test)]
mod tests {
    use super::{
        OverlayImage, OverlayImageError, OverlayPlacement, fitted_overlay_size, overlay_origin,
    };

    fn test_camera(origin: f64) -> crate::CameraFrame {
        crate::CameraFrame {
            position_meters: [origin; 3],
            target_meters: [origin, origin, origin - 1.0],
            up: [0.0, 1.0, 0.0],
            vertical_fov_radians: std::f32::consts::FRAC_PI_2,
            near_plane_meters: 0.05,
        }
    }

    #[test]
    fn world_projection_rebases_before_narrowing_and_tracks_motion() {
        for origin in [0.0, 1.0e12] {
            let camera = test_camera(origin);
            let matrix = crate::infinite_reverse_z_projection(camera, 1.0).unwrap();
            let anchor = [origin, origin, origin - 2.0];
            let projected = super::project_anchor(camera, matrix, anchor, 0.0).unwrap();
            assert_eq!(&projected[..2], &[0.0, 0.0]);
            assert!((projected[2] - 0.025).abs() < 1.0e-6);
            let shifted = super::project_anchor(
                camera,
                matrix,
                [origin + 1.0, origin + 0.5, origin - 2.0],
                0.0,
            )
            .unwrap();
            assert!((shifted[0] - 0.5).abs() < 1.0e-6);
            assert!((shifted[1] - 0.25).abs() < 1.0e-6);
            let mut moved = camera;
            moved.position_meters[0] += 1.0;
            assert!(super::project_anchor(moved, matrix, anchor, 0.0).unwrap()[0] < 0.0);
            let surface = super::project_anchor(camera, matrix, anchor, 0.25).unwrap();
            assert!(surface[2] > projected[2]);
        }
    }

    #[test]
    fn world_projection_hides_behind_near_plane_offscreen_and_invalid_targets() {
        let camera = test_camera(0.0);
        let matrix = crate::infinite_reverse_z_projection(camera, 1.0).unwrap();
        for anchor in [
            [0.0, 0.0, 1.0],
            [0.0, 0.0, -0.01],
            [3.0, 0.0, -2.0],
            [0.0, -3.0, -2.0],
            [f64::NAN, 0.0, -2.0],
            [0.0; 3],
        ] {
            assert!(
                super::project_anchor(camera, matrix, anchor, 0.0).is_none(),
                "{anchor:?}"
            );
        }
        assert!(super::project_anchor(camera, matrix, [0.0, 0.0, -2.0], -1.0).is_none());
    }

    #[test]
    fn keeps_source_size_when_the_panel_fits() {
        assert_eq!(fitted_overlay_size([1280, 720], [560, 600]), [560.0, 600.0]);
        assert_eq!(fitted_overlay_size([592, 632], [560, 600]), [560.0, 600.0]);
    }

    #[test]
    fn fits_a_wide_panel_uniformly_inside_the_horizontal_margins() {
        assert_eq!(fitted_overlay_size([640, 360], [1024, 256]), [608.0, 152.0]);
    }

    #[test]
    fn fits_tall_and_high_density_panels_inside_all_margins() {
        for (surface, source) in [([640, 360], [560, 600]), ([1280, 720], [1120, 1200])] {
            let displayed = fitted_overlay_size(surface, source);
            assert!(displayed[0] <= (surface[0] - 32) as f32);
            assert!(displayed[1] <= (surface[1] - 32) as f32);
            assert!(displayed[0] <= source[0] as f32);
            assert!(displayed[1] <= source[1] as f32);
            let width_scale = displayed[0] / source[0] as f32;
            let height_scale = displayed[1] / source[1] as f32;
            assert!((width_scale - height_scale).abs() < 1.0e-6);
            assert!((displayed[1] - (surface[1] - 32) as f32).abs() < 1.0e-4);
        }
    }

    #[test]
    fn hides_the_panel_when_the_surface_cannot_contain_both_margins() {
        assert_eq!(fitted_overlay_size([32, 360], [560, 600]), [0.0, 0.0]);
        assert_eq!(fitted_overlay_size([640, 0], [560, 600]), [0.0, 0.0]);
    }

    #[test]
    fn messages_stack_above_toolbar_after_resize_and_dpi_changes() {
        use crate::equipment_toolbar::{EquipmentToolbar, ToolbarRaster};
        for dpi in [1.0, 1.5, 2.0] {
            let mut toolbar = ToolbarRaster::default();
            toolbar.update(
                EquipmentToolbar {
                    slots: [None; 5],
                    selected: None,
                },
                dpi,
            );
            let image = toolbar.image().unwrap();
            for surface in [[1280, 720], [1920, 1080], [360, 640], [400, 180], [64, 64]] {
                let toolbar_size = fitted_overlay_size(surface, [image.width, image.height]);
                let toolbar_origin =
                    overlay_origin(surface, toolbar_size, OverlayPlacement::BottomCenter);
                let inset = toolbar.message_inset(surface);
                let message_size =
                    fitted_overlay_size([surface[0], surface[1].saturating_sub(inset)], [1200, 96]);
                let message_origin = overlay_origin(
                    surface,
                    message_size,
                    OverlayPlacement::BottomCenterInset(inset),
                );
                if message_size[1] > 0.0 {
                    assert!(message_origin[1] + message_size[1] < toolbar_origin[1]);
                    assert!(message_origin[1] >= 16.0);
                }
                assert!(
                    (toolbar_origin[0] + toolbar_size[0] * 0.5 - surface[0] as f32 * 0.5).abs()
                        < 0.001
                );
            }
        }
    }

    #[test]
    fn places_action_bars_at_the_bottom_center() {
        assert_eq!(
            overlay_origin([1280, 720], [600.0, 80.0], OverlayPlacement::BottomCenter),
            [340.0, 624.0]
        );
        assert_eq!(
            overlay_origin([1280, 720], [600.0, 80.0], OverlayPlacement::TopLeft),
            [16.0, 16.0]
        );
    }

    #[test]
    fn centered_overlay_matches_the_projection_center_after_resize() {
        for surface in [[1280, 720], [1920, 1080], [801, 603], [360, 640], [16, 16]] {
            let size = fitted_overlay_size(surface, [17, 17]);
            let origin = overlay_origin(surface, size, OverlayPlacement::Center);
            // The overlay shader maps this midpoint to NDC (0, 0), the camera ray.
            for axis in 0..2 {
                let midpoint = origin[axis] + size[axis] * 0.5;
                assert_eq!(midpoint, surface[axis] as f32 * 0.5);
                assert_eq!(midpoint / surface[axis] as f32 * 2.0 - 1.0, 0.0);
            }
        }
    }

    #[test]
    fn accepts_exact_rgba_byte_length() {
        assert_eq!(
            OverlayImage {
                width: 2,
                height: 3,
                rgba8: &[0; 24],
                revision: 1,
                placement: OverlayPlacement::TopLeft,
            }
            .validate(),
            Ok(())
        );
    }

    #[test]
    fn rejects_empty_or_incomplete_images() {
        assert_eq!(
            OverlayImage {
                width: 0,
                height: 1,
                rgba8: &[],
                revision: 0,
                placement: OverlayPlacement::TopLeft,
            }
            .validate(),
            Err(OverlayImageError::Empty)
        );
        assert_eq!(
            OverlayImage {
                width: 2,
                height: 2,
                rgba8: &[0; 15],
                revision: 0,
                placement: OverlayPlacement::TopLeft,
            }
            .validate(),
            Err(OverlayImageError::InvalidByteLength {
                expected: 16,
                actual: 15,
            })
        );
    }
}

/// Borrowed RGBA image supplied by a diagnostics or UI producer.
#[derive(Clone, Copy, Debug)]
pub struct OverlayImage<'a> {
    pub width: u32,
    pub height: u32,
    pub rgba8: &'a [u8],
    pub revision: u64,
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

pub(crate) struct OverlayRenderer {
    pipeline: wgpu::RenderPipeline,
    bind_group_layout: wgpu::BindGroupLayout,
    dimensions_buffer: wgpu::Buffer,
    texture: Option<wgpu::Texture>,
    bind_group: Option<wgpu::BindGroup>,
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
                    visibility: wgpu::ShaderStages::VERTEX,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: None,
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
            size: 16,
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        Self {
            pipeline,
            bind_group_layout,
            dimensions_buffer,
            texture: None,
            bind_group: None,
            image_size: None,
            last_revision: None,
            visible: false,
        }
    }

    pub(crate) fn prepare(
        &mut self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        surface_width: u32,
        surface_height: u32,
        image: Option<OverlayImage<'_>>,
    ) -> Result<(), OverlayImageError> {
        let Some(image) = image else {
            self.visible = false;
            return Ok(());
        };
        image.validate()?;

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

        let dimensions = [
            surface_width as f32,
            surface_height as f32,
            image.width as f32,
            image.height as f32,
        ];
        let mut bytes = [0_u8; 16];
        for (chunk, value) in bytes.chunks_exact_mut(4).zip(dimensions) {
            chunk.copy_from_slice(&value.to_ne_bytes());
        }
        queue.write_buffer(&self.dimensions_buffer, 0, &bytes);
        self.visible = true;
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
            ],
        });
        self.texture = Some(texture);
        self.bind_group = Some(bind_group);
        self.image_size = Some((width, height));
    }
}

#[cfg(test)]
mod tests {
    use super::{OverlayImage, OverlayImageError};

    #[test]
    fn accepts_exact_rgba_byte_length() {
        assert_eq!(
            OverlayImage {
                width: 2,
                height: 3,
                rgba8: &[0; 24],
                revision: 1,
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
            }
            .validate(),
            Err(OverlayImageError::InvalidByteLength {
                expected: 16,
                actual: 15,
            })
        );
    }
}

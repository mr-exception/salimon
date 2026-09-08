//! Bounded, translation-free authored sky plus physically sized local stars.
use crate::{CameraFrame, RendererError, camera_basis, encode_f32s};
use wgpu::util::DeviceExt;
pub const MAX_SKY_STARS: usize = 128;
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct SkyStar {
    /// Unit direction in the same axes as the camera, after f64 relative subtraction.
    pub direction: [f32; 3],
    pub angular_radius_radians: f32,
    /// Linear RGB radiance proxy. This is approximate presentation, not photometry.
    pub color: [f32; 3],
    pub brightness: f32,
}
#[derive(Clone, Debug, PartialEq)]
pub struct SkyScene {
    pub stars: Vec<SkyStar>,
    /// Authored galactic plane normal in camera-world axes; no navigation identities.
    pub galactic_pole: [f32; 3],
    pub background_gain: f32,
}
impl Default for SkyScene {
    fn default() -> Self {
        Self {
            stars: vec![],
            galactic_pole: [0.2, 0.93, 0.3],
            background_gain: 1.0,
        }
    }
}
pub(crate) struct SkyRenderer {
    background: wgpu::RenderPipeline,
    stars: wgpu::RenderPipeline,
    uniform: wgpu::Buffer,
    bind: wgpu::BindGroup,
    instances: wgpu::Buffer,
    count: u32,
    enabled: bool,
}
impl SkyRenderer {
    pub(crate) fn new(device: &wgpu::Device, format: wgpu::TextureFormat) -> Self {
        let uniform = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("sky basis"),
            contents: &[0; 96],
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        });
        let layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("sky layout"),
            entries: &[wgpu::BindGroupLayoutEntry {
                binding: 0,
                visibility: wgpu::ShaderStages::VERTEX_FRAGMENT,
                ty: wgpu::BindingType::Buffer {
                    ty: wgpu::BufferBindingType::Uniform,
                    has_dynamic_offset: false,
                    min_binding_size: wgpu::BufferSize::new(96),
                },
                count: None,
            }],
        });
        let bind = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("sky bind"),
            layout: &layout,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: uniform.as_entire_binding(),
            }],
        });
        let shader = device.create_shader_module(wgpu::include_wgsl!("sky.wgsl"));
        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("sky pipelines"),
            bind_group_layouts: &[Some(&layout)],
            immediate_size: 0,
        });
        let attributes = wgpu::vertex_attr_array![0 => Float32x4, 1 => Float32x4];
        let buffers = [Some(wgpu::VertexBufferLayout {
            array_stride: 32,
            step_mode: wgpu::VertexStepMode::Instance,
            attributes: &attributes,
        })];
        let make = |star: bool| {
            device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
                label: Some(if star {
                    "bounded physical stars"
                } else {
                    "authored Milky Way"
                }),
                layout: Some(&pipeline_layout),
                vertex: wgpu::VertexState {
                    module: &shader,
                    entry_point: Some(if star { "vs_star" } else { "vs_sky" }),
                    buffers: if star { &buffers } else { &[] },
                    compilation_options: Default::default(),
                },
                primitive: Default::default(),
                depth_stencil: Some(wgpu::DepthStencilState {
                    format: crate::DEPTH_FORMAT,
                    depth_write_enabled: Some(false),
                    depth_compare: Some(wgpu::CompareFunction::GreaterEqual),
                    stencil: Default::default(),
                    bias: Default::default(),
                }),
                multisample: Default::default(),
                fragment: Some(wgpu::FragmentState {
                    module: &shader,
                    entry_point: Some(if star { "fs_star" } else { "fs_sky" }),
                    targets: &[Some(wgpu::ColorTargetState {
                        format,
                        blend: Some(if star {
                            wgpu::BlendState::ALPHA_BLENDING
                        } else {
                            wgpu::BlendState::REPLACE
                        }),
                        write_mask: wgpu::ColorWrites::ALL,
                    })],
                    compilation_options: Default::default(),
                }),
                multiview_mask: None,
                cache: None,
            })
        };
        Self {
            background: make(false),
            stars: make(true),
            uniform,
            bind,
            instances: device.create_buffer(&wgpu::BufferDescriptor {
                label: Some("bounded sky stars"),
                size: (MAX_SKY_STARS * 32) as u64,
                usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
                mapped_at_creation: false,
            }),
            count: 0,
            enabled: false,
        }
    }
    pub(crate) fn prepare(
        &mut self,
        queue: &wgpu::Queue,
        camera: CameraFrame,
        size: [u32; 2],
        sky: Option<&SkyScene>,
    ) -> Result<(), RendererError> {
        self.enabled = sky.is_some();
        self.count = 0;
        let Some(sky) = sky else { return Ok(()) };
        let invalid = || {
            RendererError::new(
                "invalid sky",
                "finite unit directions, nonnegative radiance and bounded star count required",
            )
        };
        let unit = |v: [f32; 3]| {
            v.iter().all(|x| x.is_finite())
                && (v.iter().map(|x| x * x).sum::<f32>() - 1.).abs() < 0.002
        };
        let pole_length = sky.galactic_pole.iter().map(|x| x * x).sum::<f32>().sqrt();
        if sky.stars.len() > MAX_SKY_STARS
            || !pole_length.is_finite()
            || pole_length < 0.001
            || !sky.background_gain.is_finite()
            || !(0.0..=4.0).contains(&sky.background_gain)
        {
            return Err(invalid());
        }
        let basis = camera_basis(camera).map_err(invalid_camera)?;
        let tan = (camera.vertical_fov_radians * 0.5).tan();
        if !tan.is_finite() || tan <= 0. || size.contains(&0) {
            return Err(invalid());
        }
        let mut values = Vec::new();
        for (v, w) in [
            (basis.right, tan * size[0] as f32 / size[1] as f32),
            (basis.up, tan),
            (basis.forward, 0.),
            (
                sky.galactic_pole.map(|x| x / pole_length),
                sky.background_gain,
            ),
            ([size[0] as f32, size[1] as f32, 0.], 0.),
            ([0.; 3], 0.),
        ] {
            values.extend(v);
            values.push(w);
        }
        let mut stars = Vec::new();
        for s in &sky.stars {
            if !unit(s.direction)
                || !s.angular_radius_radians.is_finite()
                || !(0.0..=1.57).contains(&s.angular_radius_radians)
                || !s.brightness.is_finite()
                || !(0.0..=100.0).contains(&s.brightness)
                || !s
                    .color
                    .iter()
                    .all(|x| x.is_finite() && (0.0..=1.0).contains(x))
            {
                return Err(invalid());
            }
            stars.extend(s.direction);
            stars.push(s.angular_radius_radians);
            stars.extend(s.color);
            stars.push(s.brightness);
        }
        queue.write_buffer(&self.uniform, 0, &encode_f32s(values));
        if !stars.is_empty() {
            queue.write_buffer(&self.instances, 0, &encode_f32s(stars));
        }
        self.count = sky.stars.len() as u32;
        Ok(())
    }
    pub(crate) fn draw(&self, pass: &mut wgpu::RenderPass<'_>) {
        if !self.enabled {
            return;
        }
        pass.set_bind_group(0, &self.bind, &[]);
        pass.set_pipeline(&self.background);
        pass.draw(0..3, 0..1);
        if self.count > 0 {
            pass.set_pipeline(&self.stars);
            pass.set_vertex_buffer(0, self.instances.slice(..));
            pass.draw(0..6, 0..self.count);
        }
    }
    pub(crate) fn draws(&self) -> u32 {
        u32::from(self.enabled) + u32::from(self.enabled && self.count > 0)
    }
}
fn invalid_camera(e: crate::SceneInputError) -> RendererError {
    RendererError::new("invalid sky camera", e)
}

#[cfg(test)]
mod tests;

//! Single-scattering spherical atmosphere composited against actual opaque reverse-Z depth.
use crate::{CameraFrame, RendererError, camera_basis, encode_f32s};
use wgpu::util::DeviceExt;
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct AtmosphereScene {
    pub center_meters: [f64; 3],
    pub radius_meters: f64,
    pub scale_height_meters: f64,
    pub ceiling_meters: f64,
    pub rayleigh_per_meter: [f64; 3],
    /// Unit world direction from planet toward illuminator; no renderer ephemeris authority.
    pub sun_direction: [f32; 3],
    pub sun_color: [f32; 3],
    /// Fraction of illuminator visible after independently evaluated eclipse geometry.
    pub solar_visibility: f32,
}
pub(crate) struct AtmosphereRenderer {
    pipeline: wgpu::RenderPipeline,
    layout: wgpu::BindGroupLayout,
    uniform: wgpu::Buffer,
    bind: Option<wgpu::BindGroup>,
}
impl AtmosphereRenderer {
    pub(crate) fn new(device: &wgpu::Device, format: wgpu::TextureFormat) -> Self {
        let uniform = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("atmospheric optics"),
            contents: &[0; 128],
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        });
        let layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("atmosphere depth layout"),
            entries: &[
                wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::VERTEX_FRAGMENT,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: wgpu::BufferSize::new(128),
                    },
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 1,
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
        let shader = device.create_shader_module(wgpu::include_wgsl!("atmosphere.wgsl"));
        let pl = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("atmosphere pipeline"),
            bind_group_layouts: &[Some(&layout)],
            immediate_size: 0,
        });
        let pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("single scattering atmosphere"),
            layout: Some(&pl),
            vertex: wgpu::VertexState {
                module: &shader,
                entry_point: Some("vs_main"),
                buffers: &[],
                compilation_options: Default::default(),
            },
            primitive: Default::default(),
            depth_stencil: None,
            multisample: Default::default(),
            fragment: Some(wgpu::FragmentState {
                module: &shader,
                entry_point: Some("fs_main"),
                targets: &[Some(wgpu::ColorTargetState {
                    format,
                    blend: Some(wgpu::BlendState {
                        color: wgpu::BlendComponent {
                            src_factor: wgpu::BlendFactor::One,
                            dst_factor: wgpu::BlendFactor::OneMinusSrcAlpha,
                            operation: wgpu::BlendOperation::Add,
                        },
                        alpha: wgpu::BlendComponent::OVER,
                    }),
                    write_mask: wgpu::ColorWrites::ALL,
                })],
                compilation_options: Default::default(),
            }),
            multiview_mask: None,
            cache: None,
        });
        Self {
            pipeline,
            layout,
            uniform,
            bind: None,
        }
    }
    pub(crate) fn prepare(
        &mut self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        camera: CameraFrame,
        size: [u32; 2],
        scene: Option<AtmosphereScene>,
        depth: &wgpu::TextureView,
    ) -> Result<(), RendererError> {
        self.bind = None;
        let Some(a) = scene else { return Ok(()) };
        let invalid = || {
            RendererError::new(
                "invalid atmosphere",
                "finite physical optics, positive sphere/scale heights and unit light direction required",
            )
        };
        if !a.center_meters.iter().all(|x| x.is_finite())
            || !a.radius_meters.is_finite()
            || !(1.0..=1e12).contains(&a.radius_meters)
            || !a.scale_height_meters.is_finite()
            || a.scale_height_meters < 1.
            || !a.ceiling_meters.is_finite()
            || a.ceiling_meters < a.scale_height_meters
            || a.ceiling_meters > a.radius_meters
            || !a
                .rayleigh_per_meter
                .iter()
                .all(|x| x.is_finite() && *x >= 0. && *x <= 1.)
            || !a.sun_direction.iter().all(|x| x.is_finite())
            || (a.sun_direction.iter().map(|x| x * x).sum::<f32>() - 1.).abs() > 0.002
            || !a
                .sun_color
                .iter()
                .all(|x| x.is_finite() && *x >= 0. && *x <= 100.)
            || !a.solar_visibility.is_finite()
            || !(0.0..=1.0).contains(&a.solar_visibility)
            || size.contains(&0)
            || !camera.near_plane_meters.is_finite()
            || camera.near_plane_meters <= 0.
            || !camera.vertical_fov_radians.is_finite()
            || !(0.0..std::f32::consts::PI).contains(&camera.vertical_fov_radians)
        {
            return Err(invalid());
        }
        let basis = camera_basis(camera).map_err(|e| RendererError::new("atmosphere camera", e))?;
        let tan = (camera.vertical_fov_radians * 0.5).tan();
        let relative = std::array::from_fn::<_, 3, _>(|i| {
            (camera.position_meters[i] - a.center_meters[i]) / a.radius_meters
        });
        if !relative.iter().all(|v| v.is_finite() && v.abs() < 1e10)
            || !tan.is_finite()
            || tan <= 0.
        {
            return Err(invalid());
        }
        let values = [
            [
                basis.right[0],
                basis.right[1],
                basis.right[2],
                tan * size[0] as f32 / size[1] as f32,
            ],
            [basis.up[0], basis.up[1], basis.up[2], tan],
            [
                basis.forward[0],
                basis.forward[1],
                basis.forward[2],
                camera.near_plane_meters / a.radius_meters as f32,
            ],
            [
                relative[0] as f32,
                relative[1] as f32,
                relative[2] as f32,
                1. + (a.ceiling_meters / a.radius_meters) as f32,
            ],
            [
                a.rayleigh_per_meter[0] as f32 * a.radius_meters as f32,
                a.rayleigh_per_meter[1] as f32 * a.radius_meters as f32,
                a.rayleigh_per_meter[2] as f32 * a.radius_meters as f32,
                (a.scale_height_meters / a.radius_meters) as f32,
            ],
            [
                a.sun_direction[0],
                a.sun_direction[1],
                a.sun_direction[2],
                a.solar_visibility,
            ],
            [a.sun_color[0], a.sun_color[1], a.sun_color[2], 0.],
            [size[0] as f32, size[1] as f32, 0., 0.],
        ];
        queue.write_buffer(&self.uniform, 0, &encode_f32s(values.into_iter().flatten()));
        self.bind = Some(device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("atmosphere sampled opaque depth"),
            layout: &self.layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: self.uniform.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::TextureView(depth),
                },
            ],
        }));
        Ok(())
    }
    pub(crate) fn draw(&self, pass: &mut wgpu::RenderPass<'_>) {
        if let Some(bind) = &self.bind {
            pass.set_pipeline(&self.pipeline);
            pass.set_bind_group(0, bind, &[]);
            pass.draw(0..3, 0..1);
        }
    }
    pub(crate) fn draws(&self) -> u32 {
        u32::from(self.bind.is_some())
    }
}

#[cfg(test)]
mod tests;

//! Fixed-budget optical weather from the world's sampled field, independent of terrain LOD.
use crate::{AtmosphereScene, CameraFrame, RendererError, camera_basis, encode_f32s};
use wgpu::util::DeviceExt;
pub const WEATHER_GRID: [u32; 2] = [64, 32];
#[derive(Clone, Debug, PartialEq)]
pub struct WeatherScene {
    pub optical: Vec<[f32; 4]>,
    pub wind_temperature: Vec<[f32; 4]>,
    pub cloud_base_meters: f64,
    pub precipitation_freeze_kelvin: f32,
    pub cloud_thickness_meters: f64,
    pub time_seconds: f64,
    /// Rigid world-to-body rotation; keeps regional identity independent of camera/LOD.
    pub body_axes: [[f32; 3]; 3],
    /// Performance sampling only; never changes the supplied weather field.
    pub view_samples: u32,
}
pub(crate) struct WeatherRenderer {
    pipeline: wgpu::RenderPipeline,
    layout: wgpu::BindGroupLayout,
    uniform: wgpu::Buffer,
    optical: wgpu::Texture,
    wind: wgpu::Texture,
    bind: Option<wgpu::BindGroup>,
}
impl WeatherRenderer {
    pub(crate) fn new(device: &wgpu::Device, format: wgpu::TextureFormat) -> Self {
        let uniform = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("weather view"),
            contents: &[0; 224],
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        });
        let entry = |binding, sample_type| wgpu::BindGroupLayoutEntry {
            binding,
            visibility: wgpu::ShaderStages::FRAGMENT,
            ty: wgpu::BindingType::Texture {
                sample_type,
                view_dimension: wgpu::TextureViewDimension::D2,
                multisampled: false,
            },
            count: None,
        };
        let layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("coherent weather field"),
            entries: &[
                wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::VERTEX_FRAGMENT,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: wgpu::BufferSize::new(224),
                    },
                    count: None,
                },
                entry(1, wgpu::TextureSampleType::Depth),
                entry(2, wgpu::TextureSampleType::Float { filterable: false }),
                entry(3, wgpu::TextureSampleType::Float { filterable: false }),
            ],
        });
        let texture = |label| {
            device.create_texture(&wgpu::TextureDescriptor {
                label: Some(label),
                size: wgpu::Extent3d {
                    width: 64,
                    height: 32,
                    depth_or_array_layers: 1,
                },
                mip_level_count: 1,
                sample_count: 1,
                dimension: wgpu::TextureDimension::D2,
                format: wgpu::TextureFormat::Rgba32Float,
                usage: wgpu::TextureUsages::COPY_DST | wgpu::TextureUsages::TEXTURE_BINDING,
                view_formats: &[],
            })
        };
        let shader = device.create_shader_module(wgpu::include_wgsl!("weather.wgsl"));
        let pl = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("weather pipeline"),
            bind_group_layouts: &[Some(&layout)],
            immediate_size: 0,
        });
        let pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("weather optical volume"),
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
            optical: texture("cloud storm precipitation dust"),
            wind: texture("body wind temperature"),
            bind: None,
        }
    }
    #[allow(clippy::too_many_arguments)]
    pub(crate) fn prepare(
        &mut self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        camera: CameraFrame,
        size: [u32; 2],
        atmosphere: Option<AtmosphereScene>,
        weather: Option<&WeatherScene>,
        depth: &wgpu::TextureView,
    ) -> Result<(), RendererError> {
        self.bind = None;
        let Some(w) = weather else { return Ok(()) };
        let error = || {
            RendererError::new(
                "invalid weather presentation",
                "bounded physical atmosphere, field, orthonormal axes and sample budget required",
            )
        };
        let a = atmosphere.ok_or_else(error)?;
        if w.optical.len() != 2048
            || w.wind_temperature.len() != 2048
            || !w
                .optical
                .iter()
                .flatten()
                .all(|v| v.is_finite() && *v >= 0.)
            || !w.wind_temperature.iter().flatten().all(|v| v.is_finite())
            || w.optical
                .iter()
                .any(|v| v[0] > 1. || v[1] > 1. || v[2] > 1. || v[3] > 1.)
            || !w.precipitation_freeze_kelvin.is_finite()
            || !(0.0..=400.0).contains(&w.precipitation_freeze_kelvin)
            || w.wind_temperature
                .iter()
                .any(|v| v[..3].iter().any(|x| x.abs() > 1e5) || !(0.0..=1e5).contains(&v[3]))
            || !w.time_seconds.is_finite()
            || !w.cloud_base_meters.is_finite()
            || w.cloud_base_meters < 0.
            || !w.cloud_thickness_meters.is_finite()
            || w.cloud_thickness_meters <= 0.
            || w.cloud_base_meters + w.cloud_thickness_meters > a.ceiling_meters
            || ![8, 16, 24].contains(&w.view_samples)
        {
            return Err(error());
        }
        for i in 0..3 {
            for j in 0..3 {
                let dot: f32 = (0..3).map(|k| w.body_axes[i][k] * w.body_axes[j][k]).sum();
                if !dot.is_finite() || (dot - f32::from(i == j)).abs() > 0.002 {
                    return Err(error());
                }
            }
        }
        let [x, y, z] = w.body_axes;
        let determinant = x[0] * (y[1] * z[2] - y[2] * z[1]) - x[1] * (y[0] * z[2] - y[2] * z[0])
            + x[2] * (y[0] * z[1] - y[1] * z[0]);
        if (determinant - 1.).abs() > 0.004 {
            return Err(error());
        }
        let basis = camera_basis(camera).map_err(|e| RendererError::new("weather camera", e))?;
        let tangent = (camera.vertical_fov_radians * 0.5).tan();
        let observer: [f32; 3] = std::array::from_fn(|i| {
            ((camera.position_meters[i] - a.center_meters[i]) / a.radius_meters) as f32
        });
        let mut values = Vec::new();
        for (v, k) in [
            (basis.right, tangent * size[0] as f32 / size[1] as f32),
            (basis.up, tangent),
            (
                basis.forward,
                camera.near_plane_meters / a.radius_meters as f32,
            ),
            (observer, 1. + (a.ceiling_meters / a.radius_meters) as f32),
            (a.sun_direction, a.solar_visibility),
            (a.sun_color, a.radius_meters as f32),
            (
                w.body_axes[0],
                (w.cloud_base_meters / a.radius_meters) as f32,
            ),
            (
                w.body_axes[1],
                (w.cloud_thickness_meters / a.radius_meters) as f32,
            ),
            (
                w.body_axes[2],
                (w.time_seconds.rem_euclid(86400.) / 86400. * std::f64::consts::TAU) as f32,
            ),
            ([size[0] as f32, size[1] as f32, 0.], w.view_samples as f32),
            ([w.precipitation_freeze_kelvin, 0., 0.], 0.),
            ([0.; 3], 0.),
            ([0.; 3], 0.),
            ([0.; 3], 0.),
        ] {
            values.extend(v);
            values.push(k);
        }
        if !values.iter().all(|v| v.is_finite()) {
            return Err(error());
        }
        queue.write_buffer(&self.uniform, 0, &encode_f32s(values));
        for (texture, data) in [
            (&self.optical, &w.optical),
            (&self.wind, &w.wind_temperature),
        ] {
            queue.write_texture(
                wgpu::TexelCopyTextureInfo {
                    texture,
                    mip_level: 0,
                    origin: wgpu::Origin3d::ZERO,
                    aspect: wgpu::TextureAspect::All,
                },
                &encode_f32s(data.iter().flatten().copied()),
                wgpu::TexelCopyBufferLayout {
                    offset: 0,
                    bytes_per_row: Some(64 * 16),
                    rows_per_image: Some(32),
                },
                wgpu::Extent3d {
                    width: 64,
                    height: 32,
                    depth_or_array_layers: 1,
                },
            );
        }
        self.bind = Some(device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("coherent weather bind"),
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
                wgpu::BindGroupEntry {
                    binding: 2,
                    resource: wgpu::BindingResource::TextureView(
                        &self.optical.create_view(&Default::default()),
                    ),
                },
                wgpu::BindGroupEntry {
                    binding: 3,
                    resource: wgpu::BindingResource::TextureView(
                        &self.wind.create_view(&Default::default()),
                    ),
                },
            ],
        }));
        Ok(())
    }
    pub(crate) fn draw(&self, pass: &mut wgpu::RenderPass<'_>) {
        if let Some(bind) = &self.bind {
            pass.set_pipeline(&self.pipeline);
            pass.set_bind_group(0, bind, &[]);
            pass.draw(0..3, 0..1)
        }
    }
    pub(crate) fn draws(&self) -> u32 {
        u32::from(self.bind.is_some())
    }
}

#[cfg(test)]
mod tests;

//! Instanced, procedurally faceted segment geometry; canonical biology stays with the world.
use crate::{CameraFrame, PointLight, RendererError, camera_basis, encode_f32s};
use wgpu::util::DeviceExt;
pub const MAX_ORGANISM_SEGMENTS: usize = 16384;
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct OrganismSegment {
    pub start_meters: [f64; 3],
    pub end_meters: [f64; 3],
    pub radius_start_meters: f64,
    pub radius_end_meters: f64,
    pub color: [f32; 3],
}
fn mesh(sides: usize) -> Vec<[f32; 3]> {
    let mut v = Vec::new();
    for i in 0..sides {
        let a = i as f32 * std::f32::consts::TAU / sides as f32;
        let b = (i + 1) as f32 * std::f32::consts::TAU / sides as f32;
        let lo = [a.cos(), 0., a.sin()];
        let next = [b.cos(), 0., b.sin()];
        let hi = [lo[0], 1., lo[2]];
        let nh = [next[0], 1., next[2]];
        v.extend([
            lo,
            hi,
            next,
            next,
            hi,
            nh,
            [0., 0., 0.],
            next,
            lo,
            [0., 1., 0.],
            hi,
            nh,
        ]);
    }
    v
}
pub(crate) struct OrganismRenderer {
    pipeline: wgpu::RenderPipeline,
    uniform: wgpu::Buffer,
    bind: wgpu::BindGroup,
    vertices: [wgpu::Buffer; 2],
    instances: [wgpu::Buffer; 2],
    counts: [u32; 2],
}
impl OrganismRenderer {
    pub(crate) fn new(device: &wgpu::Device, format: wgpu::TextureFormat) -> Self {
        let uniform = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("organism view"),
            contents: &[0; 96],
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        });
        let layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("organism view layout"),
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
            label: Some("organism view bind"),
            layout: &layout,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: uniform.as_entire_binding(),
            }],
        });
        let shader = device.create_shader_module(wgpu::include_wgsl!("organisms.wgsl"));
        let pl = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("procedural organism"),
            bind_group_layouts: &[Some(&layout)],
            immediate_size: 0,
        });
        let pipeline=device.create_render_pipeline(&wgpu::RenderPipelineDescriptor{label:Some("faceted organism segments"),layout:Some(&pl),vertex:wgpu::VertexState{module:&shader,entry_point:Some("vs_main"),buffers:&[Some(wgpu::VertexBufferLayout{array_stride:12,step_mode:wgpu::VertexStepMode::Vertex,attributes:&wgpu::vertex_attr_array![0=>Float32x3]}),Some(wgpu::VertexBufferLayout{array_stride:48,step_mode:wgpu::VertexStepMode::Instance,attributes:&wgpu::vertex_attr_array![1=>Float32x4,2=>Float32x4,3=>Float32x4]})],compilation_options:Default::default()},primitive:Default::default(),depth_stencil:Some(wgpu::DepthStencilState{format:crate::DEPTH_FORMAT,depth_write_enabled:Some(true),depth_compare:Some(wgpu::CompareFunction::Greater),stencil:Default::default(),bias:Default::default()}),multisample:Default::default(),fragment:Some(wgpu::FragmentState{module:&shader,entry_point:Some("fs_main"),targets:&[Some(wgpu::ColorTargetState{format,blend:Some(wgpu::BlendState::REPLACE),write_mask:wgpu::ColorWrites::ALL})],compilation_options:Default::default()}),multiview_mask:None,cache:None});
        Self {
            pipeline,
            uniform,
            bind,
            vertices: [6, 3].map(|n| {
                device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
                    label: Some("procedural tapered prism"),
                    contents: &encode_f32s(mesh(n).into_iter().flatten()),
                    usage: wgpu::BufferUsages::VERTEX,
                })
            }),
            instances: std::array::from_fn(|_| {
                device.create_buffer(&wgpu::BufferDescriptor {
                    label: Some("bounded organism instances"),
                    size: (MAX_ORGANISM_SEGMENTS * 48) as u64,
                    usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
                    mapped_at_creation: false,
                })
            }),
            counts: [0; 2],
        }
    }
    pub(crate) fn prepare(
        &mut self,
        queue: &wgpu::Queue,
        camera: CameraFrame,
        matrix: [f32; 16],
        height: u32,
        light: Option<PointLight>,
        segments: &[OrganismSegment],
    ) -> Result<(), RendererError> {
        let invalid = || {
            RendererError::new(
                "invalid organism geometry",
                "finite bounded endpoints, positive radii, linear palette and segment budget required",
            )
        };
        if segments.len() > MAX_ORGANISM_SEGMENTS {
            return Err(invalid());
        }
        let basis = camera_basis(camera).map_err(|e| RendererError::new("organism camera", e))?;
        let mut batches: [Vec<f32>; 2] = [vec![], vec![]];
        for s in segments {
            if !s
                .start_meters
                .iter()
                .chain(&s.end_meters)
                .all(|v| v.is_finite())
                || !s.radius_start_meters.is_finite()
                || !s.radius_end_meters.is_finite()
                || !(1e-6..=1e6).contains(&s.radius_start_meters)
                || !(1e-6..=1e6).contains(&s.radius_end_meters)
                || !s
                    .color
                    .iter()
                    .all(|v| v.is_finite() && (0.0..=1.0).contains(v))
            {
                return Err(invalid());
            }
            let start: [f64; 3] =
                std::array::from_fn(|i| s.start_meters[i] - camera.position_meters[i]);
            let end: [f64; 3] =
                std::array::from_fn(|i| s.end_meters[i] - camera.position_meters[i]);
            let delta: [f64; 3] = std::array::from_fn(|i| end[i] - start[i]);
            let length = delta[0].hypot(delta[1]).hypot(delta[2]);
            if length < 1e-6 || !length.is_finite() {
                return Err(invalid());
            }
            let midpoint: [f64; 3] = std::array::from_fn(|i| (start[i] + end[i]) * 0.5);
            let distance = midpoint[0].hypot(midpoint[1]).hypot(midpoint[2]).max(0.001);
            let radius = s.radius_start_meters.max(s.radius_end_meters);
            let depth: f64 = midpoint
                .iter()
                .zip(basis.forward)
                .map(|(x, f)| x * f64::from(f))
                .sum();
            if depth + length + radius < f64::from(camera.near_plane_meters) {
                continue;
            }
            let pixels = radius / distance * f64::from(height)
                / (2. * f64::from((camera.vertical_fov_radians * 0.5).tan()));
            let lod = usize::from(pixels < 0.75);
            let values = [
                start[0] as f32,
                start[1] as f32,
                start[2] as f32,
                s.radius_start_meters as f32,
                end[0] as f32,
                end[1] as f32,
                end[2] as f32,
                s.radius_end_meters as f32,
                s.color[0],
                s.color[1],
                s.color[2],
                1.,
            ];
            if !values.iter().all(|v| v.is_finite()) {
                return Err(invalid());
            }
            batches[lod].extend(values);
        }
        let relative = light.map_or([0.3, 0.8, 0.4], |l| {
            std::array::from_fn(|i| l.position_meters[i] - camera.position_meters[i])
        });
        let n = relative[0].hypot(relative[1]).hypot(relative[2]);
        let direction = if n > 0. {
            relative.map(|v| (v / n) as f32)
        } else {
            [0., 1., 0.]
        };
        let color = light.map_or([1.; 3], |l| l.color);
        if !direction.iter().all(|v| v.is_finite())
            || !color.iter().all(|v| v.is_finite() && *v >= 0.)
        {
            return Err(invalid());
        }
        queue.write_buffer(
            &self.uniform,
            0,
            &encode_f32s(
                matrix
                    .into_iter()
                    .chain([direction[0], direction[1], direction[2], 0.])
                    .chain([color[0], color[1], color[2], 0.]),
            ),
        );
        for (i, b) in batches.iter().enumerate() {
            self.counts[i] = (b.len() / 12) as u32;
            if !b.is_empty() {
                queue.write_buffer(&self.instances[i], 0, &encode_f32s(b.iter().copied()));
            }
        }
        Ok(())
    }
    pub(crate) fn draw(&self, pass: &mut wgpu::RenderPass<'_>) {
        pass.set_pipeline(&self.pipeline);
        pass.set_bind_group(0, &self.bind, &[]);
        for i in 0..2 {
            if self.counts[i] > 0 {
                pass.set_vertex_buffer(0, self.vertices[i].slice(..));
                pass.set_vertex_buffer(1, self.instances[i].slice(..));
                pass.draw(0..[72, 36][i], 0..self.counts[i]);
            }
        }
    }
    pub(crate) fn draws(&self) -> u32 {
        self.counts.iter().filter(|n| **n > 0).count() as u32
    }
    pub(crate) fn count(&self) -> u32 {
        self.counts.iter().sum()
    }
}

#[cfg(test)]
mod tests;

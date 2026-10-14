//! Immutable patch buffers. Runtime owns selection/generation; renderer owns GPU caching.
use crate::{DEPTH_FORMAT, RendererError, encode_f32s};
use std::{collections::BTreeMap, sync::Arc};
use wgpu::util::DeviceExt;

#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub struct TerrainPatchKey {
    pub body: u8,
    /// Entire versioned definition, not merely its seed.
    pub definition: Vec<u8>,
    pub face: u8,
    pub level: u8,
    pub u: u32,
    pub v: u32,
    /// Coarse boundary fan subdivisions, left/right/bottom/top.
    pub edge_levels: [u8; 4],
}
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct TerrainVertex {
    pub offset_meters: [f32; 3],
    pub normal: [f32; 3],
    pub far_offset_meters: [f32; 3],
}
#[derive(Clone, Debug, PartialEq)]
pub struct TerrainPatch {
    pub key: TerrainPatchKey,
    /// Body-local f64 anchor and absolute body center remain separate until rebasing.
    pub body_center_meters: [f64; 3],
    pub anchor_meters: [f64; 3],
    pub vertices: Vec<TerrainVertex>,
    pub indices: Vec<u32>,
    pub material: crate::SurfaceMaterial,
}
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct TerrainMask {
    pub body_center_meters: [f64; 3],
    pub anchor_meters: [f64; 3],
    pub radius_meters: f64,
}
const MAX_PATCHES: usize = 512;
const MAX_GPU_BYTES: usize = 32 * 1024 * 1024;
const MAX_UPLOAD_BYTES: usize = 2 * 1024 * 1024;
struct GpuPatch {
    vertices: wgpu::Buffer,
    indices: wgpu::Buffer,
    count: u32,
    source: Arc<TerrainPatch>,
    bytes: usize,
}
pub(crate) struct TerrainRenderer {
    pipeline: wgpu::RenderPipeline,
    uniform: wgpu::Buffer,
    bind: wgpu::BindGroup,
    uniform_stride: usize,
    uploaded_bytes: usize,
    peak_upload_bytes: usize,
    fade_body: Option<u8>,
    fade_started: Option<std::time::Instant>,
    fraction: f32,
    cache: BTreeMap<TerrainPatchKey, GpuPatch>,
    active: Vec<TerrainPatchKey>,
}
impl TerrainRenderer {
    pub(crate) fn new(
        device: &wgpu::Device,
        format: wgpu::TextureFormat,
        spheres: &crate::spheres::SphereRenderer,
    ) -> Self {
        let layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("terrain patch uniform layout"),
            entries: &[
                wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::VERTEX_FRAGMENT,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: true,
                        min_binding_size: wgpu::BufferSize::new(160),
                    },
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 1,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Texture {
                        sample_type: wgpu::TextureSampleType::Float { filterable: true },
                        view_dimension: wgpu::TextureViewDimension::D2Array,
                        multisampled: false,
                    },
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 2,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 3,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                    count: None,
                },
            ],
        });
        let shader = device.create_shader_module(wgpu::include_wgsl!("terrain.wgsl"));
        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("terrain patch pipeline layout"),
            bind_group_layouts: &[Some(&layout)],
            immediate_size: 0,
        });
        let attributes = wgpu::vertex_attr_array![0 => Float32x3, 1 => Float32x3, 2 => Float32x3];
        let pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("terrain patches"),
            layout: Some(&pipeline_layout),
            vertex: wgpu::VertexState {
                module: &shader,
                entry_point: Some("vs_main"),
                buffers: &[Some(wgpu::VertexBufferLayout {
                    array_stride: 36,
                    step_mode: wgpu::VertexStepMode::Vertex,
                    attributes: &attributes,
                })],
                compilation_options: Default::default(),
            },
            primitive: wgpu::PrimitiveState {
                cull_mode: None,
                ..Default::default()
            },
            depth_stencil: Some(wgpu::DepthStencilState {
                format: DEPTH_FORMAT,
                depth_write_enabled: Some(true),
                depth_compare: Some(wgpu::CompareFunction::Greater),
                stencil: Default::default(),
                bias: Default::default(),
            }),
            multisample: Default::default(),
            fragment: Some(wgpu::FragmentState {
                module: &shader,
                entry_point: Some("fs_main"),
                targets: &[Some(wgpu::ColorTargetState {
                    format,
                    blend: Some(wgpu::BlendState::REPLACE),
                    write_mask: wgpu::ColorWrites::ALL,
                })],
                compilation_options: Default::default(),
            }),
            multiview_mask: None,
            cache: None,
        });
        let uniform_stride = 160_usize
            .div_ceil(device.limits().min_uniform_buffer_offset_alignment as usize)
            * device.limits().min_uniform_buffer_offset_alignment as usize;
        let uniform = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("bounded terrain uniform arena"),
            size: (uniform_stride * MAX_PATCHES) as u64,
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });
        let bind = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("shared terrain bindings"),
            layout: &layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::Buffer(wgpu::BufferBinding {
                        buffer: &uniform,
                        offset: 0,
                        size: wgpu::BufferSize::new(160),
                    }),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::TextureView(&spheres.material_view),
                },
                wgpu::BindGroupEntry {
                    binding: 2,
                    resource: wgpu::BindingResource::Sampler(&spheres.material_sampler),
                },
                wgpu::BindGroupEntry {
                    binding: 3,
                    resource: wgpu::BindingResource::Sampler(&spheres.detail_sampler),
                },
            ],
        });
        Self {
            pipeline,
            uniform,
            bind,
            uniform_stride,
            uploaded_bytes: 0,
            peak_upload_bytes: 0,
            fade_body: None,
            fade_started: None,
            fraction: 0.0,
            cache: BTreeMap::new(),
            active: Vec::new(),
        }
    }
    pub(crate) fn prepare(
        &mut self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        scene: crate::SceneFrame<'_>,
        matrix: [f32; 16],
    ) -> Result<(), RendererError> {
        let camera = scene.camera;
        let patches = scene.terrain;
        let light = scene.light;
        let mask = scene.terrain_mask;
        let invalid = |s| RendererError::new("terrain patch preparation failed", s);
        if patches.len() > MAX_PATCHES {
            return Err(invalid("active patch budget exceeded"));
        }
        self.active.clear();
        let body = patches.first().map(|p| p.key.body);
        if body != self.fade_body {
            self.fade_body = body;
            self.fade_started = None;
            self.fraction = 0.0;
        }
        if body.is_some() && self.ready(patches) && self.fade_started.is_none() {
            self.fade_started = Some(std::time::Instant::now());
        }
        self.fraction = self.fade_started.map_or(0.0, |start| {
            let t = (start.elapsed().as_secs_f32() / 0.25).clamp(0.0, 1.0);
            t * t * (3.0 - 2.0 * t)
        });

        // Evict only inactive entries, in stable key order, before bounded uploads.
        let wanted: std::collections::BTreeSet<_> = patches.iter().map(|p| p.key.clone()).collect();
        self.cache.retain(|key, _| wanted.contains(key));
        let mut uploaded = patches.len() * self.uniform_stride;
        let mut uniforms = Vec::with_capacity(patches.len() * self.uniform_stride);
        for patch in patches {
            if !self.cache.contains_key(&patch.key) {
                if patch.vertices.is_empty()
                    || patch.vertices.len() > 512
                    || patch.indices.is_empty()
                    || patch.indices.len() > 768 * 3
                    || patch.indices.len() % 3 != 0
                    || patch
                        .indices
                        .iter()
                        .any(|i| *i as usize >= patch.vertices.len())
                    || patch.vertices.iter().any(|v| {
                        v.offset_meters
                            .iter()
                            .chain(&v.normal)
                            .chain(&v.far_offset_meters)
                            .any(|x| !x.is_finite())
                    })
                    || patch
                        .body_center_meters
                        .iter()
                        .chain(&patch.anchor_meters)
                        .any(|v| !v.is_finite())
                {
                    return Err(invalid("invalid patch geometry"));
                }
                let bytes = patch.vertices.len() * 36 + patch.indices.len() * 4;
                if uploaded + bytes > MAX_UPLOAD_BYTES {
                    continue;
                }
                uploaded += bytes;
                if self.bytes() + bytes > MAX_GPU_BYTES {
                    return Err(invalid("GPU cache budget exceeded"));
                }
                let vertices = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
                    label: Some("immutable terrain vertices"),
                    contents: &encode_f32s(patch.vertices.iter().flat_map(|v| {
                        v.offset_meters
                            .into_iter()
                            .chain(v.normal)
                            .chain(v.far_offset_meters)
                    })),
                    usage: wgpu::BufferUsages::VERTEX,
                });
                let indices = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
                    label: Some("immutable terrain indices"),
                    contents: &patch
                        .indices
                        .iter()
                        .flat_map(|i| i.to_le_bytes())
                        .collect::<Vec<_>>(),
                    usage: wgpu::BufferUsages::INDEX,
                });
                self.cache.insert(
                    patch.key.clone(),
                    GpuPatch {
                        vertices,
                        indices,
                        count: patch.indices.len() as u32,
                        source: patch.clone(),
                        bytes,
                    },
                );
            }
            let gpu = &self.cache[&patch.key];
            if !Arc::ptr_eq(&gpu.source, patch)
                && (gpu.source.anchor_meters != patch.anchor_meters
                    || gpu.source.vertices != patch.vertices
                    || gpu.source.indices != patch.indices)
            {
                return Err(invalid("immutable patch key reused with changed geometry"));
            }
            let mask_offset: [f32; 3] = mask.map_or([0.0; 3], |m| {
                std::array::from_fn(|i| (m.anchor_meters[i] - patch.anchor_meters[i]) as f32)
            });
            // Subtract center-camera first, then add the local anchor in f64.
            let offset: [f32; 3] = std::array::from_fn(|i| {
                ((patch.body_center_meters[i] - camera.position_meters[i]) + patch.anchor_meters[i])
                    as f32
            });
            let light_position: [f32; 3] = light.map_or([0.0; 3], |l| {
                std::array::from_fn(|i| {
                    ((l.position_meters[i] - patch.body_center_meters[i]) - patch.anchor_meters[i])
                        as f32
                })
            });
            let light_color = light.map_or([0.0; 3], |l| l.color);
            if offset
                .iter()
                .chain(&light_position)
                .chain(&light_color)
                .any(|v| !v.is_finite())
                || light_color.iter().any(|v| *v < 0.0)
            {
                return Err(invalid("patch camera/light exceeds finite GPU range"));
            }
            let record = encode_f32s(
                matrix
                    .into_iter()
                    .chain([offset[0], offset[1], offset[2], 0.0])
                    .chain([
                        patch.material as u8 as f32,
                        light_color[0],
                        light_color[1],
                        light_color[2],
                    ])
                    .chain([light_position[0], light_position[1], light_position[2], 0.0])
                    .chain([mask_offset[0], mask_offset[1], mask_offset[2], 4096.0])
                    .chain([
                        patch.anchor_meters[0] as f32,
                        patch.anchor_meters[1] as f32,
                        patch.anchor_meters[2] as f32,
                        self.fraction,
                    ])
                    .chain([
                        patch.anchor_meters[0].rem_euclid(128.0) as f32,
                        patch.anchor_meters[1].rem_euclid(128.0) as f32,
                        patch.anchor_meters[2].rem_euclid(128.0) as f32,
                        128.0,
                    ]),
            );
            uniforms.extend(record);
            uniforms.resize((self.active.len() + 1) * self.uniform_stride, 0);
            self.active.push(patch.key.clone());
        }
        if !uniforms.is_empty() {
            queue.write_buffer(&self.uniform, 0, &uniforms);
        }
        self.uploaded_bytes = uploaded;
        self.peak_upload_bytes = self.peak_upload_bytes.max(uploaded);
        Ok(())
    }
    pub(crate) fn draw(&self, pass: &mut wgpu::RenderPass<'_>) {
        pass.set_pipeline(&self.pipeline);
        for (slot, key) in self.active.iter().enumerate() {
            let p = &self.cache[key];
            pass.set_bind_group(0, &self.bind, &[(slot * self.uniform_stride) as u32]);
            pass.set_vertex_buffer(0, p.vertices.slice(..));
            pass.set_index_buffer(p.indices.slice(..), wgpu::IndexFormat::Uint32);
            pass.draw_indexed(0..p.count, 0, 0..1);
        }
    }
    pub(crate) fn presentation_ready(&self, patches: &[Arc<TerrainPatch>]) -> bool {
        self.ready(patches) && (patches.is_empty() || self.fraction >= 1.0)
    }
    pub(crate) fn uploads(&self) -> (usize, usize) {
        (self.uploaded_bytes, self.peak_upload_bytes)
    }
    pub(crate) fn count(&self) -> u32 {
        self.active.len() as u32
    }
    pub(crate) fn ready(&self, patches: &[Arc<TerrainPatch>]) -> bool {
        patches.iter().all(|p| self.cache.contains_key(&p.key))
    }
    pub(crate) fn bytes(&self) -> usize {
        self.uniform_stride * MAX_PATCHES + self.cache.values().map(|p| p.bytes).sum::<usize>()
    }
}

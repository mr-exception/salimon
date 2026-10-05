//! Static authored resource meshes, batched into one opaque depth-tested draw.
use crate::{CameraFrame, DEPTH_FORMAT, RendererError, encode_f32s};
const FLOATS: usize = 7;
const ASSETS: [&[u8]; 14] = [
    include_bytes!("../../assets/resources/water-ice-fragment-shard/model.glb"),
    include_bytes!("../../assets/resources/water-ice-fragment-cluster/model.glb"),
    include_bytes!("../../assets/resources/silicate-fragment-slab/model.glb"),
    include_bytes!("../../assets/resources/silicate-fragment-ridge/model.glb"),
    include_bytes!("../../assets/resources/iron-fragment/model.glb"),
    include_bytes!("../../assets/resources/iron-fragment-shard/model.glb"),
    include_bytes!("../../assets/resources/water-ice-deposit-spire/model.glb"),
    include_bytes!("../../assets/resources/water-ice-deposit-crown/model.glb"),
    include_bytes!("../../assets/resources/water-ice-deposit-ridge/model.glb"),
    include_bytes!("../../assets/resources/water-ice-deposit-shelf/model.glb"),
    include_bytes!("../../assets/resources/silicate-deposit-boulder/model.glb"),
    include_bytes!("../../assets/resources/silicate-deposit-slab/model.glb"),
    include_bytes!("../../assets/resources/silicate-deposit-ridge/model.glb"),
    include_bytes!("../../assets/resources/silicate-deposit-scree/model.glb"),
];
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ResourceMesh {
    IceShard,
    IceCluster,
    SilicateSlab,
    SilicateRidge,
    IronChunk,
    IronShard,
    IceDepositSpire,
    IceDepositCrown,
    IceDepositRidge,
    IceDepositShelf,
    SilicateDepositBoulder,
    SilicateDepositSlab,
    SilicateDepositRidge,
    SilicateDepositScree,
}
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ResourceMeshInstance {
    pub mesh: ResourceMesh,
    pub center_meters: [f64; 3],
    /// Uniform visual scale; runtime inscribes deposits in their authoritative sphere.
    /// Fragments use their authoritative bounding cube side.
    pub side_meters: f64,
}
fn geometry(bytes: &[u8]) -> Result<Vec<[f32; FLOATS]>, RendererError> {
    let fail = |message: &str| RendererError::new("load resource mesh", message);
    let asset =
        gltf::Gltf::from_slice(bytes).map_err(|e| RendererError::new("load resource mesh", e))?;
    let blob = asset
        .blob
        .as_deref()
        .ok_or_else(|| fail("missing binary chunk"))?;
    let mut vertices = Vec::new();
    for node in asset.nodes() {
        if node.transform().decomposed() != ([0.0; 3], [0.0, 0.0, 0.0, 1.0], [1.0; 3]) {
            return Err(fail("expected baked identity node transforms"));
        }
        let Some(mesh) = node.mesh() else { continue };
        for primitive in mesh.primitives() {
            if primitive.mode() != gltf::mesh::Mode::Triangles {
                return Err(fail("expected triangles"));
            }
            let reader = primitive.reader(|b| (b.index() == 0).then_some(blob));
            let positions: Vec<_> = reader
                .read_positions()
                .ok_or_else(|| fail("missing positions"))?
                .collect();
            let normals: Vec<_> = reader
                .read_normals()
                .ok_or_else(|| fail("missing normals"))?
                .collect();
            let indices: Vec<u32> = reader.read_indices().map_or_else(
                || (0..positions.len() as u32).collect(),
                |v| v.into_u32().collect(),
            );
            let material = primitive.material();
            if material.alpha_mode() != gltf::material::AlphaMode::Opaque {
                return Err(fail("expected opaque material"));
            }
            let color = material.pbr_metallic_roughness().base_color_factor();
            for index in indices {
                let p = positions
                    .get(index as usize)
                    .ok_or_else(|| fail("invalid position index"))?;
                let n = normals
                    .get(index as usize)
                    .ok_or_else(|| fail("invalid normal index"))?;
                if p.iter().any(|v| !v.is_finite() || v.abs() > 0.48)
                    || n.iter().any(|v| !v.is_finite())
                {
                    return Err(fail("geometry outside centered unit cube"));
                }
                // A cheap fixed local fill emphasizes the authored planar facets.
                let shade = 0.65 + 0.35 * (n[0] * 0.3 + n[1] * 0.8 + n[2] * 0.5).clamp(0.0, 1.0);
                vertices.push([
                    p[0],
                    p[1],
                    p[2],
                    color[0] * shade,
                    color[1] * shade,
                    color[2] * shade,
                    1.0,
                ]);
            }
        }
    }
    if vertices.is_empty() {
        return Err(fail("empty geometry"));
    }
    Ok(vertices)
}
fn relative_vertices(
    meshes: &[Vec<[f32; FLOATS]>; 14],
    instances: &[ResourceMeshInstance],
    camera: CameraFrame,
) -> Result<Vec<f32>, RendererError> {
    let mut output = Vec::new();
    for instance in instances {
        if !instance.side_meters.is_finite()
            || instance.side_meters <= 0.0
            || instance.center_meters.iter().any(|v| !v.is_finite())
        {
            return Err(RendererError::new(
                "prepare resource mesh",
                "invalid pose or side",
            ));
        }
        let mesh = match instance.mesh {
            ResourceMesh::IceShard => 0,
            ResourceMesh::IceCluster => 1,
            ResourceMesh::SilicateSlab => 2,
            ResourceMesh::SilicateRidge => 3,
            ResourceMesh::IronChunk => 4,
            ResourceMesh::IronShard => 5,
            ResourceMesh::IceDepositSpire => 6,
            ResourceMesh::IceDepositCrown => 7,
            ResourceMesh::IceDepositRidge => 8,
            ResourceMesh::IceDepositShelf => 9,
            ResourceMesh::SilicateDepositBoulder => 10,
            ResourceMesh::SilicateDepositSlab => 11,
            ResourceMesh::SilicateDepositRidge => 12,
            ResourceMesh::SilicateDepositScree => 13,
        };
        for vertex in &meshes[mesh] {
            for (axis, value) in vertex.iter().take(3).enumerate() {
                output.push(
                    ((instance.center_meters[axis] - camera.position_meters[axis])
                        + f64::from(*value) * instance.side_meters) as f32,
                );
            }
            output.extend_from_slice(&vertex[3..]);
        }
    }
    Ok(output)
}
pub(crate) struct ResourceMeshRenderer {
    pipeline: wgpu::RenderPipeline,
    uniform: wgpu::Buffer,
    binding: wgpu::BindGroup,
    vertices: wgpu::Buffer,
    meshes: [Vec<[f32; FLOATS]>; 14],
    capacity: usize,
    count: u32,
    objects: u32,
}
impl ResourceMeshRenderer {
    pub(crate) fn new(
        device: &wgpu::Device,
        format: wgpu::TextureFormat,
    ) -> Result<Self, RendererError> {
        let meshes = [
            geometry(ASSETS[0])?,
            geometry(ASSETS[1])?,
            geometry(ASSETS[2])?,
            geometry(ASSETS[3])?,
            geometry(ASSETS[4])?,
            geometry(ASSETS[5])?,
            geometry(ASSETS[6])?,
            geometry(ASSETS[7])?,
            geometry(ASSETS[8])?,
            geometry(ASSETS[9])?,
            geometry(ASSETS[10])?,
            geometry(ASSETS[11])?,
            geometry(ASSETS[12])?,
            geometry(ASSETS[13])?,
        ];
        let vertices = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("Resource vertices"),
            size: 28,
            usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });
        let uniform = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("Resource camera"),
            size: 64,
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });
        let layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("Resource mesh bindings"),
            entries: &[wgpu::BindGroupLayoutEntry {
                binding: 0,
                visibility: wgpu::ShaderStages::VERTEX,
                ty: wgpu::BindingType::Buffer {
                    ty: wgpu::BufferBindingType::Uniform,
                    has_dynamic_offset: false,
                    min_binding_size: wgpu::BufferSize::new(64),
                },
                count: None,
            }],
        });
        let binding = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("Resource mesh binding"),
            layout: &layout,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: uniform.as_entire_binding(),
            }],
        });
        let shader = device.create_shader_module(wgpu::include_wgsl!("resource_mesh.wgsl"));
        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("Resource mesh layout"),
            bind_group_layouts: &[Some(&layout)],
            immediate_size: 0,
        });
        let attributes = wgpu::vertex_attr_array![
            0 => Float32x3, 1 => Float32x4
        ];
        let pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("Resource mesh pipeline"),
            layout: Some(&pipeline_layout),
            vertex: wgpu::VertexState {
                module: &shader,
                entry_point: Some("vs_main"),
                buffers: &[Some(wgpu::VertexBufferLayout {
                    array_stride: (FLOATS * size_of::<f32>()) as u64,
                    step_mode: wgpu::VertexStepMode::Vertex,
                    attributes: &attributes,
                })],
                compilation_options: Default::default(),
            },
            primitive: Default::default(),
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
                    blend: None,
                    write_mask: wgpu::ColorWrites::ALL,
                })],
                compilation_options: Default::default(),
            }),
            multiview_mask: None,
            cache: None,
        });
        Ok(Self {
            pipeline,
            uniform,
            binding,
            vertices,
            meshes,
            capacity: 1,
            count: 0,
            objects: 0,
        })
    }
    pub(crate) fn prepare(
        &mut self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        camera: CameraFrame,
        projection: [f32; 16],
        instances: &[ResourceMeshInstance],
    ) -> Result<(), RendererError> {
        let vertices = relative_vertices(&self.meshes, instances, camera)?;
        self.count = u32::try_from(vertices.len() / FLOATS)
            .map_err(|e| RendererError::new("resource vertex count", e))?;
        self.objects = u32::try_from(instances.len())
            .map_err(|e| RendererError::new("resource object count", e))?;
        if vertices.len() > self.capacity {
            self.capacity = vertices.len().next_power_of_two();
            self.vertices = device.create_buffer(&wgpu::BufferDescriptor {
                label: Some("Resource vertices"),
                size: (self.capacity * 4) as u64,
                usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
                mapped_at_creation: false,
            });
        }
        queue.write_buffer(&self.uniform, 0, &encode_f32s(projection));
        if !vertices.is_empty() {
            queue.write_buffer(&self.vertices, 0, &encode_f32s(vertices));
        }
        Ok(())
    }
    pub(crate) fn draw<'a>(&'a self, pass: &mut wgpu::RenderPass<'a>) {
        if self.count > 0 {
            pass.set_pipeline(&self.pipeline);
            pass.set_bind_group(0, &self.binding, &[]);
            pass.set_vertex_buffer(0, self.vertices.slice(..));
            pass.draw(0..self.count, 0..1);
        }
    }
    pub(crate) fn object_count(&self) -> u32 {
        self.objects
    }
    pub(crate) fn draw_count(&self) -> u32 {
        u32::from(self.count > 0)
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn authored_exports_are_distinct_and_bounded() {
        let meshes = ASSETS.map(|asset| geometry(asset).unwrap());
        for left in 0..meshes.len() {
            for right in left + 1..meshes.len() {
                assert_ne!(meshes[left], meshes[right]);
            }
        }
        for mesh in meshes {
            assert!(mesh.len() / 3 <= 240);
            assert!(mesh.iter().all(|v| v[..3].iter().all(|p| p.abs() <= 0.48)));
        }
    }
    #[test]
    fn subtracts_large_origin_before_narrowing() {
        let meshes = ASSETS.map(|asset| geometry(asset).unwrap());
        let camera = CameraFrame {
            position_meters: [1e12; 3],
            target_meters: [1e12, 1e12, 1e12 - 1.0],
            up: [0.0, 1.0, 0.0],
            vertical_fov_radians: 1.0,
            near_plane_meters: 0.1,
        };
        for mesh in [
            ResourceMesh::IceShard,
            ResourceMesh::IceCluster,
            ResourceMesh::SilicateSlab,
            ResourceMesh::SilicateRidge,
            ResourceMesh::IronChunk,
            ResourceMesh::IronShard,
            ResourceMesh::IceDepositSpire,
            ResourceMesh::IceDepositCrown,
            ResourceMesh::IceDepositRidge,
            ResourceMesh::IceDepositShelf,
            ResourceMesh::SilicateDepositBoulder,
            ResourceMesh::SilicateDepositSlab,
            ResourceMesh::SilicateDepositRidge,
            ResourceMesh::SilicateDepositScree,
        ] {
            for side in [0.01, 0.5, 3.0] {
                let instance = ResourceMeshInstance {
                    mesh,
                    center_meters: [1e12; 3],
                    side_meters: side,
                };
                let vertices = relative_vertices(&meshes, &[instance], camera).unwrap();
                assert!(
                    vertices
                        .as_chunks::<FLOATS>()
                        .0
                        .iter()
                        .all(|v| v[..3].iter().all(|p| f64::from(p.abs()) <= side * 0.481))
                );
            }
        }
    }
}

#[cfg(test)]
mod gpu_tests {
    use super::*;

    fn block_on<F: std::future::Future>(future: F) -> F::Output {
        let mut future = std::pin::pin!(future);
        let mut context = std::task::Context::from_waker(std::task::Waker::noop());
        let deadline = std::time::Instant::now() + std::time::Duration::from_secs(30);
        loop {
            match future.as_mut().poll(&mut context) {
                std::task::Poll::Ready(value) => return value,
                std::task::Poll::Pending => {
                    assert!(
                        std::time::Instant::now() < deadline,
                        "GPU initialization timed out"
                    );
                    std::thread::yield_now();
                }
            }
        }
    }

    #[test]
    #[ignore = "requires a GPU backend; run explicitly with Vulkan/lavapipe or native GPU"]
    fn headless_resource_pipeline() {
        let instance = wgpu::Instance::default();
        let adapter = block_on(instance.request_adapter(&wgpu::RequestAdapterOptions::default()))
            .expect("GPU verification requires an adapter");
        let (device, _) = block_on(adapter.request_device(&wgpu::DeviceDescriptor::default()))
            .expect("GPU verification requires a device");
        ResourceMeshRenderer::new(&device, wgpu::TextureFormat::Rgba8UnormSrgb)
            .expect("resource pipeline and shaders must validate on the GPU");
        device.poll(wgpu::PollType::wait_indefinitely()).unwrap();
    }
}

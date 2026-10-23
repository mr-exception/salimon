//! Resident authored geometry with one opaque instanced draw per visible variant.
use crate::{CameraFrame, DEPTH_FORMAT, RendererError, encode_f32s};
use wgpu::util::DeviceExt;
/// Resource batch measurements; upload time is CPU enqueue/encoding, not GPU transfer.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct ResourceBatchStats {
    pub prepare_time: std::time::Duration,
    pub upload_time: std::time::Duration,
    pub upload_bytes: u64,
    pub triangles: u32,
    /// Retained CPU instance staging and last-uploaded byte capacities.
    pub scratch_capacity_bytes: u64,
}
#[cfg(test)]
#[path = "resource_mesh/sphere_benchmark.rs"]
mod sphere_benchmark;
/// Radius of the checked-in spherical mesh at unit scale, in authored meters.
pub const SPHERE_AUTHORED_RADIUS_METERS: f64 = 0.48;
const FLOATS: usize = 7;
const ASSETS: [&[u8]; 19] = [
    // Legacy variants remain for historical geometry benchmarks/contact proxies.
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
    include_bytes!("../../assets/resources/iron-deposit-nodule/model.glb"),
    include_bytes!("../../assets/resources/iron-deposit-vein/model.glb"),
    include_bytes!("../../assets/resources/iron-deposit-ledge/model.glb"),
    include_bytes!("../../assets/resources/iron-deposit-rubble/model.glb"),
    include_bytes!("../../assets/resources/spherical-fragment/model.glb"),
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
    IronDepositNodule,
    IronDepositVein,
    IronDepositLedge,
    IronDepositRubble,
    Sphere,
}
impl ResourceMesh {
    fn index(self) -> usize {
        self as usize
    }

    /// Distance below the authored origin along a unit surface normal, at unit scale.
    /// Runtime uses this geometry query for terrain/deck placement, never GPU data.
    pub fn support_meters(self, up: [f64; 3]) -> f64 {
        self.unit_vertices()
            .iter()
            .map(|vertex| -(0..3).map(|i| vertex[i] * up[i]).sum::<f64>())
            .fold(0.0, f64::max)
    }
    /// Immutable CPU positions from the same validated triangles as the GPU batch.
    pub fn unit_vertices(self) -> &'static [[f64; 3]] {
        static MESHES: std::sync::LazyLock<[Vec<[f64; 3]>; 19]> = std::sync::LazyLock::new(|| {
            ASSETS.map(|bytes| {
                geometry(bytes)
                    .expect("validated checked-in geometry")
                    .into_iter()
                    .map(|v| [f64::from(v[0]), f64::from(v[1]), f64::from(v[2])])
                    .collect()
            })
        });
        &MESHES[self.index()]
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ResourceMeshInstance {
    pub mesh: ResourceMesh,
    /// Linear opaque material tint; shared geometry remains immutable.
    pub tint: [f32; 4],
    pub center_meters: [f64; 3],
    /// Dimensionless multiplier of authored mesh coordinates (not a diameter).
    /// Spheres use analytic radius / SPHERE_AUTHORED_RADIUS_METERS.
    pub authored_mesh_scale: f64,
    pub orientation_xyzw: [f64; 4],
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
#[cfg(test)]
fn relative_vertices(
    meshes: &[Vec<[f32; FLOATS]>; 19],
    instances: &[ResourceMeshInstance],
    camera: CameraFrame,
) -> Result<Vec<f32>, RendererError> {
    let mut output = Vec::new();
    for instance in instances {
        if !instance.authored_mesh_scale.is_finite()
            || instance.authored_mesh_scale <= 0.0
            || instance
                .tint
                .iter()
                .any(|v| !v.is_finite() || !(0.0..=1.0).contains(v))
            || instance.tint[3] != 1.0
            || instance.center_meters.iter().any(|v| !v.is_finite())
            || instance.orientation_xyzw.iter().any(|v| !v.is_finite())
            || (instance.orientation_xyzw.iter().map(|v| v * v).sum::<f64>() - 1.0).abs() > 1e-9
        {
            return Err(RendererError::new(
                "prepare resource mesh",
                "invalid pose or side",
            ));
        }
        let mesh = instance.mesh.index();
        for vertex in &meshes[mesh] {
            let q = instance.orientation_xyzw;
            let p = [
                f64::from(vertex[0]),
                f64::from(vertex[1]),
                f64::from(vertex[2]),
            ];
            let v = [q[0], q[1], q[2]];
            let cross = |a: [f64; 3], b: [f64; 3]| {
                [
                    a[1] * b[2] - a[2] * b[1],
                    a[2] * b[0] - a[0] * b[2],
                    a[0] * b[1] - a[1] * b[0],
                ]
            };
            let t = cross(v, [p[0], p[1], p[2]]).map(|v| v * 2.0);
            let c = cross(v, t);
            for axis in 0..3 {
                let rotated = p[axis] + q[3] * t[axis] + c[axis];
                output.push(
                    ((instance.center_meters[axis] - camera.position_meters[axis])
                        + rotated * instance.authored_mesh_scale) as f32,
                );
            }
            output.extend((0..4).map(|i| vertex[3 + i] * instance.tint[i]));
        }
    }
    Ok(output)
}
// Three vec4 attributes: camera-relative center/scale, quaternion, linear tint.
const INSTANCE_BYTES: usize = 48;

#[derive(Default)]
struct InstanceBatch {
    staging: Vec<u8>,
    uploaded: Vec<u8>,
}

fn prepare_instances(
    batches: &mut [InstanceBatch; 19],
    instances: &[ResourceMeshInstance],
    camera: CameraFrame,
    aspect_ratio: f32,
) -> Result<(), RendererError> {
    let basis =
        crate::camera_basis(camera).map_err(|e| RendererError::new("resource camera", e))?;
    for batch in batches.iter_mut() {
        batch.staging.clear();
    }
    for instance in instances {
        if !instance.authored_mesh_scale.is_finite()
            || instance.authored_mesh_scale <= 0.0
            || instance
                .tint
                .iter()
                .any(|v| !v.is_finite() || !(0.0..=1.0).contains(v))
            || instance.tint[3] != 1.0
            || instance.center_meters.iter().any(|v| !v.is_finite())
            || instance.orientation_xyzw.iter().any(|v| !v.is_finite())
            || (instance.orientation_xyzw.iter().map(|v| v * v).sum::<f64>() - 1.0).abs() > 1e-9
        {
            return Err(RendererError::new(
                "prepare resource mesh",
                "invalid pose or side",
            ));
        }
        // Subtract the universe-scale origin before narrowing. Camera/ship motion
        // changes only these compact records, never the immutable mesh buffers.
        let center = std::array::from_fn::<_, 3, _>(|i| {
            (instance.center_meters[i] - camera.position_meters[i]) as f32
        });
        let side = instance.authored_mesh_scale as f32;
        if center.iter().any(|v| !v.is_finite()) || !side.is_finite() || side <= 0.0 {
            return Err(RendererError::new(
                "prepare resource mesh",
                "relative pose outside f32 range",
            ));
        }
        // The authored ±0.48 cube fits this sphere at every orientation.
        // Inflate slightly for f32 transform and frustum-plane rounding.
        if !crate::instance_intersects_frustum(
            crate::GpuInstance {
                orientation: [0., 0., 0., 1.],
                relative_center: center,
                half_extents: [side * 0.481; 3],
                color: [1.0; 4],
            },
            basis,
            camera,
            aspect_ratio,
        ) {
            continue;
        }
        let batch = &mut batches[instance.mesh.index()];
        for value in center
            .into_iter()
            .chain([side])
            .chain(instance.orientation_xyzw.map(|v| v as f32))
            .chain(instance.tint)
        {
            batch.staging.extend_from_slice(&value.to_le_bytes());
        }
    }
    Ok(())
}

// Deduplicate position/material vertices once, preserving triangle order and
// facet color boundaries. Indices address the concatenated immutable stream.
fn indexed_geometry(
    meshes: &[Vec<[f32; FLOATS]>; 19],
) -> (Vec<f32>, Vec<u32>, [std::ops::Range<u32>; 19]) {
    let mut vertices = Vec::new();
    let mut indices = Vec::new();
    let ranges = std::array::from_fn(|mesh| {
        let start = indices.len() as u32;
        let mut unique = std::collections::BTreeMap::new();
        for vertex in &meshes[mesh] {
            let key = vertex.map(f32::to_bits);
            let index = *unique.entry(key).or_insert_with(|| {
                let index = (vertices.len() / FLOATS) as u32;
                vertices.extend_from_slice(vertex);
                index
            });
            indices.push(index);
        }
        start..indices.len() as u32
    });
    (vertices, indices, ranges)
}

pub(crate) struct ResourceMeshRenderer {
    pipeline: wgpu::RenderPipeline,
    uniform: wgpu::Buffer,
    binding: wgpu::BindGroup,
    vertices: wgpu::Buffer,
    indices: wgpu::Buffer,
    ranges: [std::ops::Range<u32>; 19],
    batches: [InstanceBatch; 19],
    instance_buffers: [wgpu::Buffer; 19],
    capacities: [usize; 19],
    objects: u32,
}
impl ResourceMeshRenderer {
    pub(crate) fn new(
        device: &wgpu::Device,
        format: wgpu::TextureFormat,
    ) -> Result<Self, RendererError> {
        let mut meshes: [Vec<[f32; FLOATS]>; 19] = std::array::from_fn(|_| Vec::new());
        for (mesh, asset) in meshes.iter_mut().zip(ASSETS) {
            *mesh = geometry(asset)?;
        }
        let (vertex_data, index_data, ranges) = indexed_geometry(&meshes);
        let vertices = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Resident resource vertices"),
            contents: &encode_f32s(vertex_data),
            usage: wgpu::BufferUsages::VERTEX,
        });
        let index_bytes: Vec<_> = index_data.iter().flat_map(|v| v.to_le_bytes()).collect();
        let indices = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Resident resource indices"),
            contents: &index_bytes,
            usage: wgpu::BufferUsages::INDEX,
        });
        let instance_buffers = std::array::from_fn(|_| instance_buffer(device, INSTANCE_BYTES));
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
        let instance_attributes =
            wgpu::vertex_attr_array![2 => Float32x4, 3 => Float32x4, 4 => Float32x4];
        let pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("Resource mesh pipeline"),
            layout: Some(&pipeline_layout),
            vertex: wgpu::VertexState {
                module: &shader,
                entry_point: Some("vs_main"),
                buffers: &[
                    Some(wgpu::VertexBufferLayout {
                        array_stride: (FLOATS * size_of::<f32>()) as u64,
                        step_mode: wgpu::VertexStepMode::Vertex,
                        attributes: &attributes,
                    }),
                    Some(wgpu::VertexBufferLayout {
                        array_stride: INSTANCE_BYTES as u64,
                        step_mode: wgpu::VertexStepMode::Instance,
                        attributes: &instance_attributes,
                    }),
                ],
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
            indices,
            ranges,
            batches: std::array::from_fn(|_| InstanceBatch::default()),
            instance_buffers,
            capacities: [INSTANCE_BYTES; 19],
            objects: 0,
        })
    }
    pub(crate) fn prepare(
        &mut self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        camera: CameraFrame,
        projection: [f32; 16],
        aspect_ratio: f32,
        instances: &[ResourceMeshInstance],
    ) -> Result<ResourceBatchStats, RendererError> {
        let started = std::time::Instant::now();
        prepare_instances(&mut self.batches, instances, camera, aspect_ratio)?;
        let mut objects = 0usize;
        let mut triangles = 0u64;
        for (i, batch) in self.batches.iter().enumerate() {
            let count = batch.staging.len() / INSTANCE_BYTES;
            objects += count;
            triangles += count as u64 * u64::from(self.ranges[i].len() as u32 / 3);
            if batch.staging.len() > self.capacities[i] {
                self.capacities[i] = batch.staging.len().next_power_of_two();
                self.instance_buffers[i] = instance_buffer(device, self.capacities[i]);
            }
        }
        self.objects =
            u32::try_from(objects).map_err(|e| RendererError::new("resource object count", e))?;
        let triangles = u32::try_from(triangles)
            .map_err(|e| RendererError::new("resource triangle count", e))?;
        let prepare_time = started.elapsed();
        let upload_started = std::time::Instant::now();
        queue.write_buffer(&self.uniform, 0, &encode_f32s(projection));
        let mut upload_bytes = 64;
        for (i, batch) in self.batches.iter_mut().enumerate() {
            if batch.staging != batch.uploaded {
                if !batch.staging.is_empty() {
                    queue.write_buffer(&self.instance_buffers[i], 0, &batch.staging);
                    upload_bytes += batch.staging.len() as u64;
                }
                batch.uploaded.clear();
                batch.uploaded.extend_from_slice(&batch.staging);
            }
        }
        Ok(ResourceBatchStats {
            prepare_time,
            upload_time: upload_started.elapsed(),
            upload_bytes,
            triangles,
            scratch_capacity_bytes: self
                .batches
                .iter()
                .map(|b| (b.staging.capacity() + b.uploaded.capacity()) as u64)
                .sum(),
        })
    }
    pub(crate) fn draw<'a>(&'a self, pass: &mut wgpu::RenderPass<'a>) {
        if self.objects == 0 {
            return;
        }
        pass.set_pipeline(&self.pipeline);
        pass.set_bind_group(0, &self.binding, &[]);
        pass.set_vertex_buffer(0, self.vertices.slice(..));
        pass.set_index_buffer(self.indices.slice(..), wgpu::IndexFormat::Uint32);
        for (i, batch) in self.batches.iter().enumerate() {
            let count = (batch.staging.len() / INSTANCE_BYTES) as u32;
            if count > 0 {
                pass.set_vertex_buffer(1, self.instance_buffers[i].slice(..));
                pass.draw_indexed(self.ranges[i].clone(), 0, 0..count);
            }
        }
    }
    pub(crate) fn object_count(&self) -> u32 {
        self.objects
    }
    pub(crate) fn draw_count(&self) -> u32 {
        self.batches
            .iter()
            .filter(|b| !b.staging.is_empty())
            .count() as u32
    }
}

fn instance_buffer(device: &wgpu::Device, bytes: usize) -> wgpu::Buffer {
    device.create_buffer(&wgpu::BufferDescriptor {
        label: Some("Resource instance transforms"),
        size: bytes as u64,
        usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
        mapped_at_creation: false,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    #[ignore = "run with SALIMON_GLB_CANDIDATES after offline GLB processing"]
    fn offline_glb_candidate_resources_match_imported_vertices() {
        let names = [
            "water-ice-fragment-shard",
            "water-ice-fragment-cluster",
            "silicate-fragment-slab",
            "silicate-fragment-ridge",
            "iron-fragment",
            "iron-fragment-shard",
            "water-ice-deposit-spire",
            "water-ice-deposit-crown",
            "water-ice-deposit-ridge",
            "water-ice-deposit-shelf",
            "silicate-deposit-boulder",
            "silicate-deposit-slab",
            "silicate-deposit-ridge",
            "silicate-deposit-scree",
            "iron-deposit-nodule",
            "iron-deposit-vein",
            "iron-deposit-ledge",
            "iron-deposit-rubble",
        ];
        for (original, name) in ASSETS.iter().zip(names) {
            let bytes = crate::glb_candidate_tests::candidate(&format!("resource.{name}"));
            assert_eq!(
                geometry(original).unwrap(),
                geometry(&bytes).unwrap(),
                "{name}"
            );
        }
    }

    fn test_camera() -> CameraFrame {
        CameraFrame {
            position_meters: [1e12; 3],
            target_meters: [1e12, 1e12, 1e12 - 1.0],
            up: [0.0, 1.0, 0.0],
            vertical_fov_radians: 1.0,
            near_plane_meters: 0.1,
        }
    }

    fn decode_record(bytes: &[u8]) -> [f32; 8] {
        std::array::from_fn(|i| f32::from_le_bytes(bytes[i * 4..i * 4 + 4].try_into().unwrap()))
    }

    fn shader_position(vertex: [f32; FLOATS], record: [f32; 8]) -> [f32; 3] {
        let p = [vertex[0], vertex[1], vertex[2]];
        let q = [record[4], record[5], record[6]];
        let t = crate::cross(q, p).map(|v| 2.0 * v);
        let c = crate::cross(q, t);
        std::array::from_fn(|i| record[i] + (p[i] + record[7] * t[i] + c[i]) * record[3])
    }

    #[test]
    fn indexed_variants_and_gpu_transforms_preserve_all_authored_triangles() {
        let meshes = ASSETS.map(|asset| geometry(asset).unwrap());
        let (vertices, indices, ranges) = indexed_geometry(&meshes);
        let camera = test_camera();
        for (i, mesh) in meshes.iter().enumerate() {
            let decoded: Vec<_> = indices[ranges[i].start as usize..ranges[i].end as usize]
                .iter()
                .map(|v| vertices[*v as usize * FLOATS..(*v as usize + 1) * FLOATS].to_vec())
                .collect();
            assert_eq!(decoded, mesh.iter().map(|v| v.to_vec()).collect::<Vec<_>>());
            for side in [0.01, 0.5, 3.0] {
                for orientation_xyzw in [
                    [0.0, 0.0, 0.0, 1.0],
                    [0.5, -0.5, 0.5, 0.5],
                    [0.0, 0.0, 1.0, 0.0],
                ] {
                    let instance = ResourceMeshInstance {
                        tint: [1.0; 4],
                        mesh: ALL_MESHES[i],
                        center_meters: [1e12 + 0.125, 1e12 - 0.25, 1e12 - 4.0],
                        authored_mesh_scale: side,
                        orientation_xyzw,
                    };
                    let old = relative_vertices(&meshes, &[instance], camera).unwrap();
                    let mut batches = std::array::from_fn(|_| InstanceBatch::default());
                    prepare_instances(&mut batches, &[instance], camera, 1.0).unwrap();
                    let record = decode_record(&batches[i].staging);
                    assert_eq!(&record[..3], &[0.125, -0.25, -4.0]);
                    for (v, old) in mesh.iter().zip(old.as_chunks::<FLOATS>().0) {
                        let new = shader_position(*v, record);
                        assert!((0..3).all(|axis| (new[axis] - old[axis]).abs() < 1e-6));
                        assert_eq!(&v[3..], &old[3..]);
                    }
                }
            }
        }
    }

    pub(super) const ALL_MESHES: [ResourceMesh; 19] = [
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
        ResourceMesh::IronDepositNodule,
        ResourceMesh::IronDepositVein,
        ResourceMesh::IronDepositLedge,
        ResourceMesh::IronDepositRubble,
        ResourceMesh::Sphere,
    ];

    #[test]
    fn checked_in_sphere_vertices_match_the_authored_radius_contract() {
        for vertex in ResourceMesh::Sphere.unit_vertices() {
            let radius = vertex.iter().map(|v| v * v).sum::<f64>().sqrt();
            // GLB positions are f32; allow 0.1 micrometer export rounding.
            assert!((radius - SPHERE_AUTHORED_RADIUS_METERS).abs() < 1e-7);
        }
    }

    #[test]
    fn shared_sphere_batches_different_resource_tints_without_mesh_duplication() {
        let camera = test_camera();
        let mut instances: Vec<_> = [
            [0.32, 0.68, 0.88, 1.0],
            [0.45, 0.37, 0.28, 1.0],
            [0.34, 0.29, 0.24, 1.0],
        ]
        .into_iter()
        .map(|tint| ResourceMeshInstance {
            mesh: ResourceMesh::Sphere,
            tint,
            center_meters: [1e12, 1e12, 1e12 - 4.0],
            authored_mesh_scale: 0.2,
            orientation_xyzw: [0.0, 0.0, 0.0, 1.0],
        })
        .collect();
        let mut batches = std::array::from_fn(|_| InstanceBatch::default());
        prepare_instances(&mut batches, &instances, camera, 1.0).unwrap();
        assert_eq!(batches.iter().filter(|b| !b.staging.is_empty()).count(), 1);
        let bytes = &batches[ResourceMesh::Sphere.index()].staging;
        assert_eq!(bytes.len(), 3 * INSTANCE_BYTES);
        for (record, instance) in bytes.as_chunks::<INSTANCE_BYTES>().0.iter().zip(&instances) {
            let tint = std::array::from_fn::<_, 4, _>(|i| {
                f32::from_le_bytes(record[32 + i * 4..36 + i * 4].try_into().unwrap())
            });
            assert_eq!(tint, instance.tint);
        }
        instances[0].tint[0] = f32::NAN;
        assert!(prepare_instances(&mut batches, &instances, camera, 1.0).is_err());
    }

    #[test]
    fn culling_keeps_rotated_near_plane_and_side_edges_and_never_mutates_inputs() {
        let camera = test_camera();
        let base = ResourceMeshInstance {
            tint: [1.0; 4],
            mesh: ResourceMesh::IronChunk,
            center_meters: [1e12, 1e12, 1e12 - 0.11],
            authored_mesh_scale: 0.5,
            orientation_xyzw: [0.5; 4],
        };
        let cases = [
            (base, true),
            (
                ResourceMeshInstance {
                    center_meters: [1e12, 1e12, 1e12 + 0.1],
                    ..base
                },
                true,
            ),
            (
                ResourceMeshInstance {
                    center_meters: [1e12, 1e12, 1e12 + 2.0],
                    ..base
                },
                false,
            ),
            (
                ResourceMeshInstance {
                    center_meters: [1e12 + 1.2, 1e12, 1e12 - 2.0],
                    ..base
                },
                true,
            ),
            (
                ResourceMeshInstance {
                    center_meters: [1e12 + 4.0, 1e12, 1e12 - 2.0],
                    ..base
                },
                false,
            ),
        ];
        let mut batches = std::array::from_fn(|_| InstanceBatch::default());
        for (instance, retained) in cases {
            let original = instance;
            prepare_instances(&mut batches, &[instance], camera, 1.0).unwrap();
            assert_eq!(
                batches[base.mesh.index()].staging.len(),
                usize::from(retained) * INSTANCE_BYTES
            );
            assert_eq!(instance, original);
        }
        // Validation applies even to culled instances.
        assert!(
            prepare_instances(
                &mut batches,
                &[ResourceMeshInstance {
                    center_meters: [1e12, 1e12, 1e12 + 2.0],
                    orientation_xyzw: [0.0; 4],
                    ..base
                }],
                camera,
                1.0
            )
            .is_err()
        );
    }

    #[test]
    fn instance_staging_reuses_capacity_and_changes_only_affected_variant() {
        let camera = test_camera();
        let mut instances: Vec<_> = (0..1000)
            .map(|i| ResourceMeshInstance {
                tint: [1.0; 4],
                mesh: ALL_MESHES[i % 6],
                center_meters: [1e12, 1e12, 1e12 - 4.0],
                authored_mesh_scale: 0.1,
                orientation_xyzw: [0.0, 0.0, 0.0, 1.0],
            })
            .collect();
        let mut batches: [InstanceBatch; 19] = std::array::from_fn(|_| InstanceBatch::default());
        prepare_instances(&mut batches, &instances, camera, 1.0).unwrap();
        for batch in &mut batches {
            batch.uploaded.clone_from(&batch.staging);
        }
        let capacities = batches.each_ref().map(|b| b.staging.capacity());
        prepare_instances(&mut batches, &instances, camera, 1.0).unwrap();
        assert!(batches.iter().all(|b| b.staging == b.uploaded));
        assert_eq!(capacities, batches.each_ref().map(|b| b.staging.capacity()));
        assert_eq!(
            batches.iter().map(|b| b.staging.len()).sum::<usize>(),
            1000 * INSTANCE_BYTES
        );
        instances[0].center_meters[0] += 0.125;
        prepare_instances(&mut batches, &instances, camera, 1.0).unwrap();
        assert_eq!(
            batches.iter().filter(|b| b.staging != b.uploaded).count(),
            1
        );
        let away = CameraFrame {
            target_meters: [1e12, 1e12, 1e12 + 1.0],
            ..camera
        };
        prepare_instances(&mut batches, &instances, away, 1.0).unwrap();
        assert!(batches.iter().all(|b| b.staging.is_empty()));
    }

    #[test]
    #[ignore = "opt-in release CPU preparation benchmark; no hardware FPS assertion"]
    fn resource_preparation_benchmark() {
        use std::{hint::black_box, time::Instant};
        let meshes = ASSETS.map(|asset| geometry(asset).unwrap());
        let camera = test_camera();
        for count in [500, 1000] {
            for active in [false, true] {
                for away in [false, true] {
                    let mut instances: Vec<_> = (0..count)
                        .map(|i| ResourceMeshInstance {
                            tint: [1.0; 4],
                            mesh: ALL_MESHES[i % 6],
                            center_meters: [1e12 + (i % 10) as f64 * 0.1, 1e12, 1e12 - 4.0],
                            authored_mesh_scale: 0.1,
                            orientation_xyzw: [0.5; 4],
                        })
                        .collect();
                    let camera = CameraFrame {
                        target_meters: [1e12, 1e12, 1e12 + if away { 1.0 } else { -1.0 }],
                        ..camera
                    };
                    let mut batches = std::array::from_fn(|_| InstanceBatch::default());
                    let mut old_samples = Vec::new();
                    let mut new_samples = Vec::new();
                    let mut old_bytes = 0;
                    for sample in 0..250 {
                        if active {
                            for instance in &mut instances {
                                instance.center_meters[0] += 0.000244140625;
                                let angle = sample as f64 * 0.001;
                                instance.orientation_xyzw = [0.0, angle.sin(), 0.0, angle.cos()];
                            }
                        }
                        let start = Instant::now();
                        let old = black_box(
                            relative_vertices(black_box(&meshes), black_box(&instances), camera)
                                .unwrap(),
                        );
                        let old_time = start.elapsed().as_nanos();
                        old_bytes = old.len() * 4 + 64;
                        let start = Instant::now();
                        prepare_instances(
                            black_box(&mut batches),
                            black_box(&instances),
                            camera,
                            1.0,
                        )
                        .unwrap();
                        black_box(&batches);
                        let new_time = start.elapsed().as_nanos();
                        if sample >= 50 {
                            old_samples.push(old_time);
                            new_samples.push(new_time);
                        }
                    }
                    old_samples.sort_unstable();
                    new_samples.sort_unstable();
                    let new_bytes = 64 + batches.iter().map(|b| b.staging.len()).sum::<usize>();
                    println!(
                        "count={count} state={} camera={} samples=200 old_median_ns={} old_p95_ns={} new_median_ns={} new_p95_ns={} old_upload_bytes={old_bytes} max_new_upload_bytes={new_bytes}",
                        if active {
                            "active-transforms"
                        } else {
                            "settled-transforms"
                        },
                        if away { "away" } else { "facing" },
                        old_samples[100],
                        old_samples[189],
                        new_samples[100],
                        new_samples[189]
                    );
                }
            }
        }
    }

    #[test]
    #[ignore = "opt-in authored geometry inventory; no GPU timing"]
    fn resource_geometry_inventory() {
        let meshes = ASSETS.map(|asset| geometry(asset).unwrap());
        let (_, indices, ranges) = indexed_geometry(&meshes);
        println!("mesh,triangle_corners,indexed_vertices,triangles");
        for (i, mesh) in ALL_MESHES.iter().enumerate() {
            let unique: std::collections::BTreeSet<_> = indices
                [ranges[i].start as usize..ranges[i].end as usize]
                .iter()
                .collect();
            println!(
                "{mesh:?},{},{},{}",
                meshes[i].len(),
                unique.len(),
                meshes[i].len() / 3
            );
        }
    }

    #[test]
    fn every_authored_variant_enlarges_actual_extents_and_reports_ground_support() {
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
            ResourceMesh::IronDepositNodule,
            ResourceMesh::IronDepositVein,
            ResourceMesh::IronDepositLedge,
            ResourceMesh::IronDepositRubble,
            ResourceMesh::Sphere,
        ] {
            let instance = ResourceMeshInstance {
                tint: [1.0; 4],
                mesh,
                center_meters: [1e12; 3],
                authored_mesh_scale: 0.1,
                orientation_xyzw: [0.0, 0.0, 0.0, 1.0],
            };
            let before = relative_vertices(&meshes, &[instance], camera).unwrap();
            let after = relative_vertices(
                &meshes,
                &[ResourceMeshInstance {
                    authored_mesh_scale: 1.0,
                    ..instance
                }],
                camera,
            )
            .unwrap();
            for axis in 0..3 {
                let extent = |vertices: &[f32]| {
                    let values = vertices.as_chunks::<FLOATS>().0;
                    let min = values.iter().map(|v| v[axis]).fold(f32::INFINITY, f32::min);
                    let max = values
                        .iter()
                        .map(|v| v[axis])
                        .fold(f32::NEG_INFINITY, f32::max);
                    f64::from(max - min)
                };
                assert!(extent(&before) > 0.0);
                assert!((extent(&after) / extent(&before) - 10.0).abs() < 0.00001);
            }
            for up in [
                [0.0, 1.0, 0.0],
                [0.0, 0.0, 1.0],
                [1.0, 0.0, 0.0],
                [1.0 / 3.0_f64.sqrt(); 3],
            ] {
                let support = mesh.support_meters(up);
                let vertices = geometry(ASSETS[mesh.index()]).unwrap();
                let heights: Vec<_> = vertices
                    .iter()
                    .map(|v| support + (0..3).map(|i| f64::from(v[i]) * up[i]).sum::<f64>())
                    .collect();
                assert!(heights.iter().all(|h| *h >= -1e-12));
                assert!(heights.iter().any(|h| h.abs() < 1e-12));
            }
        }
    }

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
            ResourceMesh::IronDepositNodule,
            ResourceMesh::IronDepositVein,
            ResourceMesh::IronDepositLedge,
            ResourceMesh::IronDepositRubble,
            ResourceMesh::Sphere,
        ] {
            for side in [0.01, 0.5, 3.0] {
                let instance = ResourceMeshInstance {
                    tint: [1.0; 4],
                    mesh,
                    center_meters: [1e12; 3],
                    authored_mesh_scale: side,
                    orientation_xyzw: [0.0, 0.0, 0.0, 1.0],
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

    mod geometry_evaluation;

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

    fn capture(
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        renderer: &ResourceMeshRenderer,
    ) -> Vec<u8> {
        let size = wgpu::Extent3d {
            width: 256,
            height: 256,
            depth_or_array_layers: 1,
        };
        let texture = device.create_texture(&wgpu::TextureDescriptor {
            label: Some("Resource parity image"),
            size,
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Rgba8UnormSrgb,
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::COPY_SRC,
            view_formats: &[],
        });
        let view = texture.create_view(&Default::default());
        let depth = crate::DepthTarget::new(device, 256, 256);
        let readback = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("Resource parity pixels"),
            size: 256 * 256 * 4,
            usage: wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::MAP_READ,
            mapped_at_creation: false,
        });
        let mut encoder = device.create_command_encoder(&Default::default());
        {
            let mut pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &view,
                    depth_slice: None,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(wgpu::Color::BLACK),
                        store: wgpu::StoreOp::Store,
                    },
                })],
                depth_stencil_attachment: Some(wgpu::RenderPassDepthStencilAttachment {
                    view: &depth.view,
                    depth_ops: Some(wgpu::Operations {
                        load: wgpu::LoadOp::Clear(0.0),
                        store: wgpu::StoreOp::Store,
                    }),
                    stencil_ops: None,
                }),
                ..Default::default()
            });
            renderer.draw(&mut pass);
        }
        encoder.copy_texture_to_buffer(
            wgpu::TexelCopyTextureInfo {
                texture: &texture,
                mip_level: 0,
                origin: wgpu::Origin3d::ZERO,
                aspect: wgpu::TextureAspect::All,
            },
            wgpu::TexelCopyBufferInfo {
                buffer: &readback,
                layout: wgpu::TexelCopyBufferLayout {
                    offset: 0,
                    bytes_per_row: Some(1024),
                    rows_per_image: Some(256),
                },
            },
            size,
        );
        queue.submit([encoder.finish()]);
        let (send, receive) = std::sync::mpsc::channel();
        readback.map_async(wgpu::MapMode::Read, .., move |result| {
            send.send(result).unwrap()
        });
        device.poll(wgpu::PollType::wait_indefinitely()).unwrap();
        receive.recv().unwrap().unwrap();
        let pixels = readback.get_mapped_range(..).unwrap().to_vec();
        readback.unmap();
        pixels
    }

    #[test]
    #[ignore = "requires GPU; compares resident indexed raster to pre-instancing CPU expansion"]
    fn headless_resource_raster_parity() {
        let gpu = wgpu::Instance::default();
        let adapter = block_on(gpu.request_adapter(&Default::default())).unwrap();
        let (device, queue) = block_on(adapter.request_device(&Default::default())).unwrap();
        let camera = CameraFrame {
            position_meters: [1e12; 3],
            target_meters: [1e12, 1e12, 1e12 - 1.0],
            up: [0.0, 1.0, 0.0],
            vertical_fov_radians: 1.0,
            near_plane_meters: 0.1,
        };
        let projection = crate::multiply_mat4(
            crate::infinite_reverse_z_projection(camera, 1.0).unwrap(),
            crate::view_rotation(crate::camera_basis(camera).unwrap()),
        );
        let instances: Vec<_> = tests::ALL_MESHES
            .iter()
            .enumerate()
            .map(|(i, mesh)| ResourceMeshInstance {
                tint: if *mesh == ResourceMesh::Sphere {
                    [0.25, 0.85, 1.0, 1.0]
                } else {
                    [1.0; 4]
                },
                mesh: *mesh,
                center_meters: [
                    1e12 + (i % 6) as f64 * 0.65 - 1.625,
                    1e12 + (i / 6) as f64 * 0.7 - 0.7,
                    1e12 - 4.0,
                ],
                authored_mesh_scale: 0.6,
                orientation_xyzw: if i % 2 == 0 {
                    [0.5; 4]
                } else {
                    [0.0, 0.0, 0.0, 1.0]
                },
            })
            .collect();
        let mut resident =
            ResourceMeshRenderer::new(&device, wgpu::TextureFormat::Rgba8UnormSrgb).unwrap();
        resident
            .prepare(&device, &queue, camera, projection, 1.0, &instances)
            .unwrap();
        assert_eq!(resident.object_count(), 19);
        let mut reference =
            ResourceMeshRenderer::new(&device, wgpu::TextureFormat::Rgba8UnormSrgb).unwrap();
        let meshes = ASSETS.map(|asset| geometry(asset).unwrap());
        let expanded = relative_vertices(&meshes, &instances, camera).unwrap();
        let index_bytes: Vec<_> = (0..(expanded.len() / FLOATS) as u32)
            .flat_map(u32::to_le_bytes)
            .collect();
        reference.vertices = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Pre-instancing reference vertices"),
            contents: &encode_f32s(expanded),
            usage: wgpu::BufferUsages::VERTEX,
        });
        reference.indices = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Pre-instancing reference indices"),
            contents: &index_bytes,
            usage: wgpu::BufferUsages::INDEX,
        });
        let identity_instances: Vec<_> = instances
            .iter()
            .map(|v| ResourceMeshInstance {
                tint: [1.0; 4],
                mesh: v.mesh,
                center_meters: camera.position_meters,
                authored_mesh_scale: 1.0,
                orientation_xyzw: [0.0, 0.0, 0.0, 1.0],
            })
            .collect();
        reference
            .prepare(
                &device,
                &queue,
                camera,
                projection,
                1.0,
                &identity_instances,
            )
            .unwrap();
        let actual = capture(&device, &queue, &resident);
        let expected = capture(&device, &queue, &reference);
        let colored = actual
            .as_chunks::<4>()
            .0
            .iter()
            .filter(|p| p[..3] != [0, 0, 0])
            .count();
        assert!(
            colored > 1000,
            "must rasterize real geometry rather than blank images"
        );
        // Sub-ULP arithmetic changes can move a few triangle-edge pixels; retain
        // a tight image coverage bound across native/software backends.
        let differing = actual
            .as_chunks::<4>()
            .0
            .iter()
            .zip(expected.as_chunks::<4>().0.iter())
            .filter(|(a, b)| a != b)
            .count();
        assert!(
            differing <= 32,
            "raster parity differs at {differing} of 65536 pixels"
        );
        println!("resource raster: colored={colored} differing={differing}/65536");
        if let Ok(directory) = std::env::var("SALIMON_RESOURCE_EVIDENCE_DIR") {
            std::fs::create_dir_all(&directory).unwrap();
            for (name, pixels) in [("resident.png", actual), ("cpu-reference.png", expected)] {
                let file =
                    std::fs::File::create(std::path::Path::new(&directory).join(name)).unwrap();
                let mut encoder = png::Encoder::new(file, 256, 256);
                encoder.set_color(png::ColorType::Rgba);
                encoder.set_depth(png::BitDepth::Eight);
                encoder
                    .write_header()
                    .unwrap()
                    .write_image_data(&pixels)
                    .unwrap();
            }
        }
    }
    #[test]
    #[ignore = "requires a GPU backend; run explicitly with Vulkan/lavapipe or native GPU"]
    fn headless_resource_pipeline() {
        let instance = wgpu::Instance::default();
        let adapter = block_on(instance.request_adapter(&wgpu::RequestAdapterOptions::default()))
            .expect("GPU verification requires an adapter");
        let (device, queue) = block_on(adapter.request_device(&wgpu::DeviceDescriptor::default()))
            .expect("GPU verification requires a device");
        let mut renderer = ResourceMeshRenderer::new(&device, wgpu::TextureFormat::Rgba8UnormSrgb)
            .expect("resource pipeline and shaders must validate on the GPU");
        let camera = CameraFrame {
            position_meters: [1e12; 3],
            target_meters: [1e12, 1e12, 1e12 - 1.0],
            up: [0.0, 1.0, 0.0],
            vertical_fov_radians: 1.0,
            near_plane_meters: 0.1,
        };
        let projection = crate::multiply_mat4(
            crate::infinite_reverse_z_projection(camera, 1.0).unwrap(),
            crate::view_rotation(crate::camera_basis(camera).unwrap()),
        );
        let instances: Vec<_> = tests::ALL_MESHES
            .iter()
            .cycle()
            .take(1000)
            .map(|mesh| ResourceMeshInstance {
                tint: [1.0; 4],
                mesh: *mesh,
                center_meters: [1e12, 1e12, 1e12 - 4.0],
                authored_mesh_scale: 0.1,
                orientation_xyzw: [0.5; 4],
            })
            .collect();
        let first = renderer
            .prepare(&device, &queue, camera, projection, 1.0, &instances)
            .unwrap();
        assert_eq!(first.upload_bytes, 64 + 1000 * INSTANCE_BYTES as u64);
        assert_eq!(renderer.object_count(), 1000);
        assert_eq!(renderer.draw_count(), 19);
        let unchanged = renderer
            .prepare(&device, &queue, camera, projection, 1.0, &instances)
            .unwrap();
        assert_eq!(unchanged.upload_bytes, 64);
        let turned = CameraFrame {
            target_meters: [1e12 + 0.125, 1e12, 1e12 - 1.0],
            ..camera
        };
        assert_eq!(
            renderer
                .prepare(&device, &queue, turned, projection, 1.0, &instances)
                .unwrap()
                .upload_bytes,
            64
        );
        let shifted = CameraFrame {
            position_meters: [1e12 + 0.125, 1e12, 1e12],
            target_meters: [1e12 + 0.125, 1e12, 1e12 - 1.0],
            ..camera
        };
        assert_eq!(
            renderer
                .prepare(&device, &queue, shifted, projection, 1.0, &instances)
                .unwrap()
                .upload_bytes,
            first.upload_bytes
        );
        let texture = device.create_texture(&wgpu::TextureDescriptor {
            label: Some("Resource instancing verification"),
            size: wgpu::Extent3d {
                width: 64,
                height: 64,
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Rgba8UnormSrgb,
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            view_formats: &[],
        });
        let view = texture.create_view(&Default::default());
        let depth = crate::DepthTarget::new(&device, 64, 64);
        let mut encoder = device.create_command_encoder(&Default::default());
        {
            let mut pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("Resource indexed instance verification"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &view,
                    depth_slice: None,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(wgpu::Color::BLACK),
                        store: wgpu::StoreOp::Store,
                    },
                })],
                depth_stencil_attachment: Some(wgpu::RenderPassDepthStencilAttachment {
                    view: &depth.view,
                    depth_ops: Some(wgpu::Operations {
                        load: wgpu::LoadOp::Clear(0.0),
                        store: wgpu::StoreOp::Store,
                    }),
                    stencil_ops: None,
                }),
                ..Default::default()
            });
            renderer.draw(&mut pass);
        }
        queue.submit([encoder.finish()]);
        let away = CameraFrame {
            target_meters: [1e12, 1e12, 1e12 + 1.0],
            ..camera
        };
        assert_eq!(
            renderer
                .prepare(&device, &queue, away, projection, 1.0, &instances)
                .unwrap()
                .upload_bytes,
            64
        );
        assert_eq!(renderer.object_count(), 0);
        assert_eq!(renderer.draw_count(), 0);
        assert_eq!(
            renderer
                .prepare(&device, &queue, camera, projection, 1.0, &instances)
                .unwrap()
                .upload_bytes,
            first.upload_bytes
        );
        device.poll(wgpu::PollType::wait_indefinitely()).unwrap();
    }
    #[test]
    #[ignore = "requires GPU; 500 shared spheres, unchanged upload cache and repeat pixels"]
    fn headless_five_hundred_shared_spheres() {
        let gpu = wgpu::Instance::default();
        let adapter = block_on(gpu.request_adapter(&Default::default())).unwrap();
        println!("500-sphere adapter={:?}", adapter.get_info());
        let (device, queue) = block_on(adapter.request_device(&Default::default())).unwrap();
        let camera = sphere_benchmark::camera();
        let projection = crate::multiply_mat4(
            crate::infinite_reverse_z_projection(camera, 1.0).unwrap(),
            crate::view_rotation(crate::camera_basis(camera).unwrap()),
        );
        let instances = sphere_benchmark::workload(500, false, false);
        let mut renderer =
            ResourceMeshRenderer::new(&device, wgpu::TextureFormat::Rgba8UnormSrgb).unwrap();
        let initial = renderer
            .prepare(&device, &queue, camera, projection, 1.0, &instances)
            .unwrap();
        assert_eq!(renderer.object_count(), 500);
        assert_eq!(renderer.draw_count(), 1);
        assert_eq!(initial.triangles, 40_000);
        assert_eq!(initial.upload_bytes, 64 + 500 * 48);
        let first = capture(&device, &queue, &renderer);
        assert!(
            first
                .as_chunks::<4>()
                .0
                .iter()
                .filter(|p| p[..3] != [0, 0, 0])
                .count()
                > 1000
        );
        let unchanged = renderer
            .prepare(&device, &queue, camera, projection, 1.0, &instances)
            .unwrap();
        assert_eq!(unchanged.upload_bytes, 64);
        assert_eq!(first, capture(&device, &queue, &renderer));
        println!(
            "500 spheres: triangles={} draws=1 first_upload={} unchanged_upload={} retained_cpu_bytes={}",
            initial.triangles,
            initial.upload_bytes,
            unchanged.upload_bytes,
            initial.scratch_capacity_bytes
        );
    }

    #[test]
    #[ignore = "requires GPU; shared sphere palette and repeat stability"]
    fn headless_resource_sphere_palette() {
        let gpu = wgpu::Instance::default();
        let adapter = block_on(gpu.request_adapter(&Default::default())).unwrap();
        println!("sphere palette adapter={:?}", adapter.get_info());
        let (device, queue) = block_on(adapter.request_device(&Default::default())).unwrap();
        let camera = CameraFrame {
            position_meters: [1e12; 3],
            target_meters: [1e12, 1e12, 1e12 - 1.0],
            up: [0.0, 1.0, 0.0],
            vertical_fov_radians: 1.0,
            near_plane_meters: 0.1,
        };
        let projection = crate::multiply_mat4(
            crate::infinite_reverse_z_projection(camera, 1.0).unwrap(),
            crate::view_rotation(crate::camera_basis(camera).unwrap()),
        );
        let instances: Vec<_> = [
            [0.25, 0.85, 1.0, 1.0],
            [0.45, 0.50, 0.36, 1.0],
            [0.75, 0.22, 0.06, 1.0],
        ]
        .into_iter()
        .enumerate()
        .map(|(i, tint)| ResourceMeshInstance {
            mesh: ResourceMesh::Sphere,
            tint,
            center_meters: [1e12 + (i as f64 - 1.0) * 0.8, 1e12, 1e12 - 2.5],
            authored_mesh_scale: 0.7,
            orientation_xyzw: [0.0, 0.0, 0.0, 1.0],
        })
        .collect();
        let mut renderer =
            ResourceMeshRenderer::new(&device, wgpu::TextureFormat::Rgba8UnormSrgb).unwrap();
        renderer
            .prepare(&device, &queue, camera, projection, 1.0, &instances)
            .unwrap();
        assert_eq!(renderer.draw_count(), 1);
        assert_eq!(renderer.object_count(), 3);
        let first = capture(&device, &queue, &renderer);
        assert_eq!(
            first,
            capture(&device, &queue, &renderer),
            "frozen sphere scene must not flicker"
        );
        let pixels = first.as_chunks::<4>().0;
        for region in 0..3 {
            let count = pixels
                .iter()
                .enumerate()
                .filter(|(i, p)| {
                    let x = i % 256;
                    p[..3] != [0, 0, 0] && x >= region * 85 && x < (region + 1) * 85
                })
                .count();
            assert!(
                count > 100,
                "palette region {region} must render real geometry"
            );
        }
        assert!(
            pixels
                .iter()
                .any(|p| u16::from(p[2]) > u16::from(p[0]) * 3 / 2)
        );
        assert!(pixels.iter().any(|p| p[1] > p[0] && p[1] > p[2]));
        assert!(
            pixels
                .iter()
                .any(|p| u16::from(p[0]) > u16::from(p[1]) * 3 / 2)
        );
        if let Ok(directory) = std::env::var("SALIMON_RESOURCE_EVIDENCE_DIR") {
            std::fs::create_dir_all(&directory).unwrap();
            let file = std::fs::File::create(
                std::path::Path::new(&directory).join("spherical-palette.png"),
            )
            .unwrap();
            let mut encoder = png::Encoder::new(file, 256, 256);
            encoder.set_color(png::ColorType::Rgba);
            encoder.set_depth(png::BitDepth::Eight);
            encoder
                .write_header()
                .unwrap()
                .write_image_data(&first)
                .unwrap();
        }
    }
}

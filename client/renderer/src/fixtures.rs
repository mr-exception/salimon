//! Reusable opaque interior templates. Identity/parent/pose arrive as observation;
//! this module never fabricates physical objects or derives collision authority.
use crate::{CameraFrame, DEPTH_FORMAT, RendererError, encode_f32s};
use wgpu::util::DeviceExt;
const FLOATS: usize = 10;
const TEMPLATE_COUNT: usize = 15;
pub const FIXTURE_LAYOUT_VERSION: u32 = 1;
const SHIP: &[u8] = crate::ship_mesh::SHIP_GLB;
const CABIN: &[u8] = include_bytes!("../../assets/fixtures/scout-habitation-kit/model.glb");
const KIT: &[u8] = include_bytes!("../../assets/fixtures/scout-interior-kit/model.glb");

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum FixtureMesh {
    Bunk,
    Shelf,
    Seat,
    Panel,
    PanelDamaged,
    Table,
    ScreenPanel,
    Shutter,
    LockerShell,
    LockerDoor,
    SeatBase,
    Harness,
    SeatPad,
    SeatBack,
    ShutterHandle,
}
impl FixtureMesh {
    fn index(self) -> usize {
        self as usize
    }
}
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct FixturePose {
    pub position_meters: [f64; 3],
    pub orientation_xyzw: [f64; 4],
    pub scale: f64,
}
impl Default for FixturePose {
    fn default() -> Self {
        Self {
            position_meters: [0.; 3],
            orientation_xyzw: [0., 0., 0., 1.],
            scale: 1.,
        }
    }
}
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct FixtureNode {
    pub id: u64,
    /// None attaches to the supplied ship/room root. Other IDs are local parents.
    pub parent: Option<u64>,
    pub mesh: FixtureMesh,
    pub local: FixturePose,
    pub tint: [f32; 3],
    pub powered: bool,
}
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct FixtureInstance {
    pub id: u64,
    pub mesh: FixtureMesh,
    pub pose: FixturePose,
    pub tint: [f32; 3],
    pub powered: bool,
}
fn valid_pose(p: FixturePose) -> bool {
    p.position_meters
        .iter()
        .chain(p.orientation_xyzw.iter())
        .all(|v| v.is_finite())
        && p.scale.is_finite()
        && p.scale > 0.
        && (p.orientation_xyzw.iter().map(|v| v * v).sum::<f64>() - 1.).abs() <= 1e-9
}
fn rotate(q: [f64; 4], p: [f64; 3]) -> [f64; 3] {
    let [x, y, z, w] = q;
    let t = [
        2. * (y * p[2] - z * p[1]),
        2. * (z * p[0] - x * p[2]),
        2. * (x * p[1] - y * p[0]),
    ];
    [
        p[0] + w * t[0] + y * t[2] - z * t[1],
        p[1] + w * t[1] + z * t[0] - x * t[2],
        p[2] + w * t[2] + x * t[1] - y * t[0],
    ]
}
fn compose(parent: FixturePose, local: FixturePose) -> FixturePose {
    let offset = rotate(
        parent.orientation_xyzw,
        local.position_meters.map(|v| v * parent.scale),
    );
    let [x, y, z, w] = parent.orientation_xyzw;
    let [a, b, c, d] = local.orientation_xyzw;
    FixturePose {
        position_meters: std::array::from_fn(|i| parent.position_meters[i] + offset[i]),
        orientation_xyzw: [
            w * a + x * d + y * c - z * b,
            w * b - x * c + y * d + z * a,
            w * c + x * b - y * a + z * d,
            w * d - x * a - y * b - z * c,
        ],
        scale: parent.scale * local.scale,
    }
}
/// Resolve independent room/fixture/accessory transforms in f64, preserving input
/// identity/order. Missing parents, cycles, duplicates and invalid poses fail.
pub fn resolve_fixture_hierarchy(
    root: FixturePose,
    nodes: &[FixtureNode],
) -> Result<Vec<FixtureInstance>, RendererError> {
    let fail = || {
        RendererError::new(
            "fixture hierarchy",
            "invalid root, identity, parent or pose",
        )
    };
    if !valid_pose(root) {
        return Err(fail());
    }
    let mut ids = std::collections::HashMap::new();
    for (i, n) in nodes.iter().enumerate() {
        if ids.insert(n.id, i).is_some()
            || !valid_pose(n.local)
            || n.tint
                .iter()
                .any(|v| !v.is_finite() || !(0.0..=4.0).contains(v))
        {
            return Err(fail());
        }
    }
    let mut poses = vec![None; nodes.len()];
    let mut children = vec![Vec::new(); nodes.len()];
    let mut ready = std::collections::VecDeque::new();
    for (i, n) in nodes.iter().enumerate() {
        if let Some(id) = n.parent {
            children[*ids.get(&id).ok_or_else(fail)?].push(i);
        } else {
            poses[i] = Some(compose(root, n.local));
            ready.push_back(i);
        }
    }
    let mut resolved = 0;
    while let Some(i) = ready.pop_front() {
        let pose = poses[i].expect("ready parent");
        if !valid_pose(pose) {
            return Err(fail());
        }
        resolved += 1;
        for &child in &children[i] {
            poses[child] = Some(compose(pose, nodes[child].local));
            ready.push_back(child);
        }
    }
    if resolved != nodes.len() {
        return Err(fail());
    }
    Ok(nodes
        .iter()
        .zip(poses)
        .map(|(n, p)| FixtureInstance {
            id: n.id,
            mesh: n.mesh,
            pose: p.expect("resolved"),
            tint: n.tint,
            powered: n.powered,
        })
        .collect())
}
/// Read-only authoring audit. Gameplay collision still comes from its physical owner.
pub fn fixture_template_bounds(mesh: FixtureMesh) -> Result<[f64; 6], RendererError> {
    let mut b = [
        f64::INFINITY,
        f64::NEG_INFINITY,
        f64::INFINITY,
        f64::NEG_INFINITY,
        f64::INFINITY,
        f64::NEG_INFINITY,
    ];
    for v in geometry(mesh.index())? {
        for i in 0..3 {
            b[2 * i] = b[2 * i].min(f64::from(v[i]));
            b[2 * i + 1] = b[2 * i + 1].max(f64::from(v[i]));
        }
    }
    Ok(b)
}

fn geometry(index: usize) -> Result<Vec<[f32; FLOATS]>, RendererError> {
    let (bytes, names, pivot): (&[u8], &[&str], [f64; 3]) = match index {
        0 => (
            SHIP,
            &["Starter_BunkFrame", "Starter_BunkCushion"],
            crate::habitation_geometry::BUNK_HINGE_METERS,
        ),
        1 => (KIT, &["Shelf"], [0.; 3]),
        2 => (KIT, &["Seat"], [0.; 3]),
        3 => (KIT, &["Panel"], [0.; 3]),
        4 => (KIT, &["PanelDamaged"], [0.; 3]),
        5 => (CABIN, &["Table"], [0.; 3]),
        6 => (CABIN, &["ScreenPanel"], [0.; 3]),
        7 => (CABIN, &["Shutter"], [0.; 3]),
        8 => (CABIN, &["LockerShell"], [0.; 3]),
        9 => (CABIN, &["LockerDoor"], [0.; 3]),
        10 => (CABIN, &["SeatBase"], [0.; 3]),
        11 => (CABIN, &["Harness"], [0.; 3]),
        12 => (CABIN, &["SeatPad"], [0.; 3]),
        13 => (CABIN, &["SeatBack"], [0.; 3]),
        14 => (CABIN, &["ShutterHandle"], [0.; 3]),
        _ => unreachable!(),
    };
    named_geometry(bytes, names, pivot)
}
fn named_geometry(
    bytes: &[u8],
    names: &[&str],
    pivot: [f64; 3],
) -> Result<Vec<[f32; FLOATS]>, RendererError> {
    let fail = |s: &str| RendererError::new("fixture asset", s);
    let asset =
        gltf::Gltf::from_slice(bytes).map_err(|e| RendererError::new("fixture asset", e))?;
    let blob = asset
        .blob
        .as_deref()
        .ok_or_else(|| fail("missing binary chunk"))?;
    let nodes: Vec<_> = asset.nodes().collect();
    let mut parents = vec![None; nodes.len()];
    for n in &nodes {
        for child in n.children() {
            if parents[child.index()].replace(n.index()).is_some() {
                return Err(fail("ambiguous mesh parent"));
            }
        }
    }
    let mut output = Vec::new();
    let mut found = 0;
    for node in asset
        .nodes()
        .filter(|n| n.name().is_some_and(|s| names.contains(&s)))
    {
        let mut ancestor = Some(node.index());
        let mut depth = 0;
        while let Some(index) = ancestor {
            depth += 1;
            if depth > nodes.len()
                || nodes[index].transform().decomposed() != ([0.; 3], [0., 0., 0., 1.], [1.; 3])
            {
                return Err(fail("expected acyclic baked mesh/parent transforms"));
            }
            ancestor = parents[index];
        }
        found += 1;
        let mesh = node
            .mesh()
            .ok_or_else(|| fail("missing named template mesh"))?;
        for primitive in mesh.primitives() {
            if primitive.mode() != gltf::mesh::Mode::Triangles
                || primitive.material().alpha_mode() != gltf::material::AlphaMode::Opaque
            {
                return Err(fail("opaque triangles required"));
            }
            let reader = primitive.reader(|b| (b.index() == 0).then_some(blob));
            let p: Vec<_> = reader
                .read_positions()
                .ok_or_else(|| fail("positions missing"))?
                .collect();
            let n: Vec<_> = reader
                .read_normals()
                .ok_or_else(|| fail("normals missing"))?
                .collect();
            let indices: Vec<_> = reader
                .read_indices()
                .map_or_else(|| (0..p.len() as u32).collect(), |v| v.into_u32().collect());
            let color = primitive
                .material()
                .pbr_metallic_roughness()
                .base_color_factor();
            for i in indices {
                let p = *p
                    .get(i as usize)
                    .ok_or_else(|| fail("bad position index"))?;
                let n = *n.get(i as usize).ok_or_else(|| fail("bad normal index"))?;
                if p.iter()
                    .chain(n.iter())
                    .chain(color.iter())
                    .any(|v| !v.is_finite())
                {
                    return Err(fail("nonfinite geometry"));
                }
                // Same bounded Scout cabin fill as the original opaque path.
                let smooth = |a: f32, b: f32, x: f32| {
                    let t = ((x - a) / (b - a)).clamp(0., 1.);
                    t * t * (3. - 2. * t)
                };
                let task = (1. - smooth(1., 5., (p[0] - 4.).abs()))
                    .max(1. - smooth(1., 4., (p[0] + 3.).abs()));
                output.push([
                    (f64::from(p[0]) - pivot[0]) as f32,
                    (f64::from(p[1]) - pivot[1]) as f32,
                    (f64::from(p[2]) - pivot[2]) as f32,
                    color[0],
                    color[1],
                    color[2],
                    1.,
                    0.46 + 0.18 * task + 0.12 * n[1].abs(),
                    0.34 + 0.14 * task + 0.12 * n[1].abs(),
                    0.22 + 0.09 * task + 0.12 * n[1].abs(),
                ]);
            }
        }
    }
    if found != names.len() || output.is_empty() {
        return Err(fail("template coverage mismatch"));
    }
    Ok(output)
}
/// Measured resident/staging/submission costs, not an approved hardware budget.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct FixtureBatchStats {
    pub prepare_time: std::time::Duration,
    pub upload_time: std::time::Duration,
    pub upload_bytes: u64,
    pub triangles: u32,
    pub scratch_capacity_bytes: u64,
    pub resident_geometry_bytes: u64,
    pub instance_capacity_bytes: u64,
    pub draw_calls: u32,
    pub visible_instances: u32,
}

// Three vec4 attributes: camera-relative center/scale, quaternion, linear tint.
const INSTANCE_BYTES: usize = 48;

#[derive(Default)]
struct InstanceBatch {
    staging: Vec<u8>,
    uploaded: Vec<u8>,
}

fn prepare_instances(
    batches: &mut [InstanceBatch; TEMPLATE_COUNT],
    instances: &[FixtureInstance],
    camera: CameraFrame,
    aspect_ratio: f32,
    radii: &[f32; TEMPLATE_COUNT],
) -> Result<(), RendererError> {
    if instances.len() > 16_384 {
        return Err(RendererError::new(
            "fixture frame",
            "fixture scene exceeds the 16384-record staging budget",
        ));
    }
    let basis = crate::camera_basis(camera).map_err(|e| RendererError::new("fixture camera", e))?;
    for b in batches.iter_mut() {
        b.staging.clear();
    }
    let mut ids = std::collections::HashSet::new();
    for n in instances {
        if !ids.insert(n.id)
            || !valid_pose(n.pose)
            || n.tint
                .iter()
                .any(|v| !v.is_finite() || !(0.0..=4.0).contains(v))
        {
            return Err(RendererError::new(
                "fixture instance",
                "invalid identity/pose/material",
            ));
        }
        let center = std::array::from_fn::<_, 3, _>(|i| {
            (n.pose.position_meters[i] - camera.position_meters[i]) as f32
        });
        let scale = n.pose.scale as f32;
        if center.iter().any(|v| !v.is_finite()) || !scale.is_finite() || scale <= 0. {
            return Err(RendererError::new("fixture instance", "outside f32 range"));
        }
        // Every validated template fits this conservative pivot-centered sphere.
        let extent = radii[n.mesh.index()] * scale * 1.001;
        if !crate::instance_intersects_frustum(
            crate::GpuInstance {
                orientation: [0., 0., 0., 1.],
                relative_center: center,
                half_extents: [extent; 3],
                color: [1.; 4],
            },
            basis,
            camera,
            aspect_ratio,
        ) {
            continue;
        }
        for v in center
            .into_iter()
            .chain([scale])
            .chain(n.pose.orientation_xyzw.map(|v| v as f32))
            .chain(n.tint)
            .chain([f32::from(n.powered)])
        {
            batches[n.mesh.index()]
                .staging
                .extend_from_slice(&v.to_le_bytes());
        }
    }
    Ok(())
}

// Deduplicate position/material vertices once, preserving triangle order and
// facet color boundaries. Indices address the concatenated immutable stream.
fn indexed_geometry(
    meshes: &[Vec<[f32; FLOATS]>; TEMPLATE_COUNT],
) -> (Vec<f32>, Vec<u32>, [std::ops::Range<u32>; TEMPLATE_COUNT]) {
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

pub(crate) struct FixtureMeshRenderer {
    pipeline: wgpu::RenderPipeline,
    uniform: wgpu::Buffer,
    binding: wgpu::BindGroup,
    vertices: wgpu::Buffer,
    indices: wgpu::Buffer,
    ranges: [std::ops::Range<u32>; TEMPLATE_COUNT],
    batches: [InstanceBatch; TEMPLATE_COUNT],
    instance_buffers: [wgpu::Buffer; TEMPLATE_COUNT],
    capacities: [usize; TEMPLATE_COUNT],
    objects: u32,
    radii: [f32; TEMPLATE_COUNT],
    geometry_bytes: u64,
}
impl FixtureMeshRenderer {
    pub(crate) fn new(
        device: &wgpu::Device,
        format: wgpu::TextureFormat,
    ) -> Result<Self, RendererError> {
        let mut meshes: [Vec<[f32; FLOATS]>; TEMPLATE_COUNT] = std::array::from_fn(|_| Vec::new());
        for (i, mesh) in meshes.iter_mut().enumerate() {
            *mesh = geometry(i)?;
        }
        let radii = std::array::from_fn(|i| {
            meshes[i]
                .iter()
                .map(|v| (v[0] * v[0] + v[1] * v[1] + v[2] * v[2]).sqrt())
                .fold(0.0, f32::max)
        });
        let (vertex_data, index_data, ranges) = indexed_geometry(&meshes);
        let geometry_bytes = (vertex_data.len() * 4 + index_data.len() * 4) as u64;
        let vertices = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Resident fixture vertices"),
            contents: &encode_f32s(vertex_data),
            usage: wgpu::BufferUsages::VERTEX,
        });
        let index_bytes: Vec<_> = index_data.iter().flat_map(|v| v.to_le_bytes()).collect();
        let indices = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Resident fixture indices"),
            contents: &index_bytes,
            usage: wgpu::BufferUsages::INDEX,
        });
        let instance_buffers = std::array::from_fn(|_| instance_buffer(device, INSTANCE_BYTES));
        let uniform = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("Fixture camera"),
            size: 64,
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });
        let layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("Fixture mesh bindings"),
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
            label: Some("Fixture mesh binding"),
            layout: &layout,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: uniform.as_entire_binding(),
            }],
        });
        let shader = device.create_shader_module(wgpu::include_wgsl!("fixtures.wgsl"));
        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("Fixture mesh layout"),
            bind_group_layouts: &[Some(&layout)],
            immediate_size: 0,
        });
        let attributes = wgpu::vertex_attr_array![
            0 => Float32x3, 1 => Float32x4, 2 => Float32x3
        ];
        let instance_attributes =
            wgpu::vertex_attr_array![3 => Float32x4, 4 => Float32x4, 5 => Float32x4];
        let pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("Fixture mesh pipeline"),
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
            capacities: [INSTANCE_BYTES; TEMPLATE_COUNT],
            objects: 0,
            radii,
            geometry_bytes,
        })
    }
    pub(crate) fn prepare(
        &mut self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        camera: CameraFrame,
        projection: [f32; 16],
        aspect_ratio: f32,
        instances: &[FixtureInstance],
    ) -> Result<FixtureBatchStats, RendererError> {
        let started = std::time::Instant::now();
        prepare_instances(
            &mut self.batches,
            instances,
            camera,
            aspect_ratio,
            &self.radii,
        )?;
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
            u32::try_from(objects).map_err(|e| RendererError::new("fixture object count", e))?;
        let triangles = u32::try_from(triangles)
            .map_err(|e| RendererError::new("fixture triangle count", e))?;
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
        Ok(FixtureBatchStats {
            resident_geometry_bytes: self.geometry_bytes,
            instance_capacity_bytes: self.capacities.iter().sum::<usize>() as u64,
            draw_calls: self.draw_count(),
            visible_instances: self.objects,
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
        label: Some("Fixture instance transforms"),
        size: bytes as u64,
        usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
        mapped_at_creation: false,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    fn node(id: u64, parent: Option<u64>, p: [f64; 3]) -> FixtureNode {
        FixtureNode {
            id,
            parent,
            mesh: FixtureMesh::Shelf,
            local: FixturePose {
                position_meters: p,
                ..Default::default()
            },
            tint: [1.; 3],
            powered: true,
        }
    }
    #[test]
    fn modular_shader_validates_on_the_portable_wgsl_path() {
        let module = wgpu::naga::front::wgsl::parse_str(include_str!("fixtures.wgsl")).unwrap();
        wgpu::naga::valid::Validator::new(
            wgpu::naga::valid::ValidationFlags::all(),
            wgpu::naga::valid::Capabilities::all(),
        )
        .validate(&module)
        .unwrap();
    }
    #[test]
    fn nested_rooms_and_attachments_resolve_out_of_order_without_losing_far_origin_precision() {
        let root = FixturePose {
            position_meters: [1e12, -1e12, 1e12],
            orientation_xyzw: [
                0.,
                std::f64::consts::FRAC_1_SQRT_2,
                0.,
                std::f64::consts::FRAC_1_SQRT_2,
            ],
            scale: 1.,
        };
        let result = resolve_fixture_hierarchy(
            root,
            &[
                node(20, Some(10), [0., 1., 1.]),
                node(10, None, [3., 0., 0.]),
            ],
        )
        .unwrap();
        assert_eq!(result[0].id, 20);
        assert_eq!(result[1].id, 10);
        assert_eq!(
            result[0].pose.position_meters,
            [1e12 + 1., -1e12 + 1., 1e12 - 3.]
        );
        assert_eq!(result[1].pose.position_meters, [1e12, -1e12, 1e12 - 3.]);
    }
    #[test]
    fn identity_cycles_missing_parents_bad_transforms_and_materials_fail_before_submission() {
        for nodes in [
            vec![node(1, None, [0.; 3]), node(1, None, [0.; 3])],
            vec![node(1, Some(2), [0.; 3]), node(2, Some(1), [0.; 3])],
            vec![node(1, Some(100), [0.; 3])],
        ] {
            assert!(resolve_fixture_hierarchy(FixturePose::default(), &nodes).is_err());
        }
        let mut bad = node(1, None, [0.; 3]);
        bad.local.scale = 0.;
        assert!(resolve_fixture_hierarchy(FixturePose::default(), &[bad]).is_err());
        bad.local.scale = 1.;
        bad.tint[0] = f32::NAN;
        assert!(resolve_fixture_hierarchy(FixturePose::default(), &[bad]).is_err());
    }
    #[test]
    fn independent_templates_share_geometry_and_damage_is_an_explicit_distinct_variant() {
        let meshes: Vec<_> = (0..TEMPLATE_COUNT).map(|i| geometry(i).unwrap()).collect();
        assert_eq!(meshes[..5].iter().map(|v| v.len() / 3).sum::<usize>(), 84);
        assert!(meshes[5..].iter().map(|v| v.len() / 3).sum::<usize>() <= 200);
        assert_ne!(meshes[3], meshes[4]);
        assert!(
            meshes
                .iter()
                .flatten()
                .all(|v| v.iter().all(|f| f.is_finite()))
        );
        let same = geometry(0).unwrap();
        assert_eq!(meshes[0], same);
    }
    #[test]
    fn instance_staging_rebases_before_narrowing_and_culls_without_mutating_templates() {
        let camera = CameraFrame {
            position_meters: [1e12, 0., 0.],
            target_meters: [1e12, 0., -1.],
            up: [0., 1., 0.],
            vertical_fov_radians: 1.2,
            near_plane_meters: 0.05,
        };
        let instance = FixtureInstance {
            id: 7,
            mesh: FixtureMesh::Shelf,
            pose: FixturePose {
                position_meters: [1e12 + 0.125, 0., -3.],
                ..Default::default()
            },
            tint: [1.; 3],
            powered: true,
        };
        let mut batches = std::array::from_fn(|_| InstanceBatch::default());
        prepare_instances(
            &mut batches,
            &[instance],
            camera,
            1.6,
            &[2.; TEMPLATE_COUNT],
        )
        .unwrap();
        assert_eq!(
            f32::from_le_bytes(batches[1].staging[0..4].try_into().unwrap()),
            0.125
        );
        assert_eq!(batches[1].staging.len(), INSTANCE_BYTES);
        let mut behind = instance;
        behind.pose.position_meters[2] = 100.;
        prepare_instances(&mut batches, &[behind], camera, 1.6, &[2.; TEMPLATE_COUNT]).unwrap();
        assert!(batches.iter().all(|b| b.staging.is_empty()));
        assert!(
            prepare_instances(
                &mut batches,
                &[instance, instance],
                camera,
                1.6,
                &[2.; TEMPLATE_COUNT]
            )
            .is_err()
        );
    }
}
#[cfg(test)]
#[path = "fixtures_evidence.rs"]
mod evidence;

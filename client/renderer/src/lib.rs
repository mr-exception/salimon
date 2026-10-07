//! Custom GPU presentation for the Salimon client.
//!
//! This crate owns `wgpu` resources and surface presentation. It deliberately
//! has no dependency on world, character, or ship state.

mod cockpit_instruments;
mod equipment_toolbar;
mod gpu_timing;
mod held_item;
mod overlay;
mod resource_mesh;
mod ship_mesh;
mod spheres;
mod surface_textures;

use std::error::Error;
use std::fmt;
use std::time::Duration;

pub use cockpit_instruments::{CockpitInstruments, NearbyBodyInstruments};
pub use equipment_toolbar::{EquipmentIcon, EquipmentSlot, EquipmentToolbar};
use gpu_timing::GpuTimer;
pub use held_item::HeldItemInstance;
pub use overlay::{OverlayImage, OverlayPlacement};
pub use resource_mesh::{ResourceMesh, ResourceMeshInstance};
pub use ship_mesh::ShipMeshInstance;
pub use spheres::{PointLight, SphereInstance, SurfaceMaterial};
use wgpu::util::DeviceExt;

const CLEAR_COLOR: wgpu::Color = wgpu::Color {
    r: 0.008,
    g: 0.015,
    b: 0.035,
    a: 1.0,
};
const DEPTH_FORMAT: wgpu::TextureFormat = wgpu::TextureFormat::Depth32Float;
const CAMERA_UNIFORM_SIZE: u64 = 64;
const INSTANCE_STRIDE: u64 = 48;
const CUBE_VERTEX_STRIDE: u64 = 12;
const CUBE_VERTEX_COUNT: u32 = 36;
const BASIS_EPSILON: f32 = 1.0e-5;

// Unit cuboid expanded by each instance's half-extents in the vertex shader.
const CUBE_VERTICES: [[f32; 3]; CUBE_VERTEX_COUNT as usize] = [
    // +Z
    [-1.0, -1.0, 1.0],
    [1.0, -1.0, 1.0],
    [1.0, 1.0, 1.0],
    [-1.0, -1.0, 1.0],
    [1.0, 1.0, 1.0],
    [-1.0, 1.0, 1.0],
    // -Z
    [1.0, -1.0, -1.0],
    [-1.0, -1.0, -1.0],
    [-1.0, 1.0, -1.0],
    [1.0, -1.0, -1.0],
    [-1.0, 1.0, -1.0],
    [1.0, 1.0, -1.0],
    // +X
    [1.0, -1.0, 1.0],
    [1.0, -1.0, -1.0],
    [1.0, 1.0, -1.0],
    [1.0, -1.0, 1.0],
    [1.0, 1.0, -1.0],
    [1.0, 1.0, 1.0],
    // -X
    [-1.0, -1.0, -1.0],
    [-1.0, -1.0, 1.0],
    [-1.0, 1.0, 1.0],
    [-1.0, -1.0, -1.0],
    [-1.0, 1.0, 1.0],
    [-1.0, 1.0, -1.0],
    // +Y
    [-1.0, 1.0, 1.0],
    [1.0, 1.0, 1.0],
    [1.0, 1.0, -1.0],
    [-1.0, 1.0, 1.0],
    [1.0, 1.0, -1.0],
    [-1.0, 1.0, -1.0],
    // -Y
    [-1.0, -1.0, -1.0],
    [1.0, -1.0, -1.0],
    [1.0, -1.0, 1.0],
    [-1.0, -1.0, -1.0],
    [1.0, -1.0, 1.0],
    [-1.0, -1.0, 1.0],
];

/// Camera input for one renderer frame, expressed in canonical world metres.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct CameraFrame {
    pub position_meters: [f64; 3],
    pub target_meters: [f64; 3],
    pub up: [f32; 3],
    pub vertical_fov_radians: f32,
    pub near_plane_meters: f32,
}

/// Renderer-facing cuboid instance expressed in canonical world metres.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct SceneInstance {
    pub center_meters: [f64; 3],
    pub half_extents_meters: [f32; 3],
    pub color: [f32; 4],
}

/// Borrowed scene snapshot for one renderer frame.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct SceneFrame<'a> {
    pub camera: CameraFrame,
    pub instances: &'a [SceneInstance],
    pub resource_meshes: &'a [ResourceMeshInstance],
    pub spheres: &'a [SphereInstance],
    pub light: Option<PointLight>,
    pub ship: Option<ShipMeshInstance>,
    pub held_item: Option<HeldItemInstance>,
}

/// Physical pixel dimensions for the renderer's presentation surface.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct SurfaceSize {
    width: u32,
    height: u32,
}

impl SurfaceSize {
    #[must_use]
    pub const fn new(width: u32, height: u32) -> Self {
        Self { width, height }
    }

    #[must_use]
    pub const fn is_drawable(self) -> bool {
        self.width > 0 && self.height > 0
    }
}

/// Stable information about the GPU selected for presentation.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RendererInfo {
    pub adapter_name: String,
    pub backend: String,
    pub device_type: String,
    pub timestamp_queries_supported: bool,
}

/// Availability and latest completed value for GPU pass timing.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum GpuFrameTime {
    /// The selected adapter cannot issue timestamp queries.
    Unsupported,
    /// Timestamp queries are supported but no asynchronous result is ready yet.
    Pending,
    /// Duration of the latest completed scene and overlay render pass.
    Measured(Duration),
}

/// Optional allocator totals reported by the active GPU backend.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct GpuMemoryMetrics {
    pub allocated_bytes: u64,
    pub reserved_bytes: u64,
}

/// Measurements associated with one successfully presented renderer frame.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct RenderStats {
    pub cpu_render_time: Duration,
    pub gpu_frame_time: GpuFrameTime,
    pub visible_objects: u32,
    pub rendered_objects: u32,
    pub scene_draw_calls: u32,
    pub total_draw_calls: u32,
    pub gpu_memory: Option<GpuMemoryMetrics>,
}

/// Result of attempting to render one frame.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum RenderOutcome {
    /// Commands were submitted and the surface image was presented.
    Presented(RenderStats),
    /// No drawable image is currently available; wait for a lifecycle event.
    Idle,
    /// The surface is temporarily unavailable or was reconfigured; retry later.
    Retry,
    /// The platform surface was lost and the renderer must be rebuilt.
    SurfaceLost,
}

/// Failure while initializing or using the GPU renderer.
#[derive(Debug)]
pub struct RendererError {
    context: &'static str,
    detail: String,
}

impl RendererError {
    fn new(context: &'static str, detail: impl fmt::Display) -> Self {
        Self {
            context,
            detail: detail.to_string(),
        }
    }
}

impl fmt::Display for RendererError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "{}: {}", self.context, self.detail)
    }
}

impl Error for RendererError {}

#[derive(Clone, Copy, Debug, PartialEq)]
struct CameraBasis {
    right: [f32; 3],
    up: [f32; 3],
    forward: [f32; 3],
}

#[derive(Clone, Copy, Debug, PartialEq)]
struct GpuInstance {
    relative_center: [f32; 3],
    half_extents: [f32; 3],
    color: [f32; 4],
}

#[derive(Debug, PartialEq)]
struct PreparedScene {
    view_projection: [f32; 16],
    instances: Vec<GpuInstance>,
    instance_count: u32,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum SceneInputError {
    CameraPositionNotFinite,
    CameraTargetNotFinite,
    CameraDirectionInvalid,
    CameraUpInvalid,
    CameraUpParallel,
    VerticalFieldOfViewInvalid,
    NearPlaneInvalid,
    AspectRatioInvalid,
    InstanceCountOverflow,
    InstanceCenterInvalid(usize),
    InstanceRelativeCenterOutOfRange(usize),
    InstanceHalfExtentsInvalid(usize),
    InstanceColorInvalid(usize),
}

impl fmt::Display for SceneInputError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::CameraPositionNotFinite => {
                formatter.write_str("camera position must contain only finite values")
            }
            Self::CameraTargetNotFinite => {
                formatter.write_str("camera target must contain only finite values")
            }
            Self::CameraDirectionInvalid => formatter
                .write_str("camera position and target must define a finite, nonzero direction"),
            Self::CameraUpInvalid => {
                formatter.write_str("camera up vector must be finite and nonzero")
            }
            Self::CameraUpParallel => {
                formatter.write_str("camera up vector must not be parallel to its direction")
            }
            Self::VerticalFieldOfViewInvalid => formatter.write_str(
                "camera vertical field of view must be finite and between zero and pi radians",
            ),
            Self::NearPlaneInvalid => {
                formatter.write_str("camera near plane must be finite and greater than zero")
            }
            Self::AspectRatioInvalid => {
                formatter.write_str("camera aspect ratio must be finite and greater than zero")
            }
            Self::InstanceCountOverflow => {
                formatter.write_str("scene instance count exceeds the renderer's u32 draw range")
            }
            Self::InstanceCenterInvalid(index) => {
                write!(formatter, "scene instance {index} has a non-finite center")
            }
            Self::InstanceRelativeCenterOutOfRange(index) => write!(
                formatter,
                "scene instance {index} is outside the finite f32 camera-relative range"
            ),
            Self::InstanceHalfExtentsInvalid(index) => write!(
                formatter,
                "scene instance {index} half-extents must be finite and greater than zero"
            ),
            Self::InstanceColorInvalid(index) => {
                write!(formatter, "scene instance {index} has a non-finite color")
            }
        }
    }
}

struct DepthTarget {
    _texture: wgpu::Texture,
    view: wgpu::TextureView,
}

impl DepthTarget {
    fn new(device: &wgpu::Device, width: u32, height: u32) -> Self {
        debug_assert!(width > 0 && height > 0);
        let texture = device.create_texture(&wgpu::TextureDescriptor {
            label: Some("Salimon reverse-Z depth target"),
            size: wgpu::Extent3d {
                width,
                height,
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: DEPTH_FORMAT,
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::TEXTURE_BINDING,
            view_formats: &[],
        });
        let view = texture.create_view(&wgpu::TextureViewDescriptor::default());
        Self {
            _texture: texture,
            view,
        }
    }
}

fn prepare_scene(
    scene: SceneFrame<'_>,
    aspect_ratio: f32,
) -> Result<PreparedScene, SceneInputError> {
    let basis = camera_basis(scene.camera)?;
    let view = view_rotation(basis);
    let projection = infinite_reverse_z_projection(scene.camera, aspect_ratio)?;
    let view_projection = multiply_mat4(projection, view);
    let mut instances = Vec::with_capacity(scene.instances.len());

    for (index, instance) in scene.instances.iter().copied().enumerate() {
        if !all_finite_f64(instance.center_meters) {
            return Err(SceneInputError::InstanceCenterInvalid(index));
        }
        if !instance
            .half_extents_meters
            .iter()
            .all(|value| value.is_finite() && *value > 0.0)
        {
            return Err(SceneInputError::InstanceHalfExtentsInvalid(index));
        }
        if !instance.color.iter().all(|value| value.is_finite()) {
            return Err(SceneInputError::InstanceColorInvalid(index));
        }

        let relative_center_f64 = [
            instance.center_meters[0] - scene.camera.position_meters[0],
            instance.center_meters[1] - scene.camera.position_meters[1],
            instance.center_meters[2] - scene.camera.position_meters[2],
        ];
        let relative_center = relative_center_f64.map(|value| value as f32);
        if !relative_center.iter().all(|value| value.is_finite()) {
            return Err(SceneInputError::InstanceRelativeCenterOutOfRange(index));
        }

        let gpu_instance = GpuInstance {
            relative_center,
            half_extents: instance.half_extents_meters,
            color: instance.color,
        };
        if instance_intersects_frustum(gpu_instance, basis, scene.camera, aspect_ratio) {
            instances.push(gpu_instance);
        }
    }
    let instance_count =
        u32::try_from(instances.len()).map_err(|_| SceneInputError::InstanceCountOverflow)?;

    Ok(PreparedScene {
        view_projection,
        instances,
        instance_count,
    })
}

fn instance_intersects_frustum(
    instance: GpuInstance,
    basis: CameraBasis,
    camera: CameraFrame,
    aspect_ratio: f32,
) -> bool {
    let radius = scaled_length(instance.half_extents);
    if !radius.is_finite() {
        return true;
    }

    let camera_x = dot(instance.relative_center, basis.right);
    let camera_y = dot(instance.relative_center, basis.up);
    let camera_depth = dot(instance.relative_center, basis.forward);
    if camera_depth + radius < camera.near_plane_meters {
        return false;
    }

    let vertical_tangent = (camera.vertical_fov_radians * 0.5).tan();
    let horizontal_tangent = vertical_tangent * aspect_ratio;
    let horizontal_radius = radius * (horizontal_tangent * horizontal_tangent + 1.0).sqrt();
    let vertical_radius = radius * (vertical_tangent * vertical_tangent + 1.0).sqrt();

    camera_depth * horizontal_tangent - camera_x.abs() >= -horizontal_radius
        && camera_depth * vertical_tangent - camera_y.abs() >= -vertical_radius
}

fn camera_basis(camera: CameraFrame) -> Result<CameraBasis, SceneInputError> {
    if !all_finite_f64(camera.position_meters) {
        return Err(SceneInputError::CameraPositionNotFinite);
    }
    if !all_finite_f64(camera.target_meters) {
        return Err(SceneInputError::CameraTargetNotFinite);
    }

    let direction = [
        camera.target_meters[0] - camera.position_meters[0],
        camera.target_meters[1] - camera.position_meters[1],
        camera.target_meters[2] - camera.position_meters[2],
    ];
    let forward = normalize_f64_to_f32(direction).ok_or(SceneInputError::CameraDirectionInvalid)?;
    let requested_up = normalize_f32(camera.up).ok_or(SceneInputError::CameraUpInvalid)?;
    let right_unnormalized = cross(forward, requested_up);
    if length(right_unnormalized) <= BASIS_EPSILON {
        return Err(SceneInputError::CameraUpParallel);
    }
    let right = normalize_f32(right_unnormalized).ok_or(SceneInputError::CameraUpParallel)?;
    let up = normalize_f32(cross(right, forward)).ok_or(SceneInputError::CameraUpParallel)?;

    Ok(CameraBasis { right, up, forward })
}

fn infinite_reverse_z_projection(
    camera: CameraFrame,
    aspect_ratio: f32,
) -> Result<[f32; 16], SceneInputError> {
    if !camera.vertical_fov_radians.is_finite()
        || !(0.0..std::f32::consts::PI).contains(&camera.vertical_fov_radians)
    {
        return Err(SceneInputError::VerticalFieldOfViewInvalid);
    }
    if !camera.near_plane_meters.is_finite() || camera.near_plane_meters <= 0.0 {
        return Err(SceneInputError::NearPlaneInvalid);
    }
    if !aspect_ratio.is_finite() || aspect_ratio <= 0.0 {
        return Err(SceneInputError::AspectRatioInvalid);
    }

    let y_scale = 1.0 / (camera.vertical_fov_radians * 0.5).tan();
    let x_scale = y_scale / aspect_ratio;
    if !x_scale.is_finite() || !y_scale.is_finite() {
        return Err(SceneInputError::VerticalFieldOfViewInvalid);
    }

    // Right-handed, WebGPU zero-to-one clip depth. Near maps to 1 and infinity to 0.
    Ok([
        x_scale,
        0.0,
        0.0,
        0.0,
        0.0,
        y_scale,
        0.0,
        0.0,
        0.0,
        0.0,
        0.0,
        -1.0,
        0.0,
        0.0,
        camera.near_plane_meters,
        0.0,
    ])
}

fn view_rotation(basis: CameraBasis) -> [f32; 16] {
    [
        basis.right[0],
        basis.up[0],
        -basis.forward[0],
        0.0,
        basis.right[1],
        basis.up[1],
        -basis.forward[1],
        0.0,
        basis.right[2],
        basis.up[2],
        -basis.forward[2],
        0.0,
        0.0,
        0.0,
        0.0,
        1.0,
    ]
}

fn multiply_mat4(left: [f32; 16], right: [f32; 16]) -> [f32; 16] {
    let mut product = [0.0; 16];
    for column in 0..4 {
        for row in 0..4 {
            product[column * 4 + row] = (0..4)
                .map(|index| left[index * 4 + row] * right[column * 4 + index])
                .sum();
        }
    }
    product
}

fn normalize_f64_to_f32(vector: [f64; 3]) -> Option<[f32; 3]> {
    if !all_finite_f64(vector) {
        return None;
    }
    let scale = vector.iter().copied().map(f64::abs).fold(0.0, f64::max);
    if scale == 0.0 {
        return None;
    }
    let scaled = vector.map(|value| value / scale);
    let scaled_length = scaled.iter().map(|value| value * value).sum::<f64>().sqrt();
    let normalized = scaled.map(|value| (value / scaled_length) as f32);
    normalize_f32(normalized)
}

fn normalize_f32(vector: [f32; 3]) -> Option<[f32; 3]> {
    if !vector.iter().all(|value| value.is_finite()) {
        return None;
    }
    let scale = vector.iter().copied().map(f32::abs).fold(0.0, f32::max);
    if scale == 0.0 {
        return None;
    }
    let scaled = vector.map(|value| value / scale);
    let scaled_length = length(scaled);
    Some(scaled.map(|value| value / scaled_length))
}

fn cross(left: [f32; 3], right: [f32; 3]) -> [f32; 3] {
    [
        left[1] * right[2] - left[2] * right[1],
        left[2] * right[0] - left[0] * right[2],
        left[0] * right[1] - left[1] * right[0],
    ]
}

fn dot(left: [f32; 3], right: [f32; 3]) -> f32 {
    left.into_iter()
        .zip(right)
        .map(|(left, right)| left * right)
        .sum()
}

fn length(vector: [f32; 3]) -> f32 {
    vector.iter().map(|value| value * value).sum::<f32>().sqrt()
}

fn scaled_length(vector: [f32; 3]) -> f32 {
    let scale = vector.iter().copied().map(f32::abs).fold(0.0, f32::max);
    if scale == 0.0 {
        return 0.0;
    }
    scale * length(vector.map(|value| value / scale))
}

fn all_finite_f64(vector: [f64; 3]) -> bool {
    vector.iter().all(|value| value.is_finite())
}

fn encode_f32s(values: impl IntoIterator<Item = f32>) -> Vec<u8> {
    let iterator = values.into_iter();
    let mut bytes = Vec::with_capacity(iterator.size_hint().0.saturating_mul(4));
    for value in iterator {
        bytes.extend_from_slice(&value.to_ne_bytes());
    }
    bytes
}

fn encode_instances(instances: &[GpuInstance]) -> Vec<u8> {
    let mut values = Vec::with_capacity(instances.len().saturating_mul(12));
    for instance in instances {
        values.extend_from_slice(&[
            instance.relative_center[0],
            instance.relative_center[1],
            instance.relative_center[2],
            0.0,
            instance.half_extents[0],
            instance.half_extents[1],
            instance.half_extents[2],
            0.0,
            instance.color[0],
            instance.color[1],
            instance.color[2],
            instance.color[3],
        ]);
    }
    encode_f32s(values)
}

fn scene_draw_calls(instance_count: u32) -> u32 {
    u32::from(instance_count > 0)
}

/// Owns the surface, device, queue, and renderer-facing validation scene pipeline.
pub struct Renderer {
    surface: wgpu::Surface<'static>,
    device: wgpu::Device,
    queue: wgpu::Queue,
    configuration: wgpu::SurfaceConfiguration,
    pipeline: wgpu::RenderPipeline,
    camera_buffer: wgpu::Buffer,
    camera_bind_group: wgpu::BindGroup,
    cube_vertex_buffer: wgpu::Buffer,
    instance_buffer: wgpu::Buffer,
    instance_capacity: usize,
    depth_target: DepthTarget,
    overlay: overlay::OverlayRenderer,
    action_bar: overlay::OverlayRenderer,
    reticle: overlay::OverlayRenderer,
    toolbar: overlay::OverlayRenderer,
    toolbar_raster: equipment_toolbar::ToolbarRaster,
    spheres: spheres::SphereRenderer,
    ship_mesh: ship_mesh::ShipMeshRenderer,
    held_item: held_item::HeldItemRenderer,
    resource_meshes: resource_mesh::ResourceMeshRenderer,
    gpu_timer: Option<GpuTimer>,
    cached_gpu_memory: Option<GpuMemoryMetrics>,
    presented_frames: u64,
    drawable: bool,
    info: RendererInfo,
}

impl Renderer {
    /// Creates a renderer for an owned native surface target.
    ///
    /// Passing an owned target (for example `Arc<winit::window::Window>`) gives
    /// the surface a static lifetime without unsafe code.
    pub async fn new(
        surface_target: impl Into<wgpu::SurfaceTarget<'static>>,
        initial_size: SurfaceSize,
    ) -> Result<Self, RendererError> {
        let instance = wgpu::Instance::default();
        let surface = instance
            .create_surface(surface_target)
            .map_err(|error| RendererError::new("failed to create rendering surface", error))?;

        let adapter = instance
            .request_adapter(&wgpu::RequestAdapterOptions {
                power_preference: wgpu::PowerPreference::HighPerformance,
                compatible_surface: Some(&surface),
                force_fallback_adapter: false,
                ..Default::default()
            })
            .await
            .map_err(|error| {
                RendererError::new("failed to find a compatible GPU adapter", error)
            })?;

        let timestamp_queries_supported =
            adapter.features().contains(wgpu::Features::TIMESTAMP_QUERY);
        let adapter_info = adapter.get_info();
        let info = RendererInfo {
            adapter_name: adapter_info.name,
            backend: format!("{:?}", adapter_info.backend),
            device_type: format!("{:?}", adapter_info.device_type),
            timestamp_queries_supported,
        };
        let required_features = if timestamp_queries_supported {
            wgpu::Features::TIMESTAMP_QUERY
        } else {
            wgpu::Features::empty()
        };
        let (device, queue) = adapter
            .request_device(&wgpu::DeviceDescriptor {
                label: Some("Salimon Phase 0 device"),
                required_features,
                required_limits: wgpu::Limits::default(),
                experimental_features: wgpu::ExperimentalFeatures::disabled(),
                memory_hints: wgpu::MemoryHints::MemoryUsage,
                trace: wgpu::Trace::Off,
            })
            .await
            .map_err(|error| RendererError::new("failed to create GPU device", error))?;

        let configured_width = initial_size.width.max(1);
        let configured_height = initial_size.height.max(1);
        let mut configuration = surface
            .get_default_config(&adapter, configured_width, configured_height)
            .ok_or_else(|| {
                RendererError::new(
                    "failed to configure rendering surface",
                    "adapter exposes no compatible surface format",
                )
            })?;
        configuration.present_mode = wgpu::PresentMode::AutoVsync;
        configuration.desired_maximum_frame_latency = 2;

        let camera_bind_group_layout =
            device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                label: Some("Salimon camera bind group layout"),
                entries: &[wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::VERTEX,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: wgpu::BufferSize::new(CAMERA_UNIFORM_SIZE),
                    },
                    count: None,
                }],
            });
        let camera_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("Salimon camera uniform"),
            size: CAMERA_UNIFORM_SIZE,
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });
        let camera_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("Salimon camera bind group"),
            layout: &camera_bind_group_layout,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: camera_buffer.as_entire_binding(),
            }],
        });

        let shader = device.create_shader_module(wgpu::include_wgsl!("bootstrap.wgsl"));
        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("Salimon validation scene pipeline layout"),
            bind_group_layouts: &[Some(&camera_bind_group_layout)],
            immediate_size: 0,
        });
        let pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("Salimon instanced cuboid pipeline"),
            layout: Some(&pipeline_layout),
            vertex: wgpu::VertexState {
                module: &shader,
                entry_point: Some("vs_main"),
                buffers: &[
                    Some(wgpu::VertexBufferLayout {
                        array_stride: CUBE_VERTEX_STRIDE,
                        step_mode: wgpu::VertexStepMode::Vertex,
                        attributes: &[wgpu::VertexAttribute {
                            format: wgpu::VertexFormat::Float32x3,
                            offset: 0,
                            shader_location: 0,
                        }],
                    }),
                    Some(wgpu::VertexBufferLayout {
                        array_stride: INSTANCE_STRIDE,
                        step_mode: wgpu::VertexStepMode::Instance,
                        attributes: &[
                            wgpu::VertexAttribute {
                                format: wgpu::VertexFormat::Float32x3,
                                offset: 0,
                                shader_location: 1,
                            },
                            wgpu::VertexAttribute {
                                format: wgpu::VertexFormat::Float32x3,
                                offset: 16,
                                shader_location: 2,
                            },
                            wgpu::VertexAttribute {
                                format: wgpu::VertexFormat::Float32x4,
                                offset: 32,
                                shader_location: 3,
                            },
                        ],
                    }),
                ],
                compilation_options: wgpu::PipelineCompilationOptions::default(),
            },
            primitive: wgpu::PrimitiveState::default(),
            depth_stencil: Some(wgpu::DepthStencilState {
                format: DEPTH_FORMAT,
                depth_write_enabled: Some(true),
                depth_compare: Some(wgpu::CompareFunction::Greater),
                stencil: wgpu::StencilState::default(),
                bias: wgpu::DepthBiasState::default(),
            }),
            multisample: wgpu::MultisampleState::default(),
            fragment: Some(wgpu::FragmentState {
                module: &shader,
                entry_point: Some("fs_main"),
                targets: &[Some(wgpu::ColorTargetState {
                    format: configuration.format,
                    blend: Some(wgpu::BlendState::REPLACE),
                    write_mask: wgpu::ColorWrites::ALL,
                })],
                compilation_options: wgpu::PipelineCompilationOptions::default(),
            }),
            multiview_mask: None,
            cache: None,
        });
        let cube_vertex_bytes = encode_f32s(CUBE_VERTICES.into_iter().flatten());
        let cube_vertex_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Salimon unit cuboid vertices"),
            contents: &cube_vertex_bytes,
            usage: wgpu::BufferUsages::VERTEX,
        });
        let instance_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("Salimon cuboid instances"),
            size: INSTANCE_STRIDE,
            usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });
        let depth_target = DepthTarget::new(&device, configured_width, configured_height);
        let overlay = overlay::OverlayRenderer::new(&device, configuration.format);
        let action_bar = overlay::OverlayRenderer::new(&device, configuration.format);
        let reticle = overlay::OverlayRenderer::new(&device, configuration.format);
        let toolbar = overlay::OverlayRenderer::new(&device, configuration.format);
        let spheres = spheres::SphereRenderer::new(&device, &queue, configuration.format);
        let ship_mesh = ship_mesh::ShipMeshRenderer::new(&device, configuration.format)?;
        let held_item = held_item::HeldItemRenderer::new(&device, configuration.format)?;
        let resource_meshes =
            resource_mesh::ResourceMeshRenderer::new(&device, configuration.format)?;
        let gpu_timer = timestamp_queries_supported.then(|| GpuTimer::new(&device, &queue));

        let mut renderer = Self {
            surface,
            device,
            queue,
            configuration,
            pipeline,
            camera_buffer,
            camera_bind_group,
            cube_vertex_buffer,
            instance_buffer,
            instance_capacity: 1,
            depth_target,
            overlay,
            action_bar,
            reticle,
            toolbar,
            toolbar_raster: equipment_toolbar::ToolbarRaster::default(),
            spheres,
            ship_mesh,
            held_item,
            resource_meshes,
            gpu_timer,
            cached_gpu_memory: None,
            presented_frames: 0,
            drawable: false,
            info,
        };
        renderer.resize(initial_size);
        Ok(renderer)
    }

    #[must_use]
    pub const fn info(&self) -> &RendererInfo {
        &self.info
    }

    /// Applies a new physical surface size.
    ///
    /// Zero-sized windows are marked idle because `wgpu` rejects zero-sized
    /// surface configurations. A later non-zero resize resumes presentation.
    pub fn resize(&mut self, size: SurfaceSize) {
        self.drawable = size.is_drawable();
        if !self.drawable {
            return;
        }

        self.configuration.width = size.width;
        self.configuration.height = size.height;
        self.depth_target = DepthTarget::new(&self.device, size.width, size.height);
        self.configure_surface();
    }

    /// Draws a scene, optional diagnostics/action/reticle images and typed equipment HUD.
    ///
    /// `before_present` lets the platform runtime issue its presentation
    /// notification at the exact boundary without introducing a `winit`
    /// dependency into this crate.
    #[allow(clippy::too_many_arguments)]
    pub fn render(
        &mut self,
        scene: SceneFrame<'_>,
        overlay_image: Option<OverlayImage<'_>>,
        action_bar_image: Option<OverlayImage<'_>>,
        reticle_image: Option<OverlayImage<'_>>,
        toolbar_state: Option<EquipmentToolbar>,
        scale_factor: f64,
        before_present: impl FnOnce(),
    ) -> Result<RenderOutcome, RendererError> {
        if let Some(gpu_timer) = self.gpu_timer.as_mut() {
            gpu_timer.poll(&self.device);
        }
        if !self.drawable {
            return Ok(RenderOutcome::Idle);
        }

        let frame = match self.surface.get_current_texture() {
            wgpu::CurrentSurfaceTexture::Success(frame) => frame,
            wgpu::CurrentSurfaceTexture::Suboptimal(frame) => {
                drop(frame);
                self.configure_surface();
                return Ok(RenderOutcome::Retry);
            }
            wgpu::CurrentSurfaceTexture::Timeout => return Ok(RenderOutcome::Retry),
            wgpu::CurrentSurfaceTexture::Occluded => return Ok(RenderOutcome::Idle),
            wgpu::CurrentSurfaceTexture::Outdated => {
                self.configure_surface();
                return Ok(RenderOutcome::Retry);
            }
            wgpu::CurrentSurfaceTexture::Lost => return Ok(RenderOutcome::SurfaceLost),
            wgpu::CurrentSurfaceTexture::Validation => {
                return Err(RendererError::new(
                    "failed to acquire surface image",
                    "wgpu reported a validation error",
                ));
            }
        };

        let view = frame
            .texture
            .create_view(&wgpu::TextureViewDescriptor::default());
        let cpu_render_started_at = std::time::Instant::now();
        let aspect_ratio = self.configuration.width as f32 / self.configuration.height as f32;
        let prepared_scene = prepare_scene(scene, aspect_ratio)
            .map_err(|error| RendererError::new("failed to prepare scene", error))?;
        self.spheres.prepare(
            &self.device,
            &self.queue,
            scene,
            self.configuration.width,
            self.configuration.height,
        )?;
        self.ship_mesh
            .prepare(&self.queue, scene, prepared_scene.view_projection)?;
        self.held_item
            .prepare(&self.queue, scene.camera, aspect_ratio, scene.held_item)?;
        self.resource_meshes.prepare(
            &self.device,
            &self.queue,
            scene.camera,
            prepared_scene.view_projection,
            scene.resource_meshes,
        )?;
        self.ensure_instance_capacity(prepared_scene.instances.len())?;
        self.queue.write_buffer(
            &self.camera_buffer,
            0,
            &encode_f32s(prepared_scene.view_projection),
        );
        if !prepared_scene.instances.is_empty() {
            self.queue.write_buffer(
                &self.instance_buffer,
                0,
                &encode_instances(&prepared_scene.instances),
            );
        }
        self.overlay
            .prepare(
                &self.device,
                &self.queue,
                self.configuration.width,
                self.configuration.height,
                overlay_image,
                scene.camera,
                prepared_scene.view_projection,
                &self.depth_target.view,
            )
            .map_err(|error| RendererError::new("failed to prepare overlay", error))?;
        if let Some(state) = toolbar_state {
            self.toolbar_raster.update(state, scale_factor);
        }
        let toolbar_image = toolbar_state.and_then(|_| self.toolbar_raster.image());
        self.toolbar
            .prepare(
                &self.device,
                &self.queue,
                self.configuration.width,
                self.configuration.height,
                toolbar_image,
                scene.camera,
                prepared_scene.view_projection,
                &self.depth_target.view,
            )
            .map_err(|error| RendererError::new("failed to prepare equipment toolbar", error))?;
        // Toolbar occupies the lowest HUD band; global/transient text stacks above it.
        let action_bar_image = action_bar_image.map(|mut image| {
            if toolbar_state.is_some() && image.placement == OverlayPlacement::BottomCenter {
                image.placement = OverlayPlacement::BottomCenterInset(
                    self.toolbar_raster
                        .message_inset([self.configuration.width, self.configuration.height]),
                );
            }
            image
        });
        self.action_bar
            .prepare(
                &self.device,
                &self.queue,
                self.configuration.width,
                self.configuration.height,
                action_bar_image,
                scene.camera,
                prepared_scene.view_projection,
                &self.depth_target.view,
            )
            .map_err(|error| RendererError::new("failed to prepare action bar", error))?;
        self.reticle
            .prepare(
                &self.device,
                &self.queue,
                self.configuration.width,
                self.configuration.height,
                reticle_image,
                scene.camera,
                prepared_scene.view_projection,
                &self.depth_target.view,
            )
            .map_err(|error| RendererError::new("failed to prepare reticle", error))?;
        let mut encoder = self
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("Salimon validation frame encoder"),
            });
        let timing_slot = self.gpu_timer.as_mut().and_then(GpuTimer::acquire_slot);
        {
            let timestamp_writes = timing_slot.and_then(|_| {
                self.gpu_timer.as_ref().map(|timer| {
                    let mut writes = timer.timestamp_writes();
                    writes.end_of_pass_write_index = None;
                    writes
                })
            });
            let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("Salimon validation render pass"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &view,
                    depth_slice: None,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(CLEAR_COLOR),
                        store: wgpu::StoreOp::Store,
                    },
                })],
                depth_stencil_attachment: Some(wgpu::RenderPassDepthStencilAttachment {
                    view: &self.depth_target.view,
                    depth_ops: Some(wgpu::Operations {
                        load: wgpu::LoadOp::Clear(0.0),
                        store: wgpu::StoreOp::Store,
                    }),
                    stencil_ops: None,
                }),
                timestamp_writes,
                occlusion_query_set: None,
                multiview_mask: None,
            });
            if prepared_scene.instance_count > 0 {
                render_pass.set_pipeline(&self.pipeline);
                render_pass.set_bind_group(0, &self.camera_bind_group, &[]);
                render_pass.set_vertex_buffer(0, self.cube_vertex_buffer.slice(..));
                render_pass.set_vertex_buffer(1, self.instance_buffer.slice(..));
                render_pass.draw(0..CUBE_VERTEX_COUNT, 0..prepared_scene.instance_count);
            }
            self.spheres.draw(&mut render_pass);
            self.ship_mesh.draw(&mut render_pass);
            self.resource_meshes.draw(&mut render_pass);
            self.held_item.draw(&mut render_pass);
        }
        {
            let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("Salimon overlay pass"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &view,
                    depth_slice: None,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Load,
                        store: wgpu::StoreOp::Store,
                    },
                })],
                depth_stencil_attachment: None,
                // Preserve the timing interval across scene and overlay passes.
                timestamp_writes: timing_slot.and_then(|_| {
                    self.gpu_timer.as_ref().map(|timer| {
                        let mut writes = timer.timestamp_writes();
                        writes.beginning_of_pass_write_index = None;
                        writes
                    })
                }),
                occlusion_query_set: None,
                multiview_mask: None,
            });
            self.overlay.draw(&mut render_pass);
            self.toolbar.draw(&mut render_pass);
            self.action_bar.draw(&mut render_pass);
            self.reticle.draw(&mut render_pass);
        }
        if let (Some(gpu_timer), Some(slot_index)) = (self.gpu_timer.as_ref(), timing_slot) {
            gpu_timer.resolve_and_map(&mut encoder, slot_index);
        }

        self.queue.submit([encoder.finish()]);
        let cpu_render_time = cpu_render_started_at.elapsed();
        before_present();
        self.queue.present(frame);
        self.presented_frames = self.presented_frames.saturating_add(1);
        if self.presented_frames == 1 || self.presented_frames.is_multiple_of(60) {
            self.cached_gpu_memory =
                self.device
                    .generate_allocator_report()
                    .map(|report| GpuMemoryMetrics {
                        allocated_bytes: report.total_allocated_bytes,
                        reserved_bytes: report.total_reserved_bytes,
                    });
        }

        let gpu_frame_time = match self.gpu_timer.as_ref() {
            Some(timer) => timer
                .latest()
                .map_or(GpuFrameTime::Pending, GpuFrameTime::Measured),
            None => GpuFrameTime::Unsupported,
        };
        let overlay_draw_calls = u32::from(self.overlay.is_visible())
            + u32::from(self.action_bar.is_visible())
            + u32::from(self.reticle.is_visible())
            + u32::from(self.toolbar.is_visible());
        let scene_draw_calls = scene_draw_calls(prepared_scene.instance_count)
            + scene_draw_calls(self.spheres.count())
            + self.ship_mesh.draw_count()
            + self.held_item.count()
            + self.resource_meshes.draw_count();
        let object_count = prepared_scene.instance_count
            + self.spheres.count()
            + self.ship_mesh.count()
            + self.held_item.count()
            + self.resource_meshes.object_count();
        Ok(RenderOutcome::Presented(RenderStats {
            cpu_render_time,
            gpu_frame_time,
            visible_objects: object_count,
            rendered_objects: object_count,
            scene_draw_calls,
            total_draw_calls: scene_draw_calls + overlay_draw_calls,
            gpu_memory: self.cached_gpu_memory,
        }))
    }

    fn configure_surface(&self) {
        debug_assert!(self.configuration.width > 0 && self.configuration.height > 0);
        self.surface.configure(&self.device, &self.configuration);
    }

    fn ensure_instance_capacity(&mut self, instance_count: usize) -> Result<(), RendererError> {
        let required_capacity = instance_count.max(1);
        if required_capacity <= self.instance_capacity {
            return Ok(());
        }

        let new_capacity = required_capacity
            .checked_next_power_of_two()
            .unwrap_or(required_capacity);
        let buffer_size = u64::try_from(new_capacity)
            .ok()
            .and_then(|count| count.checked_mul(INSTANCE_STRIDE))
            .ok_or_else(|| {
                RendererError::new("failed to prepare scene", "instance buffer size overflowed")
            })?;
        if buffer_size > self.device.limits().max_buffer_size {
            return Err(RendererError::new(
                "failed to prepare scene",
                format!(
                    "instance buffer requires {buffer_size} bytes but the device limit is {}",
                    self.device.limits().max_buffer_size
                ),
            ));
        }

        self.instance_buffer = self.device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("Salimon cuboid instances"),
            size: buffer_size,
            usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });
        self.instance_capacity = new_capacity;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::{
        CameraFrame, SceneFrame, SceneInputError, SceneInstance, SurfaceSize, camera_basis,
        infinite_reverse_z_projection, prepare_scene, scene_draw_calls,
    };

    const TEST_NEAR_PLANE: f32 = 0.05;

    fn camera(position_meters: [f64; 3], target_meters: [f64; 3]) -> CameraFrame {
        CameraFrame {
            position_meters,
            target_meters,
            up: [0.0, 1.0, 0.0],
            vertical_fov_radians: 60.0_f32.to_radians(),
            near_plane_meters: TEST_NEAR_PLANE,
        }
    }

    fn transform(matrix: [f32; 16], vector: [f32; 4]) -> [f32; 4] {
        std::array::from_fn(|row| {
            (0..4)
                .map(|column| matrix[column * 4 + row] * vector[column])
                .sum()
        })
    }

    fn assert_approximately_equal(actual: f32, expected: f32) {
        assert!(
            (actual - expected).abs() <= 1.0e-6,
            "expected {expected}, got {actual}"
        );
    }

    #[test]
    fn surface_size_requires_two_non_zero_dimensions() {
        assert!(SurfaceSize::new(1, 1).is_drawable());
        assert!(!SurfaceSize::new(0, 1).is_drawable());
        assert!(!SurfaceSize::new(1, 0).is_drawable());
        assert!(!SurfaceSize::new(0, 0).is_drawable());
    }

    #[test]
    fn camera_basis_is_right_handed_and_orthonormal() {
        let basis = camera_basis(camera([0.0; 3], [0.0, 0.0, -1.0])).unwrap();

        assert_eq!(basis.right, [1.0, -0.0, 0.0]);
        assert_eq!(basis.up, [0.0, 1.0, 0.0]);
        assert_eq!(basis.forward, [0.0, 0.0, -1.0]);
    }

    #[test]
    fn camera_relative_conversion_is_invariant_under_large_translation() {
        let local_camera = camera([0.0, 0.0, 0.0], [0.0, 0.0, -100.0]);
        let local_instances = [SceneInstance {
            center_meters: [10.0, -4.0, -1_000.0],
            half_extents_meters: [1.0, 2.0, 3.0],
            color: [0.2, 0.4, 0.8, 1.0],
        }];
        let local = prepare_scene(
            SceneFrame {
                camera: local_camera,
                resource_meshes: &[],
                instances: &local_instances,
                spheres: &[],
                light: None,
                ship: None,
                held_item: None,
            },
            16.0 / 9.0,
        )
        .unwrap();

        let origin = [1_000_000_000_000.0, -1_000_000_000_000.0, 500_000_000_000.0];
        let translated_camera = camera(origin, [origin[0], origin[1], origin[2] - 100.0]);
        let translated_instances = [SceneInstance {
            center_meters: [origin[0] + 10.0, origin[1] - 4.0, origin[2] - 1_000.0],
            ..local_instances[0]
        }];
        let translated = prepare_scene(
            SceneFrame {
                camera: translated_camera,
                resource_meshes: &[],
                instances: &translated_instances,
                spheres: &[],
                light: None,
                ship: None,
                held_item: None,
            },
            16.0 / 9.0,
        )
        .unwrap();

        assert_eq!(translated, local);
    }

    #[test]
    fn reverse_z_maps_near_to_one_and_has_no_finite_far_plane() {
        let camera = camera([0.0; 3], [0.0, 0.0, -1.0]);
        let projection = infinite_reverse_z_projection(camera, 16.0 / 9.0).unwrap();
        let distances = [camera.near_plane_meters, 1.0, 1_000.0, 1_000_000_000_000.0];
        let depths = distances.map(|distance| {
            let clip = transform(projection, [0.0, 0.0, -distance, 1.0]);
            clip[2] / clip[3]
        });

        assert_approximately_equal(depths[0], 1.0);
        assert!(depths.windows(2).all(|pair| pair[0] > pair[1]));
        assert!(depths.last().is_some_and(|depth| *depth > 0.0));
    }

    #[test]
    fn reverse_z_depth_spacing_covers_the_prototype_endpoints() {
        let near_step = reverse_z_distance_step(TEST_NEAR_PLANE, 12.0);
        let far_step = reverse_z_distance_step(TEST_NEAR_PLANE, 120_000_000.0);

        assert!((1.3e-6..1.4e-6).contains(&near_step));
        assert!((7.9..8.1).contains(&far_step));
    }

    #[test]
    fn scene_validation_rejects_degenerate_camera_and_instances() {
        let coincident_camera = camera([4.0, 5.0, 6.0], [4.0, 5.0, 6.0]);
        assert_eq!(
            prepare_scene(
                SceneFrame {
                    camera: coincident_camera,
                    resource_meshes: &[],
                    instances: &[],
                    spheres: &[],
                    light: None,
                    ship: None,
                    held_item: None,
                },
                1.0,
            ),
            Err(SceneInputError::CameraDirectionInvalid)
        );

        let invalid_instances = [SceneInstance {
            center_meters: [0.0, 0.0, -10.0],
            half_extents_meters: [1.0, 0.0, 1.0],
            color: [1.0; 4],
        }];
        assert_eq!(
            prepare_scene(
                SceneFrame {
                    camera: camera([0.0; 3], [0.0, 0.0, -1.0]),
                    resource_meshes: &[],
                    instances: &invalid_instances,
                    spheres: &[],
                    light: None,
                    ship: None,
                    held_item: None,
                },
                1.0,
            ),
            Err(SceneInputError::InstanceHalfExtentsInvalid(0))
        );
    }

    #[test]
    fn frustum_culling_controls_scene_counts_and_draw_calls() {
        let instances = [
            SceneInstance {
                center_meters: [0.0, 0.0, -10.0],
                half_extents_meters: [1.0; 3],
                color: [1.0; 4],
            },
            SceneInstance {
                center_meters: [2.0, 0.0, -20.0],
                half_extents_meters: [1.0; 3],
                color: [0.5; 4],
            },
            SceneInstance {
                center_meters: [0.0, 0.0, 10.0],
                half_extents_meters: [1.0; 3],
                color: [0.8; 4],
            },
            SceneInstance {
                center_meters: [100.0, 0.0, -10.0],
                half_extents_meters: [1.0; 3],
                color: [0.3; 4],
            },
        ];
        let prepared = prepare_scene(
            SceneFrame {
                camera: camera([0.0; 3], [0.0, 0.0, -1.0]),
                resource_meshes: &[],
                instances: &instances,
                spheres: &[],
                light: None,
                ship: None,
                held_item: None,
            },
            1.0,
        )
        .unwrap();

        assert_eq!(prepared.instance_count, 2);
        assert_eq!(scene_draw_calls(prepared.instance_count), 1);
        assert_eq!(scene_draw_calls(0), 0);
    }

    fn reverse_z_distance_step(near_plane: f32, distance: f32) -> f64 {
        let depth = near_plane / distance;
        let next_nearer_depth = f32::from_bits(depth.to_bits() + 1);
        let represented_distance = f64::from(near_plane) / f64::from(depth);
        let next_nearer_distance = f64::from(near_plane) / f64::from(next_nearer_depth);

        represented_distance - next_nearer_distance
    }
}

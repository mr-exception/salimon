//! Custom GPU presentation for the Salimon client.
//!
//! This crate owns `wgpu` resources and surface presentation. It deliberately
//! has no dependency on world, character, or ship state.

mod capture;
mod fixtures;
mod frame;
mod initialize;
mod surface;
pub use fixtures::{
    FIXTURE_LAYOUT_VERSION, FixtureBatchStats, FixtureInstance, FixtureMesh, FixtureNode,
    FixturePose, fixture_template_bounds, resolve_fixture_hierarchy,
};
mod cockpit_instruments;
mod scanner_hologram;
pub use scanner_hologram::{HologramContact, HologramContactState, scanner_hologram};
mod consumable_wheel;
mod equipment_toolbar;
#[allow(dead_code)]
mod habitation_geometry;
pub use consumable_wheel::{ConsumableWheel, ConsumableWheelRaster, consumable_wheel_hit_test};
mod archive_terminal;
mod gpu_timing;
mod held_item;
mod item_image;
mod machinery;
mod overlay;
#[allow(dead_code)] // Shared generated geometry; renderer consumes only animation dimensions.
mod ramp_geometry;
mod resource_mesh;
mod ship_mesh;
pub use archive_terminal::ARCHIVE_TERMINAL_METERS;
mod atmosphere;
mod organisms;
pub use organisms::{MAX_ORGANISM_SEGMENTS, OrganismSegment};
mod weather;
pub use weather::{WEATHER_GRID, WeatherScene};
mod sky;
pub use atmosphere::AtmosphereScene;
mod spheres;
pub use sky::{MAX_SKY_STARS, SkyScene, SkyStar};
mod terrain;
pub mod ui_style;
pub use terrain::{TerrainMask, TerrainPatch, TerrainPatchKey, TerrainVertex};
mod planet_style;
mod surface_textures;

use std::error::Error;
use std::fmt;
use std::time::Duration;

pub use cockpit_instruments::{
    CockpitInstruments, CoreInstruments, FabricatorInstruments, NavigationInstruments,
    NearbyBodyInstruments, PredictionInstruments,
};
pub use equipment_toolbar::{
    ConsumableIcon, EquipmentIcon, EquipmentSlot, EquipmentToolbar, toolbar_message_inset,
};
use gpu_timing::GpuTimer;
pub use held_item::HeldItemInstance;
pub use machinery::{MachineryFeedback, MachineryState};
pub use overlay::{OverlayImage, OverlayPlacement, overlay_available_size};
pub use resource_mesh::{ResourceMesh, ResourceMeshInstance, SPHERE_AUTHORED_RADIUS_METERS};
pub use ship_mesh::ShipMeshInstance;
pub use spheres::{PointLight, SphereInstance, SurfaceMaterial};

const CLEAR_COLOR: wgpu::Color = wgpu::Color {
    r: 0.008,
    g: 0.015,
    b: 0.035,
    a: 1.0,
};
const DEPTH_FORMAT: wgpu::TextureFormat = wgpu::TextureFormat::Depth32Float;
const CAMERA_UNIFORM_SIZE: u64 = 64;
const INSTANCE_STRIDE: u64 = 64;
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
    /// Unit local-to-world quaternion [x, y, z, w].
    pub orientation: [f32; 4],
    pub center_meters: [f64; 3],
    pub half_extents_meters: [f32; 3],
    pub color: [f32; 4],
}

/// Borrowed scene snapshot for one renderer frame.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct SceneFrame<'a> {
    pub sky: Option<&'a SkyScene>,
    pub atmosphere: Option<AtmosphereScene>,
    pub weather: Option<&'a WeatherScene>,
    pub organisms: &'a [OrganismSegment],
    pub camera: CameraFrame,
    pub instances: &'a [SceneInstance],
    pub resource_meshes: &'a [ResourceMeshInstance],
    pub fixtures: &'a [FixtureInstance],
    pub spheres: &'a [SphereInstance],
    pub terrain: &'a [std::sync::Arc<TerrainPatch>],
    pub terrain_mask: Option<TerrainMask>,
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
    pub driver: String,
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
    pub resource_batch: resource_mesh::ResourceBatchStats,
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
    orientation: [f32; 4],
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
    InstanceOrientationInvalid(usize),
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
            Self::InstanceOrientationInvalid(index) => write!(
                formatter,
                "scene instance {index} orientation must be a finite unit quaternion"
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
        let norm: f32 = instance.orientation.iter().map(|v| v * v).sum();
        if !instance.orientation.iter().all(|v| v.is_finite()) || (norm - 1.).abs() > 1e-5 {
            return Err(SceneInputError::InstanceOrientationInvalid(index));
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
            orientation: instance.orientation,
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
    // For squared quaternion norm n, the shader transform is (1-n)I+nR.
    // Bound its scale too, including the admitted f32 unit-norm rounding.
    let norm: f32 = instance.orientation.iter().map(|v| v * v).sum();
    let radius = scaled_length(instance.half_extents) * (norm + (1.0 - norm).abs());
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
    let mut values = Vec::with_capacity(instances.len().saturating_mul(16));
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
            instance.orientation[0],
            instance.orientation[1],
            instance.orientation[2],
            instance.orientation[3],
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
    condition_overlay: overlay::OverlayRenderer,
    overlay: overlay::OverlayRenderer,
    action_bar: overlay::OverlayRenderer,
    reticle: overlay::OverlayRenderer,
    toolbar: overlay::OverlayRenderer,
    toolbar_raster: equipment_toolbar::ToolbarRaster,
    spheres: spheres::SphereRenderer,
    sky: sky::SkyRenderer,
    atmosphere: atmosphere::AtmosphereRenderer,
    weather: weather::WeatherRenderer,
    organisms: organisms::OrganismRenderer,
    terrain: terrain::TerrainRenderer,
    ship_mesh: ship_mesh::ShipMeshRenderer,
    fixtures: fixtures::FixtureMeshRenderer,
    last_fixture_batch: FixtureBatchStats,
    held_item: held_item::HeldItemRenderer,
    resource_meshes: resource_mesh::ResourceMeshRenderer,
    gpu_timer: Option<GpuTimer>,
    cached_gpu_memory: Option<GpuMemoryMetrics>,
    presented_frames: u64,
    drawable: bool,
    info: RendererInfo,
    capture_requested: bool,
    capture_result: Option<Result<Vec<u8>, RendererError>>,
}

impl Renderer {
    pub fn request_capture(&mut self) -> Result<(), RendererError> {
        if !self
            .configuration
            .usage
            .contains(wgpu::TextureUsages::COPY_SRC)
        {
            return Err(RendererError::new(
                "frame capture failed",
                "surface does not support COPY_SRC",
            ));
        }
        self.capture_requested = true;
        self.capture_result = None;
        Ok(())
    }
    pub fn take_capture(&mut self) -> Option<Result<Vec<u8>, RendererError>> {
        self.capture_result.take()
    }

    pub fn terrain_presentation_ready(&self, patches: &[std::sync::Arc<TerrainPatch>]) -> bool {
        self.terrain.presentation_ready(patches)
    }
    /// CPU preparation including buffer creation and queue writes; not GPU execution time.
    pub fn terrain_upload_times(&self) -> (u128, u128) {
        self.terrain.upload_times()
    }

    pub fn terrain_uploads(&self) -> (usize, usize) {
        self.terrain.uploads()
    }
    pub fn terrain_gpu_bytes(&self) -> usize {
        self.terrain.bytes()
    }
    pub fn terrain_ready(&self, patches: &[std::sync::Arc<TerrainPatch>]) -> bool {
        self.terrain.ready(patches)
    }

    /// Creates a renderer for an owned native surface target.
    ///
    /// Passing an owned target (for example `Arc<winit::window::Window>`) gives
    /// the surface a static lifetime without unsafe code.
    #[must_use]
    /// Last prepared fixture workload; independent of hardware budget approval.
    pub const fn fixture_batch_stats(&self) -> FixtureBatchStats {
        self.last_fixture_batch
    }

    /// Actual resident ship buffers, uniform bytes, triangles and current draw count.
    pub fn ship_resource_stats(&self) -> [u64; 4] {
        self.ship_mesh.resource_stats()
    }

    pub const fn info(&self) -> &RendererInfo {
        &self.info
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
            orientation: [
                0.,
                std::f32::consts::FRAC_1_SQRT_2,
                0.,
                std::f32::consts::FRAC_1_SQRT_2,
            ],
            center_meters: [10.0, -4.0, -1_000.0],
            half_extents_meters: [1.0, 2.0, 3.0],
            color: [0.2, 0.4, 0.8, 1.0],
        }];
        let local = prepare_scene(
            SceneFrame {
                sky: None,
                atmosphere: None,
                weather: None,
                organisms: &[],
                camera: local_camera,
                resource_meshes: &[],
                fixtures: &[],
                instances: &local_instances,
                spheres: &[],
                terrain: &[],
                terrain_mask: None,
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
            orientation: [
                0.,
                std::f32::consts::FRAC_1_SQRT_2,
                0.,
                std::f32::consts::FRAC_1_SQRT_2,
            ],
            center_meters: [origin[0] + 10.0, origin[1] - 4.0, origin[2] - 1_000.0],
            ..local_instances[0]
        }];
        let translated = prepare_scene(
            SceneFrame {
                sky: None,
                atmosphere: None,
                weather: None,
                organisms: &[],
                camera: translated_camera,
                resource_meshes: &[],
                fixtures: &[],
                instances: &translated_instances,
                spheres: &[],
                terrain: &[],
                terrain_mask: None,
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
                    sky: None,
                    atmosphere: None,
                    weather: None,
                    organisms: &[],
                    camera: coincident_camera,
                    resource_meshes: &[],
                    fixtures: &[],
                    instances: &[],
                    spheres: &[],
                    terrain: &[],
                    terrain_mask: None,
                    light: None,
                    ship: None,
                    held_item: None,
                },
                1.0,
            ),
            Err(SceneInputError::CameraDirectionInvalid)
        );

        let invalid_instances = [SceneInstance {
            orientation: [0., 0., 0., 1.],
            center_meters: [0.0, 0.0, -10.0],
            half_extents_meters: [1.0, 0.0, 1.0],
            color: [1.0; 4],
        }];
        assert_eq!(
            prepare_scene(
                SceneFrame {
                    sky: None,
                    atmosphere: None,
                    weather: None,
                    organisms: &[],
                    camera: camera([0.0; 3], [0.0, 0.0, -1.0]),
                    resource_meshes: &[],
                    fixtures: &[],
                    instances: &invalid_instances,
                    spheres: &[],
                    terrain: &[],
                    terrain_mask: None,
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
                orientation: [0., 0., 0., 1.],
                center_meters: [0.0, 0.0, -10.0],
                half_extents_meters: [1.0; 3],
                color: [1.0; 4],
            },
            SceneInstance {
                orientation: [0., 0., 0., 1.],
                center_meters: [2.0, 0.0, -20.0],
                half_extents_meters: [1.0; 3],
                color: [0.5; 4],
            },
            SceneInstance {
                orientation: [0., 0., 0., 1.],
                center_meters: [0.0, 0.0, 10.0],
                half_extents_meters: [1.0; 3],
                color: [0.8; 4],
            },
            SceneInstance {
                orientation: [0., 0., 0., 1.],
                center_meters: [100.0, 0.0, -10.0],
                half_extents_meters: [1.0; 3],
                color: [0.3; 4],
            },
        ];
        let prepared = prepare_scene(
            SceneFrame {
                sky: None,
                atmosphere: None,
                weather: None,
                organisms: &[],
                camera: camera([0.0; 3], [0.0, 0.0, -1.0]),
                resource_meshes: &[],
                fixtures: &[],
                instances: &instances,
                spheres: &[],
                terrain: &[],
                terrain_mask: None,
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

#[cfg(test)]
mod glb_candidate_tests {
    pub fn candidate(asset: &str) -> Vec<u8> {
        let root = std::env::var_os("SALIMON_GLB_CANDIDATES").expect(
            "set SALIMON_GLB_CANDIDATES to the absolute offline candidate output directory",
        );
        std::fs::read(std::path::Path::new(&root).join(asset).join("model.glb"))
            .expect("offline candidate must exist; run process_glb.py all first")
    }
}

mod cabin_lighting;
pub use cabin_lighting::CabinLighting;

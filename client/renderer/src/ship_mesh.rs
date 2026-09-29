//! Renderer-owned loading and presentation of the checked-in Phase 0 ship GLB.

use wgpu::util::DeviceExt;

use crate::cockpit_instruments::{ATLAS_HEIGHT, ATLAS_WIDTH, InstrumentAtlas};
use crate::{CockpitInstruments, DEPTH_FORMAT, RendererError, SceneFrame, encode_f32s};

const SHIP_GLB: &[u8] = include_bytes!("../../assets/ship/export/salimon_phase0_ship.glb");
const VERTEX_FLOATS: usize = 18;
const VERTEX_STRIDE: u64 = (VERTEX_FLOATS * size_of::<f32>()) as u64;
const UNIFORM_SIZE: u64 = 160;
const OPEN_DOOR_OFFSET_METERS: f32 = 4.50;

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ShipMeshInstance {
    pub position_meters: [f64; 3],
    /// Unit quaternion `[x, y, z, w]` rotating ship-local coordinates to world.
    pub orientation: [f32; 4],
    pub door_open: bool,
    /// Read-only presentation values, kept live independently of cockpit control.
    pub instruments: CockpitInstruments,
}

#[derive(Debug)]
struct ShipGeometry {
    opaque_vertices: Vec<f32>,
    glass_vertices: Vec<f32>,
    opaque_vertex_count: u32,
    glass_vertex_count: u32,
}

fn load_geometry() -> Result<ShipGeometry, RendererError> {
    let gltf = gltf::Gltf::from_slice(SHIP_GLB)
        .map_err(|error| RendererError::new("failed to load Phase 0 ship GLB", error))?;
    let blob = gltf.blob.as_deref().ok_or_else(|| {
        RendererError::new("failed to load Phase 0 ship GLB", "GLB has no binary chunk")
    })?;
    let mut opaque_vertices = Vec::new();
    let mut glass_vertices = Vec::new();
    let mut interior_nodes = vec![false; gltf.document.nodes().len()];
    for node in gltf.document.nodes() {
        if node.name() == Some("Interior") {
            mark_interior_nodes(node, &mut interior_nodes);
        }
    }

    for node in gltf.document.nodes() {
        let Some(mesh) = node.mesh() else { continue };
        let door_flag = f32::from(node.name() == Some("Exit_Door"));
        let interior_flag = f32::from(interior_nodes[node.index()]);
        let display_panel = match node.name() {
            Some("Monitor_Center") => 1.0,
            Some("Monitor_Port") => 2.0,
            Some("Monitor_Starboard") => 3.0,
            _ => 0.0,
        };
        let matrix = node.transform().matrix();
        for primitive in mesh.primitives() {
            if primitive.mode() != gltf::mesh::Mode::Triangles {
                return Err(RendererError::new(
                    "failed to load Phase 0 ship GLB",
                    "ship meshes must use triangle primitives",
                ));
            }
            let reader = primitive.reader(|buffer| (buffer.index() == 0).then_some(blob));
            let positions: Vec<_> = reader
                .read_positions()
                .ok_or_else(|| {
                    RendererError::new(
                        "failed to load Phase 0 ship GLB",
                        "mesh is missing positions",
                    )
                })?
                .collect();
            let normals: Vec<_> = reader
                .read_normals()
                .ok_or_else(|| {
                    RendererError::new("failed to load Phase 0 ship GLB", "mesh is missing normals")
                })?
                .collect();
            let texcoords: Vec<_> = reader
                .read_tex_coords(0)
                .map(|coordinates| coordinates.into_f32().collect())
                .unwrap_or_default();
            if display_panel > 0.0 && texcoords.len() != positions.len() {
                return Err(RendererError::new(
                    "failed to load cockpit display",
                    "display surfaces require one UV coordinate per vertex",
                ));
            }
            if positions.len() != normals.len() {
                return Err(RendererError::new(
                    "failed to load Phase 0 ship GLB",
                    "position/normal counts differ",
                ));
            }
            let indices: Vec<u32> = reader.read_indices().map_or_else(
                || (0..positions.len() as u32).collect(),
                |values| values.into_u32().collect(),
            );
            let material = primitive.material();
            let color = material.pbr_metallic_roughness().base_color_factor();
            let is_glass = material.name() == Some("Cockpit Glass");
            let vertices = if is_glass {
                &mut glass_vertices
            } else {
                &mut opaque_vertices
            };
            let emissive = material.emissive_factor();
            for index in indices {
                let vertex_index = usize::try_from(index).map_err(|_| {
                    RendererError::new(
                        "failed to load Phase 0 ship GLB",
                        "index does not fit usize",
                    )
                })?;
                let position = positions.get(vertex_index).ok_or_else(|| {
                    RendererError::new(
                        "failed to load Phase 0 ship GLB",
                        "index exceeds position count",
                    )
                })?;
                let normal = normals.get(vertex_index).ok_or_else(|| {
                    RendererError::new(
                        "failed to load Phase 0 ship GLB",
                        "index exceeds normal count",
                    )
                })?;
                let transformed_position = transform_position(matrix, *position);
                let transformed_normal = normalize(transform_direction(matrix, *normal));
                let uv = texcoords.get(vertex_index).copied().unwrap_or([0.0; 2]);
                vertices.extend_from_slice(&[
                    transformed_position[0],
                    transformed_position[1],
                    transformed_position[2],
                    transformed_normal[0],
                    transformed_normal[1],
                    transformed_normal[2],
                    color[0],
                    color[1],
                    color[2],
                    color[3],
                    door_flag,
                    emissive[0],
                    emissive[1],
                    emissive[2],
                    interior_flag,
                    uv[0],
                    uv[1],
                    display_panel,
                ]);
            }
        }
    }
    let opaque_vertex_count =
        u32::try_from(opaque_vertices.len() / VERTEX_FLOATS).map_err(|_| {
            RendererError::new(
                "failed to load Phase 0 ship GLB",
                "opaque vertex count exceeds u32",
            )
        })?;
    let glass_vertex_count = u32::try_from(glass_vertices.len() / VERTEX_FLOATS).map_err(|_| {
        RendererError::new(
            "failed to load Phase 0 ship GLB",
            "glass vertex count exceeds u32",
        )
    })?;
    if opaque_vertex_count == 0 || glass_vertex_count == 0 {
        return Err(RendererError::new(
            "failed to load Phase 0 ship GLB",
            "ship must contain both opaque geometry and cockpit glass",
        ));
    }
    Ok(ShipGeometry {
        opaque_vertices,
        glass_vertices,
        opaque_vertex_count,
        glass_vertex_count,
    })
}

/// The asset's presentation grouping provides a warm cabin fill without adding
/// gameplay identity, runtime lights, or another ship draw.
fn mark_interior_nodes(node: gltf::Node<'_>, interior_nodes: &mut [bool]) {
    interior_nodes[node.index()] = true;
    for child in node.children() {
        mark_interior_nodes(child, interior_nodes);
    }
}

fn transform_position(matrix: [[f32; 4]; 4], point: [f32; 3]) -> [f32; 3] {
    [
        matrix[0][0] * point[0] + matrix[1][0] * point[1] + matrix[2][0] * point[2] + matrix[3][0],
        matrix[0][1] * point[0] + matrix[1][1] * point[1] + matrix[2][1] * point[2] + matrix[3][1],
        matrix[0][2] * point[0] + matrix[1][2] * point[1] + matrix[2][2] * point[2] + matrix[3][2],
    ]
}

fn transform_direction(matrix: [[f32; 4]; 4], direction: [f32; 3]) -> [f32; 3] {
    [
        matrix[0][0] * direction[0] + matrix[1][0] * direction[1] + matrix[2][0] * direction[2],
        matrix[0][1] * direction[0] + matrix[1][1] * direction[1] + matrix[2][1] * direction[2],
        matrix[0][2] * direction[0] + matrix[1][2] * direction[1] + matrix[2][2] * direction[2],
    ]
}

fn normalize(vector: [f32; 3]) -> [f32; 3] {
    let length = vector.iter().map(|value| value * value).sum::<f32>().sqrt();
    if length > 1.0e-6 {
        vector.map(|value| value / length)
    } else {
        [0.0, 1.0, 0.0]
    }
}

pub(crate) struct ShipMeshRenderer {
    opaque_pipeline: wgpu::RenderPipeline,
    glass_pipeline: wgpu::RenderPipeline,
    bind_group: wgpu::BindGroup,
    uniform: wgpu::Buffer,
    opaque_vertices: wgpu::Buffer,
    glass_vertices: wgpu::Buffer,
    opaque_vertex_count: u32,
    glass_vertex_count: u32,
    visible: bool,
    instrument_atlas: InstrumentAtlas,
    instrument_texture: wgpu::Texture,
}

fn create_pipeline(
    device: &wgpu::Device,
    format: wgpu::TextureFormat,
    layout: &wgpu::PipelineLayout,
    shader: &wgpu::ShaderModule,
    glass: bool,
) -> wgpu::RenderPipeline {
    device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
        label: Some(if glass {
            "Salimon Phase 0 cockpit glass pipeline"
        } else {
            "Salimon Phase 0 opaque ship pipeline"
        }),
        layout: Some(layout),
        vertex: wgpu::VertexState {
            module: shader,
            entry_point: Some("vs_main"),
            buffers: &[Some(wgpu::VertexBufferLayout {
                array_stride: VERTEX_STRIDE,
                step_mode: wgpu::VertexStepMode::Vertex,
                attributes: &[
                    wgpu::VertexAttribute {
                        format: wgpu::VertexFormat::Float32x3,
                        offset: 0,
                        shader_location: 0,
                    },
                    wgpu::VertexAttribute {
                        format: wgpu::VertexFormat::Float32x3,
                        offset: 12,
                        shader_location: 1,
                    },
                    wgpu::VertexAttribute {
                        format: wgpu::VertexFormat::Float32x4,
                        offset: 24,
                        shader_location: 2,
                    },
                    wgpu::VertexAttribute {
                        format: wgpu::VertexFormat::Float32,
                        offset: 40,
                        shader_location: 3,
                    },
                    wgpu::VertexAttribute {
                        format: wgpu::VertexFormat::Float32x3,
                        offset: 44,
                        shader_location: 4,
                    },
                    wgpu::VertexAttribute {
                        format: wgpu::VertexFormat::Float32,
                        offset: 56,
                        shader_location: 5,
                    },
                    wgpu::VertexAttribute {
                        format: wgpu::VertexFormat::Float32x3,
                        offset: 60,
                        shader_location: 6,
                    },
                ],
            })],
            compilation_options: wgpu::PipelineCompilationOptions::default(),
        },
        primitive: wgpu::PrimitiveState::default(),
        depth_stencil: Some(wgpu::DepthStencilState {
            format: DEPTH_FORMAT,
            depth_write_enabled: Some(!glass),
            depth_compare: Some(wgpu::CompareFunction::Greater),
            stencil: wgpu::StencilState::default(),
            bias: wgpu::DepthBiasState::default(),
        }),
        multisample: wgpu::MultisampleState::default(),
        fragment: Some(wgpu::FragmentState {
            module: shader,
            entry_point: Some("fs_main"),
            targets: &[Some(wgpu::ColorTargetState {
                format,
                blend: Some(if glass {
                    wgpu::BlendState::ALPHA_BLENDING
                } else {
                    wgpu::BlendState::REPLACE
                }),
                write_mask: wgpu::ColorWrites::ALL,
            })],
            compilation_options: wgpu::PipelineCompilationOptions::default(),
        }),
        multiview_mask: None,
        cache: None,
    })
}

impl ShipMeshRenderer {
    pub(crate) fn new(
        device: &wgpu::Device,
        format: wgpu::TextureFormat,
    ) -> Result<Self, RendererError> {
        let geometry = load_geometry()?;
        let uniform = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("Salimon ship camera and transform"),
            size: UNIFORM_SIZE,
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });
        let layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("Salimon ship bindings"),
            entries: &[
                wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::VERTEX_FRAGMENT,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: wgpu::BufferSize::new(UNIFORM_SIZE),
                    },
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 1,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Texture {
                        sample_type: wgpu::TextureSampleType::Float { filterable: true },
                        view_dimension: wgpu::TextureViewDimension::D2,
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
            ],
        });
        // One small, cached atlas serves all three panels in the existing opaque
        // draw. The mesh and all other ship materials remain immutable.
        let instrument_texture = device.create_texture(&wgpu::TextureDescriptor {
            label: Some("Salimon cockpit instrument atlas"),
            size: wgpu::Extent3d {
                width: ATLAS_WIDTH,
                height: ATLAS_HEIGHT,
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Rgba8UnormSrgb,
            usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
            view_formats: &[],
        });
        let instrument_view =
            instrument_texture.create_view(&wgpu::TextureViewDescriptor::default());
        let instrument_sampler = device.create_sampler(&wgpu::SamplerDescriptor {
            label: Some("Salimon cockpit screen filtering"),
            min_filter: wgpu::FilterMode::Linear,
            mag_filter: wgpu::FilterMode::Linear,
            ..Default::default()
        });
        let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("Salimon ship bind group"),
            layout: &layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: uniform.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::TextureView(&instrument_view),
                },
                wgpu::BindGroupEntry {
                    binding: 2,
                    resource: wgpu::BindingResource::Sampler(&instrument_sampler),
                },
            ],
        });
        let shader = device.create_shader_module(wgpu::include_wgsl!("ship_mesh.wgsl"));
        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("Salimon ship pipeline layout"),
            bind_group_layouts: &[Some(&layout)],
            immediate_size: 0,
        });
        let opaque_pipeline = create_pipeline(device, format, &pipeline_layout, &shader, false);
        let glass_pipeline = create_pipeline(device, format, &pipeline_layout, &shader, true);
        let opaque_vertex_bytes = encode_f32s(geometry.opaque_vertices);
        let opaque_vertices = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Salimon Phase 0 opaque ship vertices"),
            contents: &opaque_vertex_bytes,
            usage: wgpu::BufferUsages::VERTEX,
        });
        let glass_vertex_bytes = encode_f32s(geometry.glass_vertices);
        let glass_vertices = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Salimon Phase 0 cockpit glass vertices"),
            contents: &glass_vertex_bytes,
            usage: wgpu::BufferUsages::VERTEX,
        });
        Ok(Self {
            opaque_pipeline,
            glass_pipeline,
            bind_group,
            uniform,
            opaque_vertices,
            glass_vertices,
            opaque_vertex_count: geometry.opaque_vertex_count,
            glass_vertex_count: geometry.glass_vertex_count,
            visible: false,
            instrument_atlas: InstrumentAtlas::new(),
            instrument_texture,
        })
    }

    pub(crate) fn prepare(
        &mut self,
        queue: &wgpu::Queue,
        scene: SceneFrame<'_>,
        view_projection: [f32; 16],
    ) -> Result<(), RendererError> {
        let Some(ship) = scene.ship else {
            self.visible = false;
            return Ok(());
        };
        if !ship.position_meters.iter().all(|value| value.is_finite())
            || !ship.orientation.iter().all(|value| value.is_finite())
        {
            return Err(RendererError::new(
                "failed to prepare ship",
                "ship pose must be finite",
            ));
        }
        let center: [f32; 3] = std::array::from_fn(|index| {
            (ship.position_meters[index] - scene.camera.position_meters[index]) as f32
        });
        if !center.iter().all(|value| value.is_finite()) {
            return Err(RendererError::new(
                "failed to prepare ship",
                "camera-relative pose exceeds f32 range",
            ));
        }
        let axes = quaternion_axes(ship.orientation);
        if self.instrument_atlas.update(ship.instruments) {
            queue.write_texture(
                wgpu::TexelCopyTextureInfo {
                    texture: &self.instrument_texture,
                    mip_level: 0,
                    origin: wgpu::Origin3d::ZERO,
                    aspect: wgpu::TextureAspect::All,
                },
                &self.instrument_atlas.pixels,
                wgpu::TexelCopyBufferLayout {
                    offset: 0,
                    bytes_per_row: Some(ATLAS_WIDTH * 4),
                    rows_per_image: Some(ATLAS_HEIGHT),
                },
                wgpu::Extent3d {
                    width: ATLAS_WIDTH,
                    height: ATLAS_HEIGHT,
                    depth_or_array_layers: 1,
                },
            );
        }
        let mut values = Vec::with_capacity(40);
        values.extend_from_slice(&view_projection);
        values.extend_from_slice(&[
            center[0],
            center[1],
            center[2],
            0.0,
            axes[0][0],
            axes[0][1],
            axes[0][2],
            0.0,
            axes[1][0],
            axes[1][1],
            axes[1][2],
            0.0,
            axes[2][0],
            axes[2][1],
            axes[2][2],
            0.0,
            0.0,
            f32::from(ship.door_open) * OPEN_DOOR_OFFSET_METERS,
            0.0,
            0.0,
            0.0,
            0.0,
            0.0,
            0.0,
        ]);
        queue.write_buffer(&self.uniform, 0, &encode_f32s(values));
        self.visible = true;
        Ok(())
    }

    pub(crate) fn draw<'pass>(&'pass self, pass: &mut wgpu::RenderPass<'pass>) {
        if !self.visible {
            return;
        }
        pass.set_pipeline(&self.opaque_pipeline);
        pass.set_bind_group(0, &self.bind_group, &[]);
        pass.set_vertex_buffer(0, self.opaque_vertices.slice(..));
        pass.draw(0..self.opaque_vertex_count, 0..1);
        pass.set_pipeline(&self.glass_pipeline);
        pass.set_vertex_buffer(0, self.glass_vertices.slice(..));
        pass.draw(0..self.glass_vertex_count, 0..1);
    }

    pub(crate) fn count(&self) -> u32 {
        u32::from(self.visible)
    }

    pub(crate) fn draw_count(&self) -> u32 {
        u32::from(self.visible) * 2
    }
}

fn quaternion_axes(quaternion: [f32; 4]) -> [[f32; 3]; 3] {
    let length = quaternion
        .iter()
        .map(|value| value * value)
        .sum::<f32>()
        .sqrt();
    let [x, y, z, w] = if length > 1.0e-6 {
        quaternion.map(|value| value / length)
    } else {
        [0.0, 0.0, 0.0, 1.0]
    };
    [
        [
            1.0 - 2.0 * (y * y + z * z),
            2.0 * (x * y + z * w),
            2.0 * (x * z - y * w),
        ],
        [
            2.0 * (x * y - z * w),
            1.0 - 2.0 * (x * x + z * z),
            2.0 * (y * z + x * w),
        ],
        [
            2.0 * (x * z + y * w),
            2.0 * (y * z - x * w),
            1.0 - 2.0 * (x * x + y * y),
        ],
    ]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn checked_in_ship_glb_loads_as_renderable_triangles() {
        let geometry = load_geometry().expect("checked-in ship must stay renderer-compatible");
        assert!(geometry.opaque_vertex_count > geometry.glass_vertex_count);
        assert_eq!(
            geometry.opaque_vertices.len(),
            geometry.opaque_vertex_count as usize * VERTEX_FLOATS
        );
        assert_eq!(
            geometry.glass_vertices.len(),
            geometry.glass_vertex_count as usize * VERTEX_FLOATS
        );
        assert!(
            geometry
                .opaque_vertices
                .chunks_exact(VERTEX_FLOATS)
                .any(|vertex| vertex[10] == 1.0)
        );
        assert!(
            geometry
                .glass_vertices
                .chunks_exact(VERTEX_FLOATS)
                .all(|vertex| vertex[9] > 0.0 && vertex[9] < 0.5)
        );
    }

    #[test]
    fn material_emission_stays_separate_from_base_color_and_cabin_fill() {
        let geometry = load_geometry().expect("checked-in ship must load");
        let gltf = gltf::Gltf::from_slice(SHIP_GLB).expect("checked-in GLB must parse");
        let vertices: Vec<_> = geometry
            .opaque_vertices
            .chunks_exact(VERTEX_FLOATS)
            .chain(geometry.glass_vertices.chunks_exact(VERTEX_FLOATS))
            .collect();
        let emissive_materials: Vec<_> = gltf
            .materials()
            .filter(|material| material.emissive_factor().iter().any(|value| *value > 0.0))
            .collect();
        assert!(!emissive_materials.is_empty());
        for material in emissive_materials {
            let base = material.pbr_metallic_roughness().base_color_factor();
            let emission = material.emissive_factor();
            assert!(
                vertices
                    .iter()
                    .any(|vertex| { vertex[6..10] == base && vertex[11..14] == emission })
            );
        }
        assert!(vertices.iter().any(|vertex| vertex[14] == 1.0));
        assert!(vertices.iter().any(|vertex| vertex[14] == 0.0));
        assert!(vertices.iter().any(|vertex| {
            vertex[14] == 1.0 && vertex[11..14].iter().any(|value| *value > 0.0)
        }));
        assert!(vertices.iter().any(|vertex| {
            vertex[14] == 0.0 && vertex[11..14].iter().any(|value| *value > 0.0)
        }));
    }

    #[test]
    fn checked_in_ship_stays_within_the_lightweight_asset_budget() {
        let geometry = load_geometry().expect("checked-in ship must load");
        let gltf = gltf::Gltf::from_slice(SHIP_GLB).expect("checked-in GLB must parse");
        assert!(SHIP_GLB.len() <= 512 * 1024);
        assert!((geometry.opaque_vertex_count + geometry.glass_vertex_count) / 3 <= 6_000);
        assert!(
            gltf.meshes()
                .map(|mesh| mesh.primitives().len())
                .sum::<usize>()
                <= 120
        );
        assert!(gltf.materials().len() <= 13);
    }

    #[test]
    fn cockpit_panels_are_three_textured_quads_in_the_opaque_draw() {
        let geometry = load_geometry().expect("ship display contract must load");
        // Asset version 9 bakes the second 0.7 reduction and seatward shift
        // into all three assemblies; the renderer consumes final meters.
        for (panel, center, width, height, normal) in [
            (1.0, [3.95446, 1.0401, 0.0], 0.882, 0.3332, [-1.0, 0.0, 0.0]),
            (
                2.0,
                [3.70, 1.0625, 2.35],
                0.63,
                0.315,
                normalize([-0.94, 0.0, -2.35]),
            ),
            (
                3.0,
                [3.70, 1.0625, -2.35],
                0.63,
                0.315,
                normalize([-0.94, 0.0, 2.35]),
            ),
        ] {
            let vertices: Vec<_> = geometry
                .opaque_vertices
                .chunks_exact(VERTEX_FLOATS)
                .filter(|vertex| vertex[17] == panel)
                .collect();
            assert_eq!(vertices.len(), 6, "each screen is exactly two triangles");
            for vertex in &vertices {
                assert_eq!(vertex[14], 1.0, "screens are interior surfaces");
                assert!(vertex[15..17].iter().all(|uv| (0.0..=1.0).contains(uv)));
                let horizontal_offset = (vertex[15] - 0.5) * width;
                let expected_position = [
                    center[0] + normal[2] * horizontal_offset,
                    center[1] + (0.5 - vertex[16]) * height,
                    center[2] - normal[0] * horizontal_offset,
                ];
                for (axis, expected) in expected_position.into_iter().enumerate() {
                    assert!(
                        (vertex[axis] - expected).abs() < 1.0e-5,
                        "panel {panel} axis {axis} must preserve its authored dimensions and top-left UV fit"
                    );
                }
                for (axis, expected) in normal.into_iter().enumerate() {
                    assert!(
                        (vertex[axis + 3] - expected).abs() < 1.0e-5,
                        "panel {panel} must retain its pilot-facing normal"
                    );
                }
            }
            for corner in [[0.0, 0.0], [1.0, 0.0], [0.0, 1.0], [1.0, 1.0]] {
                assert!(vertices.iter().any(|vertex| vertex[15..17] == corner));
            }
        }
        assert!(
            geometry
                .glass_vertices
                .chunks_exact(VERTEX_FLOATS)
                .all(|vertex| vertex[17] == 0.0)
        );
    }

    #[test]
    fn task10_door_offset_matches_the_horizontal_asset_scale() {
        assert_eq!(OPEN_DOOR_OFFSET_METERS, 4.50);
    }
}

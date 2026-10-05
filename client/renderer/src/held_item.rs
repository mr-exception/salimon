//! Static authored handheld visual. No gameplay policy or world-space state.
use crate::{CameraFrame, DEPTH_FORMAT, RendererError, encode_f32s, infinite_reverse_z_projection};
use wgpu::util::DeviceExt;

const GLB: &[u8] = include_bytes!("../../assets/items/mining-tool/model.glb");
const FLOATS: usize = 11;

/// Presentation of the checked-in mining tool; None stows it.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct HeldItemInstance {
    pub active: bool,
}

fn geometry() -> Result<Vec<f32>, RendererError> {
    let asset =
        gltf::Gltf::from_slice(GLB).map_err(|e| RendererError::new("load item.mining-tool", e))?;
    let blob = asset
        .blob
        .as_deref()
        .ok_or_else(|| RendererError::new("load item.mining-tool", "missing binary chunk"))?;
    // This consumer deliberately requires a grip-centered, baked static mesh.
    // Reject transforms rather than silently ignoring a future authoring edit.
    for node in asset.nodes() {
        if node.transform().decomposed() != ([0.0; 3], [0.0, 0.0, 0.0, 1.0], [1.0; 3]) {
            return Err(RendererError::new(
                "load item.mining-tool",
                "nodes must retain identity transforms at SOCKET_Grip",
            ));
        }
    }
    if !asset.nodes().any(|node| node.name() == Some("SOCKET_Grip")) {
        return Err(RendererError::new(
            "load item.mining-tool",
            "missing SOCKET_Grip",
        ));
    }
    let mut vertices = Vec::new();
    for mesh in asset.meshes() {
        for primitive in mesh.primitives() {
            if primitive.mode() != gltf::mesh::Mode::Triangles {
                return Err(RendererError::new(
                    "load item.mining-tool",
                    "expected triangles",
                ));
            }
            let reader = primitive.reader(|buffer| (buffer.index() == 0).then_some(blob));
            let positions: Vec<_> = reader
                .read_positions()
                .ok_or_else(|| RendererError::new("load item.mining-tool", "missing positions"))?
                .collect();
            let normals: Vec<_> = reader
                .read_normals()
                .ok_or_else(|| RendererError::new("load item.mining-tool", "missing normals"))?
                .collect();
            let indices: Vec<u32> = reader.read_indices().map_or_else(
                || (0..positions.len() as u32).collect(),
                |i| i.into_u32().collect(),
            );
            let material = primitive.material();
            let color = material.pbr_metallic_roughness().base_color_factor();
            let indicator = f32::from(material.name() == Some("Tool_Status"));
            for index in indices {
                let p = positions.get(index as usize).ok_or_else(|| {
                    RendererError::new("load item.mining-tool", "position index out of bounds")
                })?;
                let n = normals.get(index as usize).ok_or_else(|| {
                    RendererError::new("load item.mining-tool", "normal index out of bounds")
                })?;
                vertices.extend_from_slice(p);
                vertices.extend_from_slice(n);
                vertices.extend_from_slice(&color);
                vertices.push(indicator);
            }
        }
    }
    if vertices.is_empty() || !vertices.iter().all(|v| v.is_finite()) {
        return Err(RendererError::new(
            "load item.mining-tool",
            "empty or nonfinite geometry",
        ));
    }
    Ok(vertices)
}

pub(crate) struct HeldItemRenderer {
    pipeline: wgpu::RenderPipeline,
    uniform: wgpu::Buffer,
    binding: wgpu::BindGroup,
    vertices: wgpu::Buffer,
    count: u32,
    visible: bool,
}
impl HeldItemRenderer {
    pub(crate) fn new(
        device: &wgpu::Device,
        format: wgpu::TextureFormat,
    ) -> Result<Self, RendererError> {
        let geometry = geometry()?;
        let count = u32::try_from(geometry.len() / FLOATS)
            .map_err(|e| RendererError::new("load item.mining-tool", e))?;
        let vertices = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Mining tool vertices"),
            contents: &encode_f32s(geometry),
            usage: wgpu::BufferUsages::VERTEX,
        });
        let uniform = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("Held item placement"),
            size: 80,
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });
        let layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("Held item bindings"),
            entries: &[wgpu::BindGroupLayoutEntry {
                binding: 0,
                visibility: wgpu::ShaderStages::VERTEX,
                ty: wgpu::BindingType::Buffer {
                    ty: wgpu::BufferBindingType::Uniform,
                    has_dynamic_offset: false,
                    min_binding_size: wgpu::BufferSize::new(80),
                },
                count: None,
            }],
        });
        let binding = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("Held item binding"),
            layout: &layout,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: uniform.as_entire_binding(),
            }],
        });
        let shader = device.create_shader_module(wgpu::include_wgsl!("held_item.wgsl"));
        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("Held item layout"),
            bind_group_layouts: &[Some(&layout)],
            immediate_size: 0,
        });
        let attributes = wgpu::vertex_attr_array![
            0 => Float32x3, 1 => Float32x3, 2 => Float32x4, 3 => Float32
        ];
        let pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("Held item pipeline"),
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
            count,
            visible: false,
        })
    }
    pub(crate) fn prepare(
        &mut self,
        queue: &wgpu::Queue,
        camera: CameraFrame,
        aspect: f32,
        item: Option<HeldItemInstance>,
    ) -> Result<(), RendererError> {
        self.visible = item.is_some();
        if let Some(item) = item {
            let projection = infinite_reverse_z_projection(camera, aspect)
                .map_err(|e| RendererError::new("prepare held item", e))?;
            let mut values = projection.to_vec();
            values.extend_from_slice(&[f32::from(item.active), 0.0, 0.0, 0.0]);
            queue.write_buffer(&self.uniform, 0, &encode_f32s(values));
        }
        Ok(())
    }
    pub(crate) fn draw<'a>(&'a self, pass: &mut wgpu::RenderPass<'a>) {
        if self.visible {
            pass.set_pipeline(&self.pipeline);
            pass.set_bind_group(0, &self.binding, &[]);
            pass.set_vertex_buffer(0, self.vertices.slice(..));
            pass.draw(0..self.count, 0..1);
        }
    }
    pub(crate) fn count(&self) -> u32 {
        u32::from(self.visible)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn authored_mesh_has_grip_and_separate_status_region() {
        let v = geometry().expect("validated mining tool must load");
        assert!(v.len() / FLOATS / 3 <= 2000);
        let vertices = v.as_chunks::<FLOATS>().0;
        assert!(vertices.iter().any(|p| p[10] == 1.0));
        assert!(vertices.iter().any(|p| p[10] == 0.0));
        // Every vertex stays in front of the 5cm near plane after grip placement.
        assert!(vertices.iter().all(|p| 0.55 + p[0] > 0.05));
    }
    #[test]
    fn projection_is_independent_of_camera_position_pitch_and_gravity() {
        let a = CameraFrame {
            position_meters: [0.0; 3],
            target_meters: [1.0, 0.0, 0.0],
            up: [0.0, 1.0, 0.0],
            vertical_fov_radians: 1.2,
            near_plane_meters: 0.05,
        };
        let b = CameraFrame {
            position_meters: [1e12; 3],
            target_meters: [1e12, 1e12 + 1.0, 1e12 + 0.01],
            up: [1.0, 0.0, 0.0],
            ..a
        };
        assert_eq!(
            infinite_reverse_z_projection(a, 1.6),
            infinite_reverse_z_projection(b, 1.6)
        );
    }
}

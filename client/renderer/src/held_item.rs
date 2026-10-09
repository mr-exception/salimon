//! Static authored handheld visual. No gameplay policy or world-space state.
use crate::{
    CameraFrame, DEPTH_FORMAT, RendererError, encode_f32s, infinite_reverse_z_projection,
    multiply_mat4,
};
use wgpu::util::DeviceExt;

const GLB: &[u8] = include_bytes!("../../assets/items/mining-tool/model.glb");
const FLOATS: usize = 15;

/// Presentation of the checked-in mining tool; None stows it.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct HeldItemInstance {
    pub active: bool,
}

// Column-major asset-to-view transform: asset +X forward, +Y up, +Z right.
// Work directly in the active camera's orthonormal view frame. This is equivalent
// to camera-relative basis placement followed by the scene view rotation, without
// adding/subtracting a distant absolute eye position or applying look twice.
const GRIP_TO_VIEW: [f32; 16] = [
    0.0, 0.0, -1.0, 0.0, 0.0, 1.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 0.20, -0.20, -0.55, 1.0,
];

fn model_to_clip(camera: CameraFrame, aspect: f32) -> Result<[f32; 16], RendererError> {
    let projection = infinite_reverse_z_projection(camera, aspect)
        .map_err(|e| RendererError::new("prepare held item", e))?;
    Ok(multiply_mat4(projection, GRIP_TO_VIEW))
}

fn geometry() -> Result<(Vec<f32>, crate::item_image::ItemImage), RendererError> {
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
    let image = asset
        .images()
        .next()
        .ok_or_else(|| RendererError::new("load item.mining-tool", "missing surface atlas"))?;
    if asset.images().count() != 1 {
        return Err(RendererError::new(
            "load item.mining-tool",
            "expected one surface atlas",
        ));
    }
    let gltf::image::Source::View {
        view,
        mime_type: "image/png",
    } = image.source()
    else {
        return Err(RendererError::new(
            "load item.mining-tool",
            "atlas must be embedded PNG",
        ));
    };
    let bytes = blob
        .get(view.offset()..view.offset() + view.length())
        .ok_or_else(|| RendererError::new("load item.mining-tool", "atlas range exceeds blob"))?;
    let atlas = crate::item_image::decode(bytes)?;
    if [atlas.width, atlas.height] != [256, 256]
        || atlas.pixels.as_chunks::<4>().0.iter().any(|p| p[3] != 255)
    {
        return Err(RendererError::new(
            "load item.mining-tool",
            "expected opaque 256x256 atlas",
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
            let pbr = material.pbr_metallic_roughness();
            let texture = pbr.base_color_texture().ok_or_else(|| {
                RendererError::new("load item.mining-tool", "missing base-color texture")
            })?;
            if texture.tex_coord() != 0
                || texture.texture().source().index() != image.index()
                || pbr.metallic_roughness_texture().is_some()
                || material.normal_texture().is_some()
                || material.occlusion_texture().is_some()
                || material.emissive_texture().is_some()
                || material.alpha_mode() != gltf::material::AlphaMode::Opaque
            {
                return Err(RendererError::new(
                    "load item.mining-tool",
                    "unsupported material/texture contract",
                ));
            }
            let uv: Vec<_> = reader
                .read_tex_coords(0)
                .ok_or_else(|| RendererError::new("load item.mining-tool", "missing TEXCOORD_0"))?
                .into_f32()
                .collect();
            let color = pbr.base_color_factor();
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
                vertices.extend_from_slice(uv.get(index as usize).ok_or_else(|| {
                    RendererError::new("load item.mining-tool", "UV index out of bounds")
                })?);
                vertices.extend_from_slice(&[pbr.metallic_factor(), pbr.roughness_factor()]);
            }
        }
    }
    if vertices.is_empty() || !vertices.iter().all(|v| v.is_finite()) {
        return Err(RendererError::new(
            "load item.mining-tool",
            "empty or nonfinite geometry",
        ));
    }
    Ok((vertices, atlas))
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
        queue: &wgpu::Queue,
        format: wgpu::TextureFormat,
    ) -> Result<Self, RendererError> {
        let (geometry, atlas) = geometry()?;
        let texture = surface_texture(device, queue, atlas);
        let texture_view = texture.create_view(&Default::default());
        let sampler = device.create_sampler(&wgpu::SamplerDescriptor {
            label: Some("Mining tool surface sampler"),
            mag_filter: wgpu::FilterMode::Linear,
            min_filter: wgpu::FilterMode::Linear,
            mipmap_filter: wgpu::MipmapFilterMode::Linear,
            ..Default::default()
        });
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
            entries: &[
                wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::VERTEX_FRAGMENT,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: wgpu::BufferSize::new(80),
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
        let binding = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("Held item binding"),
            layout: &layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: uniform.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::TextureView(&texture_view),
                },
                wgpu::BindGroupEntry {
                    binding: 2,
                    resource: wgpu::BindingResource::Sampler(&sampler),
                },
            ],
        });
        let shader = device.create_shader_module(wgpu::include_wgsl!("held_item.wgsl"));
        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("Held item layout"),
            bind_group_layouts: &[Some(&layout)],
            immediate_size: 0,
        });
        let attributes = wgpu::vertex_attr_array![
            0 => Float32x3, 1 => Float32x3, 2 => Float32x4, 3 => Float32,
            4 => Float32x2, 5 => Float32x2
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
            let mut values = model_to_clip(camera, aspect)?.to_vec();
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

// Build a full color-correct mip chain once at initialization. Sampling sRGB
// texels yields linear albedo; averaging in linear space avoids dark mip edges.
fn surface_texture(
    device: &wgpu::Device,
    queue: &wgpu::Queue,
    atlas: crate::item_image::ItemImage,
) -> wgpu::Texture {
    let texture = device.create_texture(&wgpu::TextureDescriptor {
        label: Some("Mining tool surface atlas"),
        size: wgpu::Extent3d {
            width: atlas.width,
            height: atlas.height,
            depth_or_array_layers: 1,
        },
        mip_level_count: 9,
        sample_count: 1,
        dimension: wgpu::TextureDimension::D2,
        format: wgpu::TextureFormat::Rgba8UnormSrgb,
        usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
        view_formats: &[],
    });
    let (mut width, mut height, mut pixels) = (atlas.width, atlas.height, atlas.pixels);
    for level in 0..9 {
        queue.write_texture(
            wgpu::TexelCopyTextureInfo {
                texture: &texture,
                mip_level: level,
                origin: wgpu::Origin3d::ZERO,
                aspect: wgpu::TextureAspect::All,
            },
            &pixels,
            wgpu::TexelCopyBufferLayout {
                offset: 0,
                bytes_per_row: Some(width * 4),
                rows_per_image: Some(height),
            },
            wgpu::Extent3d {
                width,
                height,
                depth_or_array_layers: 1,
            },
        );
        if level < 8 {
            let mut next = Vec::with_capacity((width * height) as usize);
            for y in 0..height / 2 {
                for x in 0..width / 2 {
                    for channel in 0..3 {
                        let mut sum = 0.0_f32;
                        for dy in 0..2 {
                            for dx in 0..2 {
                                let s = f32::from(
                                    pixels[(((y * 2 + dy) * width + x * 2 + dx) * 4 + channel)
                                        as usize],
                                ) / 255.0;
                                sum += if s <= 0.04045 {
                                    s / 12.92
                                } else {
                                    ((s + 0.055) / 1.055).powf(2.4)
                                };
                            }
                        }
                        let linear = sum / 4.0;
                        let s = if linear <= 0.0031308 {
                            linear * 12.92
                        } else {
                            1.055 * linear.powf(1.0 / 2.4) - 0.055
                        };
                        next.push((s * 255.0).round() as u8);
                    }
                    next.push(255);
                }
            }
            pixels = next;
            width /= 2;
            height /= 2;
        }
    }
    texture
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn authored_mesh_has_grip_and_separate_status_region() {
        let (v, atlas) = geometry().expect("validated mining tool must load");
        assert_eq!([atlas.width, atlas.height], [256, 256]);
        assert!(atlas.pixels.as_chunks::<4>().0.iter().any(|p| p[0] != p[2]));
        assert!(v.len() / FLOATS / 3 <= 3000);
        let vertices = v.as_chunks::<FLOATS>().0;
        assert!(vertices.iter().any(|p| p[10] == 1.0));
        assert!(vertices.iter().any(|p| p[10] == 0.0));
        assert!(
            vertices
                .iter()
                .all(|p| (0.0..=1.0).contains(&p[11]) && (0.0..=1.0).contains(&p[12]))
        );
        assert!(vertices.iter().any(|p| p[13] > 0.8 && p[14] < 0.4));
        assert!(vertices.iter().any(|p| p[13] == 0.0 && p[14] > 0.8));
        // Every vertex stays in front of the 5cm near plane after grip placement.
        assert!(vertices.iter().all(|p| 0.55 + p[0] > 0.05));
    }
    fn transform(matrix: [f32; 16], p: [f32; 3]) -> [f32; 4] {
        std::array::from_fn(|row| {
            matrix[row] * p[0] + matrix[4 + row] * p[1] + matrix[8 + row] * p[2] + matrix[12 + row]
        })
    }

    fn camera(yaw: f64, pitch: f64, position: [f64; 3], tilted_gravity: bool) -> CameraFrame {
        let mut forward = [
            pitch.cos() * yaw.cos(),
            pitch.sin(),
            pitch.cos() * yaw.sin(),
        ];
        let mut up = [0.0, 1.0, 0.0];
        if tilted_gravity {
            forward.swap(0, 1);
            up.swap(0, 1);
        }
        CameraFrame {
            position_meters: position,
            target_meters: std::array::from_fn(|i| position[i] + forward[i]),
            up,
            vertical_fov_radians: 1.2,
            near_plane_meters: 0.05,
        }
    }

    #[test]
    fn placement_matches_active_camera_basis_across_yaw_pitch_and_gravity() {
        let (vertices, _) = geometry().expect("validated mining tool must load");
        for yaw in [0.0, 1.2, -2.4] {
            // Character controller's supported pitch limits.
            for pitch in [
                -std::f64::consts::FRAC_PI_2 + 0.01,
                0.0,
                std::f64::consts::FRAC_PI_2 - 0.01,
            ] {
                for tilted in [false, true] {
                    for origin in [[0.0; 3], [1e12; 3]] {
                        let camera = camera(yaw, pitch, origin, tilted);
                        let basis = crate::camera_basis(camera).unwrap();
                        let projection = infinite_reverse_z_projection(camera, 1.6).unwrap();
                        let actual = model_to_clip(camera, 1.6).unwrap();
                        let world_to_clip = multiply_mat4(projection, crate::view_rotation(basis));
                        for vertex in vertices.as_chunks::<FLOATS>().0 {
                            let p = [vertex[0], vertex[1], vertex[2]];
                            // Independent oracle: place the grip and asset axes along
                            // the actual scene camera's forward/right/orthonormal up.
                            let relative = std::array::from_fn(|i| {
                                basis.forward[i] * (0.55 + p[0])
                                    + basis.right[i] * (0.20 + p[2])
                                    + basis.up[i] * (-0.20 + p[1])
                            });
                            let expected = transform(world_to_clip, relative);
                            let clip = transform(actual, p);
                            for i in 0..4 {
                                assert!((clip[i] - expected[i]).abs() < 1e-6);
                            }
                            assert!(clip[3] > camera.near_plane_meters);
                            assert!(clip[0].abs() < clip[3] && clip[1].abs() < clip[3]);
                            assert!(clip[2] > 0.0 && clip[2] < clip[3]);
                        }
                    }
                }
            }
        }
    }

    #[test]
    fn grip_follows_look_without_changing_screen_offset_or_center_aim() {
        let a = camera(0.0, 0.0, [0.0; 3], false);
        let b = camera(1.2, 1.5, [1e12; 3], true);
        let basis_a = crate::camera_basis(a).unwrap();
        let basis_b = crate::camera_basis(b).unwrap();
        assert_ne!(basis_a.forward, basis_b.forward);
        assert_eq!(
            model_to_clip(a, 1.6).unwrap(),
            model_to_clip(b, 1.6).unwrap()
        );
        for camera in [a, b] {
            let basis = crate::camera_basis(camera).unwrap();
            let vp = multiply_mat4(
                infinite_reverse_z_projection(camera, 1.6).unwrap(),
                crate::view_rotation(basis),
            );
            let aim = transform(vp, basis.forward);
            assert!(aim[0].abs() < 1e-6 && aim[1].abs() < 1e-6);
        }
    }
}

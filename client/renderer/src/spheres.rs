//! Screen-bounded analytic spheres with body-local materials and exact surface depth.

use wgpu::util::DeviceExt;

use crate::surface_textures;
use crate::{CameraFrame, DEPTH_FORMAT, RendererError, SceneFrame, camera_basis, encode_f32s};

const INSTANCE_STRIDE: u64 = 96;
const CAMERA_SIZE: u64 = 64;

/// Renderer-owned generic material styles; these are not celestial identities.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[repr(u8)]
pub enum SurfaceMaterial {
    Stone,
    Ochre,
    Oceanic,
    Slate,
    Rust,
    Emissive,
}

/// Absolute, renderer-neutral spherical surface. Radius stays f64 until preparation.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct SphereInstance {
    pub center_meters: [f64; 3],
    pub radius_meters: f64,
    pub material: SurfaceMaterial,
}

/// Unshadowed point illumination. No collision or world behavior is implied.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct PointLight {
    pub position_meters: [f64; 3],
    pub color: [f32; 3],
}

#[derive(Clone, Copy, Debug, PartialEq)]
struct PreparedSphere {
    bounds: [f32; 4],
    direction_radius: [f32; 4],
    distance_altitude_material: [f32; 4],
    light_position: [f32; 4],
    light_color: [f32; 4],
    detail_origin: [f32; 4],
}

impl PreparedSphere {
    fn values(self) -> impl Iterator<Item = f32> {
        [
            self.bounds,
            self.direction_radius,
            self.distance_altitude_material,
            self.light_position,
            self.light_color,
            self.detail_origin,
        ]
        .into_iter()
        .flatten()
    }
}

fn invalid(detail: impl std::fmt::Display) -> RendererError {
    RendererError::new("invalid sphere scene", detail)
}

fn prepare_sphere(
    sphere: SphereInstance,
    light: Option<PointLight>,
    camera: CameraFrame,
    width: u32,
    height: u32,
) -> Result<Option<PreparedSphere>, RendererError> {
    if !sphere.center_meters.iter().all(|v| v.is_finite())
        || !sphere.radius_meters.is_finite()
        || sphere.radius_meters <= 0.0
    {
        return Err(invalid(
            "sphere center must be finite and radius positive and finite",
        ));
    }
    if let Some(light) = light
        && (!light.position_meters.iter().all(|v| v.is_finite())
            || !light.color.iter().all(|v| v.is_finite() && *v >= 0.0))
    {
        return Err(invalid(
            "light position/color must be finite; color cannot be negative",
        ));
    }
    let basis = camera_basis(camera).map_err(invalid)?;
    let relative: [f64; 3] =
        std::array::from_fn(|i| sphere.center_meters[i] - camera.position_meters[i]);
    let distance = relative[0].hypot(relative[1]).hypot(relative[2]);
    let radius = sphere.radius_meters;
    // Keep cancellation-prone subtraction on the CPU. A 6 Mm radius no longer
    // quantizes a 12.125 m altitude to the 0.5 m spacing of a GPU body center.
    let altitude = distance - radius;
    let direction = if distance > 0.0 {
        relative.map(|v| v / distance)
    } else {
        [0.0, 0.0, 1.0]
    };
    let tangent_y = (f64::from(camera.vertical_fov_radians) * 0.5).tan();
    let tangent_x = tangent_y * f64::from(width) / f64::from(height);
    let view = [basis.right, basis.up, basis.forward].map(|axis| {
        relative
            .into_iter()
            .zip(axis)
            .map(|(v, a)| v * f64::from(a))
            .sum::<f64>()
    });
    if view[2] + radius < f64::from(camera.near_plane_meters) {
        return Ok(None);
    }
    let mut bounds = [-1.0_f64, -1.0, 1.0, 1.0];
    // Project a conservative camera-space box. Camera-inside and near-plane
    // crossings use the whole viewport; the analytic intersection clips it.
    if view[2] - radius > f64::from(camera.near_plane_meters) {
        for axis in 0..2 {
            let tangent = [tangent_x, tangent_y][axis];
            let coordinates = [-radius, radius].into_iter().flat_map(|offset| {
                [-radius, radius].map(|depth| (view[axis] + offset) / ((view[2] + depth) * tangent))
            });
            let min = coordinates.clone().fold(f64::INFINITY, f64::min);
            let max = coordinates.fold(f64::NEG_INFINITY, f64::max);
            let margin = 4.0 / f64::from([width, height][axis]);
            bounds[axis] = (min - margin).max(-1.0);
            bounds[axis + 2] = (max + margin).min(1.0);
        }
    }
    if bounds[0] >= bounds[2] || bounds[1] >= bounds[3] {
        return Ok(None);
    }
    let local_light = light.map_or([0.0; 3], |light| {
        std::array::from_fn(|i| (light.position_meters[i] - sphere.center_meters[i]) / radius)
    });
    let light_color = light.map_or([0.0; 3], |light| light.color);
    let prepared = PreparedSphere {
        bounds: bounds.map(|v| v as f32),
        direction_radius: [
            direction[0] as f32,
            direction[1] as f32,
            direction[2] as f32,
            radius as f32,
        ],
        distance_altitude_material: [
            distance as f32,
            altitude as f32,
            sphere.material as u8 as f32,
            f32::from(sphere.material == SurfaceMaterial::Emissive),
        ],
        light_position: [
            local_light[0] as f32,
            local_light[1] as f32,
            local_light[2] as f32,
            0.0,
        ],
        light_color: [light_color[0], light_color[1], light_color[2], 0.0],
        // Wrap camera-to-body coordinates in f64. Adding the small hit position
        // in WGSL keeps the detail tile attached to the surface during motion.
        detail_origin: [
            (-relative[0]).rem_euclid(128.0) as f32,
            (-relative[1]).rem_euclid(128.0) as f32,
            (-relative[2]).rem_euclid(128.0) as f32,
            128.0,
        ],
    };
    if !prepared.values().all(|v| v.is_finite()) || prepared.direction_radius[3] <= 0.0 {
        return Err(invalid("sphere/light exceeds finite GPU range"));
    }
    Ok(Some(prepared))
}

pub(crate) struct SphereRenderer {
    pipeline: wgpu::RenderPipeline,
    bind_group: wgpu::BindGroup,
    camera_buffer: wgpu::Buffer,
    instances: wgpu::Buffer,
    capacity: usize,
    count: u32,
}

impl SphereRenderer {
    pub(crate) fn new(
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        format: wgpu::TextureFormat,
    ) -> Self {
        let texture = device.create_texture(&wgpu::TextureDescriptor {
            label: Some("Salimon generic sphere albedo array"),
            size: wgpu::Extent3d {
                width: surface_textures::WIDTH,
                height: surface_textures::HEIGHT,
                depth_or_array_layers: surface_textures::LAYERS,
            },
            mip_level_count: surface_textures::MIP_COUNT,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            // Palette values and CPU mip averaging are both linear-light.
            format: wgpu::TextureFormat::Rgba8Unorm,
            usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
            view_formats: &[],
        });
        for layer in 0..surface_textures::LAYERS {
            for (mip, (width, height, pixels)) in
                surface_textures::mip_chain(layer).into_iter().enumerate()
            {
                queue.write_texture(
                    wgpu::TexelCopyTextureInfo {
                        texture: &texture,
                        mip_level: mip as u32,
                        origin: wgpu::Origin3d {
                            x: 0,
                            y: 0,
                            z: layer,
                        },
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
            }
        }
        let view = texture.create_view(&wgpu::TextureViewDescriptor {
            dimension: Some(wgpu::TextureViewDimension::D2Array),
            ..Default::default()
        });
        let sampler = device.create_sampler(&wgpu::SamplerDescriptor {
            label: Some("Salimon seamless trilinear sphere sampler"),
            address_mode_u: wgpu::AddressMode::Repeat,
            address_mode_v: wgpu::AddressMode::ClampToEdge,
            mag_filter: wgpu::FilterMode::Linear,
            min_filter: wgpu::FilterMode::Linear,
            mipmap_filter: wgpu::MipmapFilterMode::Linear,
            ..Default::default()
        });
        let detail_sampler = device.create_sampler(&wgpu::SamplerDescriptor {
            label: Some("Salimon periodic surface detail sampler"),
            address_mode_u: wgpu::AddressMode::Repeat,
            address_mode_v: wgpu::AddressMode::Repeat,
            mag_filter: wgpu::FilterMode::Linear,
            min_filter: wgpu::FilterMode::Linear,
            mipmap_filter: wgpu::MipmapFilterMode::Linear,
            ..Default::default()
        });
        let camera_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("Salimon sphere camera rays"),
            size: CAMERA_SIZE,
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });
        let layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("Salimon sphere bindings"),
            entries: &[
                wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: wgpu::BufferSize::new(CAMERA_SIZE),
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
        let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("Salimon sphere material bindings"),
            layout: &layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: camera_buffer.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::TextureView(&view),
                },
                wgpu::BindGroupEntry {
                    binding: 2,
                    resource: wgpu::BindingResource::Sampler(&sampler),
                },
                wgpu::BindGroupEntry {
                    binding: 3,
                    resource: wgpu::BindingResource::Sampler(&detail_sampler),
                },
            ],
        });
        let shader = device.create_shader_module(wgpu::include_wgsl!("spheres.wgsl"));
        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("Salimon analytic sphere pipeline layout"),
            bind_group_layouts: &[Some(&layout)],
            immediate_size: 0,
        });
        let attributes: [wgpu::VertexAttribute; 6] =
            std::array::from_fn(|i| wgpu::VertexAttribute {
                format: wgpu::VertexFormat::Float32x4,
                offset: i as u64 * 16,
                shader_location: i as u32,
            });
        let pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("Salimon analytic textured spheres"),
            layout: Some(&pipeline_layout),
            vertex: wgpu::VertexState {
                module: &shader,
                entry_point: Some("vs_main"),
                buffers: &[Some(wgpu::VertexBufferLayout {
                    array_stride: INSTANCE_STRIDE,
                    step_mode: wgpu::VertexStepMode::Instance,
                    attributes: &attributes,
                })],
                compilation_options: Default::default(),
            },
            primitive: wgpu::PrimitiveState::default(),
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
        let instances = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Salimon sphere instances"),
            contents: &[0; INSTANCE_STRIDE as usize],
            usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
        });
        Self {
            pipeline,
            bind_group,
            camera_buffer,
            instances,
            capacity: 1,
            count: 0,
        }
    }

    pub(crate) fn prepare(
        &mut self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        scene: SceneFrame<'_>,
        width: u32,
        height: u32,
    ) -> Result<(), RendererError> {
        let mut prepared = Vec::with_capacity(scene.spheres.len());
        for sphere in scene.spheres {
            if let Some(sphere) = prepare_sphere(*sphere, scene.light, scene.camera, width, height)?
            {
                prepared.push(sphere);
            }
        }
        self.count = u32::try_from(prepared.len()).map_err(invalid)?;
        if prepared.len() > self.capacity {
            let capacity = prepared
                .len()
                .checked_next_power_of_two()
                .ok_or_else(|| invalid("sphere capacity overflow"))?;
            let size = (capacity as u64)
                .checked_mul(INSTANCE_STRIDE)
                .ok_or_else(|| invalid("sphere buffer size overflow"))?;
            if size > device.limits().max_buffer_size {
                return Err(invalid("sphere buffer exceeds device limit"));
            }
            self.instances = device.create_buffer(&wgpu::BufferDescriptor {
                label: Some("Salimon sphere instances"),
                size,
                usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
                mapped_at_creation: false,
            });
            self.capacity = capacity;
        }
        let basis = camera_basis(scene.camera).map_err(invalid)?;
        let tangent_y = (scene.camera.vertical_fov_radians * 0.5).tan();
        let tangent_x = tangent_y * width as f32 / height as f32;
        let camera = [
            [basis.right[0], basis.right[1], basis.right[2], tangent_x],
            [basis.up[0], basis.up[1], basis.up[2], tangent_y],
            [
                basis.forward[0],
                basis.forward[1],
                basis.forward[2],
                scene.camera.near_plane_meters,
            ],
            [width as f32, height as f32, 0.0, 0.0],
        ];
        queue.write_buffer(
            &self.camera_buffer,
            0,
            &encode_f32s(camera.into_iter().flatten()),
        );
        if self.count > 0 {
            queue.write_buffer(
                &self.instances,
                0,
                &encode_f32s(prepared.into_iter().flat_map(PreparedSphere::values)),
            );
        }
        Ok(())
    }

    pub(crate) const fn count(&self) -> u32 {
        self.count
    }

    pub(crate) fn draw(&self, pass: &mut wgpu::RenderPass<'_>) {
        if self.count == 0 {
            return;
        }
        pass.set_pipeline(&self.pipeline);
        pass.set_bind_group(0, &self.bind_group, &[]);
        pass.set_vertex_buffer(0, self.instances.slice(..));
        pass.draw(0..6, 0..self.count);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    fn camera(position_meters: [f64; 3]) -> CameraFrame {
        CameraFrame {
            position_meters,
            target_meters: [
                position_meters[0],
                position_meters[1],
                position_meters[2] - 1.0,
            ],
            up: [0.0, 1.0, 0.0],
            vertical_fov_radians: std::f32::consts::FRAC_PI_3,
            near_plane_meters: 0.05,
        }
    }
    fn sphere(radius: f64, altitude: f64) -> SphereInstance {
        SphereInstance {
            center_meters: [0.0, 0.0, -radius - altitude],
            radius_meters: radius,
            material: SurfaceMaterial::Stone,
        }
    }
    /// Same stable quadratic as the shader; checks precision against f64 geometry.
    fn hit_distance(sphere: PreparedSphere, ray: [f32; 3]) -> Option<f32> {
        let [x, y, z, radius] = sphere.direction_radius;
        let [distance, altitude, _, _] = sphere.distance_altitude_material;
        let perpendicular = crate::cross(ray, [x, y, z]);
        let ratio = distance / radius;
        let discriminant = 1.0 - crate::dot(perpendicular, perpendicular) * ratio * ratio;
        if discriminant < 0.0 {
            return None;
        }
        let b = crate::dot(ray, [x, y, z]) * distance;
        let root = radius * discriminant.sqrt();
        if altitude >= 0.0 && b > 0.0 {
            Some(altitude * ((distance + radius) / (b + root)))
        } else if altitude < 0.0 {
            Some(b + root)
        } else {
            None
        }
    }

    #[test]
    fn every_showcase_radius_preserves_submeter_altitude_and_depth_across_approach() {
        for radius in [
            1_600_000.0,
            2_400_000.0,
            3_500_000.0,
            5_500_000.0,
            6_000_000.0,
            15_000_000.0,
        ] {
            for altitude in [0.125, 1.875, 12.125, radius * 0.15, 120_000_000.0] {
                let prepared =
                    prepare_sphere(sphere(radius, altitude), None, camera([0.0; 3]), 1920, 1080)
                        .unwrap()
                        .unwrap();
                let hit = hit_distance(prepared, [0.0, 0.0, -1.0]).unwrap();
                assert!(
                    (f64::from(hit) - altitude).abs() <= (altitude * 2.0e-7).max(1.0e-5),
                    "radius={radius}, altitude={altitude}, hit={hit}"
                );
                let depth = 0.05 / hit;
                assert!(depth.is_finite() && depth > 0.0 && depth < 1.0);
            }
        }
    }

    #[test]
    fn spherical_input_is_translation_invariant_at_terameter_origin() {
        let original = sphere(6_000_000.0, 12.125);
        let prepared = prepare_sphere(original, None, camera([0.0; 3]), 1920, 1080).unwrap();
        let origin = [1.0e12, -7.5e11, 2.5e11];
        let shifted = SphereInstance {
            center_meters: std::array::from_fn(|i| original.center_meters[i] + origin[i]),
            ..original
        };
        assert_eq!(
            prepared,
            prepare_sphere(shifted, None, camera(origin), 1920, 1080).unwrap()
        );
    }

    #[test]
    fn rays_miss_silhouette_and_inside_visual_volume_has_positive_exit() {
        let prepared = prepare_sphere(sphere(10.0, 100.0), None, camera([0.0; 3]), 1920, 1080)
            .unwrap()
            .unwrap();
        assert!(hit_distance(prepared, [1.0, 0.0, 0.0]).is_none());
        let inside = prepare_sphere(sphere(10.0, -10.0), None, camera([0.0; 3]), 1920, 1080)
            .unwrap()
            .unwrap();
        assert_eq!(inside.bounds, [-1.0, -1.0, 1.0, 1.0]);
        assert_eq!(hit_distance(inside, [0.0, 0.0, -1.0]), Some(10.0));
    }

    #[test]
    fn screen_bounds_cull_behind_and_offscreen_bodies_but_keep_near_plane_crossings() {
        for center in [[0.0, 0.0, 100.0], [100.0, 0.0, -10.0]] {
            let body = SphereInstance {
                center_meters: center,
                ..sphere(1.0, 10.0)
            };
            assert!(
                prepare_sphere(body, None, camera([0.0; 3]), 1920, 1080)
                    .unwrap()
                    .is_none()
            );
        }
        let close = prepare_sphere(sphere(10.0, 0.01), None, camera([0.0; 3]), 1920, 1080)
            .unwrap()
            .unwrap();
        assert_eq!(close.bounds, [-1.0, -1.0, 1.0, 1.0]);
    }

    #[test]
    fn invalid_geometry_and_light_are_rejected_before_gpu_upload() {
        for radius in [0.0, -1.0, f64::NAN, f64::INFINITY, f64::MAX] {
            assert!(
                prepare_sphere(sphere(radius, 12.0), None, camera([0.0; 3]), 1920, 1080).is_err()
            );
        }
        let light = PointLight {
            position_meters: [f64::NAN, 0.0, 0.0],
            color: [1.0; 3],
        };
        assert!(
            prepare_sphere(
                sphere(10.0, 12.0),
                Some(light),
                camera([0.0; 3]),
                1920,
                1080
            )
            .is_err()
        );
    }

    #[test]
    fn oblique_surface_ray_keeps_meter_depth_without_large_operand_cancellation() {
        let radius = 6_000_000.0;
        let altitude = 12.125;
        let prepared = prepare_sphere(sphere(radius, altitude), None, camera([0.0; 3]), 1920, 1080)
            .unwrap()
            .unwrap();
        let ray = [0.6_f32, 0.0, -0.8];
        let hit = f64::from(hit_distance(prepared, ray).unwrap());
        let b = (radius + altitude) * 0.8;
        let c = altitude * (2.0 * radius + altitude);
        let reference = c / (b + (b * b - c).sqrt());
        assert!((hit - reference).abs() < 0.00001, "{hit} vs {reference}");
    }

    #[test]
    fn detail_origin_rebases_continuously_across_a_tile_boundary() {
        let body = sphere(6_000_000.0, 12.125);
        let before = prepare_sphere(body, None, camera([127.875, 0.0, 0.0]), 1920, 1080)
            .unwrap()
            .unwrap();
        let after = prepare_sphere(body, None, camera([128.125, 0.0, 0.0]), 1920, 1080)
            .unwrap()
            .unwrap();
        let fixed_surface_x = 128.25_f32;
        let before_phase = (before.detail_origin[0] + fixed_surface_x - 127.875).rem_euclid(128.0);
        let after_phase = (after.detail_origin[0] + fixed_surface_x - 128.125).rem_euclid(128.0);
        assert_eq!(before_phase, after_phase);
    }
}

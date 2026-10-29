//! Opt-in deterministic shader evidence. Synthetic patch; no world/physics ownership.
use crate::terrain::*;
use crate::{CameraFrame, PointLight, SceneFrame, SphereInstance, SurfaceMaterial};
use std::{fmt::Write, sync::Arc};

fn block_on<F: std::future::Future>(future: F) -> F::Output {
    let mut future = std::pin::pin!(future);
    let mut context = std::task::Context::from_waker(std::task::Waker::noop());
    let deadline = std::time::Instant::now() + std::time::Duration::from_secs(30);
    loop {
        if let std::task::Poll::Ready(value) = future.as_mut().poll(&mut context) {
            return value;
        }
        assert!(
            std::time::Instant::now() < deadline,
            "GPU request timed out"
        );
        std::thread::yield_now();
    }
}

fn height(x: f32, z: f32) -> f32 {
    // Broad hill and crater on a stable 4 m grid; same positions in both revisions.
    5.0 * (-((x + 12.0).powi(2) + z * z) / 150.0).exp()
        - 4.0 * (-((x - 12.0).powi(2) + z * z) / 80.0).exp()
}
fn patch(material: SurfaceMaterial) -> Arc<TerrainPatch> {
    let mut vertices = Vec::new();
    let mut indices = Vec::new();
    for z in 0..17 {
        for x in 0..17 {
            let (x, z) = (x as f32 * 4.0 - 32.0, z as f32 * 4.0 - 32.0);
            let dx = (height(x + 0.1, z) - height(x - 0.1, z)) / 0.2;
            let dz = (height(x, z + 0.1) - height(x, z - 0.1)) / 0.2;
            let length = (dx * dx + 1.0 + dz * dz).sqrt();
            vertices.push(TerrainVertex {
                offset_meters: [x, height(x, z), z],
                normal: [-dx / length, 1.0 / length, -dz / length],
                far_offset_meters: [x, 0.0, z],
                albedo_factor: 1.0,
                strata: [0.0, 1.0, 0.0],
            });
        }
    }
    for z in 0..16 {
        for x in 0..16 {
            let a = z * 17 + x;
            indices.extend([a, a + 17, a + 1, a + 1, a + 17, a + 18]);
        }
    }
    Arc::new(TerrainPatch {
        key: TerrainPatchKey {
            body: 0,
            definition: vec![1],
            face: 0,
            level: 0,
            u: 0,
            v: 0,
            edge_levels: [0; 4],
        },
        body_center_meters: [1e12; 3],
        anchor_meters: [0.0, 1e6, 0.0],
        vertices,
        indices,
        material,
    })
}

#[test]
#[ignore = "requires GPU; SALIMON_TERRAIN_STYLE_DIR saves deterministic pixels and timestamp costs"]
fn terrain_style_reference() {
    render_reference(false);
}

#[test]
#[ignore = "requires GPU; exports day/night geology/ice and neutral controls"]
fn terrain_geology_reference() {
    render_reference(true);
}

fn render_reference(geological: bool) {
    let directory = std::path::PathBuf::from(
        std::env::var("SALIMON_TERRAIN_STYLE_DIR").expect("set evidence directory"),
    );
    std::fs::create_dir_all(&directory).unwrap();
    let gpu = wgpu::Instance::default();
    let adapter = block_on(gpu.request_adapter(&Default::default())).unwrap();
    println!("terrain style adapter={:?}", adapter.get_info());
    let timestamps = adapter.features().contains(wgpu::Features::TIMESTAMP_QUERY);
    let (device, queue) = block_on(adapter.request_device(&wgpu::DeviceDescriptor {
        required_features: if timestamps {
            wgpu::Features::TIMESTAMP_QUERY
        } else {
            wgpu::Features::empty()
        },
        ..Default::default()
    }))
    .unwrap();
    let format = wgpu::TextureFormat::Rgba8UnormSrgb;
    let mut spheres = crate::spheres::SphereRenderer::new(&device, &queue, format);
    let mut terrain = TerrainRenderer::new(&device, format, &spheres);
    let mut timer = timestamps.then(|| crate::gpu_timing::GpuTimer::new(&device, &queue));
    let size = wgpu::Extent3d {
        width: 512,
        height: 384,
        depth_or_array_layers: 1,
    };
    let texture = device.create_texture(&wgpu::TextureDescriptor {
        label: Some("terrain style evidence"),
        size,
        mip_level_count: 1,
        sample_count: 1,
        dimension: wgpu::TextureDimension::D2,
        format,
        usage: wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::COPY_SRC,
        view_formats: &[],
    });
    let view = texture.create_view(&Default::default());
    let depth = crate::DepthTarget::new(&device, size.width, size.height);
    let mut csv = String::new();
    writeln!(csv, "scene,draws,triangles,patch_gpu_bytes,uniform_upload_bytes,gpu_samples,median_gpu_ms,p95_gpu_ms").unwrap();
    let cases = if geological {
        vec![
            ("rock-day", SurfaceMaterial::Slate),
            ("rock-night", SurfaceMaterial::Slate),
            ("ice-day", SurfaceMaterial::Ice),
            ("ice-night", SurfaceMaterial::Ice),
        ]
    } else {
        vec![
            ("earth", SurfaceMaterial::Oceanic),
            ("moon", SurfaceMaterial::Slate),
            ("mars", SurfaceMaterial::Rust),
        ]
    };
    for (name, material) in cases {
        let mut transition_previous: Option<Vec<u8>> = None;
        for distance in [60.0, 79.99, 80.01, 239.99, 240.01, 300.0, 3e6] {
            let orbital = distance > 1e6;
            let mut patch = patch(material);
            if geological {
                let data = Arc::make_mut(&mut patch);
                data.key.definition = name.as_bytes().to_vec();
                for v in &mut data.vertices {
                    v.albedo_factor = if name.starts_with("ice") { 0.98 } else { 0.86 };
                    v.strata = [
                        v.offset_meters[1],
                        1.75,
                        if name.starts_with("ice") { 0.18 } else { 0.5 },
                    ];
                }
            }
            let patches = if orbital { vec![] } else { vec![patch] };
            let center = [1e12; 3];
            let target = [
                center[0],
                center[1] + if orbital { 0.0 } else { 1e6 },
                center[2],
            ];
            let position = [
                target[0] + distance * 0.3 / 0.83_f64.sqrt(),
                target[1] + distance * 0.5 / 0.83_f64.sqrt(),
                target[2] + distance * 0.7 / 0.83_f64.sqrt(),
            ];
            let camera = CameraFrame {
                position_meters: position,
                target_meters: target,
                up: [0.0, 1.0, 0.0],
                vertical_fov_radians: 1.0,
                near_plane_meters: 0.05,
            };
            let bodies = if orbital {
                vec![SphereInstance {
                    center_meters: center,
                    radius_meters: 1e6,
                    material,
                }]
            } else {
                vec![]
            };
            let scene = SceneFrame {
                sky: None,
                atmosphere: None,
                weather: None,
                camera,
                instances: &[],
                resource_meshes: &[],
                fixtures: &[],
                spheres: &bodies,
                terrain: &patches,
                terrain_mask: None,
                light: Some(PointLight {
                    position_meters: [
                        1e12 - 3e6,
                        1e12 + if name.ends_with("night") { -5e6 } else { 5e6 },
                        1e12 + 2e6,
                    ],
                    color: [1.0; 3],
                }),
                ship: None,
                held_item: None,
            };
            let aspect = size.width as f32 / size.height as f32;
            let matrix = crate::multiply_mat4(
                crate::infinite_reverse_z_projection(camera, aspect).unwrap(),
                crate::view_rotation(crate::camera_basis(camera).unwrap()),
            );
            // Lock the startup morph at its settled value; ordinary runtime remains time-based.
            terrain.fade_body = patches.first().map(|p| p.key.body);
            terrain.fade_started =
                Some(std::time::Instant::now() - std::time::Duration::from_secs(1));
            terrain.prepare(&device, &queue, scene, matrix).unwrap();
            let initial_bytes = terrain.bytes();
            terrain.prepare(&device, &queue, scene, matrix).unwrap();
            assert_eq!(terrain.bytes(), initial_bytes);
            assert_eq!(terrain.uploads().0, patches.len() * terrain.uniform_stride);
            spheres
                .prepare(&device, &queue, scene, size.width, size.height)
                .unwrap();
            let distance_label = match distance {
                60.0 => "near",
                300.0 => "far",
                79.99 => "fade-start-before",
                80.01 => "fade-start-after",
                239.99 => "fade-end-before",
                240.01 => "fade-end-after",
                _ => "orbit",
            };
            let label = format!("{name}-{distance_label}");
            let mut samples = Vec::new();
            let mut previous = None;
            for sample in 0..32 {
                let slot = timer.as_mut().map(|t| t.acquire_slot().unwrap());
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
                        timestamp_writes: timer.as_ref().map(|t| t.timestamp_writes()),
                        ..Default::default()
                    });
                    spheres.draw(&mut pass);
                    terrain.draw(&mut pass);
                }
                if let Some(t) = timer.as_ref() {
                    t.resolve_and_map(&mut encoder, slot.unwrap());
                }
                let readback = (sample < 2).then(|| {
                    crate::capture::Readback::encode(&device, &mut encoder, &texture).unwrap()
                });
                queue.submit([encoder.finish()]);
                device.poll(wgpu::PollType::wait_indefinitely()).unwrap();
                if let Some(t) = timer.as_mut() {
                    t.poll(&device);
                    if sample >= 2 {
                        samples.push(t.latest().unwrap().as_secs_f64() * 1000.0);
                    }
                }
                if let Some(readback) = readback {
                    let encoded = readback.finish(&device).unwrap();
                    let mut reader = png::Decoder::new(std::io::Cursor::new(&encoded))
                        .read_info()
                        .unwrap();
                    let mut pixels = vec![0; reader.output_buffer_size().unwrap()];
                    reader.next_frame(&mut pixels).unwrap();
                    assert!(
                        pixels
                            .as_chunks::<4>()
                            .0
                            .iter()
                            .filter(|p| p[..3] != [0, 0, 0])
                            .count()
                            > 100
                    );
                    if let Some(previous) = previous.as_ref() {
                        assert_eq!(&pixels, previous, "frozen scene changed");
                    }
                    if sample == 0 {
                        if distance_label.ends_with("after") {
                            let other = transition_previous
                                .as_ref()
                                .expect("preceding transition frame");
                            let mean_delta = pixels
                                .iter()
                                .zip(other)
                                .map(|(a, b)| u64::from(a.abs_diff(*b)))
                                .sum::<u64>() as f64
                                / pixels.len() as f64;
                            assert!(
                                mean_delta < 2.0,
                                "{label}: discontinuous shading ({mean_delta} / 255 mean pixel delta)"
                            );
                        }
                        if distance_label.starts_with("fade-") {
                            transition_previous = Some(pixels.clone());
                        }
                        std::fs::write(directory.join(format!("{label}.png")), encoded).unwrap();
                    }
                    previous = Some(pixels);
                }
            }
            samples.sort_by(f64::total_cmp);
            let median = samples.get(15).copied().unwrap_or(f64::NAN);
            let p95 = samples.get(28).copied().unwrap_or(f64::NAN);
            let triangles = if orbital { 2 } else { 512 };
            writeln!(
                csv,
                "{label},1,{triangles},{initial_bytes},{},{},{median},{p95}",
                terrain.uploads().0,
                samples.len()
            )
            .unwrap();
        }
    }
    std::fs::write(directory.join("costs.csv"), csv).unwrap();
}

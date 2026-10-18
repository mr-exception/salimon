//! Offscreen resource-only timestamp probe. Synchronization is test-only.
use super::*;

#[test]
#[ignore = "opt-in resource GPU timestamps; software GPU is not native acceptance"]
fn resource_geometry_gpu_cost() {
    let gpu = wgpu::Instance::default();
    let adapter = block_on(gpu.request_adapter(&Default::default())).unwrap();
    println!("adapter={:?}", adapter.get_info());
    if !adapter.features().contains(wgpu::Features::TIMESTAMP_QUERY) {
        println!("GPU timestamps unsupported; no timing samples collected");
        return;
    }
    let (device, queue) = block_on(adapter.request_device(&wgpu::DeviceDescriptor {
        required_features: wgpu::Features::TIMESTAMP_QUERY,
        ..Default::default()
    }))
    .unwrap();
    let texture = device.create_texture(&wgpu::TextureDescriptor {
        label: Some("Geometry evaluation color"),
        size: wgpu::Extent3d {
            width: 1920,
            height: 1080,
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
    let depth = crate::DepthTarget::new(&device, 1920, 1080);
    let mut timer = crate::gpu_timing::GpuTimer::new(&device, &queue);
    let mut renderer =
        ResourceMeshRenderer::new(&device, wgpu::TextureFormat::Rgba8UnormSrgb).unwrap();
    println!(
        "count,transforms,camera,samples,median_gpu_ms,p95_gpu_ms,p99_gpu_ms,triangles,draws,objects"
    );
    for count in [500, 1000] {
        for active in [false, true] {
            for away in [false, true] {
                let camera = CameraFrame {
                    position_meters: [1e12; 3],
                    target_meters: [1e12, 1e12, 1e12 + if away { 1.0 } else { -1.0 }],
                    up: [0.0, 1.0, 0.0],
                    vertical_fov_radians: 1.0,
                    near_plane_meters: 0.1,
                };
                let aspect = 1920.0 / 1080.0;
                let projection = crate::multiply_mat4(
                    crate::infinite_reverse_z_projection(camera, aspect).unwrap(),
                    crate::view_rotation(crate::camera_basis(camera).unwrap()),
                );
                let mut instances: Vec<_> = (0..count)
                    .map(|i| ResourceMeshInstance {
                        tint: [1.0; 4],
                        mesh: tests::ALL_MESHES[i % 6],
                        center_meters: [
                            1e12 + (i % 40) as f64 * 0.075 - 1.5,
                            1e12 + (i / 40) as f64 * 0.07 - 0.875,
                            1e12 - 4.0,
                        ],
                        authored_mesh_scale: 0.1,
                        orientation_xyzw: [0.0, 0.0, 0.0, 1.0],
                    })
                    .collect();
                let mut samples = Vec::new();
                let mut triangles = 0;
                for sample in 0..110 {
                    if active {
                        let angle = sample as f64 * 0.01;
                        for instance in &mut instances {
                            instance.orientation_xyzw = [0.0, angle.sin(), 0.0, angle.cos()];
                        }
                    }
                    let stats = renderer
                        .prepare(&device, &queue, camera, projection, aspect, &instances)
                        .unwrap();
                    triangles = stats.triangles;
                    assert_eq!(renderer.object_count(), if away { 0 } else { count as u32 });
                    let slot = timer.acquire_slot().unwrap();
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
                            depth_stencil_attachment: Some(
                                wgpu::RenderPassDepthStencilAttachment {
                                    view: &depth.view,
                                    depth_ops: Some(wgpu::Operations {
                                        load: wgpu::LoadOp::Clear(0.0),
                                        store: wgpu::StoreOp::Store,
                                    }),
                                    stencil_ops: None,
                                },
                            ),
                            timestamp_writes: Some(timer.timestamp_writes()),
                            ..Default::default()
                        });
                        renderer.draw(&mut pass);
                    }
                    timer.resolve_and_map(&mut encoder, slot);
                    queue.submit([encoder.finish()]);
                    // Drain this specific submission before reading its result;
                    // production keeps its asynchronous ring and never does this wait.
                    device.poll(wgpu::PollType::wait_indefinitely()).unwrap();
                    timer.poll(&device);
                    if sample >= 10 {
                        samples.push(
                            timer.latest().expect("timestamp readback").as_secs_f64() * 1000.0,
                        );
                    }
                }
                samples.sort_by(f64::total_cmp);
                println!(
                    "{count},{},{},100,{:.6},{:.6},{:.6},{triangles},{},{}",
                    if active { "active" } else { "static" },
                    if away { "away" } else { "facing" },
                    samples[50],
                    samples[94],
                    samples[98],
                    renderer.draw_count(),
                    renderer.object_count()
                );
                println!("samples_ms={samples:?}");
            }
        }
    }
}

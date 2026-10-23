use super::*;
use std::{
    fmt::Write,
    time::{Duration, Instant},
};
fn block_on<F: std::future::Future>(future: F) -> F::Output {
    let mut future = std::pin::pin!(future);
    let mut context = std::task::Context::from_waker(std::task::Waker::noop());
    let deadline = Instant::now() + Duration::from_secs(30);
    loop {
        if let std::task::Poll::Ready(v) = future.as_mut().poll(&mut context) {
            return v;
        }
        assert!(Instant::now() < deadline, "GPU request timed out");
        std::thread::yield_now();
    }
}

#[test]
#[ignore = "requires GPU; SALIMON_FIXTURE_EVIDENCE_DIR retains original images and timing rows"]
fn modular_fixture_gpu_evidence() {
    let directory =
        std::path::PathBuf::from(std::env::var("SALIMON_FIXTURE_EVIDENCE_DIR").unwrap());
    std::fs::create_dir_all(&directory).unwrap();
    let gpu = wgpu::Instance::default();
    let adapter = block_on(gpu.request_adapter(&Default::default())).unwrap();
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
    std::fs::write(
        directory.join("adapter.txt"),
        format!(
            "{:?}\n1280x800; fixture-only offscreen; 16 frames/case; explicit synthetic room workload\n",
            adapter.get_info()
        ),
    )
    .unwrap();
    let format = wgpu::TextureFormat::Rgba8UnormSrgb;
    let mut renderer = FixtureMeshRenderer::new(&device, format).unwrap();
    let size = wgpu::Extent3d {
        width: 1280,
        height: 800,
        depth_or_array_layers: 1,
    };
    let texture = device.create_texture(&wgpu::TextureDescriptor {
        label: Some("Scout feedback evidence"),
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
    let mut timer = timestamps.then(|| crate::gpu_timing::GpuTimer::new(&device, &queue));

    let mut csv = String::from(
        "case,sample,cpu_prepare_us,gpu_ms,draws,visible_instances,triangles,resident_geometry_bytes,instance_capacity_bytes,upload_bytes,staging_bytes\n",
    );
    let mut images = std::collections::BTreeMap::new();
    for case in [
        "intact",
        "independent-move",
        "damaged",
        "blackout",
        "expanded-500",
    ] {
        let expanded = case == "expanded-500";
        let camera = CameraFrame {
            position_meters: if expanded {
                [35., 30., 45.]
            } else {
                [0., 2.0, 5.]
            },
            target_meters: if expanded {
                [0., 6., 0.]
            } else {
                [0., 0.5, 0.]
            },
            up: [0., 1., 0.],
            vertical_fov_radians: 1.2,
            near_plane_meters: 0.05,
        };
        let mesh = |i| {
            [
                FixtureMesh::Bunk,
                FixtureMesh::Shelf,
                FixtureMesh::Seat,
                FixtureMesh::Panel,
                FixtureMesh::PanelDamaged,
            ][i]
        };
        let mut instances = if expanded {
            (0..500)
                .map(|i| FixtureInstance {
                    id: i as u64,
                    mesh: mesh(i % 5),
                    pose: FixturePose {
                        position_meters: [
                            (i % 10) as f64 * 3. - 13.5,
                            (i / 100) as f64 * 3.,
                            ((i / 10) % 10) as f64 * 3. - 13.5,
                        ],
                        ..Default::default()
                    },
                    tint: [1.; 3],
                    powered: true,
                })
                .collect::<Vec<_>>()
        } else {
            [
                FixtureMesh::Shelf,
                FixtureMesh::Seat,
                if case == "damaged" {
                    FixtureMesh::PanelDamaged
                } else {
                    FixtureMesh::Panel
                },
            ]
            .into_iter()
            .enumerate()
            .map(|(i, mesh)| FixtureInstance {
                id: i as u64,
                mesh,
                pose: FixturePose {
                    position_meters: [i as f64 * 1.2 - 1.2, 0.5, 0.],
                    ..Default::default()
                },
                tint: [1.; 3],
                powered: case != "blackout",
            })
            .collect()
        };
        if case == "independent-move" {
            instances[0].pose.position_meters[1] += 0.35;
            let a: f64 = 0.6;
            instances[1].pose.orientation_xyzw = [(a / 2.).sin(), 0., 0., (a / 2.).cos()];
        }
        let matrix = crate::multiply_mat4(
            crate::infinite_reverse_z_projection(camera, 1.6).unwrap(),
            crate::view_rotation(crate::camera_basis(camera).unwrap()),
        );
        let mut first = None;
        for sample in 0..16 {
            let stats = renderer
                .prepare(&device, &queue, camera, matrix, 1.6, &instances)
                .unwrap();
            assert!(stats.visible_instances > 0);
            assert!(stats.draw_calls <= 5);
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
                            load: wgpu::LoadOp::Clear(0.),
                            store: wgpu::StoreOp::Store,
                        }),
                        stencil_ops: None,
                    }),
                    timestamp_writes: timer.as_ref().map(|t| t.timestamp_writes()),
                    ..Default::default()
                });
                renderer.draw(&mut pass);
            }
            if let Some(t) = &timer {
                t.resolve_and_map(&mut encoder, slot.unwrap());
            }
            let readback = (sample < 2).then(|| {
                crate::capture::Readback::encode(&device, &mut encoder, &texture).unwrap()
            });
            queue.submit([encoder.finish()]);
            device.poll(wgpu::PollType::wait_indefinitely()).unwrap();
            let gpu_ms = if let Some(t) = &mut timer {
                t.poll(&device);
                t.latest()
                    .map(|d| format!("{:.6}", d.as_secs_f64() * 1000.))
                    .unwrap_or_default()
            } else {
                String::new()
            };
            writeln!(
                csv,
                "{case},{sample},{:.3},{gpu_ms},{},{},{},{},{},{},{}",
                stats.prepare_time.as_secs_f64() * 1e6,
                stats.draw_calls,
                stats.visible_instances,
                stats.triangles,
                stats.resident_geometry_bytes,
                stats.instance_capacity_bytes,
                stats.upload_bytes,
                stats.scratch_capacity_bytes
            )
            .unwrap();
            if let Some(readback) = readback {
                let png = readback.finish(&device).unwrap();
                if let Some(previous) = &first {
                    assert_eq!(&png, previous, "frozen fixture repeat changed");
                } else {
                    std::fs::write(directory.join(format!("{case}.png")), &png).unwrap();
                    first = Some(png);
                }
            }
        }
        images.insert(case, first.unwrap());
    }
    for changed in ["independent-move", "damaged", "blackout"] {
        assert_ne!(images["intact"], images[changed]);
    }
    std::fs::write(directory.join("timings.csv"), csv).unwrap();
}

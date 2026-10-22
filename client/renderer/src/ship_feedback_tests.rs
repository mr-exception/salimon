//! Explicit software/hardware GPU evidence; synthetic DTOs never certify gameplay input.
use super::*;
use crate::{CameraFrame, MachineryFeedback, MachineryState};
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
#[ignore = "requires GPU; set SALIMON_SCOUT_FEEDBACK_DIR for PNGs and raw timing CSV"]
fn scout_feedback_gpu_evidence() {
    let directory = std::path::PathBuf::from(std::env::var("SALIMON_SCOUT_FEEDBACK_DIR").unwrap());
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
            "{:?}\n1280x800; ship-only offscreen; 16 frames/case, first 2 warmup\n",
            adapter.get_info()
        ),
    )
    .unwrap();
    let format = wgpu::TextureFormat::Rgba8UnormSrgb;
    let mut renderer = ShipMeshRenderer::new(&device, format).unwrap();
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
    let idle = MachineryFeedback {
        core_charge: 0.75,
        ..Default::default()
    };
    let mut csv = String::from(
        "case,sample,cpu_prepare_us,gpu_ms,draws,vertices,vertex_bytes,uniform_bytes\n",
    );
    let mut images = std::collections::BTreeMap::new();
    for (name, eye, target, feedback, power, door, output) in [
        (
            "archive-powered",
            [-3., 1.8, -2.8],
            crate::ARCHIVE_TERMINAL_METERS,
            idle,
            true,
            0.,
            false,
        ),
        (
            "archive-blackout",
            [-3., 1.8, -2.8],
            crate::ARCHIVE_TERMINAL_METERS,
            idle,
            false,
            0.,
            false,
        ),
        (
            "core-idle",
            [-1., 1.9, -3.],
            [-1., 1.2, 0.],
            idle,
            true,
            0.,
            false,
        ),
        (
            "core-converting",
            [-1., 1.9, -3.],
            [-1., 1.2, 0.],
            MachineryFeedback {
                converting: true,
                ..idle
            },
            true,
            0.,
            false,
        ),
        (
            "core-blackout",
            [-1., 1.9, -3.],
            [-1., 1.2, 0.],
            idle,
            false,
            0.,
            false,
        ),
        (
            "fabricator-idle",
            [-3.8, 1.9, 1.2],
            [-3.8, 1.6, 3.69],
            idle,
            true,
            0.,
            false,
        ),
        (
            "fabricator-working-a",
            [-3.8, 1.9, 1.2],
            [-3.8, 1.6, 3.69],
            MachineryFeedback {
                fabricator: MachineryState::Working,
                fabrication_progress: 0.2,
                phase: 0.1,
                ..idle
            },
            true,
            0.,
            false,
        ),
        (
            "fabricator-working-b",
            [-3.8, 1.9, 1.2],
            [-3.8, 1.6, 3.69],
            MachineryFeedback {
                fabricator: MachineryState::Working,
                fabrication_progress: 0.7,
                phase: 0.7,
                ..idle
            },
            true,
            0.,
            false,
        ),
        (
            "fabricator-blocked",
            [-3.8, 1.9, 1.2],
            [-3.8, 1.6, 3.69],
            MachineryFeedback {
                fabricator: MachineryState::Blocked,
                ..idle
            },
            true,
            0.,
            true,
        ),
        (
            "fabricator-blackout",
            [-3.8, 1.9, 1.2],
            [-3.8, 1.6, 3.69],
            MachineryFeedback {
                fabricator: MachineryState::Unpowered,
                ..idle
            },
            false,
            0.,
            true,
        ),
        (
            "engine-coasting",
            [-12., 3., -7.],
            [-4., 1., 0.],
            idle,
            true,
            0.,
            false,
        ),
        (
            "engine-forward",
            [-12., 3., -7.],
            [-4., 1., 0.],
            MachineryFeedback { thrust: 1., ..idle },
            true,
            0.,
            false,
        ),
        (
            "engine-reverse",
            [-12., 3., -7.],
            [-4., 1., 0.],
            MachineryFeedback {
                thrust: -0.5,
                ..idle
            },
            true,
            0.,
            false,
        ),
        (
            "door-closed",
            [-5., 1.9, 0.],
            [-7.7, 1.35, 0.],
            idle,
            true,
            0.,
            false,
        ),
        (
            "door-open",
            [-5., 1.9, 0.],
            [-7.7, 1.35, 0.],
            idle,
            true,
            1.,
            false,
        ),
        (
            "cockpit-active",
            [2.76, 1.8, 0.],
            [5.8, 1.1, 0.],
            MachineryFeedback {
                cockpit_active: true,
                thrust: 0.5,
                ..idle
            },
            true,
            0.,
            false,
        ),
    ] {
        let camera = CameraFrame {
            position_meters: eye,
            target_meters: target,
            up: [0., 1., 0.],
            vertical_fov_radians: 70_f32.to_radians(),
            near_plane_meters: 0.05,
        };
        let scene = SceneFrame {
            camera,
            instances: &[],
            spheres: &[],
            resource_meshes: &[],
            terrain: &[],
            terrain_mask: None,
            light: None,
            held_item: None,
            ship: Some(ShipMeshInstance {
                position_meters: [0.; 3],
                orientation: [0., 0., 0., 1.],
                door_open_fraction: door,
                fabricator_output_present: output,
                fabricator_output_medical: false,
                fabricator_output_component: false,
                machinery: feedback,
                instruments: CockpitInstruments {
                    power_off: !power,
                    thruster_percentage: (feedback.thrust * 100.) as i8,
                    core_energy_capacity_joules: 60_000_000_000,
                    core_energy_stored_joules: (feedback.core_charge as f64 * 60_000_000_000.)
                        as u64,
                    core_status: crate::CoreInstruments {
                        conversion_watts: if feedback.converting {
                            200_000_000.
                        } else {
                            0.
                        },
                        conversion_remaining_joules: if feedback.converting {
                            1_000_000_000.
                        } else {
                            0.
                        },
                        ..Default::default()
                    },
                    fabricator: crate::FabricatorInstruments {
                        status: match feedback.fabricator {
                            MachineryState::Idle => 0,
                            MachineryState::Working => 1,
                            MachineryState::Blocked => 3,
                            MachineryState::Unpowered => 2,
                        },
                        jobs: u32::from(feedback.fabricator == MachineryState::Working),
                        remaining_seconds: if feedback.fabricator == MachineryState::Working {
                            ((1. - feedback.fabrication_progress) * 10.).ceil() as u32
                        } else {
                            0
                        },
                    },
                    ..Default::default()
                },
            }),
        };
        let matrix = crate::multiply_mat4(
            crate::infinite_reverse_z_projection(camera, 1.6).unwrap(),
            crate::view_rotation(crate::camera_basis(camera).unwrap()),
        );
        let mut first = None;
        for sample in 0..16 {
            let started = Instant::now();
            renderer.prepare(&queue, scene, matrix).unwrap();
            let cpu = started.elapsed().as_secs_f64() * 1e6;
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
            let cost = if let Some(t) = &mut timer {
                t.poll(&device);
                t.latest()
                    .map(|v| format!("{:.6}", v.as_secs_f64() * 1000.))
                    .unwrap_or_default()
            } else {
                String::new()
            };
            writeln!(
                csv,
                "{name},{sample},{cpu:.3},{cost},{},{},{},{}",
                renderer.draw_count(),
                renderer.opaque_vertex_count + renderer.glass_vertex_count,
                (renderer.opaque_vertex_count + renderer.glass_vertex_count) as u64 * VERTEX_STRIDE,
                UNIFORM_SIZE
            )
            .unwrap();
            if let Some(r) = readback {
                let png = r.finish(&device).unwrap();
                if let Some(first) = &first {
                    assert!(&png == first, "frozen {name} differs");
                } else {
                    std::fs::write(directory.join(format!("{name}.png")), &png).unwrap();
                    first = Some(png);
                }
            }
        }
        images.insert(name, first.unwrap());
    }
    std::fs::write(directory.join("timings.csv"), &csv).unwrap();
    for (a, b) in [
        ("core-idle", "core-converting"),
        ("core-idle", "core-blackout"),
        ("fabricator-working-a", "fabricator-working-b"),
        ("fabricator-idle", "fabricator-blocked"),
        ("engine-coasting", "engine-forward"),
        ("door-closed", "door-open"),
    ] {
        assert!(images[a] != images[b], "{a}/{b} feedback was invisible");
    }
    std::fs::write(directory.join("timings.csv"), csv).unwrap();
}

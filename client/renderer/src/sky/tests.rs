use super::*;
fn block_on<F: std::future::Future>(f: F) -> F::Output {
    let mut f = std::pin::pin!(f);
    let mut c = std::task::Context::from_waker(std::task::Waker::noop());
    let deadline = std::time::Instant::now() + std::time::Duration::from_secs(30);
    loop {
        if let std::task::Poll::Ready(v) = f.as_mut().poll(&mut c) {
            return v;
        }
        assert!(std::time::Instant::now() < deadline);
        std::thread::yield_now();
    }
}
#[test]
#[ignore = "production Vulkan sky captures; set SALIMON_SKY_DIR"]
fn sky_production_reference() {
    let instance = wgpu::Instance::default();
    let adapter = block_on(instance.request_adapter(&Default::default())).unwrap();
    println!("sky adapter {:?}", adapter.get_info());
    let (device, queue) = block_on(adapter.request_device(&Default::default())).unwrap();
    let format = wgpu::TextureFormat::Rgba8UnormSrgb;
    let mut renderer = SkyRenderer::new(&device, format);
    let texture = device.create_texture(&wgpu::TextureDescriptor {
        label: Some("sky evidence"),
        size: wgpu::Extent3d {
            width: 512,
            height: 384,
            depth_or_array_layers: 1,
        },
        mip_level_count: 1,
        sample_count: 1,
        dimension: wgpu::TextureDimension::D2,
        format,
        usage: wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::COPY_SRC,
        view_formats: &[],
    });
    let depth = crate::DepthTarget::new(&device, 512, 384);
    let view = texture.create_view(&Default::default());
    let camera = CameraFrame {
        position_meters: [0.; 3],
        target_meters: [0., 0., -1.],
        up: [0., 1., 0.],
        vertical_fov_radians: 1.2,
        near_plane_meters: 0.1,
    };
    let directory = std::path::PathBuf::from(std::env::var("SALIMON_SKY_DIR").unwrap());
    std::fs::create_dir_all(&directory).unwrap();
    let mut sky = SkyScene::default();
    sky.stars.push(SkyStar {
        direction: [0., 0., -1.],
        angular_radius_radians: 0.018,
        color: [1., 0.7, 0.4],
        brightness: 1.,
    });
    let mut reference = None;
    for (name, shift, blocked) in [
        ("sky", 0., false),
        ("translated", 1e12, false),
        ("occluded", 0., true),
    ] {
        let mut camera = camera;
        camera.position_meters = [shift; 3];
        camera.target_meters = [shift, shift, shift - 1.];
        renderer
            .prepare(&queue, camera, [512, 384], Some(&sky))
            .unwrap();
        assert_eq!(renderer.draws(), 2);
        let started = std::time::Instant::now();
        let mut encoder = device.create_command_encoder(&Default::default());
        {
            let mut pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("production sky"),
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
                        load: wgpu::LoadOp::Clear(if blocked { 1. } else { 0. }),
                        store: wgpu::StoreOp::Store,
                    }),
                    stencil_ops: None,
                }),
                timestamp_writes: None,
                occlusion_query_set: None,
                multiview_mask: None,
            });
            renderer.draw(&mut pass);
        }
        let read = crate::capture::Readback::encode(&device, &mut encoder, &texture).unwrap();
        queue.submit([encoder.finish()]);
        let png = read.finish(&device).unwrap();
        println!(
            "{name}: capture {:.3}ms (includes synchronous readback)",
            started.elapsed().as_secs_f64() * 1000.
        );
        let mut reader = png::Decoder::new(std::io::Cursor::new(&png))
            .read_info()
            .unwrap();
        let mut pixels = vec![0; reader.output_buffer_size().unwrap()];
        let info = reader.next_frame(&mut pixels).unwrap();
        pixels.truncate(info.buffer_size());
        if blocked {
            assert!(
                pixels
                    .as_chunks::<4>()
                    .0
                    .iter()
                    .all(|p| p[..3] == [0, 0, 0])
            );
        } else if let Some(ref r) = reference {
            assert_eq!(&pixels, r);
        } else {
            assert!(
                pixels
                    .as_chunks::<4>()
                    .0
                    .iter()
                    .filter(|p| p[0] > 150)
                    .count()
                    > 50
            );
            reference = Some(pixels);
        }
        std::fs::write(directory.join(format!("{name}.png")), png).unwrap();
    }
    let mut invalid = sky.clone();
    invalid.stars[0].direction = [0.; 3];
    assert!(
        renderer
            .prepare(&queue, camera, [512, 384], Some(&invalid))
            .is_err()
    );
    invalid = sky.clone();
    invalid.stars = vec![sky.stars[0]; MAX_SKY_STARS + 1];
    assert!(
        renderer
            .prepare(&queue, camera, [512, 384], Some(&invalid))
            .is_err()
    );
}

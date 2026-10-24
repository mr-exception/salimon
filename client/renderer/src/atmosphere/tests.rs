use super::*;
fn block_on<F: std::future::Future>(f: F) -> F::Output {
    let mut f = std::pin::pin!(f);
    let mut c = std::task::Context::from_waker(std::task::Waker::noop());
    let end = std::time::Instant::now() + std::time::Duration::from_secs(30);
    loop {
        if let std::task::Poll::Ready(v) = f.as_mut().poll(&mut c) {
            return v;
        }
        assert!(std::time::Instant::now() < end);
        std::thread::yield_now();
    }
}
#[test]
#[ignore = "production atmosphere captures; set SALIMON_ATMOSPHERE_DIR"]
fn atmosphere_production_reference() {
    let gpu = wgpu::Instance::default();
    let adapter = block_on(gpu.request_adapter(&Default::default())).unwrap();
    println!("atmosphere {:?}", adapter.get_info());
    let (device, queue) = block_on(adapter.request_device(&Default::default())).unwrap();
    let format = wgpu::TextureFormat::Rgba8UnormSrgb;
    let mut renderer = AtmosphereRenderer::new(&device, format);
    let texture = device.create_texture(&wgpu::TextureDescriptor {
        label: Some("atmosphere evidence"),
        size: wgpu::Extent3d {
            width: 384,
            height: 256,
            depth_or_array_layers: 1,
        },
        mip_level_count: 1,
        sample_count: 1,
        dimension: wgpu::TextureDimension::D2,
        format,
        usage: wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::COPY_SRC,
        view_formats: &[],
    });
    let view = texture.create_view(&Default::default());
    let depth = crate::DepthTarget::new(&device, 384, 256);
    let directory = std::path::PathBuf::from(std::env::var("SALIMON_ATMOSPHERE_DIR").unwrap());
    std::fs::create_dir_all(&directory).unwrap();
    let atmosphere = AtmosphereScene {
        center_meters: [0.; 3],
        radius_meters: 6371000.,
        scale_height_meters: 8000.,
        ceiling_meters: 96000.,
        rayleigh_per_meter: [5.8e-6, 13.5e-6, 33.1e-6],
        sun_direction: [0., 1., 0.],
        sun_color: [1.; 3],
        solar_visibility: 1.,
    };
    let mut means = std::collections::BTreeMap::new();
    let mut upper_sky = std::collections::BTreeMap::new();
    for (name, altitude, visibility, sun, blocked) in [
        ("day", 2., 1., 1., false),
        ("red-star", 2., 1., 1., false),
        ("night", 2., 1., -1., false),
        ("eclipse", 2., 0., 1., false),
        ("descent", 20000., 1., 1., false),
        ("orbit", 120000., 1., 1., false),
        ("near-occlusion", 2., 1., 1., true),
        ("vacuum", 2., 1., 1., false),
    ] {
        let camera = CameraFrame {
            position_meters: [0., 6371000. + altitude, 0.],
            target_meters: [0., 6371000. + altitude + 2000., -10000.],
            up: [0., 1., 0.],
            vertical_fov_radians: 1.2,
            near_plane_meters: 0.1,
        };
        let mut a = atmosphere;
        a.solar_visibility = visibility;
        a.sun_direction = [0., sun, 0.];
        if name == "red-star" {
            a.sun_color = [1., 0.2, 0.05];
        }
        renderer
            .prepare(
                &device,
                &queue,
                camera,
                [384, 256],
                if name == "vacuum" { None } else { Some(a) },
                &depth.view,
            )
            .unwrap();
        let mut encoder = device.create_command_encoder(&Default::default());
        {
            let _pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("opaque depth fixture"),
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
        }
        {
            let mut pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("production atmospheric integration"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &view,
                    depth_slice: None,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Load,
                        store: wgpu::StoreOp::Store,
                    },
                })],
                depth_stencil_attachment: None,
                timestamp_writes: None,
                occlusion_query_set: None,
                multiview_mask: None,
            });
            renderer.draw(&mut pass);
        }
        let read = crate::capture::Readback::encode(&device, &mut encoder, &texture).unwrap();
        let start = std::time::Instant::now();
        queue.submit([encoder.finish()]);
        let png = read.finish(&device).unwrap();
        println!(
            "{name}: synchronous submit/readback {:.3} ms",
            start.elapsed().as_secs_f64() * 1000.
        );
        let mut reader = png::Decoder::new(std::io::Cursor::new(&png))
            .read_info()
            .unwrap();
        let mut pixels = vec![0; reader.output_buffer_size().unwrap()];
        reader.next_frame(&mut pixels).unwrap();
        let rgb = pixels.as_chunks::<4>().0;
        let mean = std::array::from_fn::<_, 3, _>(|i| {
            rgb.iter().map(|p| f64::from(p[i])).sum::<f64>() / rgb.len() as f64
        });
        means.insert(name, mean);
        upper_sky.insert(
            name,
            rgb[..384 * 32].iter().map(|p| f64::from(p[2])).sum::<f64>() / (384. * 32.),
        );
        std::fs::write(directory.join(format!("{name}.png")), png).unwrap();
    }
    println!("pixel means {means:?}");
    assert!(means["day"][2] > means["day"][0] + 5.);
    assert!(means["day"][2] > means["night"][2] + 10.);
    assert!(means["red-star"][0] > means["red-star"][2]);
    assert_eq!(means["eclipse"], [0.; 3]);
    assert_eq!(means["vacuum"], [0.; 3]);
    assert!(means["near-occlusion"][2] < 1.);
    println!("upper sky {upper_sky:?}");
    assert!(upper_sky["descent"] < upper_sky["day"]);
    let camera = CameraFrame {
        position_meters: [0., 6371002., 0.],
        target_meters: [0., 6371002., -1.],
        up: [0., 1., 0.],
        vertical_fov_radians: 1.,
        near_plane_meters: 0.1,
    };
    let mut bad = atmosphere;
    bad.solar_visibility = f32::NAN;
    assert!(
        renderer
            .prepare(&device, &queue, camera, [384, 256], Some(bad), &depth.view)
            .is_err()
    );
}

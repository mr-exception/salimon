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
#[ignore = "production atmosphere captures; set SALIMON_WEATHER_DIR"]
fn weather_production_reference() {
    let gpu = wgpu::Instance::default();
    let adapter = block_on(gpu.request_adapter(&Default::default())).unwrap();
    println!("weather {:?}", adapter.get_info());
    let (device, queue) = block_on(adapter.request_device(&Default::default())).unwrap();
    let format = wgpu::TextureFormat::Rgba8UnormSrgb;
    let mut renderer = WeatherRenderer::new(&device, format);
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
    let directory = std::path::PathBuf::from(std::env::var("SALIMON_WEATHER_DIR").unwrap());
    std::fs::create_dir_all(&directory).unwrap();
    let mut atmosphere = AtmosphereScene {
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
    let mut original = None;
    let mut observer_direction: [f64; 3] = [0., 1., 0.];
    let mut base = WeatherScene {
        optical: vec![[0.85, 0.5, 0., 0.]; 2048],
        wind_temperature: vec![[5., 0., 0., 285.]; 2048],
        cloud_base_meters: 4000.,
        precipitation_freeze_kelvin: 273.15,
        cloud_thickness_meters: 2000.,
        time_seconds: 3600.,
        body_axes: [[1., 0., 0.], [0., 1., 0.], [0., 0., 1.]],
        view_samples: 16,
    };
    if let Ok(path) = std::env::var("SALIMON_WEATHER_FRAME_FILE") {
        let data: serde_json::Value =
            serde_json::from_slice(&std::fs::read(path).unwrap()).unwrap();
        observer_direction = serde_json::from_value(data["observer_direction"].clone()).unwrap();
        atmosphere.sun_direction = observer_direction.map(|x| x as f32);
        base.optical = serde_json::from_value(data["optical"].clone()).unwrap();
        base.wind_temperature = serde_json::from_value(data["wind_temperature"].clone()).unwrap();
        base.cloud_base_meters = data["cloud_base_meters"].as_f64().unwrap();
        base.cloud_thickness_meters = data["cloud_thickness_meters"].as_f64().unwrap();
        base.time_seconds = data["time_seconds"].as_f64().unwrap();
        base.precipitation_freeze_kelvin =
            data["precipitation_freeze_kelvin"].as_f64().unwrap() as f32;
        println!(
            "actual world/runtime frame definition {} epoch {}",
            data["definition"], data["epoch"]
        );
    }
    for (name, altitude, kind, samples) in [
        ("clouds", 2., 0, 16),
        ("reload", 2., 0, 16),
        ("day-end", 2., 0, 16),
        ("day-start", 2., 0, 16),
        ("low-quality", 2., 0, 8),
        ("dust", 2., 1, 16),
        ("rain", 2., 2, 16),
        ("orbit", 120000., 0, 16),
        ("clear", 2., 3, 16),
    ] {
        let tangent = [observer_direction[2], 0., -observer_direction[0]];
        let norm = tangent[0].hypot(tangent[2]);
        let tangent = if norm < 0.001 {
            [0., 0., -1.]
        } else {
            tangent.map(|v| v / norm)
        };
        let position = observer_direction.map(|v| v * (6371000. + altitude));
        let target = std::array::from_fn(|i| {
            position[i] + observer_direction[i] * 2000. + tangent[i] * 10000.
        });
        let camera = CameraFrame {
            position_meters: position,
            target_meters: target,
            up: observer_direction.map(|v| v as f32),
            vertical_fov_radians: 1.2,
            near_plane_meters: 0.1,
        };
        let mut weather = base.clone();
        weather.view_samples = samples;
        if name == "day-end" {
            weather.time_seconds = 86399.999;
        }
        if name == "day-start" {
            weather.time_seconds = 86400.001;
        }
        if kind == 1 {
            weather.optical.fill([0., 0.5, 0., 0.0001]);
        }
        if kind == 2 {
            weather.optical.fill([0.1, 0.8, 0.001, 0.]);
        }
        if kind == 3 {
            weather.optical.fill([0.; 4]);
        }
        renderer
            .prepare(
                &device,
                &queue,
                camera,
                [384, 256],
                Some(atmosphere),
                Some(&weather),
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
                        load: wgpu::LoadOp::Clear(0.),
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
                label: Some("production weather"),
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
        if name == "clouds" {
            original = Some(pixels.clone());
        }
        if name == "reload" {
            assert_eq!(original.as_ref().unwrap(), &pixels);
        }
        assert_eq!(weather.optical.len(), 2048);
        std::fs::write(directory.join(format!("{name}.png")), png).unwrap();
    }
    println!("pixel means {means:?}");
    assert!(means["clouds"][2] > 20.);
    assert!(means["dust"][0] > means["dust"][2]);
    assert!(means["rain"][2] > 1.);
    assert_eq!(means["clear"], [0.; 3]);
    assert!((means["day-end"][2] - means["day-start"][2]).abs() < 0.5);
    let camera = CameraFrame {
        position_meters: [0., 6371002., 0.],
        target_meters: [0., 6371002., -1.],
        up: [0., 1., 0.],
        vertical_fov_radians: 1.,
        near_plane_meters: 0.1,
    };
    assert!(
        renderer
            .prepare(
                &device,
                &queue,
                camera,
                [384, 256],
                None,
                Some(&base),
                &depth.view
            )
            .is_err()
    );
    let mut bad = base.clone();
    bad.optical.pop();
    assert!(
        renderer
            .prepare(
                &device,
                &queue,
                camera,
                [384, 256],
                Some(atmosphere),
                Some(&bad),
                &depth.view
            )
            .is_err()
    );
    bad = base;
    bad.body_axes[0] = [0.; 3];
    assert!(
        renderer
            .prepare(
                &device,
                &queue,
                camera,
                [384, 256],
                Some(atmosphere),
                Some(&bad),
                &depth.view
            )
            .is_err()
    );
}

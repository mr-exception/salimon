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
fn procedural_prism_lods_have_closed_bounded_identical_envelopes() {
    for sides in [3, 6] {
        let m = mesh(sides);
        assert_eq!(m.len(), sides * 12);
        assert!(m.iter().flatten().all(|v| v.is_finite() && v.abs() <= 1.));
        assert!(m.iter().any(|v| v[1] == 0.) && m.iter().any(|v| v[1] == 1.));
    }
}
#[test]
#[ignore = "actual world/runtime specimen capture; set SALIMON_ORGANISMS_FILE and SALIMON_ORGANISM_DIR"]
fn organism_production_reference() {
    let models: serde_json::Value = serde_json::from_slice(
        &std::fs::read(std::env::var("SALIMON_ORGANISMS_FILE").unwrap()).unwrap(),
    )
    .unwrap();
    let directory = std::path::PathBuf::from(std::env::var("SALIMON_ORGANISM_DIR").unwrap());
    std::fs::create_dir_all(&directory).unwrap();
    let gpu = wgpu::Instance::default();
    let adapter = block_on(gpu.request_adapter(&Default::default())).unwrap();
    println!("organisms {:?}", adapter.get_info());
    let (device, queue) = block_on(adapter.request_device(&Default::default())).unwrap();
    let format = wgpu::TextureFormat::Rgba8UnormSrgb;
    let mut renderer = OrganismRenderer::new(&device, format);
    let texture = device.create_texture(&wgpu::TextureDescriptor {
        label: Some("organism evidence"),
        size: wgpu::Extent3d {
            width: 512,
            height: 512,
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
    let depth = crate::DepthTarget::new(&device, 512, 512);
    let mut hashes = std::collections::BTreeSet::new();
    for model in models.as_array().unwrap() {
        let segments: Vec<_> = model["segments"]
            .as_array()
            .unwrap()
            .iter()
            .map(|v| OrganismSegment {
                start_meters: serde_json::from_value(v["start"].clone()).unwrap(),
                end_meters: serde_json::from_value(v["end"].clone()).unwrap(),
                radius_start_meters: v["r0"].as_f64().unwrap(),
                radius_end_meters: v["r1"].as_f64().unwrap(),
                color: serde_json::from_value(v["color"].clone()).unwrap(),
            })
            .collect();
        let origin: [f64; 3] = model
            .get("origin")
            .map(|v| serde_json::from_value(v.clone()).unwrap())
            .unwrap_or([0.; 3]);
        let axes: [[f64; 3]; 3] = model
            .get("axes")
            .map(|v| serde_json::from_value(v.clone()).unwrap())
            .unwrap_or([[1., 0., 0.], [0., 1., 0.], [0., 0., 1.]]);
        let size = segments
            .iter()
            .flat_map(|s| {
                s.start_meters
                    .into_iter()
                    .enumerate()
                    .map(|(i, v)| (v - origin[i]).abs())
                    .chain(
                        s.end_meters
                            .into_iter()
                            .enumerate()
                            .map(|(i, v)| (v - origin[i]).abs()),
                    )
            })
            .fold(0.5, f64::max)
            * 1.5;
        let camera = CameraFrame {
            position_meters: std::array::from_fn(|i| {
                origin[i] + size * (axes[0][i] * 0.6 + axes[1][i] * 0.7 + axes[2][i] * 2.)
            }),
            target_meters: std::array::from_fn(|i| origin[i] + axes[1][i] * size * 0.35),
            up: axes[1].map(|v| v as f32),
            vertical_fov_radians: 1.,
            near_plane_meters: 0.01,
        };
        let scene = crate::SceneFrame {
            sky: None,
            atmosphere: None,
            weather: None,
            organisms: &segments,
            camera,
            instances: &[],
            resource_meshes: &[],
            fixtures: &[],
            spheres: &[],
            terrain: &[],
            terrain_mask: None,
            light: None,
            ship: None,
            held_item: None,
        };
        let matrix = crate::prepare_scene(scene, 1.).unwrap().view_projection;
        renderer
            .prepare(&queue, camera, matrix, 512, None, &segments)
            .unwrap();
        assert_eq!(renderer.count(), segments.len() as u32);
        assert!(renderer.draws() <= 2);
        let mut encoder = device.create_command_encoder(&Default::default());
        {
            let mut pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("actual organism grammar"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &view,
                    depth_slice: None,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(wgpu::Color {
                            r: 0.008,
                            g: 0.015,
                            b: 0.035,
                            a: 1.,
                        }),
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
            renderer.draw(&mut pass);
        }
        let read = crate::capture::Readback::encode(&device, &mut encoder, &texture).unwrap();
        let start = std::time::Instant::now();
        queue.submit([encoder.finish()]);
        let png = read.finish(&device).unwrap();
        println!(
            "{} segments={} submit/readback {:.3}ms",
            model["name"],
            segments.len(),
            start.elapsed().as_secs_f64() * 1000.
        );
        let mut reader = png::Decoder::new(std::io::Cursor::new(&png))
            .read_info()
            .unwrap();
        let mut pixels = vec![0; reader.output_buffer_size().unwrap()];
        reader.next_frame(&mut pixels).unwrap();
        assert!(
            pixels
                .as_chunks::<4>()
                .0
                .iter()
                .filter(|p| p[1] > 60)
                .count()
                > 20
        );
        hashes.insert(png.clone());
        std::fs::write(
            directory.join(format!("{}.png", model["name"].as_str().unwrap())),
            png,
        )
        .unwrap();
        let far = CameraFrame {
            position_meters: std::array::from_fn(|i| {
                origin[i] + axes[1][i] * size + axes[2][i] * 1e6
            }),
            target_meters: std::array::from_fn(|i| origin[i] + axes[1][i] * size),
            ..camera
        };
        let scene = crate::SceneFrame {
            camera: far,
            ..scene
        };
        renderer
            .prepare(
                &queue,
                far,
                crate::prepare_scene(scene, 1.).unwrap().view_projection,
                512,
                None,
                &segments,
            )
            .unwrap();
        assert_eq!(renderer.count(), segments.len() as u32);
        assert_eq!(renderer.counts[1], segments.len() as u32);
    }
    assert!(!hashes.is_empty());
    assert_eq!(hashes.len(), models.as_array().unwrap().len());
}

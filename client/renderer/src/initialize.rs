//! GPU device and immutable pipeline initialization.
use super::{
    CAMERA_UNIFORM_SIZE, CUBE_VERTEX_STRIDE, CUBE_VERTICES, DEPTH_FORMAT, DepthTarget, GpuTimer,
    INSTANCE_STRIDE, Renderer, RendererError, RendererInfo, SurfaceSize, encode_f32s,
    equipment_toolbar, fixtures, held_item, overlay, resource_mesh, ship_mesh, spheres, terrain,
};
use wgpu::util::DeviceExt;

impl Renderer {
    pub async fn new(
        surface_target: impl Into<wgpu::SurfaceTarget<'static>>,
        initial_size: SurfaceSize,
    ) -> Result<Self, RendererError> {
        let instance = wgpu::Instance::default();
        let surface = instance
            .create_surface(surface_target)
            .map_err(|error| RendererError::new("failed to create rendering surface", error))?;

        let adapter = instance
            .request_adapter(&wgpu::RequestAdapterOptions {
                power_preference: wgpu::PowerPreference::HighPerformance,
                compatible_surface: Some(&surface),
                force_fallback_adapter: false,
                ..Default::default()
            })
            .await
            .map_err(|error| {
                RendererError::new("failed to find a compatible GPU adapter", error)
            })?;

        let timestamp_queries_supported =
            adapter.features().contains(wgpu::Features::TIMESTAMP_QUERY);
        let adapter_info = adapter.get_info();
        let info = RendererInfo {
            adapter_name: adapter_info.name,
            driver: format!("{} / {}", adapter_info.driver, adapter_info.driver_info),
            backend: format!("{:?}", adapter_info.backend),
            device_type: format!("{:?}", adapter_info.device_type),
            timestamp_queries_supported,
        };
        let required_features = if timestamp_queries_supported {
            wgpu::Features::TIMESTAMP_QUERY
        } else {
            wgpu::Features::empty()
        };
        let (device, queue) = adapter
            .request_device(&wgpu::DeviceDescriptor {
                label: Some("Salimon Phase 0 device"),
                required_features,
                required_limits: wgpu::Limits::default(),
                experimental_features: wgpu::ExperimentalFeatures::disabled(),
                memory_hints: wgpu::MemoryHints::MemoryUsage,
                trace: wgpu::Trace::Off,
            })
            .await
            .map_err(|error| RendererError::new("failed to create GPU device", error))?;

        let configured_width = initial_size.width.max(1);
        let configured_height = initial_size.height.max(1);
        let mut configuration = surface
            .get_default_config(&adapter, configured_width, configured_height)
            .ok_or_else(|| {
                RendererError::new(
                    "failed to configure rendering surface",
                    "adapter exposes no compatible surface format",
                )
            })?;
        if surface
            .get_capabilities(&adapter)
            .usages
            .contains(wgpu::TextureUsages::COPY_SRC)
        {
            configuration.usage |= wgpu::TextureUsages::COPY_SRC;
        }
        configuration.present_mode = wgpu::PresentMode::AutoVsync;
        configuration.desired_maximum_frame_latency = 2;

        let camera_bind_group_layout =
            device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                label: Some("Salimon camera bind group layout"),
                entries: &[wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::VERTEX,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: wgpu::BufferSize::new(CAMERA_UNIFORM_SIZE),
                    },
                    count: None,
                }],
            });
        let camera_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("Salimon camera uniform"),
            size: CAMERA_UNIFORM_SIZE,
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });
        let camera_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("Salimon camera bind group"),
            layout: &camera_bind_group_layout,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: camera_buffer.as_entire_binding(),
            }],
        });

        let shader = device.create_shader_module(wgpu::include_wgsl!("bootstrap.wgsl"));
        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("Salimon validation scene pipeline layout"),
            bind_group_layouts: &[Some(&camera_bind_group_layout)],
            immediate_size: 0,
        });
        let pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("Salimon instanced cuboid pipeline"),
            layout: Some(&pipeline_layout),
            vertex: wgpu::VertexState {
                module: &shader,
                entry_point: Some("vs_main"),
                buffers: &[
                    Some(wgpu::VertexBufferLayout {
                        array_stride: CUBE_VERTEX_STRIDE,
                        step_mode: wgpu::VertexStepMode::Vertex,
                        attributes: &[wgpu::VertexAttribute {
                            format: wgpu::VertexFormat::Float32x3,
                            offset: 0,
                            shader_location: 0,
                        }],
                    }),
                    Some(wgpu::VertexBufferLayout {
                        array_stride: INSTANCE_STRIDE,
                        step_mode: wgpu::VertexStepMode::Instance,
                        attributes: &[
                            wgpu::VertexAttribute {
                                format: wgpu::VertexFormat::Float32x3,
                                offset: 0,
                                shader_location: 1,
                            },
                            wgpu::VertexAttribute {
                                format: wgpu::VertexFormat::Float32x3,
                                offset: 16,
                                shader_location: 2,
                            },
                            wgpu::VertexAttribute {
                                format: wgpu::VertexFormat::Float32x4,
                                offset: 32,
                                shader_location: 3,
                            },
                            wgpu::VertexAttribute {
                                format: wgpu::VertexFormat::Float32x4,
                                offset: 48,
                                shader_location: 4,
                            },
                        ],
                    }),
                ],
                compilation_options: wgpu::PipelineCompilationOptions::default(),
            },
            primitive: wgpu::PrimitiveState::default(),
            depth_stencil: Some(wgpu::DepthStencilState {
                format: DEPTH_FORMAT,
                depth_write_enabled: Some(true),
                depth_compare: Some(wgpu::CompareFunction::Greater),
                stencil: wgpu::StencilState::default(),
                bias: wgpu::DepthBiasState::default(),
            }),
            multisample: wgpu::MultisampleState::default(),
            fragment: Some(wgpu::FragmentState {
                module: &shader,
                entry_point: Some("fs_main"),
                targets: &[Some(wgpu::ColorTargetState {
                    format: configuration.format,
                    blend: Some(wgpu::BlendState::REPLACE),
                    write_mask: wgpu::ColorWrites::ALL,
                })],
                compilation_options: wgpu::PipelineCompilationOptions::default(),
            }),
            multiview_mask: None,
            cache: None,
        });
        let cube_vertex_bytes = encode_f32s(CUBE_VERTICES.into_iter().flatten());
        let cube_vertex_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Salimon unit cuboid vertices"),
            contents: &cube_vertex_bytes,
            usage: wgpu::BufferUsages::VERTEX,
        });
        let instance_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("Salimon cuboid instances"),
            size: INSTANCE_STRIDE,
            usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });
        let depth_target = DepthTarget::new(&device, configured_width, configured_height);
        let condition_overlay = overlay::OverlayRenderer::new(&device, configuration.format);
        let overlay = overlay::OverlayRenderer::new(&device, configuration.format);
        let action_bar = overlay::OverlayRenderer::new(&device, configuration.format);
        let reticle = overlay::OverlayRenderer::new(&device, configuration.format);
        let toolbar = overlay::OverlayRenderer::new(&device, configuration.format);
        let weather = crate::weather::WeatherRenderer::new(&device, configuration.format);
        let atmosphere = crate::atmosphere::AtmosphereRenderer::new(&device, configuration.format);
        let sky = crate::sky::SkyRenderer::new(&device, configuration.format);
        let spheres = spheres::SphereRenderer::new(&device, &queue, configuration.format);
        let terrain = terrain::TerrainRenderer::new(&device, configuration.format, &spheres);
        let ship_mesh = ship_mesh::ShipMeshRenderer::new(&device, configuration.format)?;
        let fixtures = fixtures::FixtureMeshRenderer::new(&device, configuration.format)?;
        let held_item = held_item::HeldItemRenderer::new(&device, &queue, configuration.format)?;
        let resource_meshes =
            resource_mesh::ResourceMeshRenderer::new(&device, configuration.format)?;
        let gpu_timer = timestamp_queries_supported.then(|| GpuTimer::new(&device, &queue));

        let mut renderer = Self {
            surface,
            device,
            queue,
            configuration,
            pipeline,
            camera_buffer,
            camera_bind_group,
            cube_vertex_buffer,
            instance_buffer,
            instance_capacity: 1,
            depth_target,
            condition_overlay,
            overlay,
            action_bar,
            reticle,
            toolbar,
            toolbar_raster: equipment_toolbar::ToolbarRaster::default(),
            spheres,
            sky,
            atmosphere,
            weather,
            terrain,
            ship_mesh,
            fixtures,
            last_fixture_batch: Default::default(),
            held_item,
            resource_meshes,
            gpu_timer,
            cached_gpu_memory: None,
            presented_frames: 0,
            drawable: false,
            info,
            capture_requested: false,
            capture_result: None,
        };
        renderer.resize(initial_size);
        Ok(renderer)
    }
}
